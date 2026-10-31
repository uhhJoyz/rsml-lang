//! A deterministic fuzzer for the macros and the formatter.
//!
//! Inputs: for every block in this repository (tests, examples, the
//! formatter's fixtures, README), its prefixes at token boundaries with
//! the open brackets closed (what an editor sends while you type) and
//! random mutations (tokens deleted, doubled, swapped, glued to or
//! spaced from their neighbours); plus expressions generated from a small
//! grammar of every notation, so new syntax is exercised before any test
//! uses it.
//!
//! Checks:
//! * `expand` / `expand_with` and `expand_items` never panic, as rustc
//!   runs them and as rust-analyzer does;
//! * for an input that expands without an error, the formatter does not
//!   panic or refuse it, is idempotent, and keeps the expansion (markers
//!   aside), under the default config and, in turn, every config of
//!   tests/corpus.rs.
//!
//! `cargo test` runs a sample in a few seconds (of the toy examples,
//! examples/toy, the blocks of one file in four). `FUZZ_ITERS=n` runs `n`
//! mutations per block (and `n` times as many generated inputs, and every
//! prefix, of every file):
//!
//! ```sh
//! FUZZ_ITERS=200 cargo test --release -p rsml-fmt --test fuzz -- --nocapture
//! ```

use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use rsml_fmt::{Block, BlockKind, Config, blocks, format_source};

/// Mutations per block, and the scale of everything else, by default.
const DEFAULT_ITERS: usize = 3;

/// The configs of tests/corpus.rs, with the Rust in `rsml!` kept as written
/// (rustfmt is the corpus test's business, and slow to run this often).
const CONFIGS: &[&str] = &[
    "",
    "max_width = 60",
    "max_width = 40; indent = 4; block_indent = 2",
    "max_width = 24",
    "pipe.indent = align; pipe.layout = wrap; apply.position = leading; compose.indent = align",
    "collections.style = block; collections.separator_position = before; \
     collections.trailing_semicolon = vertical; collections.space_inside_lists = true; \
     collections.space_inside_arrays = true; collections.layout = wrap",
    "if.style = keyword_first; match.leading_bar = always; match.align_arrows = true; \
     match.body = next_line; match.cases = vertical; match.case_indent = 2; begin_end_hug = false",
    "let.body = fit; let.in_position = end_of_line; toplevel.break = fit; sequence.break = fit; \
     sequence.style = separator; toplevel.double_semi = always; \
     toplevel.double_semi_position = own_line; let.and_blank_line = true",
    "if.style = compact; app.args = fit_or_vertical; begin_end = parens; \
     op_section_spaces = minimal; colon_spacing = after; fun.hug_last_arg = false; \
     let.fun_on_eq_line = false; collections.space_inside_records = false",
    "max_width = 30; arithmetic.force_break_at = 1; pipe.force_break_at = 1; \
     boolean.position = trailing; cons.position = leading; comparison.indent = align; \
     custom_operators.layout = wrap",
    "single_line_blocks = never; if.style = vertical; preserve_blank_lines = false; \
     toplevel.blank_lines = always; fn_single_line = true; comments = rust",
];

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
}

fn iters() -> usize {
    std::env::var("FUZZ_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_ITERS)
}

/// The `.rs` files under `dir`, in its subdirectories too, sorted.
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

fn corpus() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut out = Vec::new();
    for dir in ["tests", "examples", "rsml-fmt/tests/fixtures"] {
        let mut paths = Vec::new();
        rs_files(&root.join(dir), &mut paths);
        for p in paths {
            out.push((
                p.display().to_string(),
                std::fs::read_to_string(&p).unwrap(),
            ));
        }
    }
    let readme = std::fs::read_to_string(root.join("README.md")).unwrap();
    for (i, block) in readme.split("```rust\n").skip(1).enumerate() {
        let code = block.split("```").next().unwrap();
        out.push((format!("README.md block {}", i), code.to_string()));
    }
    out
}

/// What a piece of input is: the contents of an `ml!` block, or the body
/// of an `rsml!` function (with the text of the whole `rsml!` around it).
#[derive(Clone)]
struct Origin {
    name: String,
    kind: BlockKind,
    params: Vec<String>,
    /// The `rsml! { .. }` contents before and after the body's braces.
    items: Option<(String, String)>,
}

/// Token ranges and groups `(open, close, closer)` of `ts`, by byte offset.
fn walk(ts: TokenStream, toks: &mut Vec<(usize, usize)>, groups: &mut Vec<(usize, usize, char)>) {
    for t in ts {
        match t {
            TokenTree::Group(g) => {
                let o = g.span_open().byte_range();
                let c = g.span_close().byte_range();
                let closer = match g.delimiter() {
                    Delimiter::Parenthesis => ')',
                    Delimiter::Bracket => ']',
                    Delimiter::Brace => '}',
                    Delimiter::None => ' ',
                };
                toks.push((o.start, o.end));
                toks.push((c.start, c.end));
                groups.push((o.start, c.start, closer));
                walk(g.stream(), toks, groups);
            }
            t => {
                let r = t.span().byte_range();
                toks.push((r.start, r.end));
            }
        }
    }
}

/// Prefixes (every `step`-th) and `n` mutations of a block's text.
fn variants(inner: &str, step: usize, n: usize, rng: &mut Rng) -> Vec<String> {
    let Ok(ts) = inner.parse::<TokenStream>() else {
        return Vec::new();
    };
    let (mut toks, mut groups) = (Vec::new(), Vec::new());
    walk(ts, &mut toks, &mut groups);
    toks.sort();
    let mut out = Vec::new();
    for (k, &(_, e)) in toks.iter().enumerate() {
        if k % step != 0 {
            continue;
        }
        let mut s = inner[..e].to_string();
        let mut open: Vec<&(usize, usize, char)> =
            groups.iter().filter(|g| g.0 < e && g.1 >= e).collect();
        open.sort_by_key(|g| std::cmp::Reverse(g.0));
        for g in open {
            s.push(' ');
            s.push(g.2);
        }
        out.push(s);
    }
    let leaves: Vec<(usize, usize)> = toks
        .iter()
        .copied()
        .filter(|(s, e)| !matches!(&inner[*s..*e], "(" | ")" | "[" | "]" | "{" | "}"))
        .collect();
    if leaves.is_empty() {
        return out;
    }
    for _ in 0..n {
        let a = *rng.pick(&leaves);
        let b = *rng.pick(&leaves);
        let s = match rng.below(6) {
            // Delete a token.
            0 => format!("{}{}", &inner[..a.0], &inner[a.1..]),
            // Double it, spaced.
            1 => format!("{} {} {}", &inner[..a.1], &inner[a.0..a.1], &inner[a.1..]),
            // Swap two.
            2 => {
                let (x, y) = if a.0 < b.0 { (a, b) } else { (b, a) };
                if x == y || x.1 > y.0 {
                    continue;
                }
                format!(
                    "{}{}{}{}{}",
                    &inner[..x.0],
                    &inner[y.0..y.1],
                    &inner[x.1..y.0],
                    &inner[x.0..x.1],
                    &inner[y.1..]
                )
            }
            // Glue a token to the one after it.
            3 => {
                let rest = &inner[a.1..];
                let trimmed = rest.trim_start_matches([' ', '\t']);
                format!("{}{}", &inner[..a.1], trimmed)
            }
            // Space it from its neighbours.
            4 => format!("{} {} {}", &inner[..a.0], &inner[a.0..a.1], &inner[a.1..]),
            // Put another token after it.
            _ => {
                let t = rng.pick(&[
                    ";", ";;", "&", "!", "*", "-", "|", "->", "(* c *)", "// c\n", ".", "..", "_",
                    "()", ",", "?", "begin", "end", "in", "then", "fun x ->", "x", "1",
                ]);
                format!("{} {} {}", &inner[..a.1], t, &inner[a.1..])
            }
        };
        out.push(s);
    }
    out
}

/// Expressions from a small grammar of every notation, with random
/// spacing where spacing matters.
struct Gen<'a> {
    rng: &'a mut Rng,
}

impl Gen<'_> {
    fn sp(&mut self) -> &'static str {
        if self.rng.chance(30) { "" } else { " " }
    }

    fn name(&mut self) -> String {
        self.rng
            .pick(&["x", "y", "xs", "n", "f", "g", "s", "p", "o", "r", "acc"])
            .to_string()
    }

    fn atom(&mut self, d: usize) -> String {
        if d == 0 {
            return match self.rng.below(3) {
                0 => self.name(),
                1 => self
                    .rng
                    .pick(&[
                        "1", "0", "2.0", "1.", "\"s\"", "'c'", "true", "()", "1L", "[]",
                    ])
                    .to_string(),
                _ => self
                    .rng
                    .pick(&[
                        ".len",
                        ".0",
                        "( + )",
                        "( * )",
                        "(.len)",
                        "Vec::new ()",
                        "i32::MAX",
                        "f _1 _0",
                    ])
                    .to_string(),
            };
        }
        let d = d - 1;
        let r = self.rng.below(32);
        match r {
            0 => self.name(),
            1 => self
                .rng
                .pick(&["1", "0", "2.0", "1.", "\"s\"", "'c'", "true", "()", "1L"])
                .to_string(),
            2 => format!("[{}]", self.list(d, "; ")),
            3 => format!("[|{}|]", self.list(d, "; ")),
            4 => format!("({}, {})", self.expr(d), self.expr(d)),
            5 => format!("Some{}({})", self.sp(), self.expr(d)),
            6 => format!("Point {{ x = {}; y }}", self.expr(d)),
            7 => format!("{{ p with x = {} }}", self.expr(d)),
            8 => self
                .rng
                .pick(&[
                    ".len", ".0", ".iter()", ".map(f)", "( + )", "( * )", "( - )", "(.len)",
                ])
                .to_string(),
            9 => format!("({})", self.expr(d)),
            10 => format!("begin {} end", self.seq(d)),
            11 => self
                .rng
                .pick(&[
                    "Vec::new ()",
                    "Vec.new ()",
                    "std::mem::swap",
                    "i32::MAX",
                    "Vec::<u8>::new ()",
                ])
                .to_string(),
            12 => format!("f{}({})", self.sp(), self.list(d, ", ")),
            13 => format!("{}[{}]", self.name(), self.expr(d)),
            14 => format!("{}.({})", self.name(), self.expr(d)),
            15 => format!(
                "{}.{}",
                self.name(),
                self.rng
                    .pick(&["len ()", "len()", "0", "x", "iter().count()", "await"])
            ),
            16 => format!("{}?", self.name()),
            17 => format!("!{}", self.atom(d)),
            18 => format!("*{}", self.atom(d)),
            19 => format!("&{}{}", self.rng.pick(&["", "mut ", "&"]), self.atom(d)),
            20 => format!("({} : {})", self.expr(d), self.ty()),
            21 => format!("{} _", self.name()),
            22 => "f _1 _0".to_string(),
            23 => format!("println!(\"{{}}\", {})", self.expr(d)),
            24 => format!("assert_eq!({}, {})", self.expr(d), self.expr(d)),
            25 => format!("vec![{}; {}]", self.expr(d), self.expr(d)),
            26 => format!("unsafe {{ {} }}", self.seq(d)),
            27 => format!("async move {{ {} }}", self.expr(d)),
            28 => format!(
                "({}{}..{}{})",
                self.expr(d),
                self.sp(),
                self.sp(),
                self.expr(d)
            ),
            29 => format!("({} as i64)", self.expr(d)),
            30 => format!("~x:{}", self.atom(d)),
            _ => format!("{} (* note *)", self.name()),
        }
    }

    fn ty(&mut self) -> String {
        self.rng
            .pick(&[
                "i64",
                "&str",
                "Vec<i64>",
                "i64 -> i64",
                "i64 * String",
                "&&str",
                "Option<'a>",
                "_",
            ])
            .to_string()
    }

    fn list(&mut self, d: usize, sep: &str) -> String {
        let n = self.rng.below(3);
        (0..n).map(|_| self.expr(d)).collect::<Vec<_>>().join(sep)
    }

    fn app(&mut self, d: usize) -> String {
        let mut s = self.atom(d);
        for _ in 0..self.rng.below(3) {
            s.push(' ');
            s.push_str(&self.atom(d));
        }
        s
    }

    fn pat(&mut self) -> String {
        self.rng
            .pick(&[
                "_",
                "x",
                "Some x",
                "Some(x, y)",
                "None",
                "[]",
                "x :: rest",
                "(a, b)",
                "1..=3",
                "'a' .. 'z'",
                "0. ..= 1.",
                "A | B",
                "&x",
                "Point { x; _ }",
                "p when p > 0",
                "p if p > 0",
                "Shape::Rect(w, h)",
                "Shape::Circle r",
            ])
            .to_string()
    }

    fn expr(&mut self, d: usize) -> String {
        if d == 0 {
            return self.atom(0);
        }
        let d = d - 1;
        match self.rng.below(30) {
            0..=4 => self.app(d + 1),
            5 => {
                let op = self.rng.pick(&[
                    "+", "-", "*", "/", "+.", "**", "=", "<>", "==", "<", ">=", "&&", "||", "or",
                    "mod", "land", "lsl", "::", "@", "^", "|>", "@@", ">>", "<<", "+!",
                ]);
                format!("{} {} {}", self.app(d), op, self.app(d))
            }
            6 => format!("let {} = {} in {}", self.name(), self.expr(d), self.expr(d)),
            7 => format!(
                "let {}f x y = {} in {}",
                self.rng.pick(&["", "mut "]),
                self.expr(d),
                self.expr(d)
            ),
            8 => format!("let rec go (n : i64) : i64 = {} in go 1", self.expr(d)),
            9 if self.rng.below(2) == 0 => format!(
                "fun {} -> {}",
                self.rng
                    .pick(&["x", "x y", "(x : i64)", "()", "move x", "(a, b)"]),
                self.expr(d)
            ),
            // Rust's spelling of the same closures.
            9 => format!(
                "{} {}",
                self.rng.pick(&[
                    "|x|", "|x, y|", "|x: i64|", "||", "move |x|", "|(a, b)|", "|&x|"
                ]),
                self.expr(d)
            ),
            10 => format!(
                "function {} -> {} | _ -> {}",
                self.pat(),
                self.expr(d),
                self.expr(d)
            ),
            11 => format!(
                "if {} then {} else {}",
                self.expr(d),
                self.expr(d),
                self.expr(d)
            ),
            12 if self.rng.below(3) == 0 => format!(
                // Nested without parentheses: `A::Y` is the outer `match`'s.
                "match {} with A::X -> match {} with B::P -> {} | _ -> {} | A::Y -> {}",
                self.expr(d),
                self.expr(d),
                self.expr(d),
                self.expr(d),
                self.expr(d)
            ),
            12 => format!(
                "match {} with {} -> {} | _ -> {}",
                self.expr(d),
                self.pat(),
                self.expr(d),
                self.expr(d)
            ),
            13 => format!("for i = 0 to {} do {} done", self.expr(d), self.seq(d)),
            14 => format!("for i = {} downto 0 do {} done", self.expr(d), self.seq(d)),
            15 => format!(
                "for {} in {} do {} done",
                self.pat(),
                self.expr(d),
                self.seq(d)
            ),
            16 => format!("while {} do {} done", self.expr(d), self.seq(d)),
            17 => format!("{} <- {}", self.name(), self.expr(d)),
            18 => format!("let r = ref {} in r := !r + 1; incr r; !r", self.expr(d)),
            // `|>.map`: Rust lexes one operator `|>.`.
            19 => format!(
                "{} |>{}.map(fun x -> {}) |>{}.collect::<Vec<_>>()",
                self.name(),
                self.sp(),
                self.expr(d),
                self.sp()
            ),
            20 => format!("return {}", self.expr(d)),
            21 => format!("({} : i64 -> i64)", self.expr(d)),
            22 => format!("use std::cmp::*; {}", self.expr(d)),
            23 => format!(
                "not {} || -{} < -. {}",
                self.atom(d),
                self.atom(d),
                self.atom(d)
            ),
            24 => format!(
                "assert ({}); if {} then {}",
                self.expr(d),
                self.expr(d),
                self.expr(d)
            ),
            25 => format!("let (a, b) = {} in a", self.expr(d)),
            26 => format!(
                "let rec ev n = {} and od n = {} in ev 1",
                self.expr(d),
                self.expr(d)
            ),
            // A defined `( |>. )` is used as written, glued or not.
            27 => format!(
                "let ( {0} ) a b = {1} in 1 {0}{2}{3}",
                self.rng.pick(&["+!", "|>."]),
                self.expr(d),
                self.sp(),
                self.atom(d)
            ),
            28 => format!(
                "map (_ * 2) {} |> f(_, {}) |> (_.len())",
                self.atom(d),
                self.atom(d)
            ),
            _ => self.seq(d),
        }
    }

    fn seq(&mut self, d: usize) -> String {
        let n = 1 + self.rng.below(3);
        (0..n).map(|_| self.expr(d)).collect::<Vec<_>>().join("; ")
    }

    fn toplevel(&mut self) -> String {
        let mut parts = Vec::new();
        if self.rng.chance(20) {
            parts.push("external h : i64 -> i64 -> i64".to_string());
        }
        if self.rng.chance(20) {
            parts.push(format!("let z = {}", self.expr(2)));
        }
        parts.push(self.expr(3));
        if self.rng.chance(15) {
            parts.push(self.expr(2));
        }
        parts.join(" ;; ")
    }
}

fn panic_message(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// The source of a file holding the input as its origin's kind of block.
fn file_for(text: &str, o: &Origin) -> String {
    match o.kind {
        BlockKind::Ml => format!("fn fuzz() {{\n    let _ = ml! {{\n{}\n    }};\n}}\n", text),
        BlockKind::FnBody => {
            let params: Vec<String> = o.params.iter().map(|p| format!("{}: T", p)).collect();
            format!(
                "rsml! {{\n    fn fuzz({}) {{\n{}\n    }}\n}}\n",
                params.join(", "),
                text
            )
        }
    }
}

struct Fuzz {
    configs: Vec<Config>,
    seen: HashSet<String>,
    runs: usize,
    formatted: usize,
    problems: Vec<String>,
}

impl Fuzz {
    fn fail(&mut self, what: &str, o: &Origin, input: &str, detail: &str) {
        self.problems.push(format!(
            "{} ({})\n    input: {}\n    {}",
            what,
            o.name,
            input.replace('\n', "⏎"),
            detail.replace('\n', "⏎")
        ));
    }

    fn run(&mut self, input: &str, o: &Origin, rng: &mut Rng) {
        if !self.seen.insert(format!("{:?}{}", o.kind, input)) {
            return;
        }
        let Ok(ts) = input.parse::<TokenStream>() else {
            return;
        };
        self.runs += 1;
        let locals: Vec<(String, Option<Vec<usize>>)> =
            o.params.iter().map(|p| (p.clone(), None)).collect();
        let mut expansion = None;
        for editor in [false, true] {
            rsml_compiler::force_editor_mode(Some(editor));
            let r = catch_unwind(AssertUnwindSafe(|| {
                rsml_compiler::expand_with(ts.clone(), &locals).to_string()
            }));
            match r {
                Ok(out) if !editor => expansion = Some(out),
                Ok(_) => {}
                Err(e) => {
                    let mode = if editor { "rust-analyzer" } else { "rustc" };
                    self.fail(
                        &format!("PANIC in expand ({})", mode),
                        o,
                        input,
                        &panic_message(e),
                    );
                }
            }
            if let Some((before, after)) = &o.items {
                let items = format!("{}{{\n{}\n}}{}", before, input, after);
                if let Ok(its) = items.parse::<TokenStream>() {
                    let r = catch_unwind(AssertUnwindSafe(|| {
                        rsml_compiler::expand_items(its).to_string()
                    }));
                    if let Err(e) = r {
                        let mode = if editor { "rust-analyzer" } else { "rustc" };
                        self.fail(
                            &format!("PANIC in expand_items ({})", mode),
                            o,
                            input,
                            &panic_message(e),
                        );
                    }
                }
            }
        }
        rsml_compiler::force_editor_mode(None);
        let Some(expansion) = expansion else { return };
        if expansion.contains("compile_error") {
            return;
        }
        // The default config, and one of the others in turn.
        self.formatted += 1;
        let other = 1 + rng.below(self.configs.len() - 1);
        for k in [0, other] {
            self.format(input, o, k);
        }
    }

    fn format(&mut self, input: &str, o: &Origin, k: usize) {
        let cfg = self.configs[k].clone();
        let src = file_for(input, o);
        let r = catch_unwind(AssertUnwindSafe(|| format_source(&src, &cfg)));
        let once = match r {
            Err(e) => {
                return self.fail(
                    &format!("PANIC in the formatter [{}]", CONFIGS[k]),
                    o,
                    input,
                    &panic_message(e),
                );
            }
            Ok(Err(e)) => {
                return self.fail(&format!("FORMATTER ERROR [{}]", CONFIGS[k]), o, input, &e);
            }
            Ok(Ok(out)) => out,
        };
        if let Some(w) = once.warnings.first() {
            // A mutated `rsmlfmt:` comment is rightly refused.
            if w.message.starts_with("bad `rsmlfmt:` comment") {
                return;
            }
            let detail = w.message.clone();
            return self.fail(
                &format!("NOT FORMATTED [{}]", CONFIGS[k]),
                o,
                input,
                &detail,
            );
        }
        let twice = match format_source(&once.text, &cfg) {
            Ok(t) => t.text,
            Err(e) => {
                return self.fail(
                    &format!("FORMATTER ERROR, second run [{}]", CONFIGS[k]),
                    o,
                    input,
                    &e,
                );
            }
        };
        if twice != once.text {
            return self.fail(
                &format!("NOT IDEMPOTENT [{}]", CONFIGS[k]),
                o,
                input,
                &once.text,
            );
        }
        let key = |text: &str| -> Vec<String> {
            blocks(text)
                .map(|bs| {
                    bs.iter()
                        .map(|b: &Block| rsml_fmt::expansion_key(b.group.stream(), b))
                        .collect()
                })
                .unwrap_or_default()
        };
        if key(&src) != key(&once.text) {
            self.fail(
                &format!("CHANGES THE EXPANSION [{}]", CONFIGS[k]),
                o,
                input,
                &once.text,
            );
        }
    }
}

#[test]
fn fuzz() {
    let n = iters();
    let long = n > DEFAULT_ITERS;
    // Panics are caught and reported; their messages are noise here.
    std::panic::set_hook(Box::new(|_| {}));
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut f = Fuzz {
        configs: CONFIGS
            .iter()
            .map(|c| {
                Config::default()
                    .with_overrides(c)
                    .and_then(|c| c.with_overrides("rust_items = preserve"))
                    .unwrap()
            })
            .collect(),
        seen: HashSet::new(),
        runs: 0,
        formatted: 0,
        problems: Vec::new(),
    };
    let mut toy = 0;
    for (name, src) in corpus() {
        if name.contains("/examples/toy/") {
            toy += 1;
            if !long && toy % 4 != 0 {
                continue;
            }
        }
        let Ok(bs) = blocks(&src) else { continue };
        for b in bs {
            let o = b.group.span_open().byte_range().end;
            let c = b.group.span_close().byte_range().start;
            let inner = src[o..c].to_string();
            let items = enclosing_items(&src, &b);
            let origin = Origin {
                name: name.clone(),
                kind: b.kind,
                params: b.params.clone(),
                items,
            };
            let step = if long { 1 } else { 9 };
            for v in variants(&inner, step, n, &mut rng) {
                f.run(&v, &origin, &mut rng);
            }
        }
    }
    let generated = Origin {
        name: "the grammar".to_string(),
        kind: BlockKind::Ml,
        params: Vec::new(),
        items: None,
    };
    let mut generated_runs = 0;
    for _ in 0..100 * n {
        let text = Gen { rng: &mut rng }.toplevel();
        let before = f.runs;
        f.run(&text, &generated, &mut rng);
        // And what an editor sends while it is being typed.
        if rng.chance(25) {
            for v in variants(&text, 3, 1, &mut rng) {
                f.run(&v, &generated, &mut rng);
            }
        }
        generated_runs += f.runs - before;
    }
    let _ = std::panic::take_hook();
    println!(
        "fuzz: {} inputs ({} generated), {} formatted (under 2 configs each), {} problems",
        f.runs,
        generated_runs,
        f.formatted,
        f.problems.len()
    );
    let show = std::env::var("FUZZ_SHOW")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    for p in f.problems.iter().take(show) {
        println!("{}", p);
    }
    assert!(
        f.problems.is_empty(),
        "{} problems (see the output above)",
        f.problems.len()
    );
}

/// For a function body in `rsml!`, the text of the `rsml!` contents before
/// and after the body's braces.
fn enclosing_items(src: &str, b: &Block) -> Option<(String, String)> {
    if b.kind != BlockKind::FnBody {
        return None;
    }
    let body = b.group.span().byte_range();
    // The innermost `rsml! { .. }` around the body.
    let ts: TokenStream = src.parse().ok()?;
    let mut groups = Vec::new();
    rsml_fmt::rustfmt::rsml_groups(ts, &mut groups);
    let g = groups.into_iter().find(|g| {
        let r = g.span().byte_range();
        r.start < body.start && body.end < r.end
    })?;
    let r = g.span().byte_range();
    Some((
        src[r.start + 1..body.start].to_string(),
        src[body.end..r.end - 1].to_string(),
    ))
}

//! Recursive-descent parser for the OCaml subset, following the precedence
//! table of the OCaml manual (section "Expressions").

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use proc_macro2::{Delimiter, Group, Ident, Literal, Span, TokenStream};
use quote::quote_spanned;

use crate::arms;
use crate::ast::*;
use crate::lexer::{Op, Tok, adjacent, ident_str, is_capitalized, lex, to_stream, word};
use crate::types::{GenericArg, Type};

pub struct Error {
    pub span: Span,
    pub msg: String,
}

impl Error {
    pub fn new(span: Span, msg: impl Into<String>) -> Error {
        Error {
            span,
            msg: msg.into(),
        }
    }

    pub fn to_compile_error(&self) -> TokenStream {
        let msg = format!("rsml: {}", self.msg);
        quote_spanned!(self.span=> ::core::compile_error!(#msg))
    }
}

pub type R<T> = Result<T, Error>;

/// The name, type and Rust path of an `external` declaration.
type External = (Name, Type, Option<(String, Span)>);

/// OCaml keywords. Sorted (see the tests).
const KEYWORDS: &[&str] = &[
    "and",
    "as",
    "asr",
    "assert",
    "begin",
    "do",
    "done",
    "downto",
    "else",
    "end",
    "exception",
    "external",
    "false",
    "for",
    "fun",
    "function",
    "if",
    "in",
    "land",
    "lazy",
    "let",
    "lor",
    "lsl",
    "lsr",
    "lxor",
    "match",
    "mod",
    "module",
    "of",
    "or",
    "rec",
    "then",
    "til",
    "to",
    "true",
    "type",
    "val",
    "when",
    "while",
    "with",
];

pub fn is_keyword(s: &str) -> bool {
    // Asked for nearly every token; the list is sorted (see the tests).
    KEYWORDS.binary_search(&s).is_ok()
}

/// The identifier that stands in for a name or expression missing from
/// unfinished code (see [`Parser::set_recover`]).
pub const MISSING: &str = "__rsml_missing";

/// Keywords that cannot name a field or method (Rust reserves them too), so
/// in unfinished code `xs.` followed by one of them is missing its name.
const NEVER_FIELDS: &[&str] = &[
    "as", "async", "break", "continue", "do", "else", "false", "for", "fun", "if", "in", "let",
    "match", "mod", "return", "true", "type", "unsafe", "while",
];

/// Rust keywords that are not OCaml keywords and that are Rust syntax:
/// none of them can name a value, binding, field or method (but written raw,
/// `r#fn`). Some start syntax of their own here (`return`, `break`,
/// `continue`, `unsafe { .. }`, `async { .. }`, `fun move`), `ref` is OCaml's
/// `ref e`, `await` follows a `.` and `dyn` is in types; the others are Rust
/// written in OCaml code by mistake, and get an error saying so. `self`,
/// `Self`, `super` and `crate` are not here: they start paths. Sorted.
pub const RUST_KEYWORDS: &[&str] = &[
    "async", "await", "break", "const", "continue", "dyn", "enum", "extern", "fn", "impl", "loop",
    "move", "pub", "ref", "return", "static", "struct", "trait", "unsafe", "use", "where",
];

/// Words that Rust reserves (in every edition, or from 2018 or 2024 on)
/// without syntax that rsml uses: ordinary names in OCaml code (`let gen =
/// 3`, `rng.gen ()`), emitted as raw identifiers (`r#gen`), which every
/// edition accepts. A macro cannot tell which edition calls it, so this is
/// the same for all. Sorted.
pub const RESERVED: &[&str] = &[
    "abstract", "become", "box", "final", "gen", "macro", "override", "priv", "try", "typeof",
    "unsized", "virtual", "yield",
];

/// A Rust keyword written as an identifier (not as a raw `r#name`).
fn is_rust_keyword(i: &Ident) -> bool {
    RUST_KEYWORDS.binary_search(&i.to_string().as_str()).is_ok()
}

/// Whether a field or method name after `.` is a Rust keyword (OCaml ones
/// included: `p.type` is no Rust), other than `await`.
fn is_keyword_field(i: &Ident) -> bool {
    const SHARED: &[&str] = &[
        "as", "crate", "else", "false", "for", "if", "in", "let", "match", "mod", "mut", "self",
        "Self", "super", "true", "type", "while",
    ];
    i != "await" && (is_rust_keyword(i) || SHARED.contains(&i.to_string().as_str()))
}

/// The error for a Rust keyword where a name is expected.
fn rust_keyword_error(i: &Ident) -> Error {
    let kw = i.to_string();
    let hint = match kw.as_str() {
        "fn" => "; a closure is `|x| e`".to_string(),
        "move" => "; a `move` closure is `move |x| e`".to_string(),
        "loop" => "; an endless loop is `while true do .. done`".to_string(),
        "use" => "; `use path;` starts an expression or a toplevel phrase".to_string(),
        "await" => "; write `e.await`".to_string(),
        "ref" => {
            "; `ref e` makes a mutable local, and matching on `&e` binds by reference".to_string()
        }
        "unsafe" | "async" => format!("; a block is `{} {{ e }}`", kw),
        "const" | "static" | "struct" | "enum" | "trait" | "impl" | "extern" | "pub" => {
            "; Rust items go outside the rsml code (or in `rsml!`)".to_string()
        }
        _ => String::new(),
    };
    Error::new(
        i.span(),
        format!(
            "`{0}` is a Rust keyword, not a name (as a name: `r#{0}`){1}",
            kw, hint
        ),
    )
}

/// The error for an operator starting with `!` (but `!=`) defined or used
/// as one. In OCaml such an operator is a prefix operator; here `!` before
/// an expression is a dereference, and a run of `!`, `*` and `&` is that
/// many dereferences and borrows (`!*p` is `*(*p)`): it could never be used.
pub fn bang_op_error(op: &Op) -> Option<Error> {
    if !op.text.starts_with('!') || op.text == "!=" {
        return None;
    }
    Some(Error::new(
        op.span(),
        format!(
            "`{0}` cannot be an operator: one starting with `!` is a prefix operator, \
             and here `!` before an expression is a dereference (`!r` is `*r`), so `{0}` could \
             never be used; define a function instead",
            op.text
        ),
    ))
}

/// Compound assignments, which OCaml does not have, with the OCaml operator
/// for `x <- x op e`.
const ASSIGN_OPS: &[(&str, &str)] = &[
    ("<<=", "lsl"),
    (">>=", "lsr"),
    ("+=", "+"),
    ("-=", "-"),
    ("*=", "*"),
    ("/=", "/"),
    ("%=", "mod"),
    ("&=", "land"),
    ("|=", "lor"),
    ("^=", "lxor"),
];

/// The error for `x += e` and the like, when no such operator is defined.
pub fn assign_op_error(op: &Op) -> Option<Error> {
    let (text, ocaml) = ASSIGN_OPS.iter().find(|(t, _)| op.text.starts_with(t))?;
    let r = match *text {
        "+=" => " (for a `ref`: `r := !r + e`, or `incr r`)",
        "-=" => " (for a `ref`: `r := !r - e`, or `decr r`)",
        _ => "",
    };
    Some(Error::new(
        op.span(),
        format!(
            "rsml has no `{0}`: write `x <- x {1} e`{2}, or define the operator with `let ( {0} ) a b = ..`",
            text, ocaml, r
        ),
    ))
}

/// Operators that Rust (as OCaml) lexes together with the `.` of a method or
/// field section written right after them: `xs |>.iter()` has the operator
/// `|>.`. Unless such an operator is defined, it is the operator without
/// the `.`, then the section (see [`Parser::split_glued_section`]).
const GLUED_SECTION_OPS: &[&str] = &["|>.", ">>.", "<<.", "@@."];

/// The error for `|>.` and the like, when no such operator is defined and
/// no name or number follows the `.` (`xs |>. iter`).
pub fn glued_section_error(op: &Op) -> Option<Error> {
    if !GLUED_SECTION_OPS.contains(&op.text.as_str()) {
        return None;
    }
    Some(Error::new(
        op.span(),
        format!(
            "unknown operator `{0}`; for a method or field section after `{1}`, write `{1} .name` \
             or `{1} .0` (the name right after the `.`), or define the operator with \
             `let ( {0} ) a b = ..`",
            op.text,
            &op.text[..2]
        ),
    ))
}

/// The scope entry of a user-defined operator (no identifier has a `:`).
fn op_key(op: &Op) -> String {
    format!("op:{}", op.text)
}

/// Keyword operators that can appear in an operator section, e.g. `( mod )`.
const KEYWORD_OPS: &[&str] = &["mod", "land", "lor", "lxor", "lsl", "lsr", "asr", "or"];

// Binary operator levels, loosest first. Ranges (`a..b`) are between
// comparisons and `@`, and do not chain.
const OR: u8 = 1;
const AND: u8 = 2;
const CMP: u8 = 3;
const RANGE: u8 = 4;
const CONCAT: u8 = 5;
const CONS: u8 = 6;
const ADD: u8 = 7;
const MUL: u8 = 8;
const POW: u8 = 9;

#[derive(PartialEq, Clone, Copy)]
enum Assoc {
    Left,
    Right,
}

fn binop_info(op: &str) -> Option<(u8, Assoc)> {
    use Assoc::*;
    match op {
        "->" | "|" | ":=" | "<-" | "&" | "=>" | ".." | "..=" => return None,
        "||" => return Some((OR, Right)),
        "&&" => return Some((AND, Right)),
        "::" => return Some((CONS, Right)),
        "!=" => return Some((CMP, Left)),
        _ => {}
    }
    if op.starts_with("**") {
        return Some((POW, Right));
    }
    match op.chars().next()? {
        '=' | '<' | '>' | '|' | '&' | '$' => Some((CMP, Left)),
        '@' | '^' => Some((CONCAT, Right)),
        '+' | '-' => Some((ADD, Left)),
        '*' | '/' | '%' => Some((MUL, Left)),
        _ => None,
    }
}

/// Precedence level (higher binds tighter) and right-associativity of an
/// infix operator, symbolic or keyword (`mod`, `lsl`, ..).
pub fn infix_level(op: &str) -> Option<(u8, bool)> {
    let info = match op {
        "mod" | "land" | "lor" | "lxor" => Some((MUL, Assoc::Left)),
        "lsl" | "lsr" | "asr" => Some((POW, Assoc::Right)),
        "or" => Some((OR, Assoc::Right)),
        _ => binop_info(op),
    };
    info.map(|(level, assoc)| (level, assoc == Assoc::Right))
}

pub struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    end: Span,
    /// Inside Rust-style argument lists `f.m(a, b)` a comma separates arguments.
    comma_stop: bool,
    /// Names bound by the ml code being parsed; used to tell the cons `h::t`
    /// apart from the Rust path `module::item`. Also the operators it
    /// defines ([`op_key`]), for [`Parser::split_glued_section`].
    scope: Vec<String>,
    /// Keep surface syntax for the formatter (see [`Surface`]).
    surface: bool,
    /// Operator characters that will not appear in the expansion (shared
    /// with sub-parsers); see [`Parser::unmapped_operators`].
    unmapped: Rc<RefCell<Vec<Span>>>,
    /// Continue past missing names and expressions (see [`Parser::set_recover`]).
    recover: bool,
    /// The errors recovered from (shared with sub-parsers).
    recovered: Rc<RefCell<Vec<Error>>>,
    /// Keywords and symbols with no Rust counterpart (shared with
    /// sub-parsers); see [`Parser::marks`].
    marks: Rc<RefCell<Vec<Mark>>>,
    /// Answers of [`Parser::touches`], by token: 0 not asked yet, else
    /// 1 + `None` / `Some(false)` / `Some(true)`.
    touch: RefCell<Vec<u8>>,
    /// Holes parsed and not yet in a section (shared with sub-parsers): while
    /// there are none, [`Parser::close_holes`] has nothing to look for.
    open_holes: Rc<Cell<usize>>,
    /// The `match`es and `function`s whose cases are being parsed, innermost
    /// last, with `None` for a barrier (see [`crate::arms`]). Brackets are
    /// barriers too: a sub-parser starts with none.
    levels: Vec<Option<arms::Level>>,
}

/// A keyword or symbol of the OCaml code (`then`, the `|` of a match arm)
/// that the expansion has nothing for, and the Rust token an editor should
/// color it like.
#[derive(Clone)]
pub struct Mark {
    /// `if`, `match`, `break`, `loop` or `let` (keywords), `|` or `=>`.
    pub like: &'static str,
    /// One per character.
    pub spans: Vec<Span>,
}

/// The Rust keyword each OCaml keyword is colored like: control flow as
/// control flow, the rest as `let`.
fn keyword_mark(kw: &str) -> Option<&'static str> {
    Some(match kw {
        "then" | "when" => "if",
        "with" | "function" => "match",
        "done" => "break",
        "to" | "downto" | "til" => "loop",
        "begin" | "end" | "fun" | "rec" | "and" | "external" => "let",
        _ => return None,
    })
}

impl Parser {
    pub fn new(toks: Vec<Tok>, end: Span) -> Parser {
        Parser {
            toks,
            pos: 0,
            end,
            comma_stop: false,
            scope: Vec::new(),
            surface: false,
            unmapped: Rc::default(),
            recover: false,
            recovered: Rc::default(),
            marks: Rc::default(),
            touch: RefCell::default(),
            open_holes: Rc::default(),
            levels: Vec::new(),
        }
    }

    /// Parses unfinished code, as an editor sends it while completing: a
    /// missing name (after `.` or `::`) or expression becomes [`MISSING`]
    /// and the error is recorded instead of ending the parse. The rest of
    /// the expansion then still exists for the editor to complete against.
    pub fn set_recover(&mut self) {
        self.recover = true;
    }

    /// The errors recovered from so far.
    pub fn take_recovered(&self) -> Vec<Error> {
        std::mem::take(&mut self.recovered.borrow_mut())
    }

    /// With recovery on, records `err` and returns the stand-in name.
    fn missing(&self, err: Error) -> R<Ident> {
        if !self.recover {
            return Err(err);
        }
        self.recovered.borrow_mut().push(err);
        Ok(Ident::new(MISSING, Span::call_site()))
    }

    /// An error in code that parses all the same: with recovery on, it is
    /// recorded and the parse goes on as if it were not there.
    fn recoverable(&self, err: Error) -> R<()> {
        if !self.recover {
            return Err(err);
        }
        self.recovered.borrow_mut().push(err);
        Ok(())
    }

    /// Spans of operator characters that the expansion turns into
    /// something else (`|>` into a call). Editors color tokens inside a
    /// macro call by what they become, so these need a stand-in.
    pub fn unmapped_operators(&self) -> Vec<Span> {
        self.unmapped.borrow().clone()
    }

    fn unmapped(&self, spans: &[Span]) {
        self.unmapped.borrow_mut().extend_from_slice(spans);
    }

    /// The OCaml keywords and symbols that the expansion has no Rust token
    /// for, in source order. Editors color tokens inside a macro call by
    /// what they become, so these need a stand-in.
    pub fn marks(&self) -> Vec<Mark> {
        self.marks.borrow().clone()
    }

    fn mark(&self, like: &'static str, spans: Vec<Span>) {
        self.marks.borrow_mut().push(Mark { like, spans });
    }

    /// Marks the keyword at the current position, if it needs one.
    fn mark_keyword(&self) {
        if let Some(Tok::Ident(i)) = self.peek()
            && let Some(like) = keyword_mark(&word(i))
        {
            self.mark(like, vec![i.span()]);
        }
    }

    /// Marks a `->` at the current position, colored like `=>`.
    fn mark_arrow(&self) {
        if self.peek_op("->") {
            self.mark_op("=>");
        }
    }

    /// Marks the operator at the current position (`->` or `|`) like `like`.
    fn mark_op(&self, like: &'static str) {
        if let Some(Tok::Op(o)) = self.peek() {
            self.mark(like, o.spans.clone());
        }
    }

    /// Parses for the formatter: syntax the macro desugars while parsing
    /// (pipes, `begin .. end`, labels, toplevel phrases) is kept as written.
    /// The result is not meant for code generation.
    pub fn set_surface(&mut self) {
        self.surface = true;
    }

    /// Declares names bound outside the parsed code (Rust locals).
    pub fn bind_names(&mut self, names: impl IntoIterator<Item = String>) {
        self.scope.extend(names);
    }

    fn sub(&self, g: &Group) -> Parser {
        self.sub_toks(lex(g.stream()), g.span_close())
    }

    fn sub_toks(&self, toks: Vec<Tok>, end: Span) -> Parser {
        Parser {
            toks,
            pos: 0,
            end,
            comma_stop: false,
            scope: self.scope.clone(),
            surface: self.surface,
            unmapped: self.unmapped.clone(),
            recover: self.recover,
            recovered: self.recovered.clone(),
            marks: self.marks.clone(),
            touch: RefCell::default(),
            open_holes: self.open_holes.clone(),
            levels: Vec::new(),
        }
    }

    // ----- token helpers -------------------------------------------------

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn peek_at(&self, n: usize) -> Option<&Tok> {
        self.toks.get(self.pos + n)
    }

    fn bump(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    /// The end of the token before the current one.
    /// Whether token `k` is written right after token `k - 1`, with no
    /// space ([`adjacent`]; `None` when unknown). Answers are kept: under
    /// rust-analyzer, each formats two spans to find their positions.
    fn touches(&self, k: usize) -> Option<bool> {
        if k == 0 || k >= self.toks.len() {
            return None;
        }
        let mut cache = self.touch.borrow_mut();
        if cache.len() < self.toks.len() {
            cache.resize(self.toks.len(), 0);
        }
        match cache[k] {
            1 => return None,
            2 => return Some(false),
            3 => return Some(true),
            _ => {}
        }
        let r = adjacent(tok_end(&self.toks[k - 1]), self.toks[k].span());
        cache[k] = match r {
            None => 1,
            Some(false) => 2,
            Some(true) => 3,
        };
        r
    }

    /// Whether the `try` here is OCaml's `try e with ..`: a `with` follows it
    /// (before `;;`) that no `match` after it takes.
    fn try_with_follows(&self) -> bool {
        let mut depth = 0usize;
        for t in &self.toks[self.pos + 1..] {
            match t {
                Tok::Ident(i) if i == "match" || i == "try" => depth += 1,
                Tok::Ident(i) if i == "with" => match depth.checked_sub(1) {
                    Some(d) => depth = d,
                    None => return true,
                },
                Tok::Op(o) if o.text == ";;" => return false,
                _ => {}
            }
        }
        false
    }

    /// Whether the operator at `k` is written with no space on either side
    /// (`Vec::new`, not `x :: xs`).
    fn tight_at(&self, k: usize) -> Option<bool> {
        let l = self.touches(k)?;
        let r = self.touches(k + 1)?;
        Some(l && r)
    }

    /// Token `k` was split in two: the new second half, at `k + 1`, touches
    /// the first; the rest keep their answers.
    fn split_token(&mut self, k: usize, a: Op, b: Op) {
        self.toks[k] = Tok::Op(a);
        self.toks.insert(k + 1, Tok::Op(b));
        let mut cache = self.touch.borrow_mut();
        if cache.len() > k + 1 {
            cache.insert(k + 1, 0);
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }

    fn span(&self) -> Span {
        self.peek().map(|t| t.span()).unwrap_or(self.end)
    }

    fn err<T>(&self, msg: impl Into<String>) -> R<T> {
        Err(Error::new(self.span(), msg))
    }

    fn peek_kw(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(i)) if i == kw)
    }

    fn eat_kw(&mut self, kw: &str) -> bool {
        if self.peek_kw(kw) {
            self.mark_keyword();
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Expects a keyword; with recovery on, code that ends before it (`if c`
    /// with no `then` yet) is unfinished rather than wrong.
    fn expect_kw(&mut self, kw: &str) -> R<Span> {
        let sp = self.span();
        if self.eat_kw(kw) {
            Ok(sp)
        } else if self.recover && self.at_end() {
            self.missing(Error::new(sp, format!("expected `{}`", kw)))?;
            Ok(sp)
        } else {
            self.err(format!("expected `{}`", kw))
        }
    }

    fn peek_op(&self, text: &str) -> bool {
        matches!(self.peek(), Some(Tok::Op(o)) if o.text == text)
    }

    fn peek_op_starts(&self, prefix: &str) -> bool {
        matches!(self.peek(), Some(Tok::Op(o)) if o.text.starts_with(prefix))
    }

    fn eat_op(&mut self, text: &str) -> bool {
        if self.peek_op(text) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Expects an operator, recovering at the end like [`Parser::expect_kw`].
    fn expect_op(&mut self, text: &str) -> R<Span> {
        let sp = self.span();
        if self.eat_op(text) {
            Ok(sp)
        } else if self.recover && self.at_end() {
            self.missing(Error::new(sp, format!("expected `{}`", text)))?;
            Ok(sp)
        } else {
            self.err(format!("expected `{}`", text))
        }
    }

    /// If the current operator starts with `prefix`, split it so that the
    /// current token is exactly `prefix` (Rust glues `?.` and `::<` together).
    fn split_op_prefix(&mut self, prefix: &str) -> bool {
        let Some(Tok::Op(o)) = self.peek() else {
            return false;
        };
        if o.text == prefix {
            return true;
        }
        if !o.text.starts_with(prefix) {
            return false;
        }
        let (a, b) = o.split(prefix.chars().count());
        self.split_token(self.pos, a, b);
        true
    }

    /// `xs |>.iter()`, `t |>.0`, `.iter >>.count`: a section written with no
    /// space after `|>`, `>>`, `<<` or `@@` is lexed into the operator
    /// (`|>.`), in OCaml too. Unless that operator is defined (as `x += e`
    /// is an error unless `( += )` is), it is split into the operator and
    /// the section's `.`, when a name or number follows the `.` (not for
    /// float operators such as `+.`). While recovering, `xs |>.` with no
    /// name yet is split too, so that it completes like `xs |> .`.
    fn split_glued_section(&mut self) {
        let Some(Tok::Op(o)) = self.peek() else {
            return;
        };
        if !GLUED_SECTION_OPS.contains(&o.text.as_str()) || self.in_scope(&op_key(o)) {
            return;
        }
        let next = self.pos + 1;
        let name = match self.toks.get(next) {
            Some(Tok::Ident(i)) => !is_keyword(&word(i)),
            Some(Tok::Lit(l)) => l.to_string().starts_with(|c: char| c.is_ascii_digit()),
            _ => false,
        };
        let spaced = self.touches(next) == Some(false);
        // Unfinished: nothing after the `.` yet, or the name still to come
        // before a space (as in `xs |> .‸ ys`).
        let unfinished = self.recover && (next >= self.toks.len() || spaced);
        if (name && !spaced) || unfinished {
            let (a, b) = o.split(2);
            self.split_token(self.pos, a, b);
        }
    }

    fn expect_end(&self) -> R<()> {
        match self.peek() {
            None => Ok(()),
            Some(t) => Err(Error::new(t.span(), "unexpected token")),
        }
    }

    fn expect_ident(&mut self) -> R<Ident> {
        match self.peek() {
            Some(Tok::Ident(i)) if is_rust_keyword(i) => Err(rust_keyword_error(i)),
            Some(Tok::Ident(i)) if !is_keyword(&word(i)) => {
                let i = i.clone();
                self.pos += 1;
                Ok(i)
            }
            _ => self.err("expected an identifier"),
        }
    }

    fn in_scope(&self, name: &str) -> bool {
        self.scope.iter().any(|s| s == name)
    }

    fn push_pat_names<'a>(&mut self, pats: impl IntoIterator<Item = &'a Pat>) -> usize {
        let len = self.scope.len();
        for p in pats {
            let mut names = Vec::new();
            p.names(&mut names);
            self.scope.extend(names.iter().map(ident_str));
        }
        len
    }

    fn push_binding_names(&mut self, bs: &[Binding]) -> usize {
        let len = self.scope.len();
        for b in bs {
            match &b.kind {
                BindKind::Func {
                    name: Name::Ident(i),
                    ..
                } => self.scope.push(ident_str(i)),
                BindKind::Func {
                    name: Name::Op(o), ..
                } => self.scope.push(op_key(o)),
                BindKind::Value { pat, .. } => {
                    self.push_pat_names([pat]);
                }
            }
        }
        len
    }

    /// An `external` operator is in scope after its declaration. (Only
    /// operators: an `external` value is no local, for [`Parser::in_scope`].)
    fn push_external_name(&mut self, name: &Name) -> usize {
        let len = self.scope.len();
        if let Name::Op(o) = name {
            self.scope.push(op_key(o));
        }
        len
    }

    // ----- nested cases --------------------------------------------------

    /// Parses `f` behind a barrier: a case after it cannot leave for an
    /// enclosing `match` (see [`crate::arms`]), as the text after `f`'s
    /// expression must be a keyword (`in`, `then`, `with`, `do`, ..).
    fn barrier<T>(&mut self, f: impl FnOnce(&mut Parser) -> R<T>) -> R<T> {
        self.levels.push(None);
        let r = f(self);
        self.levels.pop();
        r
    }

    /// Whether the cases of a `match` or `function` are being parsed here,
    /// with no barrier in between: a `|` may then start the next case.
    fn in_cases(&self) -> bool {
        matches!(self.levels.last(), Some(Some(_)))
    }

    /// At the `|` before a case: whether that case belongs to an enclosing
    /// `match` rather than the innermost one (see [`crate::arms`]). The
    /// pattern is parsed ahead and the position restored.
    fn case_belongs_outside(&mut self) -> bool {
        // Only an enclosing `match` with no barrier in between can take it.
        if !matches!(self.levels.iter().rev().nth(1), Some(Some(_))) {
            return false;
        }
        let save = self.pos;
        let recovered_len = self.recovered.borrow().len();
        let marks_len = self.marks.borrow().len();
        let unmapped_len = self.unmapped.borrow().len();
        self.pos += 1;
        let pat = self.parse_pattern();
        self.pos = save;
        self.recovered.borrow_mut().truncate(recovered_len);
        self.marks.borrow_mut().truncate(marks_len);
        self.unmapped.borrow_mut().truncate(unmapped_len);
        pat.is_ok_and(|p| arms::belongs_outside(&self.levels, &p))
    }

    // ----- top level -----------------------------------------------------

    /// The body of `ml! { .. }`: a sequence of OCaml toplevel phrases
    /// (`let` without `in`, `external`, `use`, `;;`-separated expressions).
    pub fn parse_toplevel(&mut self) -> R<Expr> {
        while self.eat_op(";;") {}
        if self.at_end() {
            return Ok(Expr::Unit(self.end));
        }
        let span = self.span();
        if self.eat_kw("external") {
            let (name, ty, rust) = self.parse_external()?;
            let len = self.push_external_name(&name);
            let body = if self.eat_kw("in") {
                self.parse_expr()?
            } else {
                self.parse_toplevel()?
            };
            self.scope.truncate(len);
            return Ok(Expr::External {
                name,
                ty,
                rust,
                body: Box::new(body),
                span,
            });
        }
        if self.peek_kw("use") {
            let (tree, semi) = self.parse_use()?;
            let body = self.parse_toplevel()?;
            return Ok(Expr::Use {
                tree,
                body: Box::new(body),
                span,
                semi,
            });
        }
        if self.peek_kw("let") {
            let save = self.pos;
            let scope_len = self.scope.len();
            let recovered_len = self.recovered.borrow().len();
            let marks_len = self.marks.borrow().len();
            self.pos += 1;
            let (rec_, bindings) = self.parse_bindings()?;
            if self.peek_kw("in") {
                // An ordinary `let .. in` expression: reparse it as one.
                self.pos = save;
                self.scope.truncate(scope_len);
                self.recovered.borrow_mut().truncate(recovered_len);
                self.marks.borrow_mut().truncate(marks_len);
            } else {
                self.check_rust_let(&bindings)?;
                let len = self.push_binding_names(&bindings);
                let body = self.parse_toplevel()?;
                self.scope.truncate(len);
                return Ok(Expr::Let {
                    rec_,
                    bindings,
                    body: Box::new(body),
                    span,
                });
            }
        }
        let e = self.parse_expr()?;
        if self.eat_op(";;") {
            while self.eat_op(";;") {}
            if self.at_end() {
                return Ok(e);
            }
            let rest = self.parse_toplevel()?;
            return Ok(Expr::Seq(Box::new(e), Box::new(rest)));
        }
        match self.peek() {
            None => Ok(e),
            Some(t) => Err(Error::new(
                t.span(),
                "unexpected token (missing `;;` or `in`?)",
            )),
        }
    }

    /// A toplevel `let x = e; more` without `in` is Rust's `let x = e;`: in
    /// OCaml the `;` makes `more` part of the value. (`let () = a; b`,
    /// binding no name, is the OCaml idiom it looks like.)
    fn check_rust_let(&self, bindings: &[Binding]) -> R<()> {
        for b in bindings {
            if let BindKind::Value { pat, .. } = &b.kind
                && is_seq(&b.value)
            {
                let mut names = Vec::new();
                pat.names(&mut names);
                if !names.is_empty() {
                    self.recoverable(rust_let_error(b))?;
                }
            }
        }
        Ok(())
    }

    /// `external name : type [= "path"]`, after the keyword.
    fn parse_external(&mut self) -> R<External> {
        let name = self.parse_name()?;
        self.expect_op(":")?;
        let ty = self.parse_type()?;
        let rust = if self.eat_op("=") {
            match self.bump() {
                Some(Tok::Lit(l)) => Some((unquote(&l.to_string()), l.span())),
                _ => return self.err("expected a string naming the Rust function"),
            }
        } else {
            None
        };
        Ok((name, ty, rust))
    }

    /// The block as toplevel phrases, for a surface parse.
    pub fn parse_phrases(&mut self) -> R<Vec<Phrase>> {
        let mut out = Vec::new();
        loop {
            while self.eat_op(";;") {}
            if self.at_end() {
                return Ok(out);
            }
            let span = self.span();
            if self.eat_kw("external") {
                let (name, ty, rust) = self.parse_external()?;
                self.push_external_name(&name);
                out.push(Phrase::External {
                    name,
                    ty,
                    rust,
                    span,
                });
                // `external .. in e` is the same as `external .. ;; e`.
                if self.eat_kw("in") {
                    out.push(Phrase::Expr(self.parse_expr()?));
                }
                continue;
            }
            if self.peek_kw("use") {
                let (tree, _) = self.parse_use()?;
                out.push(Phrase::Use { tree, span });
                continue;
            }
            if self.peek_kw("let") {
                let save = self.pos;
                let scope_len = self.scope.len();
                let recovered_len = self.recovered.borrow().len();
                let marks_len = self.marks.borrow().len();
                self.pos += 1;
                let (rec_, bindings) = self.parse_bindings()?;
                if self.peek_kw("in") {
                    self.pos = save;
                    self.scope.truncate(scope_len);
                    self.recovered.borrow_mut().truncate(recovered_len);
                    self.marks.borrow_mut().truncate(marks_len);
                } else {
                    self.check_rust_let(&bindings)?;
                    self.push_binding_names(&bindings);
                    out.push(Phrase::Let {
                        rec_,
                        bindings,
                        span,
                    });
                    continue;
                }
            }
            out.push(Phrase::Expr(self.parse_expr()?));
            if !self.at_end() && !self.peek_op(";;") {
                return self.err("unexpected token (missing `;;` or `in`?)");
            }
        }
    }

    /// `use tree;`, from the `use`: the use tree as Rust tokens (checked by
    /// rustc), and the span of the `;`.
    fn parse_use(&mut self) -> R<(TokenStream, Span)> {
        let kw = self.span();
        self.pos += 1;
        let start = self.pos;
        while let Some(t) = self.peek() {
            if matches!(t, Tok::Op(o) if o.text.starts_with(';')) {
                break;
            }
            let ok = use_tree_token(t);
            if !ok && self.pos == start {
                return self.err("expected a Rust path after `use` (`use std::cmp::max;`)");
            }
            if !ok {
                return self.err("expected `;` after the `use` (`use std::cmp::max;`)");
            }
            self.pos += 1;
        }
        let mut tree = to_stream(&self.toks[start..self.pos]);
        // Unfinished: `use`, `use std::` (while a name is being typed).
        let unfinished = match self.toks[start..self.pos].last() {
            None => Some(kw),
            Some(Tok::Op(o)) if o.text.ends_with("::") => Some(o.span()),
            _ => None,
        };
        if let Some(at) = unfinished {
            let name = self.missing(Error::new(at, "expected a path after `use`"))?;
            tree.extend(quote::quote!(#name));
        }
        let semi = self.span();
        if self.peek_op(";") || self.peek_op(";;") {
            self.pos += 1;
        } else if self.recover && self.at_end() {
            self.missing(Error::new(semi, "expected `;` after the `use`"))?;
        } else {
            return self.err("expected `;` after the `use`");
        }
        Ok((tree, semi))
    }

    fn parse_name(&mut self) -> R<Name> {
        if let Some(Tok::Group(g)) = self.peek()
            && g.delimiter() == Delimiter::Parenthesis
            && let Some(op) = self.sub(g).single_op()
        {
            if let Some(e) = bang_op_error(&op) {
                return Err(e);
            }
            self.pos += 1;
            return Ok(Name::Op(op));
        }
        Ok(Name::Ident(self.expect_ident()?))
    }

    /// `( op )` with nothing else inside.
    fn single_op(&self) -> Option<Op> {
        if self.toks.len() != 1 {
            return None;
        }
        match &self.toks[0] {
            Tok::Op(o) if !matches!(o.text.as_str(), "," | ";" | ";;" | "." | ":") => {
                Some(o.clone())
            }
            Tok::Ident(i) if KEYWORD_OPS.contains(&word(i).as_str()) => {
                Some(Op::new(&word(i), i.span()))
            }
            _ => None,
        }
    }

    // ----- let bindings --------------------------------------------------

    fn parse_bindings(&mut self) -> R<(bool, Vec<Binding>)> {
        let rec_ = self.eat_kw("rec");
        let len = self.scope.len();
        let mut bindings = Vec::new();
        loop {
            bindings.push(self.barrier(|p| p.parse_binding(rec_))?);
            if !self.eat_kw("and") {
                break;
            }
        }
        self.scope.truncate(len);
        Ok((rec_, bindings))
    }

    fn can_start_param_at(&self, n: usize) -> bool {
        match self.peek_at(n) {
            Some(Tok::Ident(i)) => {
                let s = word(i);
                !is_keyword(&s) && (s == "_" || !is_capitalized(i))
            }
            Some(Tok::Group(_)) | Some(Tok::Lit(_)) => true,
            Some(Tok::Op(o)) => o.text == "~",
            _ => false,
        }
    }

    /// Parameters; a labeled one (`~x`, `~x:p`, `~(x : t)`) is a
    /// [`Pat::Label`], which [`Parser::split_labels`] takes apart.
    fn parse_params(&mut self) -> R<Vec<Pat>> {
        let mut params = Vec::new();
        while self.can_start_param_at(0) {
            if self.peek_op("~") {
                let span = self.span();
                self.pos += 1;
                let (label, pat) = if matches!(self.peek(), Some(Tok::Group(_))) {
                    (None, Some(self.parse_pat_atom()?))
                } else {
                    let label = self.expect_ident()?;
                    let pat = if self.eat_op(":") {
                        Some(self.parse_pat_atom()?)
                    } else {
                        None
                    };
                    (Some(label), pat)
                };
                params.push(Pat::Label {
                    label,
                    pat: pat.map(Box::new),
                    span,
                });
                continue;
            }
            params.push(self.parse_pat_atom()?);
        }
        Ok(params)
    }

    /// The patterns of `params` without their labels, and the label of
    /// each: `~x` is `x` labeled `x`, `~x:p` is `p` labeled `x`, and
    /// `~(x : t)` is `(x : t)` labeled `x`. A surface parse keeps the
    /// labels in the patterns, for the formatter.
    fn split_labels(&self, params: Vec<Pat>) -> R<(Vec<Pat>, Vec<Option<Ident>>)> {
        if self.surface {
            return Ok((params, Vec::new()));
        }
        let mut pats = Vec::new();
        let mut labels = Vec::new();
        for p in params {
            let Pat::Label { label, pat, span } = p else {
                pats.push(p);
                labels.push(None);
                continue;
            };
            let (label, pat) = match (label, pat) {
                (Some(l), Some(p)) => (l, *p),
                (Some(l), None) => (l.clone(), Pat::Var(l)),
                (None, Some(p)) => match p.strip() {
                    Pat::Var(l) => (l.clone(), *p),
                    _ => {
                        return Err(Error::new(
                            span,
                            "a labeled parameter is `~x`, `~x:p` or `~(x : t)`",
                        ));
                    }
                },
                (None, None) => unreachable!("`~` without a label or a pattern"),
            };
            pats.push(pat);
            labels.push(Some(label));
        }
        Ok((pats, labels))
    }

    fn parse_binding(&mut self, rec_: bool) -> R<Binding> {
        let span = self.span();
        // let ( +! ) a b = ..
        if let Some(Tok::Group(g)) = self.peek()
            && g.delimiter() == Delimiter::Parenthesis
            && let Some(op) = self.sub(g).single_op()
        {
            if let Some(e) = bang_op_error(&op) {
                return Err(e);
            }
            self.pos += 1;
            let (params, labels) = self.parse_params().and_then(|ps| self.split_labels(ps))?;
            let ty = if self.eat_op(":") {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect_op("=")?;
            let len = self.push_pat_names(&params);
            let value = self.parse_expr()?;
            self.scope.truncate(len);
            let kind = BindKind::Func {
                name: Name::Op(op),
                params,
                labels,
                mutable: false,
            };
            return Ok(Binding {
                kind,
                ty,
                value,
                span,
            });
        }
        // let f x y = .., or let mut f x = .. (Rust extension)
        let m = usize::from(self.peek_kw("mut"));
        if let Some(Tok::Ident(i)) = self.peek_at(m) {
            let s = word(i);
            if !is_keyword(&s) && s != "_" && !is_capitalized(i) && self.can_start_param_at(m + 1) {
                if is_rust_keyword(i) {
                    return Err(rust_keyword_error(i));
                }
                if m == 1 && rec_ {
                    return Err(Error::new(
                        self.span(),
                        "a `let rec` function is a Rust `fn`, which cannot be `mut` \
                         (and cannot capture anything to mutate)",
                    ));
                }
                let name = i.clone();
                self.pos += m + 1;
                if rec_ {
                    self.scope.push(s);
                }
                let (params, labels) = self.parse_params().and_then(|ps| self.split_labels(ps))?;
                let ty = if self.eat_op(":") {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect_op("=")?;
                let len = self.push_pat_names(&params);
                let value = self.parse_expr()?;
                self.scope.truncate(len);
                let kind = BindKind::Func {
                    name: Name::Ident(name),
                    params,
                    labels,
                    mutable: m == 1,
                };
                return Ok(Binding {
                    kind,
                    ty,
                    value,
                    span,
                });
            }
        }
        // let mut x = .. (Rust extension)
        if self.peek_kw("mut") && matches!(self.peek_at(1), Some(Tok::Ident(_))) {
            self.pos += 1;
            let name = self.expect_ident()?;
            let ty = if self.eat_op(":") {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect_op("=")?;
            let value = self.parse_expr()?;
            let kind = BindKind::Value {
                pat: Pat::Var(name),
                mutable: true,
            };
            return Ok(Binding {
                kind,
                ty,
                value,
                span,
            });
        }
        // let pattern = ..
        let pat = self.parse_pattern()?;
        if rec_ {
            self.push_pat_names([&pat]);
        }
        let ty = if self.eat_op(":") {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_op("=")?;
        let value = self.parse_expr()?;
        Ok(Binding {
            kind: BindKind::Value {
                pat,
                mutable: false,
            },
            ty,
            value,
            span,
        })
    }

    // ----- expressions ---------------------------------------------------

    pub fn parse_expr(&mut self) -> R<Expr> {
        // `use tree; e`: Rust's `use`, for the rest of the expression.
        if self.peek_kw("use") {
            let span = self.span();
            let (tree, semi) = self.parse_use()?;
            let body = self.parse_expr()?;
            return Ok(Expr::Use {
                tree,
                body: Box::new(body),
                span,
                semi,
            });
        }
        let e = self.parse_expr_noseq()?;
        if self.peek_op(";") {
            let semi = self.span();
            self.pos += 1;
            // `a; ; b` is not `a;; b`, and in `a; + b` the operand is missing:
            // OCaml rejects both (and the expression would end before them).
            if self.peek_op(";") {
                self.recoverable(Error::new(
                    self.span(),
                    "`; ;` is two `;`s: write `;;` (with no space) to end a toplevel phrase, \
                     or one `;`",
                ))?;
            } else if !self.can_start_expr()
                && (self.peek_binop().is_some()
                    || self.peek_op(":=")
                    || self.peek_op("<-")
                    || self.peek_op(","))
            {
                self.recoverable(Error::new(
                    self.span(),
                    "expected an expression after `;`, before this",
                ))?;
            }
            if !self.can_start_expr() {
                if self.recover {
                    // The next item may still be coming: give `a;` the shape
                    // `a; b` will have (no error, it is valid as it is).
                    let rest = Expr::Unit(semi);
                    return Ok(self.close_holes(Expr::Seq(Box::new(e), Box::new(rest))));
                }
                return Ok(e);
            }
            let rest = self.parse_expr()?;
            return Ok(self.close_holes(Expr::Seq(Box::new(e), Box::new(rest))));
        }
        Ok(e)
    }

    /// Where a full expression ends (parentheses, list items, `let` values
    /// and bodies, branches, operands of `|>` `@@` `>>` `<<`, ..), the holes
    /// in it make it a [`Expr::Section`]. A hole alone, perhaps annotated,
    /// stays a hole: it belongs to the expression around it.
    fn close_holes(&self, mut e: Expr) -> Expr {
        if e.is_bare_hole() || self.open_holes.get() == 0 {
            return e;
        }
        let mut holes = Vec::new();
        e.visit_holes(&mut |_, h| holes.push(h));
        let Some(first) = holes.first() else {
            return e;
        };
        self.open_holes
            .set(self.open_holes.get().saturating_sub(holes.len()));
        let span = first.span;
        // `_` takes the lowest number that is neither written nor taken.
        let written: Vec<usize> = holes.iter().filter_map(|h| h.written).collect();
        let mut taken = Vec::new();
        let mut next = 0;
        let mut assigned = Vec::new();
        for h in &holes {
            let i = match h.written {
                Some(n) => n,
                None => {
                    while written.contains(&next) || taken.contains(&next) {
                        next += 1;
                    }
                    taken.push(next);
                    next
                }
            };
            assigned.push(i);
        }
        let arity = assigned.iter().max().unwrap() + 1;
        let mut params = vec![None; arity];
        for (h, i) in holes.iter().zip(&assigned) {
            if params[*i].is_none() {
                params[*i] = h.ty.clone();
            }
        }
        let mut k = 0;
        e.visit_holes(&mut |hole, _| {
            hole.set_hole_index(assigned[k]);
            k += 1;
        });
        Expr::Section {
            body: Box::new(e),
            params,
            span,
        }
    }

    fn can_start_expr(&self) -> bool {
        self.can_start_expr_at(0)
    }

    fn can_start_expr_at(&self, n: usize) -> bool {
        match self.peek_at(n) {
            None | Some(Tok::TyVar(..)) => false,
            Some(Tok::Ident(i)) => {
                let s = word(i);
                !is_keyword(&s)
                    || matches!(
                        s.as_str(),
                        "true"
                            | "false"
                            | "begin"
                            | "let"
                            | "match"
                            | "fun"
                            | "function"
                            | "if"
                            | "while"
                            | "for"
                            | "assert"
                    )
            }
            Some(Tok::Lit(_)) | Some(Tok::Group(_)) => true,
            // A closure; among cases, `|` is the next case's (`return` and
            // `a;` end there), so a closure after them needs parentheses.
            Some(Tok::Op(o)) if closure_bar(&o.text) => o.text == "||" || !self.in_cases(),
            Some(Tok::Op(o)) => {
                matches!(o.text.as_str(), "-" | "-." | "::" | ".")
                    || o.text.chars().all(|c| matches!(c, '!' | '*' | '&'))
                    || range_op(&o.text).is_some()
            }
        }
    }

    /// Everything except `;` sequencing (the level of `if` branches and list items).
    fn parse_expr_noseq(&mut self) -> R<Expr> {
        let lhs = self.parse_tuple()?;
        // `:=` and `<-` become `=`, which keeps the first character's span.
        if let Some(Tok::Op(o)) = self.peek()
            && (o.text == ":=" || o.text == "<-")
        {
            self.unmapped(&o.spans[1..]);
        }
        let e = if self.peek_op(":=") {
            let sp = self.span();
            self.pos += 1;
            let rhs = self.parse_expr_noseq()?;
            Expr::RefAssign(Box::new(lhs), Box::new(rhs), sp)
        } else if self.peek_op("<-") {
            let sp = self.span();
            self.pos += 1;
            let rhs = self.parse_expr_noseq()?;
            Expr::Assign(Box::new(lhs), Box::new(rhs), sp)
        } else {
            lhs
        };
        Ok(self.close_holes(e))
    }

    fn parse_tuple(&mut self) -> R<Expr> {
        let first = self.parse_bin(OR)?;
        if self.comma_stop || !self.peek_op(",") {
            return Ok(first);
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            items.push(self.parse_bin(OR)?);
        }
        Ok(Expr::Tuple(items))
    }

    fn peek_binop(&self) -> Option<(u8, Assoc)> {
        match self.peek()? {
            Tok::Op(o) => binop_info(&o.text),
            Tok::Ident(i) => match word(i).as_str() {
                "mod" | "land" | "lor" | "lxor" => Some((MUL, Assoc::Left)),
                "lsl" | "lsr" | "asr" => Some((POW, Assoc::Right)),
                "or" => Some((OR, Assoc::Right)),
                _ => None,
            },
            _ => None,
        }
    }

    fn parse_bin(&mut self, min: u8) -> R<Expr> {
        let mut lhs = if min <= RANGE && self.peek_range() {
            self.parse_range(None)?
        } else {
            self.parse_unary()?
        };
        loop {
            if min <= RANGE && self.peek_range() {
                if matches!(lhs, Expr::Range { .. }) {
                    return self.err("ranges do not chain: put one in parentheses");
                }
                lhs = self.parse_range(Some(lhs))?;
                continue;
            }
            self.split_glued_section();
            let Some((lvl, assoc)) = self.peek_binop() else {
                break;
            };
            if lvl < min {
                break;
            }
            let op = match self.bump() {
                Some(Tok::Op(o)) => o,
                Some(Tok::Ident(i)) => Op::new(&word(&i), i.span()),
                _ => unreachable!(),
            };
            let rhs = self.parse_bin(if assoc == Assoc::Left { lvl + 1 } else { lvl })?;
            lhs = self.binop(op, lhs, rhs);
        }
        Ok(lhs)
    }

    /// Whether `..` or `..=` comes next (perhaps glued to what follows it,
    /// as in `..-1`).
    fn peek_range(&self) -> bool {
        matches!(self.peek(), Some(Tok::Op(o)) if range_op(&o.text).is_some())
    }

    /// `lo..hi`, from the `..` on; the end is optional (`lo..`, `..`) when
    /// what follows cannot start an expression.
    fn parse_range(&mut self, lo: Option<Expr>) -> R<Expr> {
        let Some(Tok::Op(o)) = self.peek() else {
            unreachable!()
        };
        let text = range_op(&o.text).unwrap();
        self.split_op_prefix(text);
        let Some(Tok::Op(op)) = self.bump() else {
            unreachable!()
        };
        let hi = if self.can_start_expr() && !self.peek_range() {
            Some(Box::new(self.parse_bin(RANGE + 1)?))
        } else if op.text == "..=" {
            let at = self.span();
            Some(Box::new(Expr::Var(self.missing(Error::new(
                at,
                "expected the end of the range after `..=`",
            ))?)))
        } else {
            None
        };
        Ok(Expr::Range {
            lo: lo.map(Box::new),
            hi,
            op,
        })
    }

    fn parse_unary(&mut self) -> R<Expr> {
        if let Some(Tok::Op(o)) = self.peek()
            && (o.text == "-" || o.text == "-.")
        {
            let o = o.clone();
            self.pos += 1;
            let e = self.parse_bin(POW)?;
            return Ok(Expr::Neg(Box::new(e), o));
        }
        if self.closure_at(0) {
            if !self.in_cases() || self.peek_op("||") {
                return self.parse_closure();
            }
            // Among cases, a `|` with no body before it may be the next
            // case's (`Some x -> ‸ | None -> 0`, being written): a closure
            // only if one parses, with no `->` or guard after it.
            if let Some(e) = self.closure_among_cases() {
                return Ok(e);
            }
        }
        if let Some(Tok::Ident(i)) = self.peek() {
            match word(i).as_str() {
                "let" => return self.parse_let_expr(),
                "fun" => return self.parse_fun(),
                "function" => {
                    let span = self.span();
                    self.mark_keyword();
                    self.pos += 1;
                    let arms = self.parse_arms()?;
                    return Ok(Expr::Function { arms, span });
                }
                "match" => {
                    let span = self.span();
                    self.pos += 1;
                    if !self.recover && self.block_before("with") {
                        return Err(Error::new(
                            span,
                            "this is Rust's `match x { p => e }`: write `match x with p -> e | ..`",
                        ));
                    }
                    let scrut = self.barrier(Parser::parse_expr)?;
                    self.expect_kw("with")?;
                    let arms = self.parse_arms()?;
                    return Ok(Expr::Match {
                        scrut: Box::new(scrut),
                        arms,
                        span,
                    });
                }
                "if" => return self.parse_if(),
                "while" => {
                    let span = self.span();
                    self.pos += 1;
                    let cond = self.barrier(Parser::parse_expr)?;
                    self.expect_kw("do")?;
                    let body = self.parse_loop_body()?;
                    return Ok(Expr::While {
                        cond: Box::new(cond),
                        body: Box::new(body),
                        span,
                    });
                }
                "for" => return self.parse_for(),
                "return" | "break" | "continue" => {
                    let kind = match word(i).as_str() {
                        "return" => Jump::Return,
                        "break" => Jump::Break,
                        _ => Jump::Continue,
                    };
                    let span = self.span();
                    self.pos += 1;
                    // The value extends like an `if` branch, and is left out
                    // when nothing that can start one follows.
                    let value = if kind != Jump::Continue && self.can_start_expr() {
                        Some(Box::new(self.parse_expr_noseq()?))
                    } else {
                        None
                    };
                    return Ok(Expr::Jump { kind, value, span });
                }
                // Else `try` is a name (`r#try`, see `RESERVED`).
                "try" if self.try_with_follows() => {
                    return self.err(
                        "`try .. with` is not supported: Rust has no exceptions, use `Result` and `match`",
                    );
                }
                _ => {}
            }
        }
        self.parse_app()
    }

    fn parse_let_expr(&mut self) -> R<Expr> {
        let span = self.span();
        self.pos += 1;
        let (rec_, bindings) = self.parse_bindings()?;
        if !self.peek_kw("in")
            && !(self.recover && self.at_end())
            && let Some(b) = bindings.iter().find(|b| is_seq(&b.value))
        {
            return Err(rust_let_error(b));
        }
        self.expect_kw("in")?;
        let len = self.push_binding_names(&bindings);
        let body = self.parse_expr()?;
        self.scope.truncate(len);
        Ok(Expr::Let {
            rec_,
            bindings,
            body: Box::new(body),
            span,
        })
    }

    fn parse_fun(&mut self) -> R<Expr> {
        let span = self.span();
        self.mark_keyword();
        self.pos += 1;
        // `fun move x -> ..` (Rust extension); `fun move -> ..` is a parameter named `move`.
        let is_move = self.peek_kw("move") && self.can_start_param_at(1);
        if is_move {
            self.pos += 1;
        }
        let (params, labels) = self.parse_params().and_then(|ps| self.split_labels(ps))?;
        if params.is_empty() {
            return self.err("expected a parameter after `fun`");
        }
        self.mark_arrow();
        self.expect_op("->")?;
        let len = self.push_pat_names(&params);
        let body = self.parse_expr()?;
        self.scope.truncate(len);
        Ok(Expr::Fun {
            params,
            labels,
            body: Box::new(body),
            span,
            is_move,
            bars: None,
        })
    }

    /// Whether a Rust-style closure starts at `peek_at(n)`: `|x| e`, `|| e`,
    /// `move |x| e`.
    fn closure_at(&self, n: usize) -> bool {
        match self.peek_at(n) {
            Some(Tok::Op(o)) => closure_bar(&o.text),
            Some(Tok::Ident(i)) if i == "move" => {
                matches!(self.peek_at(n + 1), Some(Tok::Op(o)) if closure_bar(&o.text))
            }
            _ => false,
        }
    }

    /// `|p, q: t| body`, `|| body` or `move |p| body`: the same closure as
    /// `fun p (q : t) -> body`, its body reaching as far.
    fn parse_closure(&mut self) -> R<Expr> {
        let span = self.span();
        let is_move = self.peek_kw("move");
        if is_move {
            self.pos += 1;
        }
        let open = self.span();
        let mut params = Vec::new();
        let close = if self.peek_op("||") {
            self.pos += 1;
            open
        } else {
            self.split_op_prefix("|");
            self.pos += 1;
            while !self.peek_op_starts("|") && !self.at_end() {
                let p = self.parse_pat_cons()?;
                let p = if self.eat_op(":") {
                    Pat::Constraint(Box::new(p), self.parse_type()?)
                } else {
                    p
                };
                params.push(p);
                if !self.eat_op(",") {
                    break;
                }
            }
            let close = self.span();
            if self.split_op_prefix("|") {
                self.pos += 1;
            } else if self.recover && self.at_end() {
                self.missing(Error::new(close, "expected `|`"))?;
            } else {
                return self.err("expected `,` or the `|` that closes the closure's parameters");
            }
            close
        };
        if params.is_empty() {
            params.push(Pat::Unit(open));
        }
        if self.peek_op("->") {
            return self.err(
                "a closure's body comes right after its parameters: `|x| e` (or `fun x -> e`)",
            );
        }
        let len = self.push_pat_names(&params);
        let body = self.parse_expr()?;
        self.scope.truncate(len);
        Ok(Expr::Fun {
            labels: vec![None; params.len()],
            params,
            body: Box::new(body),
            span,
            is_move,
            bars: Some((open, close)),
        })
    }

    /// [`Parser::parse_closure`] where a `|` could start the next case: the
    /// closure, or `None` (and the position unchanged) if what follows is
    /// no closure or reads as a case (`| A | B -> e`).
    fn closure_among_cases(&mut self) -> Option<Expr> {
        let save = self.pos;
        let scope_len = self.scope.len();
        let recovered_len = self.recovered.borrow().len();
        let marks_len = self.marks.borrow().len();
        let unmapped_len = self.unmapped.borrow().len();
        let holes = self.open_holes.get();
        let parsed = self.parse_closure();
        let case_follows = self.peek_op("->") || self.peek_kw("when") || self.peek_kw("if");
        match parsed {
            Ok(e) if !case_follows => Some(e),
            _ => {
                self.pos = save;
                self.scope.truncate(scope_len);
                self.recovered.borrow_mut().truncate(recovered_len);
                self.marks.borrow_mut().truncate(marks_len);
                self.unmapped.borrow_mut().truncate(unmapped_len);
                self.open_holes.set(holes);
                None
            }
        }
    }

    fn parse_arms(&mut self) -> R<Vec<Arm>> {
        self.levels.push(Some(arms::Level::default()));
        let arms = self.parse_cases();
        self.levels.pop();
        arms
    }

    /// The cases of a `match` or `function`, up to the first that belongs
    /// to an enclosing one (see [`crate::arms`]).
    fn parse_cases(&mut self) -> R<Vec<Arm>> {
        if self.peek_op("|") {
            self.mark_op("|");
            self.pos += 1;
        }
        let mut arms = Vec::new();
        loop {
            let pat = self.parse_pattern()?;
            if let Some(Some(level)) = self.levels.last_mut() {
                level.add(&pat);
            }
            let len = self.push_pat_names([&pat]);
            // `when`, or Rust's `if`.
            let mut guard_if = None;
            let guard = if self.eat_kw("when") {
                Some(self.barrier(Parser::parse_expr)?)
            } else if self.peek_kw("if") {
                guard_if = Some(self.span());
                self.pos += 1;
                Some(self.barrier(Parser::parse_expr)?)
            } else {
                None
            };
            if guard.is_none()
                && let Some(Some(level)) = self.levels.last_mut()
            {
                level.add_unguarded(&pat);
            }
            let mut arrow = Vec::new();
            if self.recover && !self.peek_op("->") {
                // Unfinished: `match o with Some x` (the body is missing too).
                self.missing(Error::new(self.span(), "expected `->`"))?;
            } else {
                if let Some(Tok::Op(o)) = self.peek() {
                    arrow = o.spans.clone();
                }
                self.expect_op("->")?;
            }
            let body = self.parse_expr()?;
            self.scope.truncate(len);
            arms.push(Arm {
                pat,
                guard,
                guard_if,
                body,
                arrow,
            });
            if !self.peek_op("|") || self.case_belongs_outside() {
                break;
            }
            self.mark_op("|");
            self.pos += 1;
        }
        Ok(arms)
    }

    fn parse_if(&mut self) -> R<Expr> {
        let span = self.span();
        self.pos += 1;
        if !self.recover && self.peek_kw("let") && self.block_before("in") {
            return Err(Error::new(
                self.span(),
                "this is Rust's `if let`: write `match e with p -> .. | _ -> ..`",
            ));
        }
        let cond = self.barrier(Parser::parse_expr)?;
        self.expect_kw("then")?;
        let then = self.parse_expr_noseq()?;
        let els = if self.eat_kw("else") {
            Some(Box::new(self.parse_expr_noseq()?))
        } else {
            None
        };
        Ok(Expr::If {
            cond: Box::new(cond),
            then: Box::new(then),
            els,
            span,
        })
    }

    /// Whether, before the keyword `kw` (`with` after `match`, `in` after
    /// `if let`), a Rust block `{ .. }` comes, or `then`, or the end: the
    /// shape of Rust's `match x { .. }` and `if let p = e { .. }`, which
    /// is no OCaml.
    fn block_before(&self, kw: &str) -> bool {
        for k in self.pos..self.toks.len() {
            match &self.toks[k] {
                Tok::Ident(i) if i == kw => return false,
                Tok::Ident(i) if i == "then" => return kw == "in",
                Tok::Group(_) if self.is_block_brace(k) => return true,
                _ => {}
            }
        }
        kw == "in"
    }

    /// Whether the token at `k` is a `{ .. }` that is not a record
    /// (`Point { .. }`, `{ p with .. }`), a macro's (`m! { .. }`) or a
    /// block of `unsafe` or `async`: a Rust block.
    fn is_block_brace(&self, k: usize) -> bool {
        let Tok::Group(g) = &self.toks[k] else {
            return false;
        };
        if g.delimiter() != Delimiter::Brace {
            return false;
        }
        let after_name = match k.checked_sub(1).map(|j| &self.toks[j]) {
            Some(Tok::Ident(i)) => {
                is_capitalized(i) || matches!(i.to_string().as_str(), "unsafe" | "async" | "move")
            }
            Some(Tok::Op(o)) => o.text == "!" || o.text.ends_with('>'),
            _ => false,
        };
        !after_name
            && !lex(g.stream())
                .iter()
                .any(|t| matches!(t, Tok::Ident(i) if i == "with"))
    }

    fn parse_loop_body(&mut self) -> R<Expr> {
        let sp = self.span();
        if self.eat_kw("done") {
            return Ok(Expr::Unit(sp));
        }
        let body = self.barrier(Parser::parse_expr)?;
        self.expect_kw("done")?;
        Ok(body)
    }

    fn parse_for(&mut self) -> R<Expr> {
        let span = self.span();
        self.pos += 1;
        let counting = matches!(self.peek(), Some(Tok::Ident(_)))
            && matches!(self.peek_at(1), Some(Tok::Op(o)) if o.text == "=");
        if !counting {
            // for pat in iter do body done
            let pat = self.parse_pattern()?;
            self.expect_kw("in")?;
            let iter = self.barrier(Parser::parse_expr)?;
            self.expect_kw("do")?;
            let len = self.push_pat_names([&pat]);
            let body = self.parse_loop_body()?;
            self.scope.truncate(len);
            return Ok(Expr::ForIn {
                pat,
                iter: Box::new(iter),
                body: Box::new(body),
                span,
            });
        }
        let var = self.expect_ident()?;
        self.expect_op("=")?;
        let from = self.barrier(Parser::parse_expr)?;
        let dir = if self.eat_kw("to") {
            ForDir::To
        } else if self.eat_kw("downto") {
            ForDir::Downto
        } else if self.eat_kw("til") {
            ForDir::Til
        } else {
            return self.err("expected `to`, `downto` or `til`");
        };
        let to = self.barrier(Parser::parse_expr)?;
        self.expect_kw("do")?;
        self.scope.push(ident_str(&var));
        let body = self.parse_loop_body()?;
        self.scope.pop();
        Ok(Expr::For {
            var,
            from: Box::new(from),
            to: Box::new(to),
            dir,
            body: Box::new(body),
            span,
        })
    }

    fn can_start_arg(&self) -> bool {
        match self.peek() {
            // `use` starts the next phrase (`let x = 1` then `use m;`).
            Some(Tok::Ident(i)) if i == "use" => false,
            Some(Tok::Ident(i)) => {
                let s = word(i);
                !is_keyword(&s)
                    || s == "true"
                    || s == "false"
                    || s == "begin"
                    || self.assert_macro_at(0)
            }
            Some(Tok::Lit(_)) | Some(Tok::Group(_)) => true,
            // `f &x` borrows; `a && b` stays the boolean operator. `!` is
            // never an infix operator, so a run starting with it (`!!x`,
            // `!*x`, `!&x`) always starts an argument.
            Some(Tok::Op(o)) => {
                matches!(o.text.as_str(), "~" | "&")
                    || (o.text.starts_with('!')
                        && o.text.chars().all(|c| matches!(c, '!' | '*' | '&')))
            }
            _ => false,
        }
    }

    fn parse_app(&mut self) -> R<Expr> {
        if self.peek_kw("assert") && !self.assert_macro_at(0) {
            let sp = self.span();
            self.pos += 1;
            let e = self.parse_postfix()?;
            return Ok(Expr::Assert(Box::new(e), sp));
        }
        if self.peek_kw("lazy") {
            return self.err("`lazy` is not supported");
        }
        let head = self.parse_postfix()?;
        let mut args = Vec::new();
        while self.can_start_arg() {
            args.push(self.parse_arg()?);
        }
        // `f |x| e`: `|` cannot follow an application, but as the next case.
        if !self.in_cases() && !self.peek_op("||") && self.closure_at(0) {
            return Err(closure_paren_error(self.span()));
        }
        let mut e = if args.is_empty() {
            head
        } else {
            Expr::App(Box::new(head), args)
        };
        // `e as t` (Rust extension), binding tighter than every binary operator.
        while self.eat_kw("as") {
            let ty = self.parse_type_atom()?;
            e = Expr::Cast(Box::new(e), ty);
        }
        Ok(e)
    }

    fn parse_arg(&mut self) -> R<Expr> {
        if self.surface && self.peek_op("~") {
            let span = self.span();
            self.pos += 1;
            let (label, value) = if matches!(self.peek(), Some(Tok::Group(_))) {
                (None, Some(self.parse_postfix()?))
            } else {
                let label = self.expect_ident()?;
                let value = if self.eat_op(":") {
                    Some(self.parse_postfix()?)
                } else {
                    None
                };
                (Some(label), value)
            };
            return Ok(Expr::Surface(Box::new(Surface::Label {
                label,
                value,
                span,
            })));
        }
        if self.peek_op("~") {
            // `~x:e`, `~x` (`~x:x`) or `~(x : t)` (`~x:(x : t)`): given to the
            // callee's parameter labeled `x` (see `Cx::apply_labeled`).
            let span = self.span();
            self.pos += 1;
            let (label, value) = if matches!(self.peek(), Some(Tok::Group(_))) {
                let value = self.parse_postfix()?;
                let name = match value.strip() {
                    Expr::Var(l) => Some(l.clone()),
                    Expr::Constraint(inner, _) => match inner.strip() {
                        Expr::Var(l) => Some(l.clone()),
                        _ => None,
                    },
                    _ => None,
                };
                let Some(label) = name else {
                    return Err(Error::new(
                        span,
                        "a labeled argument is `~x:e`, `~x` or `~(x : t)`",
                    ));
                };
                (label, value)
            } else {
                let label = self.expect_ident()?;
                let value = if self.eat_op(":") {
                    self.parse_postfix()?
                } else {
                    Expr::Var(label.clone())
                };
                (label, value)
            };
            return Ok(Expr::Labeled {
                label,
                value: Box::new(value),
                span,
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> R<Expr> {
        let mut e = self.parse_prefix_atom()?;
        loop {
            // `Some(x)`, written against the constructor, is an atom like a
            // Rust call: in `Some(x).unwrap()` the method is the option's.
            // The argument keeps OCaml's rules (`Some(1, 2)` holds a pair).
            if let Some(Tok::Group(g)) = self.peek()
                && self.ctor_call_follows(&e, g)
            {
                let g = g.clone();
                self.pos += 1;
                let arg = self.parse_paren(&g)?;
                e = Expr::App(Box::new(e), vec![arg]);
                continue;
            }
            if let Some(Tok::Group(g)) = self.peek()
                && self.rust_call_follows(&e, g)
            {
                let g = g.clone();
                self.pos += 1;
                e = if g.delimiter() == Delimiter::Parenthesis {
                    let (args, trailing_comma) = self.parse_call_args(&g)?;
                    let args = if self.surface {
                        args
                    } else if args.is_empty() {
                        // `f()` is `f ()`: no arguments, or `()` to a
                        // function that takes one.
                        vec![Expr::Unit(g.span())]
                    } else {
                        // `f(())` passes the unit value, as `f (())` does.
                        args.into_iter()
                            .map(|a| match a {
                                Expr::Unit(sp) => Expr::Paren(Box::new(a), sp),
                                a => a,
                            })
                            .collect()
                    };
                    Expr::Call {
                        func: Box::new(e),
                        args,
                        trailing_comma,
                        span: g.span(),
                    }
                } else {
                    let mut sub = self.sub(&g);
                    let i = sub.parse_expr()?;
                    sub.expect_end()?;
                    Expr::Index(Box::new(e), Box::new(i), true)
                };
                continue;
            }
            if self.peek_op(".") {
                match self.peek_at(1).cloned() {
                    // `xs. len` with a space: the name belongs after the dot.
                    Some(Tok::Ident(name))
                        if self.touches(self.pos + 1) == Some(false) && !self.recover =>
                    {
                        return Err(Error::new(
                            name.span(),
                            "no space is allowed between `.` and the field or method name",
                        ));
                    }
                    Some(Tok::Ident(name))
                        if !(self.recover
                            && (NEVER_FIELDS.contains(&word(&name).as_str())
                                || self.touches(self.pos + 1) == Some(false))) =>
                    {
                        if is_keyword_field(&name) {
                            return Err(rust_keyword_error(&name));
                        }
                        self.pos += 2;
                        if name == "await" {
                            e = Expr::Await(Box::new(e), name.span());
                            continue;
                        }
                        let mut turbofish = None;
                        if self.peek_op_starts("::<")
                            || (self.peek_op("::")
                                && matches!(self.peek_at(1), Some(Tok::Op(o)) if o.text.starts_with('<')))
                        {
                            self.split_op_prefix("::");
                            self.pos += 1;
                            turbofish = Some(self.collect_generics()?);
                        }
                        let is_call = match self.peek() {
                            Some(Tok::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                                turbofish.is_some() || self.touches(self.pos).unwrap_or(true)
                            }
                            _ => false,
                        };
                        if is_call {
                            let Some(Tok::Group(g)) = self.bump() else {
                                unreachable!()
                            };
                            let (args, trailing_comma) = self.parse_call_args(&g)?;
                            e = Expr::Method {
                                recv: Box::new(e),
                                name,
                                turbofish,
                                args,
                                trailing_comma,
                            };
                        } else if turbofish.is_some() {
                            return self.err("expected `(` after turbofish");
                        } else {
                            e = Expr::Field(Box::new(e), name);
                        }
                    }
                    Some(Tok::Lit(l)) => {
                        let idx = tuple_index(&l)?;
                        self.pos += 2;
                        e = Expr::TupleIndex(Box::new(e), idx);
                    }
                    Some(Tok::Group(g)) if g.delimiter() != Delimiter::Brace => {
                        self.pos += 2;
                        let mut sub = self.sub(&g);
                        let i = sub.parse_expr()?;
                        sub.expect_end()?;
                        e = if g.delimiter() == Delimiter::Bracket {
                            // OCaml's `s.[i]`, the byte at `i` as a character:
                            // Rust strings are not indexed by position.
                            self.recoverable(Error::new(
                                g.span(),
                                "strings are not indexed by position in Rust: write `s.as_bytes()[i]` for the byte, or `s.chars().nth(i)` for the character",
                            ))?;
                            Expr::StrIndex(Box::new(e), Box::new(i))
                        } else {
                            Expr::Index(Box::new(e), Box::new(i), false)
                        };
                    }
                    _ if self.recover => {
                        let dot = self.span();
                        self.pos += 1;
                        let name = self.missing(Error::new(
                            dot,
                            "expected a field or method name after `.`",
                        ))?;
                        e = Expr::Field(Box::new(e), name);
                    }
                    _ => break,
                }
            } else if self.peek_op_starts("?") {
                let sp = self.span();
                self.split_op_prefix("?");
                self.pos += 1;
                e = Expr::Try(Box::new(e), sp);
            } else {
                break;
            }
        }
        Ok(e)
    }

    /// Rust-style `(a, b)` argument list of a call, and whether a `,`
    /// ends it.
    fn parse_call_args(&self, g: &Group) -> R<(Vec<Expr>, bool)> {
        let mut sub = self.sub(g);
        sub.comma_stop = true;
        let mut args = Vec::new();
        while !sub.at_end() {
            args.push(sub.parse_expr_noseq()?);
            if !sub.eat_op(",") {
                break;
            }
        }
        sub.expect_end()?;
        let trailing =
            !args.is_empty() && matches!(sub.toks.last(), Some(Tok::Op(o)) if o.text == ",");
        Ok((args, trailing))
    }

    /// Whether the group `g`, next, is a Rust-style call `f(a, b)` or index
    /// `v[i]` of `e`: it is written right after `e`, which ends in a name
    /// (`f`, `x.f`, `t.0`), a path's generic arguments (`size_of::<T>`) or
    /// a closing `)` or `]`. A constructor's `(` is its argument instead
    /// (see [`Parser::ctor_call_follows`]), and `[|..|]` is always an array.
    fn rust_call_follows(&self, e: &Expr, g: &Group) -> bool {
        match g.delimiter() {
            Delimiter::Parenthesis => {}
            Delimiter::Bracket => {
                let toks = lex(g.stream());
                let bar = |t: &Tok| matches!(t, Tok::Op(o) if o.text.starts_with('|'));
                if toks.first().is_some_and(bar) && toks.last().is_some_and(bar) {
                    return false;
                }
            }
            _ => return false,
        }
        let Some(prev) = self.pos.checked_sub(1).map(|i| &self.toks[i]) else {
            return false;
        };
        let callee = match prev {
            Tok::Ident(i) => !is_keyword(&word(i)),
            // `f::<T>(x)`, and `x?(a)` / `f(x)?[0]` (the `?` of `Expr::Try`).
            Tok::Op(o) => o.text.ends_with('>') || (o.text == "?" && matches!(e, Expr::Try(..))),
            Tok::Group(p) => matches!(p.delimiter(), Delimiter::Parenthesis | Delimiter::Bracket),
            Tok::Lit(_) => matches!(e, Expr::TupleIndex(..)),
            Tok::TyVar(..) => false,
        };
        let ctor = matches!(e.strip(), Expr::Path(p) if is_capitalized(p.last()));
        callee && !ctor && self.touches(self.pos) == Some(true)
    }

    /// Whether the group `g`, next, holds the argument of the constructor
    /// `e`, written right after it: `Some(x)`, `Shape::Rect(w, h)`,
    /// `(Some)(x)`.
    fn ctor_call_follows(&self, e: &Expr, g: &Group) -> bool {
        g.delimiter() == Delimiter::Parenthesis
            && matches!(e.strip(), Expr::Path(p) if is_capitalized(p.last()))
            && self.touches(self.pos) == Some(true)
    }

    fn parse_prefix_atom(&mut self) -> R<Expr> {
        if let Some(Tok::Op(o)) = self.peek() {
            let text = o.text.clone();
            let sp = o.span();
            // Rust glues prefix operators together (`&*x`, `!*x`, `**x`):
            // one at a time.
            if text.chars().count() > 1
                && text.chars().all(|c| matches!(c, '!' | '*' | '&'))
                && !text.chars().all(|c| c == '!')
                && text != "&&"
            {
                let first = &text[..1];
                self.split_op_prefix(first);
                return self.parse_prefix_atom();
            }
            // Rust's `*e`, which binds like Rust's (`*x.f` is `*(x.f)`).
            if text == "*" {
                self.pos += 1;
                let inner = self.parse_postfix()?;
                return Ok(Expr::Deref(Box::new(inner), sp));
            }
            if text.starts_with('!') && text.chars().all(|c| c == '!') {
                self.pos += 1;
                let mut e = self.parse_prefix_atom()?;
                for _ in 0..text.len() {
                    e = Expr::Deref(Box::new(e), sp);
                }
                return Ok(e);
            }
            if text == "&" || text == "&&" {
                self.pos += 1;
                let mutable = self.eat_kw("mut");
                let inner = self.parse_postfix()?;
                let mut e = Expr::Borrow {
                    mutable,
                    expr: Box::new(inner),
                    span: sp,
                };
                if text == "&&" {
                    e = Expr::Borrow {
                        mutable: false,
                        expr: Box::new(e),
                        span: sp,
                    };
                }
                return Ok(e);
            }
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> R<Expr> {
        // Unfinished code: an operand is missing before the end or before
        // a token that cannot start one (`let x = in`, `xs |> |> f`).
        if self.recover && (!self.can_start_expr() || self.peek_range()) {
            let at = self.span();
            return Ok(Expr::Var(
                self.missing(Error::new(at, "expected an expression"))?,
            ));
        }
        let Some(tok) = self.bump() else {
            return Err(Error::new(self.end, "expected an expression"));
        };
        match tok {
            Tok::Lit(l) => Ok(Expr::Lit(l)),
            Tok::Ident(i) => {
                let s = word(&i);
                match s.as_str() {
                    "true" => Ok(Expr::Bool(true, i.span())),
                    "false" => Ok(Expr::Bool(false, i.span())),
                    "begin" => {
                        self.mark("let", vec![i.span()]);
                        let end = self.span();
                        if self.eat_kw("end") {
                            if self.surface {
                                return Ok(Expr::Surface(Box::new(Surface::Begin {
                                    body: None,
                                    begin: i.span(),
                                    end,
                                })));
                            }
                            return Ok(Expr::Unit(i.span()));
                        }
                        let e = self.barrier(Parser::parse_expr)?;
                        let end = self.expect_kw("end")?;
                        if self.surface {
                            return Ok(Expr::Surface(Box::new(Surface::Begin {
                                body: Some(e),
                                begin: i.span(),
                                end,
                            })));
                        }
                        Ok(Expr::Paren(Box::new(e), i.span()))
                    }
                    "assert" if self.macro_bang_follows(&i) => self.finish_macro(Path {
                        leading_colon: false,
                        dotted: false,
                        segs: vec![PathSeg {
                            ident: i,
                            generics: None,
                        }],
                    }),
                    _ if is_keyword(&s) => {
                        Err(Error::new(i.span(), format!("unexpected keyword `{}`", s)))
                    }
                    "unsafe" | "async" if is_rust_keyword(&i) => self.parse_rust_block(i),
                    "return" | "break" | "continue" if is_rust_keyword(&i) => Err(Error::new(
                        i.span(),
                        format!("put `{}` in parentheses here: `({} ..)`", s, s),
                    )),
                    "move" if matches!(self.peek(), Some(Tok::Op(o)) if closure_bar(&o.text)) => {
                        Err(closure_paren_error(i.span()))
                    }
                    // `fn f x = ..`: Rust's spelling of a local function.
                    "fn" if matches!(self.peek(), Some(Tok::Ident(n)) if !is_keyword(&word(n))) => {
                        let Some(Tok::Ident(name)) = self.peek() else {
                            unreachable!()
                        };
                        let name = ident_str(name);
                        let param = if name == "x" { "y" } else { "x" };
                        Err(Error::new(
                            i.span(),
                            format!(
                                "`fn` is a Rust keyword, not a name (as a name: `r#fn`); a named local \
                                 function is `let {} {} = .. in ..`, and a closure is `|x| e`",
                                name, param
                            ),
                        ))
                    }
                    // OCaml's `ref`, as in `let r = ref 0`.
                    "ref" => self.parse_lower(i),
                    _ if is_rust_keyword(&i) => Err(rust_keyword_error(&i)),
                    _ if is_capitalized(&i) => self.parse_path_expr(i, false),
                    _ => self.parse_lower(i),
                }
            }
            Tok::Op(o) if o.text == "::" => match self.peek() {
                Some(Tok::Ident(i)) => {
                    let i = i.clone();
                    self.pos += 1;
                    self.parse_path_expr(i, true)
                }
                _ => {
                    let i = self.missing(Error::new(o.span(), "expected a path after `::`"))?;
                    self.parse_path_expr(i, true)
                }
            },
            Tok::Op(o) if o.text == "." => self.parse_method_ref(o.span()),
            Tok::Group(g) => match g.delimiter() {
                Delimiter::Parenthesis => self.parse_paren(&g),
                Delimiter::Bracket => self.parse_bracket(&g),
                Delimiter::Brace => self.parse_brace(&g),
                Delimiter::None => Ok(Expr::Rust(g)),
            },
            // Among cases, the `|` of the next case, after a missing body.
            Tok::Op(o) if closure_bar(&o.text) && self.in_cases() => {
                Err(Error::new(o.span(), "expected an expression before `|`"))
            }
            Tok::Op(o) if closure_bar(&o.text) => Err(closure_paren_error(o.span())),
            Tok::Op(o) => Err(Error::new(
                o.span(),
                format!("unexpected operator `{}`", o.text),
            )),
            Tok::TyVar(_, sp) => Err(Error::new(sp, "unexpected type variable")),
        }
    }

    /// `unsafe { e }`, `async { e }` or `async move { e }`, after the
    /// first keyword. The braces hold OCaml, as `begin .. end` does.
    fn parse_rust_block(&mut self, kw: Ident) -> R<Expr> {
        let mut kind = if kw == "unsafe" {
            RustBlock::Unsafe
        } else {
            RustBlock::Async
        };
        if kind == RustBlock::Async
            && self.peek_kw("move")
            && matches!(self.peek_at(1), Some(Tok::Group(g)) if g.delimiter() == Delimiter::Brace)
        {
            self.pos += 1;
            kind = RustBlock::AsyncMove;
        }
        let g = match self.peek() {
            Some(Tok::Group(g)) if g.delimiter() == Delimiter::Brace => g.clone(),
            _ => return Err(rust_keyword_error(&kw)),
        };
        self.pos += 1;
        let mut sub = self.sub(&g);
        let body = if sub.at_end() {
            None
        } else {
            let e = sub.parse_expr()?;
            sub.expect_end()?;
            Some(Box::new(e))
        };
        Ok(Expr::RustBlock {
            kind,
            body,
            span: kw.span(),
            brace: g.span(),
        })
    }

    /// `.name` (or `.name::<T>`) where an expression starts: the method as
    /// a function of its receiver and arguments, `.push xs 1` being
    /// `xs.push(1)`. (After an expression, `.name` is a field or method of
    /// that expression, so this can only start an expression.)
    fn parse_method_ref(&mut self, dot: Span) -> R<Expr> {
        // `.0`: a tuple field as a function of the tuple.
        if let Some(Tok::Lit(l)) = self.peek() {
            if self.touches(self.pos) == Some(false) && !self.recover {
                return Err(Error::new(
                    l.span(),
                    "no space is allowed between `.` and the field",
                ));
            }
            let idx = tuple_index(l)?;
            self.pos += 1;
            return Ok(Expr::FieldRef { idx, span: dot });
        }
        // The name is written right after the dot, so in `. xs` (the name
        // still to come) `xs` is the receiver.
        let mut written = false;
        let name = match self.peek() {
            Some(Tok::Ident(i)) if is_rust_keyword(i) && i != "await" => {
                return Err(rust_keyword_error(i));
            }
            Some(Tok::Ident(i))
                if !is_keyword(&word(i)) && self.touches(self.pos) != Some(false) =>
            {
                let i = i.clone();
                self.pos += 1;
                written = true;
                i
            }
            Some(Tok::Ident(i)) if !is_keyword(&word(i)) && !self.recover => {
                return Err(Error::new(
                    i.span(),
                    "no space is allowed between `.` and the method name",
                ));
            }
            _ => self.missing(Error::new(dot, "expected a method name after `.`"))?,
        };
        let mut turbofish = None;
        if self.peek_op_starts("::<")
            || (self.peek_op("::")
                && matches!(self.peek_at(1), Some(Tok::Op(o)) if o.text.starts_with('<')))
        {
            self.split_op_prefix("::");
            self.pos += 1;
            turbofish = Some(self.collect_generics()?);
        }
        // `.map(f)`, with its arguments written Rust style.
        let args = match self.peek() {
            Some(Tok::Group(g))
                if written
                    && g.delimiter() == Delimiter::Parenthesis
                    && self.touches(self.pos) == Some(true) =>
            {
                let g = g.clone();
                self.pos += 1;
                Some(self.parse_call_args(&g)?.0)
            }
            _ => None,
        };
        Ok(Expr::MethodRef {
            name,
            turbofish,
            args,
            span: dot,
        })
    }

    fn parse_lower(&mut self, i: Ident) -> R<Expr> {
        let s = word(&i);
        if let Some(digits) = s.strip_prefix('_')
            && (digits.is_empty() || digits.chars().all(|c| c.is_ascii_digit()))
        {
            let written = match digits.parse::<usize>() {
                Ok(n) => Some(n),
                Err(_) if digits.is_empty() => None,
                Err(_) => return Err(Error::new(i.span(), "placeholder number too large")),
            };
            self.open_holes.set(self.open_holes.get() + 1);
            return Ok(Expr::Hole {
                written,
                index: None,
                span: i.span(),
            });
        }
        if self.macro_bang_follows(&i) {
            let path = Path {
                leading_colon: false,
                dotted: false,
                segs: vec![PathSeg {
                    ident: i,
                    generics: None,
                }],
            };
            return self.finish_macro(path);
        }
        if self.rust_path_follows(&i) {
            return self.parse_path_expr(i, false);
        }
        // Unfinished `std::` (for a name bound here it is a cons).
        if self.recover && !self.in_scope(&s) && self.unfinished_path(false) {
            return self.parse_path_expr(i, false);
        }
        Ok(Expr::Var(i))
    }

    fn macro_bang_follows(&self, last: &Ident) -> bool {
        match (self.peek(), self.peek_at(1)) {
            (Some(Tok::Op(o)), Some(Tok::Group(_))) if o.text == "!" => {
                adjacent(last.span(), o.span()).unwrap_or(true)
            }
            _ => false,
        }
    }

    /// Whether the token at `n` is `assert` written as the macro `assert!(..)`
    /// rather than OCaml's `assert e`.
    fn assert_macro_at(&self, n: usize) -> bool {
        match (self.peek_at(n), self.peek_at(n + 1), self.peek_at(n + 2)) {
            (Some(Tok::Ident(i)), Some(Tok::Op(_)), Some(Tok::Group(_)))
                if i == "assert"
                    && matches!(self.peek_at(n + 1), Some(Tok::Op(o)) if o.text == "!") =>
            {
                self.touches(self.pos + n + 1).unwrap_or(true)
            }
            _ => false,
        }
    }

    fn finish_macro(&mut self, path: Path) -> R<Expr> {
        let bang = self.span();
        self.pos += 1;
        let Some(Tok::Group(group)) = self.bump() else {
            unreachable!()
        };
        if is_std_macro(&path) {
            let vec = path.last() == "vec";
            return self.parse_std_macro(path, bang, group, vec);
        }
        Ok(Expr::Macro { path, bang, group })
    }

    /// The arguments of a std macro, as OCaml expressions separated by `,`
    /// (each parsed up to a `,` outside brackets, so `HashMap::<K, V>::new ()`
    /// and `a < b` are single arguments).
    fn parse_std_macro(&mut self, path: Path, bang: Span, group: Group, vec: bool) -> R<Expr> {
        let mut sub = self.sub(&group);
        sub.comma_stop = true;
        let mut args = Vec::new();
        let mut repeat = false;
        let mut trailing_comma = false;
        // Named arguments follow a format string.
        let mut format = false;
        while !sub.at_end() {
            let named = match (sub.peek(), sub.peek_at(1)) {
                (Some(Tok::Ident(n)), Some(Tok::Op(o))) if format && o.text == "=" => {
                    Some(n.clone())
                }
                _ => None,
            };
            if let Some(n) = named {
                if is_keyword(&word(&n)) || is_rust_keyword(&n) {
                    return Err(rust_keyword_error(&n));
                }
                sub.pos += 2;
                args.push(MacroArg::Named(n, sub.parse_expr_noseq()?));
            } else {
                let string = matches!(sub.peek(), Some(Tok::Lit(l)) if is_string(l))
                    && match sub.peek_at(1) {
                        None => true,
                        Some(Tok::Op(o)) => o.text == ",",
                        _ => false,
                    };
                format |= string;
                args.push(MacroArg::Expr(sub.parse_expr_noseq()?));
            }
            if vec && args.len() == 1 && sub.eat_op(";") {
                repeat = true;
                args.push(MacroArg::Expr(sub.parse_expr_noseq()?));
                break;
            }
            if !sub.eat_op(",") {
                break;
            }
            trailing_comma = sub.at_end();
        }
        sub.expect_end()?;
        Ok(Expr::StdMacro {
            path,
            bang,
            group,
            args,
            repeat,
            trailing_comma,
        })
    }

    /// Whether `ident::next` is a Rust path rather than the cons `ident :: next`.
    /// Paths are written without spaces around `::`, and a name bound by the
    /// ml code is always a value, never a module.
    fn rust_path_follows(&self, first: &Ident) -> bool {
        let Some(Tok::Op(o)) = self.peek() else {
            return false;
        };
        if o.text.starts_with("::<") {
            return true;
        }
        if o.text != "::" {
            return false;
        }
        let Some(Tok::Ident(_)) = self.peek_at(1) else {
            return false;
        };
        match self.tight_at(self.pos) {
            Some(t) => t && (is_capitalized(first) || !self.in_scope(&ident_str(first))),
            None => is_capitalized(first),
        }
    }

    fn parse_path_expr(&mut self, first: Ident, leading_colon: bool) -> R<Expr> {
        let path = self.parse_path(first, leading_colon)?;
        if self.macro_bang_follows(path.last()) {
            return self.finish_macro(path);
        }
        if is_capitalized(path.last())
            && let Some(Tok::Group(g)) = self.peek()
            && g.delimiter() == Delimiter::Brace
        {
            let g = g.clone();
            self.pos += 1;
            let mut sub = self.sub(&g);
            let fields = sub.parse_record_fields()?;
            let trailing_semi = matches!(sub.toks.last(), Some(Tok::Op(o)) if o.text == ";");
            return Ok(Expr::Record {
                ctor: path,
                fields,
                trailing_semi,
            });
        }
        Ok(Expr::Path(path))
    }

    /// Paths: OCaml style `Vec.new`, `Color.Red` (capitalized segments joined
    /// by `.`) or Rust style `std::mem::swap`, `Vec::<u8>::new`.
    fn parse_path(&mut self, first: Ident, leading_colon: bool) -> R<Path> {
        let mut segs = vec![PathSeg {
            ident: first,
            generics: None,
        }];
        let mut rust_style = leading_colon;
        let mut dotted = false;
        loop {
            let last = segs.last().unwrap().ident.clone();
            if self.peek_op_starts("::<")
                || (self.peek_op("::")
                    && matches!(self.peek_at(1), Some(Tok::Op(o)) if o.text.starts_with('<')))
            {
                self.split_op_prefix("::");
                self.pos += 1;
                segs.last_mut().unwrap().generics = Some(self.collect_generics()?);
                rust_style = true;
                continue;
            }
            if self.recover && self.unfinished_path(false) {
                let Some(Tok::Op(o)) = self.bump() else {
                    unreachable!()
                };
                let msg = format!("expected a name after `{}`", o.text);
                segs.push(PathSeg {
                    ident: self.missing(Error::new(o.span(), msg))?,
                    generics: None,
                });
                dotted |= o.text == ".";
                break;
            }
            if let (Some(Tok::Op(o)), Some(Tok::Ident(n))) = (self.peek(), self.peek_at(1)) {
                if o.text == "::" {
                    // After generic arguments (`Vec::<u8>::new`), from the `>`.
                    let ok = match self.tight_at(self.pos) {
                        Some(t) => t,
                        None => rust_style || is_capitalized(&last),
                    };
                    if !ok {
                        break;
                    }
                    if is_rust_keyword(n) {
                        return Err(rust_keyword_error(n));
                    }
                    let n = n.clone();
                    self.pos += 2;
                    segs.push(PathSeg {
                        ident: n,
                        generics: None,
                    });
                    rust_style = true;
                    continue;
                }
                if o.text == "." && is_capitalized(&last) {
                    if is_rust_keyword(n) {
                        return Err(rust_keyword_error(n));
                    }
                    let n = n.clone();
                    self.pos += 2;
                    dotted = true;
                    let lower = !is_capitalized(&n);
                    segs.push(PathSeg {
                        ident: n,
                        generics: None,
                    });
                    if lower {
                        break;
                    }
                    continue;
                }
            }
            break;
        }
        Ok(Path {
            leading_colon,
            dotted,
            segs,
        })
    }

    /// Whether the current `::` (or `.` after a capitalized name), written
    /// right after the path so far, is a separator whose next segment is
    /// still to come: nothing that could be one follows it, or only after a
    /// space (`Vec::`, `Vec.`, `Vec::<u8>::`, `geometry:: &shape`, `std::
    /// in`). `x :: xs`, with a space before `::`, is a cons.
    fn unfinished_path(&self, pattern: bool) -> bool {
        let (Some(Tok::Op(o)), true) = (self.peek(), self.pos > 0) else {
            return false;
        };
        let capitalized = matches!(&self.toks[self.pos - 1], Tok::Ident(i) if is_capitalized(i));
        if !(o.text == "::" || (o.text == "." && capitalized))
            || self.touches(self.pos) == Some(false)
        {
            return false;
        }
        let Some(next) = self.peek_at(1) else {
            return true;
        };
        if self.touches(self.pos + 1) == Some(false) {
            return true;
        }
        match next {
            Tok::Ident(n) => is_keyword(&word(n)),
            // In a pattern, `A::[x]` is a cons.
            Tok::Op(n) if pattern => !matches!(n.text.as_str(), "-" | "&" | "&&"),
            _ if pattern => false,
            _ => !self.can_start_expr_at(1),
        }
    }

    /// Collects Rust generic arguments starting at the current `<`.
    fn collect_generics(&mut self) -> R<TokenStream> {
        let mut out: Vec<Tok> = Vec::new();
        let mut depth: i32 = 0;
        loop {
            let Some(tok) = self.bump() else {
                return Err(Error::new(self.end, "unclosed `<`"));
            };
            let Tok::Op(o) = tok else {
                out.push(tok);
                continue;
            };
            let chars: Vec<char> = o.text.chars().collect();
            let mut i = 0;
            let mut close = None;
            while i < chars.len() {
                if chars[i] == '-' && chars.get(i + 1) == Some(&'>') {
                    i += 2;
                    continue;
                }
                match chars[i] {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    _ => {}
                }
                i += 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            match close {
                Some(n) if n < chars.len() => {
                    // `>::` in `Vec::<u8>::new`: keep the `::` for the path.
                    let (a, b) = o.split(n);
                    out.push(Tok::Op(a.clone()));
                    self.split_token(self.pos - 1, a, b);
                    return Ok(to_stream(&out));
                }
                Some(_) => {
                    out.push(Tok::Op(o));
                    return Ok(to_stream(&out));
                }
                None => out.push(Tok::Op(o)),
            }
        }
    }

    fn parse_paren(&mut self, g: &Group) -> R<Expr> {
        let mut sub = self.sub(g);
        if sub.at_end() {
            return Ok(Expr::Unit(g.span()));
        }
        if let Some(op) = sub.single_op() {
            return Ok(Expr::OpSection(op));
        }
        let e = sub.parse_expr()?;
        let e = if sub.eat_op(":") {
            Expr::Constraint(Box::new(e), sub.parse_type()?)
        } else {
            e
        };
        sub.expect_end()?;
        Ok(Expr::Paren(Box::new(e), g.span()))
    }

    fn parse_bracket(&mut self, g: &Group) -> R<Expr> {
        let toks = lex(g.stream());
        let span = g.span();
        let bar = |t: &Tok| matches!(t, Tok::Op(o) if o.text == "|");
        if toks.len() == 1 && matches!(&toks[0], Tok::Op(o) if o.text == "||") {
            return Ok(Expr::Array(Vec::new(), span));
        }
        if toks.len() >= 2 && bar(&toks[0]) && bar(toks.last().unwrap()) {
            let inner = toks[1..toks.len() - 1].to_vec();
            let mut sub = self.sub_toks(inner, g.span_close());
            return Ok(Expr::Array(sub.parse_semi_list(true)?, span));
        }
        let mut sub = self.sub_toks(toks, g.span_close());
        Ok(Expr::List(sub.parse_semi_list(false)?, span))
    }

    fn parse_semi_list(&mut self, array: bool) -> R<Vec<Expr>> {
        let mut items = Vec::new();
        while !self.at_end() {
            let start = self.pos;
            let item = self.parse_expr_noseq()?;
            if matches!(item, Expr::Tuple(_)) {
                self.comma_in_list(start, array)?;
            }
            items.push(item);
            if !self.eat_op(";") {
                break;
            }
        }
        self.expect_end()?;
        Ok(items)
    }

    /// `[1, 2]` is the list of one pair `[(1, 2)]` in OCaml, which a Rust
    /// reader would not guess: an error at the first comma after `start`
    /// (the item just parsed, a tuple), in expressions and patterns alike.
    fn comma_in_list(&self, start: usize, array: bool) -> R<()> {
        let Some(Tok::Op(comma)) = self.toks[start..self.pos]
            .iter()
            .find(|t| matches!(t, Tok::Op(o) if o.text == ","))
        else {
            return Ok(());
        };
        let msg = if array {
            "array items are separated by `;`: write `[|1; 2|]`, or `[|(1, 2)|]` for an array of one pair"
        } else {
            "list items are separated by `;`: write `[1; 2]`, or `[(1, 2)]` for a list of one pair"
        };
        self.recoverable(Error::new(comma.span(), msg))
    }

    fn parse_brace(&mut self, g: &Group) -> R<Expr> {
        let toks = lex(g.stream());
        let Some(with) = toks
            .iter()
            .position(|t| matches!(t, Tok::Ident(i) if i == "with"))
        else {
            // `{ x = 1 }` is a record without its name; `{ a; b }` and
            // `{ f x }` are Rust blocks.
            let record = matches!(&toks[..], [Tok::Ident(_), Tok::Op(o), ..] if o.text == "=");
            return Err(Error::new(
                g.span(),
                if record {
                    "record expressions need the struct name in Rust: write `Point { x = 1; y = 2 }`"
                } else {
                    "`{ .. }` is not a block in rsml: write `begin .. end` or `( .. )` \
                     (a record needs its struct name: `Point { x = 1 }`)"
                },
            ));
        };
        self.mark("match", vec![toks[with].span()]);
        let mut base = self.sub_toks(toks[..with].to_vec(), toks[with].span());
        let b = base.parse_expr()?;
        base.expect_end()?;
        let mut rest = self.sub_toks(toks[with + 1..].to_vec(), g.span_close());
        let mut fields = Vec::new();
        for (f, v) in rest.parse_record_fields()? {
            match v {
                Some(v) => fields.push((f, v)),
                None => fields.push((f.clone(), Expr::Var(f))),
            }
        }
        Ok(Expr::RecordUpdate {
            base: Box::new(b),
            fields,
            span: g.span(),
        })
    }

    fn parse_record_fields(&mut self) -> R<Vec<(Ident, Option<Expr>)>> {
        let mut fields = Vec::new();
        while !self.at_end() {
            let f = self.expect_ident()?;
            let v = if self.eat_op("=") {
                Some(self.parse_expr_noseq()?)
            } else {
                None
            };
            fields.push((f, v));
            if !self.eat_op(";") {
                break;
            }
        }
        self.expect_end()?;
        Ok(fields)
    }

    // ----- patterns ------------------------------------------------------

    pub fn parse_pattern(&mut self) -> R<Pat> {
        let mut p = self.parse_pat_or()?;
        while self.eat_kw("as") {
            let name = self.expect_ident()?;
            p = Pat::Alias(Box::new(p), name);
        }
        Ok(p)
    }

    fn parse_pat_or(&mut self) -> R<Pat> {
        let first = self.parse_pat_tuple()?;
        if !self.peek_op("|") {
            return Ok(first);
        }
        let mut alts = vec![first];
        while self.peek_op("|") {
            self.mark_op("|");
            self.pos += 1;
            alts.push(self.parse_pat_tuple()?);
        }
        Ok(Pat::Or(alts))
    }

    fn parse_pat_tuple(&mut self) -> R<Pat> {
        let first = self.parse_pat_cons()?;
        if self.comma_stop || !self.peek_op(",") {
            return Ok(first);
        }
        let mut items = vec![first];
        while self.eat_op(",") {
            items.push(self.parse_pat_cons()?);
        }
        Ok(Pat::Tuple(items))
    }

    fn parse_pat_cons(&mut self) -> R<Pat> {
        let head = self.parse_pat_app()?;
        if let Some(Tok::Op(o)) = self.peek()
            && o.text == "::"
        {
            let sp = o.span();
            self.unmapped(&o.spans);
            self.pos += 1;
            let tail = self.parse_pat_cons()?;
            return Ok(Pat::Cons(Box::new(head), Box::new(tail), sp));
        }
        Ok(head)
    }

    fn can_start_pat_atom(&self) -> bool {
        match self.peek() {
            Some(Tok::Ident(i)) => {
                let s = word(i);
                !is_keyword(&s) || s == "true" || s == "false"
            }
            Some(Tok::Lit(_)) | Some(Tok::Group(_)) => true,
            Some(Tok::Op(o)) => o.text == "-" || is_amps(&o.text),
            _ => false,
        }
    }

    fn parse_pat_app(&mut self) -> R<Pat> {
        if let Some(Tok::Ident(i)) = self.peek() {
            let s = word(i);
            if s == "lazy" || s == "exception" {
                return self.err(format!("`{}` patterns are not supported", s));
            }
            if is_capitalized(i) || self.lower_pat_path_follows(i, 1) {
                let i = i.clone();
                self.pos += 1;
                let path = self.parse_pat_path(i)?;
                if let Some(p) = self.parse_ctor_record(&path)? {
                    return Ok(p);
                }
                if self.can_start_pat_atom() {
                    let arg = self.parse_pat_atom()?;
                    return Ok(Pat::Ctor(path, Some(Box::new(arg))));
                }
                return Ok(Pat::Ctor(path, None));
            }
        }
        self.parse_pat_atom()
    }

    fn parse_ctor_record(&mut self, path: &Path) -> R<Option<Pat>> {
        let Some(Tok::Group(g)) = self.peek() else {
            return Ok(None);
        };
        if g.delimiter() != Delimiter::Brace {
            return Ok(None);
        }
        let g = g.clone();
        self.pos += 1;
        let mut sub = self.sub(&g);
        let mut fields = Vec::new();
        let mut wild = false;
        while !sub.at_end() {
            if sub.peek_kw("_") {
                sub.pos += 1;
                wild = true;
            } else {
                let f = sub.expect_ident()?;
                let p = if sub.eat_op("=") {
                    Some(sub.parse_pattern()?)
                } else {
                    None
                };
                fields.push((f, p));
            }
            if !sub.eat_op(";") {
                break;
            }
        }
        sub.expect_end()?;
        Ok(Some(Pat::CtorRecord(path.clone(), fields, wild)))
    }

    /// Constructor paths in patterns: `Some`, `Color.Red`, `Ordering::Less`.
    fn parse_pat_path(&mut self, first: Ident) -> R<Path> {
        let mut segs = vec![PathSeg {
            ident: first,
            generics: None,
        }];
        let mut dotted = false;
        loop {
            let last = segs.last().unwrap().ident.clone();
            // Unfinished: `Ordering::` with no pattern after it.
            if self.recover && self.unfinished_path(true) {
                let Some(Tok::Op(o)) = self.bump() else {
                    unreachable!()
                };
                let msg = format!("expected a name after `{}`", o.text);
                segs.push(PathSeg {
                    ident: self.missing(Error::new(o.span(), msg))?,
                    generics: None,
                });
                dotted |= o.text == ".";
                break;
            }
            let (Some(Tok::Op(o)), Some(next @ Tok::Ident(n))) = (self.peek(), self.peek_at(1))
            else {
                break;
            };
            let ok = match o.text.as_str() {
                "." => is_capitalized(&last),
                "::" => tight(last.span(), o, next).unwrap_or(true),
                _ => false,
            };
            if !ok {
                break;
            }
            if is_rust_keyword(n) {
                return Err(rust_keyword_error(n));
            }
            dotted |= o.text == ".";
            let n = n.clone();
            self.pos += 2;
            segs.push(PathSeg {
                ident: n,
                generics: None,
            });
        }
        Ok(Path {
            leading_colon: false,
            dotted,
            segs,
        })
    }

    /// Whether the lowercase `first`, followed by the tokens from
    /// `peek_at(at)`, starts a path pattern (`std::cmp::Ordering::Less`,
    /// `i32::MAX`) rather than a cons (`x::rest`): its `::` chain, written
    /// without spaces, reaches a capitalized name.
    fn lower_pat_path_follows(&self, first: &Ident, at: usize) -> bool {
        let mut last = first;
        let mut k = at;
        loop {
            match (self.peek_at(k), self.peek_at(k + 1)) {
                (Some(Tok::Op(o)), Some(next @ Tok::Ident(n)))
                    if o.text == "::" && tight(last.span(), o, next).unwrap_or(true) =>
                {
                    if is_capitalized(n) {
                        return true;
                    }
                    last = n;
                    k += 2;
                }
                _ => return false,
            }
        }
    }

    fn parse_pat_atom(&mut self) -> R<Pat> {
        // Unfinished: `| ` before the end or `->`.
        if self.recover && (self.at_end() || self.peek_op("->")) {
            let at = self.span();
            return Ok(Pat::Var(
                self.missing(Error::new(at, "expected a pattern"))?,
            ));
        }
        let Some(tok) = self.bump() else {
            return Err(Error::new(self.end, "expected a pattern"));
        };
        match tok {
            Tok::Ident(i) => {
                let s = word(&i);
                match s.as_str() {
                    "_" => Ok(Pat::Wild(i.span())),
                    "true" => Ok(Pat::Bool(true, i.span())),
                    "false" => Ok(Pat::Bool(false, i.span())),
                    "mut" => Ok(Pat::Mut(self.expect_ident()?)),
                    _ if is_keyword(&s) => {
                        Err(Error::new(i.span(), format!("unexpected keyword `{}`", s)))
                    }
                    _ if is_rust_keyword(&i) => Err(rust_keyword_error(&i)),
                    _ if is_capitalized(&i) || self.lower_pat_path_follows(&i, 0) => {
                        let path = self.parse_pat_path(i)?;
                        match self.parse_ctor_record(&path)? {
                            Some(p) => Ok(p),
                            None => Ok(Pat::Ctor(path, None)),
                        }
                    }
                    _ => Ok(Pat::Var(i)),
                }
            }
            Tok::Lit(l) => self.finish_lit_pat(Pat::Lit { lit: l, neg: false }),
            Tok::Op(o) if o.text == "-" => match self.bump() {
                Some(Tok::Lit(l)) => self.finish_lit_pat(Pat::Lit { lit: l, neg: true }),
                _ => Err(Error::new(o.span(), "expected a literal after `-`")),
            },
            // `&p`, `&&p`, `&&&p`: Rust glues the `&`s together.
            Tok::Op(o) if is_amps(&o.text) => {
                let mut p = self.parse_pat_atom()?;
                for _ in 0..o.text.len() {
                    p = Pat::Ref(Box::new(p), o.span());
                }
                Ok(p)
            }
            Tok::Group(g) => match g.delimiter() {
                Delimiter::Parenthesis | Delimiter::None => {
                    let mut sub = self.sub(&g);
                    if sub.at_end() {
                        return Ok(Pat::Unit(g.span()));
                    }
                    let p = sub.parse_pattern()?;
                    let p = if sub.eat_op(":") {
                        Pat::Constraint(Box::new(p), sub.parse_type()?)
                    } else {
                        p
                    };
                    sub.expect_end()?;
                    Ok(Pat::Paren(Box::new(p)))
                }
                Delimiter::Bracket => {
                    let toks = lex(g.stream());
                    let bar = |t: &Tok| matches!(t, Tok::Op(o) if o.text == "|");
                    if toks.len() == 1 && matches!(&toks[0], Tok::Op(o) if o.text == "||") {
                        return Ok(Pat::Array(Vec::new(), g.span()));
                    }
                    let (array, inner) =
                        if toks.len() >= 2 && bar(&toks[0]) && bar(toks.last().unwrap()) {
                            (true, toks[1..toks.len() - 1].to_vec())
                        } else {
                            (false, toks)
                        };
                    let mut sub = self.sub_toks(inner, g.span_close());
                    let mut items = Vec::new();
                    while !sub.at_end() {
                        let start = sub.pos;
                        let item = sub.parse_pattern()?;
                        if matches!(item, Pat::Tuple(_)) {
                            sub.comma_in_list(start, array)?;
                        }
                        items.push(item);
                        if !sub.eat_op(";") {
                            break;
                        }
                    }
                    sub.expect_end()?;
                    Ok(if array {
                        Pat::Array(items, g.span())
                    } else {
                        Pat::List(items, g.span())
                    })
                }
                Delimiter::Brace => Err(Error::new(
                    g.span(),
                    "record patterns need the struct name in Rust: write `Point { x; y }`",
                )),
            },
            other => Err(Error::new(other.span(), "expected a pattern")),
        }
    }

    /// `lo..hi` or `lo..=hi` after a literal, as in Rust: `..` leaves `hi`
    /// out (however it is spaced), `..=` includes it.
    fn finish_lit_pat(&mut self, lo: Pat) -> R<Pat> {
        let Some(op) = (match self.peek() {
            Some(Tok::Op(o)) => range_op(&o.text),
            _ => None,
        }) else {
            return Ok(lo);
        };
        self.split_op_prefix(op);
        self.pos += 1;
        let hi = match self.bump() {
            Some(Tok::Lit(l)) => Pat::Lit { lit: l, neg: false },
            Some(Tok::Op(o)) if o.text == "-" => match self.bump() {
                Some(Tok::Lit(l)) => Pat::Lit { lit: l, neg: true },
                _ => return self.err("expected a literal"),
            },
            _ => return self.err("expected the end of the range"),
        };
        Ok(Pat::Range(Box::new(lo), Box::new(hi), op == "..="))
    }

    // ----- types ---------------------------------------------------------

    pub fn parse_type(&mut self) -> R<Type> {
        let t = self.parse_type_tuple()?;
        self.mark_arrow();
        if self.eat_op("->") {
            let r = self.parse_type()?;
            return Ok(Type::Arrow(Box::new(t), Box::new(r)));
        }
        Ok(t)
    }

    fn parse_type_tuple(&mut self) -> R<Type> {
        let first = self.parse_type_atom()?;
        if !self.peek_op("*") {
            return Ok(first);
        }
        let mut items = vec![first];
        while self.eat_op("*") {
            items.push(self.parse_type_atom()?);
        }
        Ok(Type::Tuple(items))
    }

    pub fn parse_type_atom(&mut self) -> R<Type> {
        // Unfinished: `(x : )`, `let x : = ..`.
        if self.recover
            && match self.peek() {
                None => true,
                Some(Tok::Op(o)) => !(matches!(o.text.as_str(), "::" | "!") || is_amps(&o.text)),
                _ => false,
            }
        {
            let at = self.span();
            let i = self.missing(Error::new(at, "expected a type"))?;
            return Ok(Type::Path {
                tokens: quote::quote!(#i),
                args: None,
            });
        }
        let Some(tok) = self.bump() else {
            return Err(Error::new(self.end, "expected a type"));
        };
        match tok {
            Tok::TyVar(i, _) => Ok(Type::Var(i)),
            Tok::Ident(i) if i == "_" => Ok(Type::Infer(i.span())),
            Tok::Ident(i) => self.parse_type_path(i, false),
            Tok::Op(o) if o.text == "::" => match self.bump() {
                Some(Tok::Ident(i)) => self.parse_type_path(i, true),
                _ => Err(Error::new(o.span(), "expected a path")),
            },
            Tok::Op(o) if is_amps(&o.text) => {
                let lifetime = match self.peek() {
                    Some(Tok::TyVar(l, _)) => {
                        let l = l.clone();
                        self.pos += 1;
                        Some(l)
                    }
                    _ => None,
                };
                let mutable = self.eat_kw("mut");
                let inner = self.parse_type_atom()?;
                let mut t = Type::Ref {
                    lifetime,
                    mutable,
                    inner: Box::new(inner),
                    span: o.span(),
                };
                // `&&T`, `&&&T`: the lifetime and `mut` are the innermost's.
                for _ in 1..o.text.len() {
                    t = Type::Ref {
                        lifetime: None,
                        mutable: false,
                        inner: Box::new(t),
                        span: o.span(),
                    };
                }
                Ok(t)
            }
            Tok::Op(o) if o.text == "!" => Ok(Type::Rust(to_stream(&[Tok::Op(o)]))),
            Tok::Group(g) => match g.delimiter() {
                Delimiter::Parenthesis => {
                    let mut sub = self.sub(&g);
                    if sub.at_end() {
                        return Ok(Type::Unit(g.span()));
                    }
                    let first = sub.parse_type()?;
                    if sub.peek_op(",") {
                        let mut items = vec![first];
                        while sub.eat_op(",") {
                            items.push(sub.parse_type()?);
                        }
                        sub.expect_end()?;
                        return Ok(Type::ParenTuple(items));
                    }
                    sub.expect_end()?;
                    Ok(Type::Paren(Box::new(first)))
                }
                Delimiter::Bracket => {
                    // `[t]` and `[t; N]`.
                    let mut sub = self.sub(&g);
                    let elem = sub.parse_type()?;
                    let len = if sub.eat_op(";") {
                        // `[T; ]` is no Rust type; while it is being typed,
                        // it is unfinished.
                        if sub.at_end() {
                            sub.recoverable(Error::new(
                                g.span_close(),
                                "expected the array's length after `;`",
                            ))?;
                        }
                        Some(to_stream(&sub.toks[sub.pos..]))
                    } else {
                        None
                    };
                    if len.is_none() {
                        sub.expect_end()?;
                    }
                    Ok(Type::Slice {
                        elem: Box::new(elem),
                        len,
                    })
                }
                _ => Err(Error::new(g.span(), "expected a type")),
            },
            other => Err(Error::new(other.span(), "expected a type")),
        }
    }

    fn parse_type_path(&mut self, first: Ident, leading_colon: bool) -> R<Type> {
        let mut toks: Vec<Tok> = Vec::new();
        if leading_colon {
            toks.push(Tok::Op(Op::new("::", first.span())));
        }
        toks.push(Tok::Ident(first));
        loop {
            if self.recover && self.unfinished_path(false) {
                let Some(Tok::Op(o)) = self.bump() else {
                    unreachable!()
                };
                let i = self.missing(Error::new(o.span(), "expected a name after `::`"))?;
                toks.push(Tok::Op(Op::new("::", o.span())));
                toks.push(Tok::Ident(i));
                break;
            }
            let sep = match (self.peek(), self.peek_at(1)) {
                (Some(Tok::Op(o)), Some(Tok::Ident(_))) if o.text == "::" || o.text == "." => {
                    o.span()
                }
                _ => break,
            };
            self.pos += 1;
            let Some(Tok::Ident(n)) = self.bump() else {
                unreachable!()
            };
            toks.push(Tok::Op(Op::new("::", sep)));
            toks.push(Tok::Ident(n));
        }
        let mut args = None;
        if self.peek_op_starts("<") {
            // Generic arguments are types (so `Vec<'a>` has a type variable)
            // when they parse as such; anything else is kept as Rust.
            let (save_pos, save_toks) = (self.pos, self.toks.clone());
            let marks_len = self.marks.borrow().len();
            match self.parse_generic_args() {
                Ok(a) => args = Some(a),
                Err(_) => {
                    self.pos = save_pos;
                    self.toks = save_toks;
                    self.marks.borrow_mut().truncate(marks_len);
                    toks.extend(lex(self.collect_generics()?));
                }
            }
        }
        Ok(Type::Path {
            tokens: to_stream(&toks),
            args,
        })
    }

    /// `<a, 'b, Item = c>` after a type path.
    fn parse_generic_args(&mut self) -> R<Vec<GenericArg>> {
        self.split_op_prefix("<");
        self.expect_op("<")?;
        let mut args = Vec::new();
        loop {
            if self.split_op_prefix(">") {
                self.pos += 1;
                return Ok(args);
            }
            let arg = match (self.peek().cloned(), self.peek_at(1)) {
                (Some(Tok::TyVar(l, sp)), _) if l == "static" => {
                    self.pos += 1;
                    GenericArg::Lifetime(l, sp)
                }
                (Some(Tok::Ident(name)), Some(Tok::Op(o))) if o.text == "=" => {
                    self.pos += 2;
                    GenericArg::Binding(name, self.parse_type()?)
                }
                _ => GenericArg::Type(self.parse_type()?),
            };
            args.push(arg);
            if !self.eat_op(",") && !self.peek_op_starts(">") {
                return self.err("expected `,` or `>`");
            }
        }
    }
}

/// Whether a token can be part of a Rust use tree: names, `::`, `*`, `,`
/// and `{ .. }` holding such tokens.
fn use_tree_token(t: &Tok) -> bool {
    match t {
        Tok::Ident(_) => true,
        Tok::Group(g) => {
            g.delimiter() == Delimiter::Brace && lex(g.stream()).iter().all(use_tree_token)
        }
        Tok::Op(o) => o.text.chars().all(|c| matches!(c, ':' | '*' | ',')),
        Tok::Lit(_) | Tok::TyVar(..) => false,
    }
}

/// The first token of a closure: `|` (`|&` in `|&x|`, which Rust glues
/// together) or `||`.
fn closure_bar(op: &str) -> bool {
    op == "||"
        || op
            .strip_prefix('|')
            .is_some_and(|rest| rest.chars().all(|c| c == '&'))
}

/// A closure where an argument or operand starts (`f |x| e`, `&|x| e`).
fn closure_paren_error(span: Span) -> Error {
    Error::new(
        span,
        "a closure here goes in parentheses, as `fun` does: `f (|x| e)`",
    )
}

/// The range operator an operator token starts with: `..`, or `..=` (Rust
/// glues `..` to a following operator character, as in `..-1`; `...` is
/// not a range).
fn range_op(text: &str) -> Option<&'static str> {
    if text.starts_with("..=") {
        Some("..=")
    } else if text.starts_with("..") && !text.starts_with("...") {
        Some("..")
    } else {
        None
    }
}

/// Macros whose arguments are OCaml expressions: std's formatting,
/// assertion and panicking macros, `dbg!` and `vec!`.
const STD_MACROS: &[&str] = &[
    "print",
    "println",
    "eprint",
    "eprintln",
    "format",
    "format_args",
    "write",
    "writeln",
    "panic",
    "unreachable",
    "todo",
    "unimplemented",
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "dbg",
    "vec",
];

/// `println`, `std::println` or `::core::assert_eq` (`std`, `core` or
/// `alloc`), without generic arguments.
fn is_std_macro(p: &Path) -> bool {
    let named = |seg: &PathSeg, names: &[&str]| {
        seg.generics.is_none() && names.contains(&seg.ident.to_string().as_str())
    };
    match &p.segs[..] {
        [m] => !p.leading_colon && named(m, STD_MACROS),
        [krate, m] => named(krate, &["std", "core", "alloc"]) && named(m, STD_MACROS),
        _ => false,
    }
}

/// A string literal (a format string): `"..."` or `r"..."`, `r#"..."#`.
fn is_string(l: &Literal) -> bool {
    let s = l.to_string();
    s.starts_with('"') || s.starts_with("r\"") || s.starts_with("r#")
}

/// `&`, `&&`, `&&&`, ..: references, glued together by Rust's lexer.
fn is_amps(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c == '&')
}

/// The fields of `t.0` or `t.0.1` (which Rust lexes as `t`, `.`, `0.1`).
fn tuple_index(l: &Literal) -> R<Vec<(u32, Span)>> {
    let text = l.to_string();
    let mut idx = Vec::new();
    for part in text.split('.') {
        match part.parse::<u32>() {
            Ok(n) => idx.push((n, l.span())),
            Err(_) => return Err(Error::new(l.span(), "expected a tuple index")),
        }
    }
    Ok(idx)
}

/// A sequence `a; b`, perhaps a function of its holes.
fn is_seq(e: &Expr) -> bool {
    match e {
        Expr::Seq(..) => true,
        Expr::Section { body, .. } => is_seq(body),
        _ => false,
    }
}

fn rust_let_error(b: &Binding) -> Error {
    Error::new(
        b.span,
        "this reads as Rust's `let x = e;`, but after `=` a `;` makes the rest part of the value: \
         write `let x = e in ..`",
    )
}

/// Where a token ends: the last character of an operator, the closing
/// bracket of a group.
fn tok_end(t: &Tok) -> Span {
    match t {
        Tok::Op(o) => o.last_span(),
        Tok::Group(g) => g.span_close(),
        t => t.span(),
    }
}

/// Whether `a :: b` is written with no space on either side of `::`.
fn tight(a_end: Span, op: &Op, b: &Tok) -> Option<bool> {
    let l = adjacent(a_end, op.span())?;
    let r = adjacent(op.last_span(), b.span())?;
    Some(l && r)
}

impl Parser {
    fn binop(&self, op: Op, lhs: Expr, rhs: Expr) -> Expr {
        // The operands of `|>`, `@@`, `>>` and `<<` are functions and their
        // arguments: each is a full expression for holes.
        let (lhs, rhs) = match op.text.as_str() {
            "|>" | "@@" | ">>" | "<<" => (self.close_holes(lhs), self.close_holes(rhs)),
            _ => (lhs, rhs),
        };
        if self.surface {
            return Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        if matches!(op.text.as_str(), "|>" | "@@" | ">>" | "<<") {
            self.unmapped(&op.spans);
        }
        desugar_binop(op, lhs, rhs)
    }
}

fn desugar_binop(op: Op, lhs: Expr, rhs: Expr) -> Expr {
    let span = op.span();
    match op.text.as_str() {
        "|>" => Expr::App(Box::new(rhs), vec![lhs]),
        "@@" => Expr::App(Box::new(lhs), vec![rhs]),
        ">>" => Expr::Compose {
            first: Box::new(lhs),
            second: Box::new(rhs),
            span,
        },
        "<<" => Expr::Compose {
            first: Box::new(rhs),
            second: Box::new(lhs),
            span,
        },
        _ => Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    }
}

/// The contents of a plain (non-raw) string literal, without unescaping.
/// Only used for `external` paths, which never contain escapes.
fn unquote(s: &str) -> String {
    s.trim_matches('"').to_string()
}

#[cfg(test)]
mod tests {
    /// The word lists are searched by bisection.
    #[test]
    fn word_lists_are_sorted() {
        for list in [super::KEYWORDS, super::RUST_KEYWORDS, super::RESERVED] {
            assert!(list.windows(2).all(|w| w[0] < w[1]), "{:?}", list);
        }
    }
}

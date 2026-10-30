//! Formats every `ml!` block and `rsml!` function in this repository (tests,
//! examples, README) under many configurations, and checks that formatting
//! is idempotent and that every formatted block expands to exactly the same
//! Rust as the original.
//!
//! The toy examples (examples/toy) are many, so `cargo test` formats each
//! under the default config and one other, in turn; `CORPUS_ALL=1` formats
//! them under every config, as scripts/check_all.sh does:
//!
//! ```sh
//! CORPUS_ALL=1 cargo test --release -p rsml-fmt --test corpus
//! ```

use std::path::{Path, PathBuf};

use rsml_fmt::{Block, Config, blocks, format_source};

/// The `.rs` files under `dir`, in its subdirectories too, sorted.
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<_> = entries.map(|e| e.unwrap().path()).collect();
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

/// A block's expansion, without the editor markers (see
/// [`rsml_fmt::expansion_key`]).
fn expansion(b: &Block) -> String {
    rsml_fmt::expansion_key(b.group.stream(), b)
}

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

/// The first differing line, with context.
fn first_difference(a: &str, b: &str) -> String {
    let (al, bl): (Vec<_>, Vec<_>) = (a.lines().collect(), b.lines().collect());
    let i = al
        .iter()
        .zip(&bl)
        .position(|(x, y)| x != y)
        .unwrap_or(al.len().min(bl.len()));
    let lo = i.saturating_sub(3);
    format!(
        "--- first pass, line {}\n{}\n--- second pass\n{}",
        i + 1,
        al[lo..(i + 4).min(al.len())].join("\n"),
        bl[lo..(i + 4).min(bl.len())].join("\n")
    )
}

/// The configs `name` is formatted under (`toy` counts the toy files).
fn configs_for(name: &str, toy: &mut usize) -> Vec<&'static str> {
    if !name.contains("/examples/toy/") || std::env::var_os("CORPUS_ALL").is_some() {
        return CONFIGS.to_vec();
    }
    *toy += 1;
    vec![CONFIGS[0], CONFIGS[1 + *toy % (CONFIGS.len() - 1)]]
}

#[test]
fn formatting_keeps_meaning_and_is_idempotent() {
    let corpus = corpus();
    assert!(corpus.len() > 5);
    let mut blocks_seen = 0;
    let mut toy = 0;
    for (name, src) in &corpus {
        let before: Vec<String> = blocks(src).unwrap().iter().map(expansion).collect();
        blocks_seen += before.len();
        for overrides in configs_for(name, &mut toy) {
            let cfg = Config::default().with_overrides(overrides).unwrap();
            let once = format_source(src, &cfg).unwrap();
            // Only blocks the macro rejects too are left unformatted.
            let rejected = before
                .iter()
                .filter(|e| e.contains("compile_error"))
                .count();
            assert_eq!(
                once.warnings.len(),
                rejected,
                "{} [{}]: {:?}",
                name,
                overrides,
                once.warnings
            );
            let after: Vec<String> = blocks(&once.text)
                .unwrap_or_else(|e| panic!("{} [{}]: {}\n{}", name, overrides, e, once.text))
                .iter()
                .map(expansion)
                .collect();
            assert_eq!(before.len(), after.len(), "{} [{}]", name, overrides);
            for (i, (a, b)) in before.iter().zip(&after).enumerate() {
                assert_eq!(
                    a, b,
                    "{} [{}]: block {} expands differently after formatting:\n{}",
                    name, overrides, i, once.text
                );
            }
            let twice = format_source(&once.text, &cfg).unwrap();
            assert!(
                once.text == twice.text,
                "{} [{}]: not idempotent\n{}",
                name,
                overrides,
                first_difference(&once.text, &twice.text)
            );
            for line in once.text.lines() {
                assert!(
                    !line.ends_with(' '),
                    "{} [{}]: trailing whitespace: {:?}",
                    name,
                    overrides,
                    line
                );
            }
        }
    }
    assert!(blocks_seen > 100, "only {} blocks", blocks_seen);
}

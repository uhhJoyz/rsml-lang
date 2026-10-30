//! A formatter for the OCaml in Rust files: `ml! { .. }` blocks and the
//! function bodies inside `rsml! { .. }`. rustfmt leaves macro bodies
//! alone, so this formats exactly those and nothing else.
//!
//! Blocks are parsed with the macros' own parser (in surface mode, which
//! keeps pipes, `begin .. end`, labels and parentheses as written), so the
//! formatter accepts what the macros accept, and a formatted block expands
//! to the same Rust as the original.
//!
//! Options come from `rsmlfmt.toml` (see [`Config`] and [`DEFAULT_TOML`]).
//! A comment at the start of a block can adjust them for that block:
//!
//! ```text
//! ml! {
//!     (* rsmlfmt: skip *)
//!     ..
//! }
//! ml! {
//!     (* rsmlfmt: max_width = 60; pipe.indent = align *)
//!     ..
//! }
//! ```
//!
//! At the start of an `rsml! { .. }` block, such a comment applies to every
//! function in it.

pub mod config;
mod doc;
mod printer;
pub mod rustfmt;
mod source;

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use rsml_compiler::lexer::lex;
use rsml_compiler::parser::Parser;

pub use config::{Config, DEFAULT_TOML};
use doc::{Opts, render};
use printer::Printer;
pub use rustfmt::Env;
use source::{Source, range};

/// A block that was left as it is.
#[derive(Debug, Clone)]
pub struct Warning {
    pub line: usize,
    pub message: String,
}

pub struct Formatted {
    pub text: String,
    pub warnings: Vec<Warning>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockKind {
    /// `ml! { .. }`
    Ml,
    /// A function body inside `rsml! { .. }`.
    FnBody,
}

/// A piece of OCaml in a Rust file.
#[derive(Clone)]
pub struct Block {
    pub kind: BlockKind,
    /// The delimited group holding the code.
    pub group: Group,
    /// Names in scope (parameters of an `rsml!` function).
    pub params: Vec<String>,
    /// For an `rsml!` function: its parameters with their shapes, and the
    /// functions of the same `rsml!` it can call, as the macro expands it.
    pub body: Option<rsml_compiler::items::FnBody>,
    /// `rsmlfmt:` directives of the enclosing `rsml!` block.
    inherited: Vec<Directive>,
}

#[derive(Clone, Debug, PartialEq)]
enum Directive {
    Skip,
    Set(String),
}

/// Formats every block in a Rust source file (the Rust inside `rsml!` too,
/// see [`config::RustItems`]), for a file of edition 2024 with no
/// rustfmt.toml. Blocks that do not parse are left unchanged and reported
/// as warnings.
pub fn format_source(src: &str, cfg: &Config) -> Result<Formatted, String> {
    format_source_in(src, cfg, &Env::default())
}

/// [`format_source`] for a file in `env` (its edition and rustfmt config).
pub fn format_source_in(src: &str, cfg: &Config, env: &Env) -> Result<Formatted, String> {
    let mut warnings = Vec::new();
    let rust = match cfg.rust_items {
        config::RustItems::Rustfmt => rust_items(src, cfg, env, &mut warnings)?,
        config::RustItems::Preserve => None,
    };
    let mut text = rust.unwrap_or_else(|| src.to_string());
    // A block's layout depends on the column it starts at, and formatting
    // a block moves the blocks after it on its last line (two `ml!`s in
    // one call): those are formatted again, until nothing moves.
    for round in 0.. {
        let (next, moved, block_warnings) = format_blocks(&text, cfg)?;
        text = next;
        if !moved || round == 3 {
            warnings.extend(block_warnings);
            break;
        }
    }
    Ok(Formatted { text, warnings })
}

/// One pass over the blocks of `src`: the new text, whether a changed
/// block moved a block after it on the same line, and the warnings.
fn format_blocks(src: &str, cfg: &Config) -> Result<(String, bool, Vec<Warning>), String> {
    let blocks = blocks(src)?;
    let mut warnings = Vec::new();
    let mut edits = Vec::new();
    for b in &blocks {
        match format_block(src, b, cfg) {
            Ok(Some(text)) => edits.push((range(b.group.span()), text)),
            Ok(None) => {}
            Err(message) => warnings.push(Warning {
                line: b.group.span().start().line,
                message,
            }),
        }
    }
    let moved = blocks.iter().any(|b| {
        let start = range(b.group.span()).start;
        let line_start = src[..start].rfind('\n').map_or(0, |i| i + 1);
        edits
            .iter()
            .any(|(r, new)| r.end <= start && r.end > line_start && src[r.clone()] != **new)
    });
    let mut text = src.to_string();
    for (r, new) in edits.into_iter().rev() {
        text.replace_range(r, &new);
    }
    Ok((text, moved, warnings))
}

/// The file with the Rust of its `rsml!` blocks formatted by rustfmt, or
/// `None` if nothing changes. A block rustfmt cannot format is left as it
/// is, with a warning (once, if rustfmt is missing).
fn rust_items(
    src: &str,
    cfg: &Config,
    env: &Env,
    warnings: &mut Vec<Warning>,
) -> Result<Option<String>, String> {
    let ts: TokenStream = src
        .parse()
        .map_err(|e: proc_macro2::LexError| format!("not valid Rust tokens: {:?}", e))?;
    let mut groups = Vec::new();
    rustfmt::rsml_groups(ts, &mut groups);
    let mut edits = Vec::new();
    for g in &groups {
        if directives(&Source::new(src, g)).contains(&Directive::Skip) {
            continue;
        }
        match rustfmt::rsml_block(src, g, cfg, env) {
            Ok(Some(text)) => edits.push((range(g.span()), text)),
            Ok(None) => {}
            Err(e) => {
                let missing = matches!(e, rustfmt::Failure::Missing(_));
                if !missing || !RUSTFMT_MISSING.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    warnings.push(Warning {
                        line: g.span().start().line,
                        message: format!("Rust in `rsml!` left as written: {}", e),
                    });
                }
                if missing {
                    break;
                }
            }
        }
    }
    if edits.is_empty() {
        return Ok(None);
    }
    let mut text = src.to_string();
    for (r, new) in edits.into_iter().rev() {
        text.replace_range(r, &new);
    }
    Ok(Some(text))
}

/// Whether a missing rustfmt was reported already.
static RUSTFMT_MISSING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Formats bare OCaml (the inside of a block), starting at column 0.
pub fn format_ml(code: &str, cfg: &Config) -> Result<String, String> {
    let mut cfg = cfg.clone();
    cfg.block_indent = 0;
    cfg.single_line_blocks = config::SingleLineBlocks::Never;
    let wrapped = format!("ml! {{\n{}\n}}", code);
    let out = format_source(&wrapped, &cfg)?;
    if let Some(w) = out.warnings.first() {
        return Err(w.message.clone());
    }
    let body = out
        .text
        .strip_prefix("ml! {\n")
        .and_then(|t| t.strip_suffix("\n}"))
        .unwrap_or(&out.text);
    Ok(body.to_string())
}

/// The OCaml blocks of a Rust source file, in order.
pub fn blocks(src: &str) -> Result<Vec<Block>, String> {
    let ts: TokenStream = src
        .parse()
        .map_err(|e: proc_macro2::LexError| format!("not valid Rust tokens: {:?}", e))?;
    let mut out = Vec::new();
    find_blocks(src, ts, &mut out);
    Ok(out)
}

fn find_blocks(src: &str, ts: TokenStream, out: &mut Vec<Block>) {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut i = 0;
    while i < toks.len() {
        // The blocks in a `macro_rules!` are templates (`ml! { $e + 1 }`),
        // made into code only where the macro is used: left alone.
        if let (
            TokenTree::Ident(kw),
            Some(TokenTree::Punct(bang)),
            Some(TokenTree::Ident(_)),
            Some(TokenTree::Group(_)),
        ) = (&toks[i], toks.get(i + 1), toks.get(i + 2), toks.get(i + 3))
            && kw == "macro_rules"
            && bang.as_char() == '!'
        {
            i += 4;
            continue;
        }
        if let (TokenTree::Ident(name), Some(TokenTree::Punct(bang)), Some(TokenTree::Group(g))) =
            (&toks[i], toks.get(i + 1), toks.get(i + 2))
            && bang.as_char() == '!'
            && (name == "ml" || name == "rsml")
        {
            if name == "ml" {
                out.push(Block {
                    kind: BlockKind::Ml,
                    group: g.clone(),
                    params: Vec::new(),
                    body: None,
                    inherited: Vec::new(),
                });
            } else {
                let inherited = directives(&Source::new(src, g));
                for f in rsml_compiler::items::fn_bodies(g.stream()) {
                    out.push(Block {
                        kind: BlockKind::FnBody,
                        group: f.body.clone(),
                        params: f.params.iter().map(|(n, _)| n.clone()).collect(),
                        body: Some(f),
                        inherited: inherited.clone(),
                    });
                }
            }
            i += 3;
            continue;
        }
        if let TokenTree::Group(g) = &toks[i] {
            find_blocks(src, g.stream(), out);
        }
        i += 1;
    }
}

/// `rsmlfmt:` comments before the first token of a block.
fn directives(source: &Source) -> Vec<Directive> {
    let first = source.first_code();
    source
        .comments
        .iter()
        .filter(|c| c.range.start < first)
        .filter_map(|c| {
            let text = source.slice(c.range.clone());
            let inner = text
                .strip_prefix("(*")
                .and_then(|t| t.strip_suffix("*)"))
                .or_else(|| text.strip_prefix("/*").and_then(|t| t.strip_suffix("*/")))
                .or_else(|| text.strip_prefix("//"))?;
            let rest = inner.trim().strip_prefix("rsmlfmt:")?.trim();
            Some(if rest == "skip" {
                Directive::Skip
            } else {
                Directive::Set(rest.to_string())
            })
        })
        .collect()
}

/// The new text of a block's group, or `None` to leave it alone.
fn format_block(src: &str, b: &Block, cfg: &Config) -> Result<Option<String>, String> {
    let Some(text) = layout_block(src, b, cfg)? else {
        return Ok(None);
    };
    // A safety net: formatting never changes what the block means. A
    // result that would expand to different Rust is a formatter bug, and
    // the block is left as written.
    let new: TokenStream = text
        .parse()
        .map_err(|_| "not formatted: the result would not lex (a formatter bug)".to_string())?;
    let new = match new.into_iter().next() {
        Some(TokenTree::Group(g)) => g.stream(),
        _ => TokenStream::new(),
    };
    if expansion_key(b.group.stream(), b) != expansion_key(new, b) {
        return Err(
            "not formatted: the result would expand differently (a formatter bug)".to_string(),
        );
    }
    Ok(Some(text))
}

/// A block's expansion as text, without the markers that only tell editors
/// how to color keywords and operators (formatting may spell `begin .. end`
/// as `( .. )` or add a leading `|`, which changes them).
#[doc(hidden)]
pub fn expansion_key(ts: TokenStream, b: &Block) -> String {
    let ts = match b.kind {
        BlockKind::Ml => rsml_compiler::expand(ts),
        BlockKind::FnBody => match &b.body {
            Some(f) => rsml_compiler::expand_in(ts, &f.params, f.known.clone()),
            None => {
                let params: Vec<_> = b.params.iter().map(|p| (p.clone(), None)).collect();
                rsml_compiler::expand_with(ts, &params)
            }
        },
    };
    let mut ts = without_markers(ts);
    // The markers come with a block around the expansion.
    loop {
        let mut it = ts.clone().into_iter();
        match (it.next(), it.next()) {
            (Some(TokenTree::Group(g)), None) if g.delimiter() == Delimiter::Brace => {
                ts = g.stream()
            }
            _ => break,
        }
    }
    ts.to_string()
}

/// Drops the `const _: usize = ..;` and `const _: () = { .. };` markers.
fn without_markers(ts: TokenStream) -> TokenStream {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut out = TokenStream::new();
    let mut i = 0;
    while i < toks.len() {
        let marker = matches!(
            (&toks[i], toks.get(i + 1)),
            (TokenTree::Ident(c), Some(TokenTree::Ident(u))) if c == "const" && u == "_"
        );
        if marker {
            while i < toks.len() && !matches!(&toks[i], TokenTree::Punct(p) if p.as_char() == ';') {
                i += 1;
            }
            i += 1;
            continue;
        }
        out.extend([match &toks[i] {
            TokenTree::Group(g) => {
                let mut n = Group::new(g.delimiter(), without_markers(g.stream()));
                n.set_span(g.span());
                TokenTree::Group(n)
            }
            t => t.clone(),
        }]);
        i += 1;
    }
    out
}

/// The new text of a block's group, or `None` to leave it alone.
fn layout_block(src: &str, b: &Block, cfg: &Config) -> Result<Option<String>, String> {
    let source = Source::new(src, &b.group);
    let mut cfg = cfg.clone();
    for d in b.inherited.iter().chain(&directives(&source)) {
        match d {
            Directive::Skip => return Ok(None),
            Directive::Set(text) => {
                cfg = cfg
                    .with_overrides(text)
                    .map_err(|e| format!("bad `rsmlfmt:` comment: {}", e))?;
            }
        }
    }
    let mut parser = Parser::new(lex(b.group.stream()), b.group.span_close());
    parser.set_surface();
    parser.bind_names(b.params.iter().cloned());
    let phrases = parser.parse_phrases().map_err(|e| {
        format!(
            "not formatted, does not parse: {} (line {})",
            e.msg,
            e.span.start().line
        )
    })?;
    if phrases.is_empty() && source.comments.is_empty() {
        return Ok(None);
    }
    let mut printer = Printer::new(&cfg, &source);
    let body = printer.phrases(&phrases);
    let doc = printer.with_rest_comments(body);

    let (open, close) = match b.group.delimiter() {
        Delimiter::Parenthesis => ("(", ")"),
        Delimiter::Bracket => ("[", "]"),
        _ => ("{", "}"),
    };
    let open_r = range(b.group.span_open());
    let close_r = range(b.group.span_close());
    let line_start = src[..open_r.start].rfind('\n').map_or(0, |i| i + 1);
    let line_indent: String = src[line_start..]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    let opts = |indent: usize| Opts {
        width: cfg.max_width,
        indent,
        start_col: indent,
        max_blank_lines: cfg.max_blank_lines,
    };

    let one_line = match b.kind {
        BlockKind::Ml => match cfg.single_line_blocks {
            config::SingleLineBlocks::WhenFits => true,
            config::SingleLineBlocks::Preserve => !src[open_r.start..close_r.end].contains('\n'),
            config::SingleLineBlocks::Never => false,
        },
        BlockKind::FnBody => cfg.fn_single_line,
    };
    if one_line && !doc.has_hard() {
        let inner = render(&doc, &opts(0), true);
        let pad = if open == "{" { " " } else { "" };
        let suffix = src[close_r.end..].split('\n').next().unwrap_or("");
        let width = source.column(open_r.start)
            + 2
            + 2 * pad.len()
            + inner.chars().count()
            + suffix.trim_end().chars().count();
        if width <= cfg.max_width {
            return Ok(Some(format!("{open}{pad}{inner}{pad}{close}")));
        }
    }

    let body_indent = line_indent.chars().count() + cfg.block_indent;
    let body = render(&doc, &opts(body_indent), false);
    let mut text = format!(
        "{open}\n{}{body}\n{line_indent}{close}",
        " ".repeat(body_indent)
    );
    if src.contains("\r\n") {
        text = text.replace('\n', "\r\n");
    }
    Ok(Some(text))
}

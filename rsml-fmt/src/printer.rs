//! Lays out a surface syntax tree as a [`Doc`], following the [`Config`].
//!
//! The tree comes from the macros' own parser in surface mode, so it keeps
//! the source's parentheses and the formatter never has to decide where
//! they are needed: printing a tree and parsing the result gives the same
//! tree. Docs are built in source order, which is what lets comments be
//! emitted at the first token after them.

use std::ops::Range;

use proc_macro2::{Ident, Span};
use rsml_compiler::ast::*;
use rsml_compiler::lexer::{Op, is_capitalized};
use rsml_compiler::parser::infix_level;
use rsml_compiler::types::{GenericArg, Type};

use crate::config::*;
use crate::doc::*;
use crate::source::{Source, range, stream_range};

pub struct Printer<'a> {
    cfg: &'a Config,
    src: &'a Source<'a>,
    /// Comments already printed.
    done: Vec<bool>,
    next_id: usize,
    /// The `&` (by position) that starts an application's argument, where
    /// the parser reads `&&` as `a && b`: `f & &x` keeps its space.
    arg_amp: Option<usize>,
    /// The `match`es and `function`s (by position) that end a case before
    /// another, without parentheses: their cases go one per line.
    vertical: Vec<usize>,
}

fn start_of(span: Span) -> usize {
    range(span).start
}

fn end_of(span: Span) -> usize {
    range(span).end
}

/// Where an expression starts in the source.
fn expr_start(e: &Expr) -> usize {
    start_of(match e {
        Expr::Var(i) | Expr::Tmp(i, _) => i.span(),
        Expr::Path(p) => p.segs[0].ident.span(),
        Expr::Lit(l) => l.span(),
        Expr::Bool(_, s)
        | Expr::Unit(s)
        | Expr::List(_, s)
        | Expr::Array(_, s)
        | Expr::Paren(_, s)
        | Expr::Deref(_, s)
        | Expr::Assert(_, s) => *s,
        Expr::Tuple(es) => return expr_start(&es[0]),
        Expr::Range { lo: Some(e), .. } => return expr_start(e),
        Expr::Range { lo: None, op, .. } => op.span(),
        Expr::App(e, _)
        | Expr::Call { func: e, .. }
        | Expr::Seq(e, _)
        | Expr::Binary { lhs: e, .. }
        | Expr::Compose { first: e, .. }
        | Expr::Field(e, _)
        | Expr::TupleIndex(e, _)
        | Expr::Try(e, _)
        | Expr::Await(e, _)
        | Expr::Constraint(e, _)
        | Expr::Cast(e, _)
        | Expr::Index(e, _, _)
        | Expr::StrIndex(e, _)
        | Expr::Assign(e, _, _)
        | Expr::RefAssign(e, _, _)
        | Expr::Method { recv: e, .. } => return expr_start(e),
        Expr::Fun { span, .. }
        | Expr::Function { span, .. }
        | Expr::Let { span, .. }
        | Expr::External { span, .. }
        | Expr::Use { span, .. }
        | Expr::If { span, .. }
        | Expr::Match { span, .. }
        | Expr::For { span, .. }
        | Expr::ForIn { span, .. }
        | Expr::Jump { span, .. }
        | Expr::RustBlock { span, .. }
        | Expr::While { span, .. }
        | Expr::RecordUpdate { span, .. }
        | Expr::Borrow { span, .. } => *span,
        Expr::Neg(_, op) | Expr::OpSection(op) => op.span(),
        Expr::Record { ctor: p, .. }
        | Expr::Macro { path: p, .. }
        | Expr::StdMacro { path: p, .. } => p.segs[0].ident.span(),
        Expr::Rust(g) => g.span(),
        Expr::Hole { span, .. } | Expr::MethodRef { span, .. } | Expr::FieldRef { span, .. } => {
            *span
        }
        Expr::Section { body, .. } => return expr_start(body),
        Expr::Surface(s) => match &**s {
            Surface::Begin { begin, .. } => *begin,
            Surface::Label { span, .. } => *span,
        },
        Expr::Labeled { span, .. } | Expr::TailCall { span, .. } => *span,
    })
}

/// Where a path ends in the source, generic arguments included.
fn path_end(p: &Path) -> usize {
    let seg = p.segs.last().unwrap();
    match seg.generics.as_ref().and_then(stream_range) {
        Some(r) => r.end,
        None => end_of(seg.ident.span()),
    }
}

/// Where a constructor applied like a function ends: `Some`,
/// `Shape::Rect` or `(Some)`.
fn ctor_end(e: &Expr) -> Option<usize> {
    let ctor = |e: &Expr| matches!(e, Expr::Path(p) if is_capitalized(p.last()));
    match e {
        Expr::Path(p) if ctor(e) => Some(path_end(p)),
        Expr::Paren(inner, sp) if ctor(inner.strip()) => Some(end_of(*sp)),
        _ => None,
    }
}

/// Where a pattern starts in the source, when known.
fn pat_start(p: &Pat) -> Option<usize> {
    Some(start_of(match p {
        Pat::Wild(s) | Pat::Bool(_, s) | Pat::Unit(s) | Pat::List(_, s) | Pat::Array(_, s) => *s,
        Pat::Ref(_, s) | Pat::Label { span: s, .. } => *s,
        Pat::Var(i) => i.span(),
        Pat::Lit { lit, neg: false } => lit.span(),
        Pat::Ctor(path, _) | Pat::CtorRecord(path, _, _) => path.segs[0].ident.span(),
        Pat::Paren(p) | Pat::Cons(p, _, _) | Pat::Alias(p, _) | Pat::Range(p, _, _) => {
            return pat_start(p);
        }
        Pat::Constraint(p, _) => return pat_start(p),
        Pat::Tuple(ps) | Pat::Or(ps) => return ps.first().and_then(pat_start),
        Pat::Mut(_) | Pat::Lit { .. } => return None,
    }))
}

fn phrase_start(p: &Phrase) -> usize {
    match p {
        Phrase::Let { span, .. } | Phrase::External { span, .. } | Phrase::Use { span, .. } => {
            start_of(*span)
        }
        Phrase::Expr(e) => expr_start(e),
    }
}

/// `fun` / `function` (not in parentheses).
fn is_lambda(e: &Expr) -> bool {
    matches!(e, Expr::Fun { .. } | Expr::Function { .. })
}

/// Whether `text` starts with a closing bracket, `end` or `done`.
fn closes(text: &str) -> bool {
    if text.starts_with([')', ']', '}']) || text.starts_with("|]") {
        return true;
    }
    ["end", "done"].iter().any(|kw| {
        text.strip_prefix(kw)
            .is_some_and(|rest| !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
    })
}

/// The path that `e`'s text starts with, perhaps in a tuple's first item.
fn leading_path(e: &Expr) -> Option<&Path> {
    match leftmost(e) {
        Expr::Path(p) => Some(p),
        Expr::Tuple(es) => leading_path(&es[0]),
        _ => None,
    }
}

/// The subexpression whose text starts `e`'s (`x` in `x.f() + 1`).
fn leftmost(e: &Expr) -> &Expr {
    match e {
        Expr::Range { lo: Some(e), .. }
        | Expr::App(e, _)
        | Expr::Call { func: e, .. }
        | Expr::Seq(e, _)
        | Expr::Binary { lhs: e, .. }
        | Expr::Compose { first: e, .. }
        | Expr::Field(e, _)
        | Expr::TupleIndex(e, _)
        | Expr::Try(e, _)
        | Expr::Await(e, _)
        | Expr::Cast(e, _)
        | Expr::Index(e, _, _)
        | Expr::StrIndex(e, _)
        | Expr::Assign(e, _, _)
        | Expr::RefAssign(e, _, _)
        | Expr::Method { recv: e, .. } => leftmost(e),
        Expr::Section { body, .. } => leftmost(body),
        e => e,
    }
}

/// A character that continues an OCaml operator.
fn is_op_char(c: char) -> bool {
    "!$%&*+-./:<=>?@^|~#".contains(c)
}

/// The first character a pattern prints, when it is an operator's.
fn pat_first_char(p: &Pat) -> Option<char> {
    match p {
        Pat::Lit { neg: true, .. } => Some('-'),
        Pat::Ref(..) => Some('&'),
        Pat::Label { .. } => Some('~'),
        Pat::Ctor(path, _) | Pat::CtorRecord(path, _, _) if path.leading_colon => Some(':'),
        Pat::Range(p, _, _) | Pat::Cons(p, _, _) | Pat::Alias(p, _) | Pat::Constraint(p, _) => {
            pat_first_char(p)
        }
        Pat::Or(ps) => ps.first().and_then(pat_first_char),
        _ => None,
    }
}

/// Whether the expression's text starts with an operator character (or
/// the `.` of a method), which would merge with a preceding `[|`, `-`, ...
fn starts_with_op(e: &Expr) -> bool {
    match e {
        Expr::Neg(..)
        | Expr::Deref(..)
        | Expr::Borrow { .. }
        | Expr::MethodRef { .. }
        | Expr::FieldRef { .. } => true,
        // `|x| e`
        Expr::Fun {
            bars: Some(_),
            is_move: false,
            ..
        } => true,
        Expr::Surface(s) => matches!(**s, Surface::Label { .. }),
        Expr::Range { lo: None, .. } => true,
        Expr::Range { lo: Some(e), .. } => starts_with_op(e),
        Expr::App(e, _)
        | Expr::Call { func: e, .. }
        | Expr::Seq(e, _)
        | Expr::Binary { lhs: e, .. }
        | Expr::Field(e, _)
        | Expr::TupleIndex(e, _)
        | Expr::Try(e, _)
        | Expr::Await(e, _)
        | Expr::Cast(e, _)
        | Expr::Index(e, _, _)
        | Expr::StrIndex(e, _)
        | Expr::Assign(e, _, _)
        | Expr::RefAssign(e, _, _)
        | Expr::Method { recv: e, .. } => starts_with_op(e),
        Expr::Tuple(es) => starts_with_op(&es[0]),
        Expr::Section { body, .. } => starts_with_op(body),
        _ => false,
    }
}

/// Whether the expression's text ends with an operator character, which
/// would merge with a following `|]`.
fn ends_with_op(e: &Expr) -> bool {
    match e {
        Expr::Try(..) | Expr::Cast(..) | Expr::Range { hi: None, .. } => true,
        _ => tail(e).is_some_and(ends_with_op),
    }
}

/// Whether `e` would take the phrase after it as its last operand: it ends
/// with an open range (`x..`) or a `return` / `break` with no value, so
/// `let a = x..` then `let b = ..` needs the `;;` between them.
fn takes_next(e: &Expr) -> bool {
    match e {
        Expr::Range { hi: None, .. } | Expr::Jump { value: None, .. } => true,
        _ => tail(e).is_some_and(takes_next),
    }
}

/// The `match` or `function` that `e` ends with, whose cases a `|` after
/// `e` could continue.
fn tail_cases(e: &Expr) -> Option<&Expr> {
    match e {
        Expr::Match { .. } | Expr::Function { .. } => Some(e),
        _ => tail(e).and_then(tail_cases),
    }
}

/// The expression that `e` ends with, when `e` ends with one of its own.
fn tail(e: &Expr) -> Option<&Expr> {
    match e {
        Expr::App(_, args) => args.last(),
        Expr::Binary { rhs: e, .. }
        | Expr::Seq(_, e)
        | Expr::Neg(e, _)
        | Expr::Deref(e, _)
        | Expr::Borrow { expr: e, .. }
        | Expr::Assign(_, e, _)
        | Expr::RefAssign(_, e, _) => Some(e),
        Expr::Tuple(es) => es.last(),
        Expr::Section { body, .. } => Some(body),
        // Constructs that end with an expression of theirs.
        Expr::Fun { body: e, .. } | Expr::Let { body: e, .. } => Some(e),
        Expr::External { body: e, .. } | Expr::Use { body: e, .. } => Some(e),
        Expr::If { then, els, .. } => Some(els.as_deref().unwrap_or(then)),
        Expr::Function { arms, .. } | Expr::Match { arms, .. } => arms.last().map(|a| &a.body),
        Expr::Assert(e, _) => Some(e),
        Expr::Range { hi: Some(e), .. } | Expr::Jump { value: Some(e), .. } => Some(e),
        _ => None,
    }
}

impl<'a> Printer<'a> {
    pub fn new(cfg: &'a Config, src: &'a Source<'a>) -> Printer<'a> {
        Printer {
            cfg,
            src,
            done: vec![false; src.comments.len()],
            next_id: 0,
            arg_amp: None,
            vertical: Vec::new(),
        }
    }

    fn id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    fn colon(&self) -> &'static str {
        match self.cfg.colon_spacing {
            ColonSpacing::Both => " : ",
            ColonSpacing::After => ": ",
        }
    }

    // ----- comments ------------------------------------------------------

    /// Comments before `pos` that are not printed yet. With `blank_ok` (at
    /// places that are line breaks anyway) blank lines before them and
    /// before `pos` are kept.
    fn comments_before(&mut self, pos: usize, blank_ok: bool) -> Doc {
        let blank_ok = blank_ok && self.cfg.preserve_blank_lines;
        let mut out = Vec::new();
        for i in 0..self.src.comments.len() {
            let start = self.src.comments[i].range.start;
            if start >= pos {
                break;
            }
            if !self.done[i] {
                self.done[i] = true;
                out.push(self.comment(i, blank_ok));
            }
        }
        if blank_ok && pos < self.src.text.len() {
            let prev = self.src.prev_end(pos);
            let n = self.src.newlines(prev, pos);
            if !self.src.is_block_start(prev) && n >= 2 {
                out.push(Doc::BlankLines(n - 1));
            }
        }
        concat(out)
    }

    /// A comment is kept with the code that follows it on its line
    /// (`(* c *) x`), else on its own line if it was, else at the end of
    /// the line it followed code on.
    fn comment(&self, i: usize, blank_ok: bool) -> Doc {
        let c = &self.src.comments[i];
        let (text, line_comment) = self.comment_text(i);
        let prev = self.src.prev_end(c.range.start);
        let newlines = self.src.newlines(prev, c.range.start);
        let rest = self.src.text[c.range.end..]
            .split('\n')
            .next()
            .unwrap_or("")
            .trim_start();
        // A closing bracket (or `end`, `done`) is not code the comment
        // belongs to: the comment stays after what it followed.
        let next = self.src.text[c.range.end..].trim_start_matches([' ', '\t']);
        let at_close = self.src.text.len() - next.len() == self.src.block_close() || closes(next);
        let code_follows = !at_close
            && !rest.is_empty()
            && !rest.starts_with("//")
            && !rest.starts_with("/*")
            && !rest.starts_with("(*");
        let first_line = !text.contains('\n');
        if first_line && code_follows && !line_comment {
            // After code on its line, it is spaced from it like a trailing
            // comment (`f (* c *) .0`, not `f(* c *) .0`).
            if newlines == 0 && !self.src.is_block_start(prev) {
                return Doc::Inline(text);
            }
            return concat(vec![text_doc(&text), text_doc(" ")]);
        }
        if !first_line || newlines > 0 || self.src.is_block_start(prev) {
            return Doc::OwnLine {
                blank_before: if blank_ok && !self.src.is_block_start(prev) {
                    newlines.saturating_sub(1)
                } else {
                    0
                },
                src_col: self.src.column(c.range.start),
                text,
            };
        }
        Doc::Trailing { text, line_comment }
    }

    /// The text of comment `i`, and whether it is a `//` comment, which a
    /// line break must follow. With `comments = "rust"`, `(* c *)` is
    /// rewritten: as `// c` when it has a line to itself (and no line
    /// break inside), else as `/* c */`, so the layout stays as it was. A
    /// space after `//` or `/*` keeps it from being a doc comment
    /// (`(** c *)` is `/* * c */`), and a comment holding `/*` or `*/`
    /// stays as it is.
    fn comment_text(&self, i: usize) -> (String, bool) {
        let c = &self.src.comments[i];
        let text = self.src.slice(c.range.clone());
        if self.cfg.comments == Comments::Preserve || !text.starts_with("(*") {
            return (text.to_string(), c.line_comment);
        }
        let inner = &text[2..text.len() - 2];
        let rest_of_line = self.src.text[c.range.end..]
            .split('\n')
            .next()
            .unwrap_or("");
        let line_start = self.src.text[..c.range.start]
            .rfind('\n')
            .map_or(0, |k| k + 1);
        let before = &self.src.text[line_start..c.range.start];
        let doc_like = |s: &str| s.starts_with(['*', '!', '/']);
        if !inner.contains('\n') && rest_of_line.trim().is_empty() && before.trim().is_empty() {
            let pad = if doc_like(inner) { " " } else { "" };
            return (format!("//{}{}", pad, inner.trim_end()), true);
        }
        if inner.contains("/*") || inner.contains("*/") {
            return (text.to_string(), false);
        }
        let pad = if doc_like(inner) { " " } else { "" };
        (format!("/*{}{}*/", pad, inner), false)
    }

    /// Comments right after `end` on the same line.
    fn trailing_after(&mut self, end: usize) -> Doc {
        let mut out = Vec::new();
        let mut at = end;
        for i in 0..self.src.comments.len() {
            let c = &self.src.comments[i];
            if self.done[i] || c.range.start < at {
                continue;
            }
            let gap = &self.src.text[at..c.range.start];
            if !gap.chars().all(|ch| ch == ' ' || ch == '\t')
                || self.src.slice(c.range.clone()).contains('\n')
            {
                break;
            }
            self.done[i] = true;
            let (text, line_comment) = self.comment_text(i);
            out.push(Doc::Trailing { text, line_comment });
            at = c.range.end;
        }
        concat(out)
    }

    /// The block's `body` and the comments not printed yet, at its end. A
    /// comment that ended the last line of code after a `;;`, which is not
    /// printed at the end, ends that line, and is read back as part of the
    /// last phrase: it goes inside its groups (as between phrases).
    pub fn with_rest_comments(&mut self, body: Doc) -> Doc {
        let mut out = Vec::new();
        for i in 0..self.src.comments.len() {
            if !self.done[i] {
                self.done[i] = true;
                out.push(self.comment(i, self.cfg.preserve_blank_lines));
            }
        }
        let (trailing, rest) = split_trailing(concat(out));
        if matches!(trailing, Doc::Nil) {
            return concat(vec![body, rest]);
        }
        concat(vec![attach_trailing(body, trailing), rest])
    }

    /// Source text at `r` that is printed as is: comments inside it count
    /// as printed.
    fn verbatim(&mut self, r: Range<usize>, keep_lines: bool) -> Doc {
        let before = self.comments_before(r.start, false);
        for (i, c) in self.src.comments.iter().enumerate() {
            if c.range.start >= r.start && c.range.end <= r.end {
                self.done[i] = true;
            }
        }
        let text = self.src.slice(r.clone()).to_string();
        let body = if text.contains('\n') {
            Doc::Verbatim {
                text,
                src_col: (!keep_lines).then(|| self.src.column(r.start)),
            }
        } else {
            Doc::Text(text)
        };
        let after = self.trailing_after(r.end);
        concat(vec![before, body, after])
    }

    /// Marks the comments inside `r` as printed (they are in its text).
    fn mark_done(&mut self, r: Range<usize>) {
        for (i, c) in self.src.comments.iter().enumerate() {
            if c.range.start >= r.start && c.range.end <= r.end {
                self.done[i] = true;
            }
        }
    }

    /// A token: its text, with the comments around it.
    fn leaf(&mut self, r: Range<usize>, s: impl Into<String>) -> Doc {
        let before = self.comments_before(r.start, false);
        let after = self.trailing_after(r.end);
        concat(vec![before, text_doc(s), after])
    }

    /// `()`, with the comments written inside it kept there:
    /// `Some((* c *)) .f` is not `Some() (* c *) .f`.
    fn unit(&mut self, r: Range<usize>) -> Doc {
        if !self.src.text[r.clone()].starts_with('(') {
            return self.leaf(r, "()");
        }
        let before = self.comments_before(r.start, false);
        let inside = self.comments_before(r.end - 1, false);
        let after = self.trailing_after(r.end);
        concat(vec![before, text_doc("("), inside, text_doc(")"), after])
    }

    /// The `.` written against the field or method `name`, with the
    /// comments before it: in `x // c ⏎ .f`, `// c` stays before `.f`.
    fn dot_before(&mut self, name: &Ident) -> Doc {
        let at = start_of(name.span()).saturating_sub(1);
        self.leaf(at..at + 1, ".")
    }

    fn ident(&mut self, i: &Ident) -> Doc {
        self.leaf(range(i.span()), i.to_string())
    }

    fn keyword(&mut self, span: Span, kw: &str) -> Doc {
        let r = range(span);
        self.leaf(r.start..r.start + kw.len(), kw)
    }

    // ----- toplevel ------------------------------------------------------

    pub fn phrases(&mut self, phrases: &[Phrase]) -> Doc {
        let cfg = self.cfg.toplevel.clone();
        let always = cfg.break_ == Break::Always;
        let sep = if always { Doc::HardLine } else { Doc::Line };
        let mut out = Vec::new();
        let mut last = 0;
        for (i, ph) in phrases.iter().enumerate() {
            if i > 0 {
                // `;;` is needed before an expression, and after one or a
                // `let` that would take the next phrase (`let a = x..`); a
                // `use` ends with its own `;`.
                let semi = match cfg.double_semi {
                    _ if matches!(phrases[i - 1], Phrase::Use { .. }) => false,
                    DoubleSemi::Always => true,
                    DoubleSemi::Minimal => {
                        matches!(ph, Phrase::Expr(_))
                            || match &phrases[i - 1] {
                                Phrase::Expr(_) => true,
                                Phrase::Let { bindings, .. } => {
                                    bindings.last().is_some_and(|b| takes_next(&b.value))
                                }
                                _ => false,
                            }
                    }
                };
                if semi {
                    match cfg.double_semi_position {
                        DoubleSemiPosition::EndOfLine => out.push(text_doc(" ;;")),
                        DoubleSemiPosition::OwnLine => {
                            out.push(Doc::HardLine);
                            out.push(text_doc(";;"));
                        }
                    }
                }
                out.push(sep.clone());
                let start = match ph {
                    Phrase::Expr(e) => self.start(e),
                    ph => phrase_start(ph),
                };
                let blank = always && cfg.blank_lines == ToplevelBlankLines::Preserve;
                if always && cfg.blank_lines == ToplevelBlankLines::Always {
                    out.push(Doc::BlankLines(1));
                }
                let mut c = self.comments_before(start, blank);
                // With no `;;` printed, a comment that followed it (`f () ;;
                // // c`) ends the previous phrase's last line, and is read
                // back as part of it: it goes inside its groups.
                if !semi {
                    let (trailing, rest) = split_trailing(c);
                    if !matches!(trailing, Doc::Nil) {
                        let prev = std::mem::replace(&mut out[last], Doc::Nil);
                        out[last] = attach_trailing(prev, trailing);
                    }
                    c = rest;
                }
                out.push(c);
            }
            last = out.len();
            out.push(self.phrase(ph));
        }
        if always {
            concat(out)
        } else {
            group(concat(out))
        }
    }

    fn phrase(&mut self, ph: &Phrase) -> Doc {
        match ph {
            Phrase::Let {
                rec_,
                bindings,
                span,
            } => self.let_bindings(*rec_, bindings, *span).0,
            Phrase::External {
                name,
                ty,
                rust,
                span,
            } => self.external(name, ty, rust, *span),
            Phrase::Use { tree, span } => self.use_item(tree, *span),
            Phrase::Expr(e) => self.expr(e),
        }
    }

    /// `use tree;`, the tree in rustfmt's spelling: `a::b::{c, d as e}`.
    fn use_item(&mut self, tree: &proc_macro2::TokenStream, span: Span) -> Doc {
        fn print(ts: proc_macro2::TokenStream, out: &mut String) {
            use proc_macro2::{Spacing, TokenTree};
            // A `:` not joined to the next one stays apart from it.
            let mut apart = false;
            for t in ts {
                if apart && matches!(&t, TokenTree::Punct(p) if p.as_char() != ',') {
                    out.push(' ');
                }
                apart = matches!(&t, TokenTree::Punct(p) if p.spacing() == Spacing::Alone);
                match t {
                    TokenTree::Ident(i) => {
                        if out.ends_with(|c: char| c.is_alphanumeric() || c == '_') {
                            out.push(' ');
                        }
                        out.push_str(&i.to_string());
                    }
                    TokenTree::Punct(p) if p.as_char() == ',' => out.push_str(", "),
                    TokenTree::Punct(p) => out.push(p.as_char()),
                    proc_macro2::TokenTree::Group(g) => {
                        out.push('{');
                        print(g.stream(), out);
                        let n = out.trim_end_matches(' ').len();
                        out.truncate(n);
                        out.push('}');
                    }
                    proc_macro2::TokenTree::Literal(l) => out.push_str(&l.to_string()),
                }
            }
        }
        let kw = self.keyword(span, "use");
        let mut text = String::new();
        print(tree.clone(), &mut text);
        concat(vec![kw, text_doc(" "), text_doc(text), text_doc(";")])
    }

    fn external(
        &mut self,
        name: &Name,
        ty: &Type,
        rust: &Option<(String, Span)>,
        span: Span,
    ) -> Doc {
        let kw = self.keyword(span, "external");
        let n = self.name(name);
        let t = self.ty(ty);
        let mut parts = vec![kw, text_doc(" "), n, text_doc(self.colon()), t];
        if let Some((_, sp)) = rust {
            parts.push(text_doc(" = "));
            parts.push(self.verbatim(range(*sp), true));
        }
        concat(parts)
    }

    fn name(&mut self, n: &Name) -> Doc {
        match n {
            Name::Ident(i) => self.ident(i),
            Name::Op(op) => self.op_section(op),
        }
    }

    // ----- let -----------------------------------------------------------

    /// `let [rec] a = .. and b = ..`, and the group id of the last binding
    /// (`in` goes on its own line when that one spans several lines).
    fn let_bindings(&mut self, rec_: bool, bindings: &[Binding], span: Span) -> (Doc, usize) {
        let mut id = 0;
        let mut parts = Vec::new();
        for (i, b) in bindings.iter().enumerate() {
            let kw = if i == 0 {
                let k = self.keyword(span, "let");
                concat(vec![k, text_doc(if rec_ { " rec " } else { " " })])
            } else {
                parts.push(if self.cfg.let_.and_blank_line {
                    Doc::BlankLines(1)
                } else {
                    Doc::HardLine
                });
                text_doc("and ")
            };
            id = self.id();
            let d = self.binding(kw, b);
            parts.push(group_with_id(d, Some(id)));
        }
        (concat(parts), id)
    }

    fn binding(&mut self, kw: Doc, b: &Binding) -> Doc {
        let mut head = vec![kw];
        match &b.kind {
            BindKind::Func {
                name,
                params,
                mutable,
                ..
            } => {
                if *mutable {
                    head.push(text_doc("mut "));
                }
                head.push(self.name(name));
                for p in params {
                    head.push(text_doc(" "));
                    head.push(self.pat(p));
                }
            }
            BindKind::Value { pat, mutable } => {
                if *mutable {
                    head.push(text_doc("mut "));
                }
                head.push(self.pat(pat));
            }
        }
        if let Some(t) = &b.ty {
            head.push(text_doc(self.colon()));
            head.push(self.ty(t));
        }
        head.push(text_doc(" ="));
        let indent = self.cfg.let_.binding_indent;
        self.after_eq(concat(head), &b.value, indent)
    }

    /// `head` (ending in `=`) and `value`: on one line, or the value below,
    /// indented by `indent`. Lambdas, and calls ending in a lambda, start on
    /// the `=` line.
    fn after_eq(&mut self, head: Doc, value: &Expr, indent: usize) -> Doc {
        let lambda = is_lambda(value);
        let v = self.expr(value);
        if lambda && self.cfg.let_.fun_on_eq_line {
            return concat(vec![head, text_doc(" "), v]);
        }
        if self.ends_in_lambda(value) {
            let same_line = concat(vec![head.clone(), text_doc(" "), v.clone()]);
            let below = concat(vec![head, nest(indent, concat(vec![Doc::HardLine, v]))]);
            return choice(vec![same_line.clone(), same_line, below]);
        }
        group(concat(vec![head, nest(indent, concat(vec![Doc::Line, v]))]))
    }

    /// A lambda, or (with `begin_end = "parens"`) `begin` around one, which
    /// prints as the same `(fun ..)`.
    fn is_lambda_as_printed(&self, e: &Expr) -> bool {
        is_lambda(e) || self.begin_as_parens(e).is_some_and(is_lambda)
    }

    /// The body of a `begin .. end` printed as `( .. )`.
    fn begin_as_parens<'e>(&self, e: &'e Expr) -> Option<&'e Expr> {
        match e {
            Expr::Surface(s) if self.cfg.begin_end == BeginEnd::Parens => match &**s {
                Surface::Begin { body: Some(b), .. } => Some(b),
                _ => None,
            },
            _ => None,
        }
    }

    /// `(fun ..)` / `(function ..)`, also when written `begin fun .. end`
    /// and printed with parentheses.
    fn is_paren_lambda(&self, e: &Expr) -> bool {
        match e {
            Expr::Paren(inner, _) => self.is_lambda_as_printed(inner),
            _ => self.begin_as_parens(e).is_some_and(is_lambda),
        }
    }

    /// A call or operator chain whose last argument is a lambda that can
    /// keep the line open (`Iterator::map _ (fun x ->`).
    fn ends_in_lambda(&self, e: &Expr) -> bool {
        if !self.cfg.fun.hug_last_arg {
            return false;
        }
        match e {
            Expr::App(_, args) => args.last().is_some_and(|a| self.is_paren_lambda(a)),
            Expr::Method { args, .. } | Expr::Call { args, .. } => {
                args.last().is_some_and(is_lambda)
            }
            Expr::Binary { rhs, .. } => is_lambda(rhs) || self.ends_in_lambda(rhs),
            _ => false,
        }
    }

    fn let_in(&mut self, rec_: bool, bindings: &[Binding], body: &Expr, span: Span) -> Doc {
        let (b, id) = self.let_bindings(rec_, bindings, span);
        let in_ = match self.cfg.let_.in_position {
            InPosition::Auto => if_group_breaks(
                id,
                concat(vec![Doc::HardLine, text_doc("in")]),
                text_doc(" in"),
            ),
            InPosition::EndOfLine => text_doc(" in"),
        };
        self.then_body(concat(vec![b, in_]), body)
    }

    /// `head` (ending in `in`) and the body after it.
    fn then_body(&mut self, head: Doc, body: &Expr) -> Doc {
        let start = self.start(body);
        match self.cfg.let_.body {
            LetBody::AlwaysBreak => {
                let c = self.comments_before(start, true);
                let body = self.expr(body);
                concat(vec![head, Doc::HardLine, c, body])
            }
            LetBody::Fit => {
                let c = self.comments_before(start, false);
                let body = self.expr(body);
                group(concat(vec![head, Doc::Line, c, body]))
            }
        }
    }

    // ----- expressions ---------------------------------------------------

    pub fn expr(&mut self, e: &Expr) -> Doc {
        // Comments before the expression go outside its indentation, and so
        // do those inside the path it starts with, which the path prints
        // before itself (`String. (* c *) from x`): read back, they come
        // before the expression.
        let start = self.start(e);
        let before = match leading_path(e) {
            Some(p) if self.path_start(p) == start => self.leading_path_comments(p, start),
            _ => self.comments_before(start, false),
        };
        let d = self.expr_inner(e);
        match before {
            Doc::Concat(ref v) if v.is_empty() => d,
            before => concat(vec![before, d]),
        }
    }

    fn expr_inner(&mut self, e: &Expr) -> Doc {
        let ind = self.cfg.indent;
        match e {
            Expr::Var(i) | Expr::Tmp(i, _) => self.ident(i),
            Expr::Path(p) => self.path(p),
            Expr::Lit(l) => self.verbatim(range(l.span()), true),
            Expr::Bool(b, sp) => self.leaf(range(*sp), if *b { "true" } else { "false" }),
            Expr::Unit(sp) => self.unit(range(*sp)),
            Expr::Paren(inner, sp) => self.paren(inner, *sp),
            Expr::Tuple(items) => {
                let items: Vec<Doc> = items.iter().map(|x| self.expr(x)).collect();
                group(concat(join(items, concat(vec![text_doc(","), Doc::Line]))))
            }
            Expr::List(items, sp) => {
                let space = self.cfg.collections.space_inside_lists;
                let items = items.iter().map(|x| self.expr(x)).collect();
                self.collection(range(*sp), "[", "]", items, space)
            }
            Expr::Array(items, sp) => {
                let space = self.cfg.collections.space_inside_arrays
                    || items.first().is_some_and(starts_with_op)
                    || items.last().is_some_and(ends_with_op);
                let items = items.iter().map(|x| self.expr(x)).collect();
                self.collection(range(*sp), "[|", "|]", items, space)
            }
            Expr::App(head, args) => self.app(head, args),
            // `f(a)` keeps its `(` against `f`: with a space it would be
            // OCaml's `f (a)`.
            Expr::Call {
                func, args, span, ..
            } => {
                let f = self.expr(func);
                let close = self.src.matching_close(start_of(*span));
                let a = self.call_args(args, close);
                concat(vec![f, a])
            }
            Expr::Range { lo, hi, op } => {
                // `0. ..1.`, not `0...1.` (which is `0 ... 1.`).
                let float_dot = lo.is_some()
                    && self.src.text[..start_of(op.span())]
                        .trim_end()
                        .ends_with('.');
                let space = lo.as_deref().is_some_and(ends_with_op)
                    || hi.as_deref().is_some_and(starts_with_op)
                    || float_dot;
                let sp = text_doc(if space { " " } else { "" });
                let mut parts = Vec::new();
                if let Some(lo) = lo {
                    parts.push(self.expr(lo));
                    parts.push(sp.clone());
                }
                parts.push(self.leaf(start_of(op.span())..end_of(op.last_span()), op.text.clone()));
                if let Some(hi) = hi {
                    parts.push(sp);
                    parts.push(self.expr(hi));
                }
                concat(parts)
            }
            Expr::Jump { kind, value, span } => {
                let kw = self.keyword(*span, kind.keyword());
                match value {
                    Some(v) => {
                        let v = self.expr(v);
                        concat(vec![kw, text_doc(" "), v])
                    }
                    None => kw,
                }
            }
            Expr::RustBlock {
                kind,
                body,
                span,
                brace,
            } => {
                let kw = self.keyword(
                    *span,
                    if *kind == RustBlock::Unsafe {
                        "unsafe"
                    } else {
                        "async"
                    },
                );
                let head = if *kind == RustBlock::AsyncMove {
                    concat(vec![kw, text_doc(" move {")])
                } else {
                    concat(vec![kw, text_doc(" {")])
                };
                let close = end_of(*brace) - 1;
                let Some(body) = body else {
                    let c = self.comments_before(close, false);
                    return concat(vec![head, c, text_doc("}")]);
                };
                let b = self.expr(body);
                let c = self.comments_before(close, false);
                group(concat(vec![
                    head,
                    nest(ind, concat(vec![Doc::Line, b, c])),
                    Doc::Line,
                    text_doc("}"),
                ]))
            }
            Expr::Fun {
                params,
                body,
                span,
                is_move,
                bars,
                ..
            } => self.lambda(params, body, *span, *is_move, *bars),
            Expr::Function { arms, span } => {
                let kw = self.keyword(*span, "function");
                let vertical = self.vertical.contains(&start_of(*span));
                self.cases(kw, arms, vertical)
            }
            Expr::Let {
                rec_,
                bindings,
                body,
                span,
            } => self.let_in(*rec_, bindings, body, *span),
            Expr::External {
                name,
                ty,
                rust,
                body,
                span,
            } => {
                let head = self.external(name, ty, rust, *span);
                self.then_body(concat(vec![head, text_doc(" in")]), body)
            }
            Expr::Use {
                tree, body, span, ..
            } => {
                let head = self.use_item(tree, *span);
                self.then_body(head, body)
            }
            Expr::If { .. } => self.if_(e),
            Expr::Match { scrut, arms, span } => {
                let kw = self.keyword(*span, "match");
                let s = align(self.expr(scrut));
                let head = concat(vec![kw, text_doc(" "), s, text_doc(" with")]);
                let vertical = self.vertical.contains(&start_of(*span));
                self.cases(head, arms, vertical)
            }
            Expr::Seq(..) => self.seq(e),
            Expr::Binary { .. } => self.chain(e),
            Expr::Compose { first, second, .. } => {
                let a = self.expr(first);
                let b = self.expr(second);
                concat(vec![a, text_doc(" >> "), b])
            }
            Expr::Neg(inner, op) => {
                let o = self.leaf(start_of(op.span())..end_of(op.last_span()), op.text.clone());
                // `- ::cmp`, not `-::cmp` (one operator to the lexer).
                let space = op.text == "-."
                    || starts_with_op(inner)
                    || self.first_char(inner).is_some_and(is_op_char);
                let i = self.expr(inner);
                concat(vec![o, text_doc(if space { " " } else { "" }), i])
            }
            // `!e` (OCaml) or `*e` (Rust), as written.
            Expr::Deref(inner, sp) => {
                let at = start_of(*sp);
                let star = self.src.text[at..].starts_with('*');
                let o = self.leaf(at..at + 1, if star { "*" } else { "!" });
                let space = self.glues(inner);
                let i = self.expr(inner);
                concat(vec![o, text_doc(if space { " " } else { "" }), i])
            }
            Expr::Borrow {
                mutable,
                expr,
                span,
            } => {
                let o = self.leaf(start_of(*span)..start_of(*span) + 1, "&");
                // At the start of an argument, `&&x` would be `&&`, and `&*x`
                // a user-defined `&*`.
                let arg = self.arg_amp == Some(start_of(*span));
                let space = !*mutable
                    && (self.glues(expr) || arg && self.first_char(expr).is_some_and(is_op_char));
                let i = self.expr(expr);
                let m = if *mutable {
                    "mut "
                } else if space {
                    " "
                } else {
                    ""
                };
                concat(vec![o, text_doc(m), i])
            }
            Expr::Field(..)
            | Expr::TupleIndex(..)
            | Expr::Method { .. }
            | Expr::Index(..)
            | Expr::StrIndex(..)
            | Expr::Try(..)
            | Expr::Await(..) => self.postfix(e),
            Expr::Assign(lhs, rhs, _) => self.assign(lhs, " <-", rhs),
            Expr::RefAssign(lhs, rhs, _) => self.assign(lhs, " :=", rhs),
            Expr::OpSection(op) => self.op_section(op),
            Expr::Constraint(inner, ty) => {
                let i = self.expr(inner);
                let t = self.ty(ty);
                concat(vec![i, text_doc(self.colon()), t])
            }
            Expr::Cast(inner, ty) => {
                let i = self.expr(inner);
                let t = self.ty(ty);
                concat(vec![i, text_doc(" as "), t])
            }
            Expr::Record { ctor, fields, .. } => {
                let p = self.path(ctor);
                let mut items = Vec::new();
                for (f, v) in fields {
                    let name = self.ident(f);
                    items.push(match v {
                        Some(v) => {
                            let v = self.expr(v);
                            concat(vec![name, text_doc(" = "), v])
                        }
                        None => name,
                    });
                }
                let space = self.cfg.collections.space_inside_records;
                let braces = self.braces_after(path_end(ctor));
                let body = self.collection(braces, "{", "}", items, space);
                concat(vec![p, text_doc(" "), body])
            }
            Expr::RecordUpdate { base, fields, span } => {
                let open = self.leaf(start_of(*span)..start_of(*span) + 1, "{");
                let b = self.expr(base);
                let mut items = Vec::new();
                for (f, v) in fields {
                    let name = self.ident(f);
                    let punned =
                        matches!(v, Expr::Var(x) if start_of(x.span()) == start_of(f.span()));
                    items.push(if punned {
                        name
                    } else {
                        let v = self.expr(v);
                        concat(vec![name, text_doc(" = "), v])
                    });
                }
                let sp = if self.cfg.collections.space_inside_records {
                    " "
                } else {
                    ""
                };
                let fields = concat(join(items, concat(vec![text_doc(";"), Doc::Line])));
                group(concat(vec![
                    open,
                    text_doc(sp),
                    align(concat(vec![
                        b,
                        text_doc(" with"),
                        nest(ind, concat(vec![Doc::Line, fields])),
                    ])),
                    text_doc(sp),
                    text_doc("}"),
                ]))
            }
            Expr::For {
                var,
                from,
                to,
                dir,
                body,
                span,
            } => {
                let kw = self.keyword(*span, "for");
                let v = self.ident(var);
                let f = self.expr(from);
                let t = self.expr(to);
                let hug = self.hugs(body);
                let b = self.expr(body);
                let dir = format!(" {} ", dir.keyword());
                group(concat(vec![
                    kw,
                    text_doc(" "),
                    v,
                    text_doc(" = "),
                    f,
                    text_doc(&dir),
                    t,
                    text_doc(" do"),
                    loop_body(ind, hug, b),
                ]))
            }
            Expr::ForIn {
                pat,
                iter,
                body,
                span,
            } => {
                let kw = self.keyword(*span, "for");
                let p = self.pat(pat);
                let i = align(self.expr(iter));
                let hug = self.hugs(body);
                let b = self.expr(body);
                group(concat(vec![
                    kw,
                    text_doc(" "),
                    p,
                    text_doc(" in "),
                    i,
                    text_doc(" do"),
                    loop_body(ind, hug, b),
                ]))
            }
            Expr::While { cond, body, span } => {
                let kw = self.keyword(*span, "while");
                let c = align(self.expr(cond));
                let hug = self.hugs(body);
                let b = self.expr(body);
                group(concat(vec![
                    kw,
                    text_doc(" "),
                    c,
                    text_doc(" do"),
                    loop_body(ind, hug, b),
                ]))
            }
            // Macros with Rust arguments are kept as written.
            Expr::Macro { path, group, .. } => self.verbatim_macro(path, group),
            Expr::StdMacro { .. } => self.std_macro(e),
            Expr::Rust(g) => text_doc(g.to_string()),
            Expr::Assert(inner, sp) => {
                let kw = self.keyword(*sp, "assert");
                let i = self.expr(inner);
                concat(vec![kw, text_doc(" "), i])
            }
            Expr::Hole { written, span, .. } => {
                let s = match written {
                    Some(n) => format!("_{}", n),
                    None => "_".to_string(),
                };
                self.leaf(range(*span), s)
            }
            // A section is its body; the holes say it is a function.
            Expr::Section { body, .. } => self.expr_inner(body),
            Expr::MethodRef {
                name,
                turbofish,
                args,
                span,
            } => {
                let dot = self.leaf(range(*span), ".");
                let n = self.ident(name);
                let tf = match turbofish.as_ref().and_then(stream_range) {
                    Some(r) => concat(vec![text_doc("::"), self.verbatim(r, true)]),
                    None => Doc::Nil,
                };
                let a = match args {
                    Some(args) => {
                        let close = self.args_close(name, turbofish);
                        self.call_args(args, close)
                    }
                    None => Doc::Nil,
                };
                concat(vec![dot, n, tf, a])
            }
            Expr::FieldRef { idx, span } => {
                let dot = self.leaf(range(*span), ".");
                let s: Vec<String> = idx.iter().map(|(n, _)| n.to_string()).collect();
                let f = self.leaf(range(idx[0].1), s.join("."));
                concat(vec![dot, f])
            }
            Expr::Surface(s) => self.surface(s),
            Expr::Labeled { .. } => unreachable!("a surface parse keeps `Surface::Label`"),
            Expr::TailCall { .. } => unreachable!("only the code generator makes tail calls"),
        }
    }

    fn surface(&mut self, s: &Surface) -> Doc {
        match s {
            Surface::Begin { body, begin, end } => {
                let parens = self.cfg.begin_end == BeginEnd::Parens;
                let Some(body) = body else {
                    if parens {
                        return self.leaf(start_of(*begin)..end_of(*end), "()");
                    }
                    let b = self.keyword(*begin, "begin");
                    let e = self.keyword(*end, "end");
                    return concat(vec![b, text_doc(" "), e]);
                };
                if parens {
                    // As `paren` prints `( .. )`, which is what it becomes.
                    let before = self.comments_before(start_of(*begin), false);
                    let after_open = self.trailing_after(end_of(*begin));
                    let paren = text_doc(self.open_paren(body));
                    let (open, lead) = if after_open.has_hard() {
                        (paren, concat(vec![after_open, Doc::HardLine]))
                    } else {
                        (concat(vec![paren, after_open]), Doc::Nil)
                    };
                    let open = concat(vec![before, open]);
                    let b = self.expr(body);
                    let before_end = self.comments_before(start_of(*end), false);
                    let close = self.leaf(range(*end), ")");
                    let b = concat(vec![lead, b, before_end]);
                    let b = if self.is_lambda_as_printed(body) {
                        b
                    } else {
                        align(b)
                    };
                    return concat(vec![open, b, close]);
                }
                let b = self.keyword(*begin, "begin");
                let inner = self.expr(body);
                let before_end = self.comments_before(start_of(*end), false);
                let e = self.keyword(*end, "end");
                group(concat(vec![
                    b,
                    nest(self.cfg.indent, concat(vec![Doc::Line, inner, before_end])),
                    Doc::Line,
                    e,
                ]))
            }
            Surface::Label { label, value, span } => {
                let tilde = self.leaf(start_of(*span)..start_of(*span) + 1, "~");
                let mut parts = vec![tilde];
                if let Some(l) = label {
                    parts.push(self.ident(l));
                    if let Some(v) = value {
                        // `~f:.len` would lex as `:.`, `~f:!r` as `:!`.
                        let space = self.first_char(v).is_some_and(is_op_char);
                        parts.push(text_doc(if space { ": " } else { ":" }));
                    }
                }
                if let Some(v) = value {
                    parts.push(self.expr(v));
                }
                concat(parts)
            }
        }
    }

    /// Where a path starts: its first name, or its leading `::`.
    fn path_start(&self, p: &Path) -> usize {
        let start = start_of(p.segs[0].ident.span());
        if p.leading_colon {
            return self.src.text[..start].trim_end().len().saturating_sub(2);
        }
        start
    }

    fn path(&mut self, p: &Path) -> Doc {
        let start = self.path_start(p);
        let mut s = String::new();
        if p.leading_colon {
            s.push_str("::");
        }
        let mut end = start;
        for (i, seg) in p.segs.iter().enumerate() {
            if i > 0 {
                let before = self.src.text[..start_of(seg.ident.span())].trim_end();
                s.push_str(if before.ends_with('.') { "." } else { "::" });
            }
            s.push_str(&seg.ident.to_string());
            end = end_of(seg.ident.span());
            if let Some(g) = &seg.generics
                && let Some(r) = stream_range(g)
            {
                s.push_str("::");
                s.push_str(self.src.slice(r.clone()));
                self.mark_done(r.clone());
                end = r.end;
            }
        }
        let pre = self.path_comments(start, end);
        concat(vec![pre, self.leaf(start..end, s)])
    }

    /// [`Printer::path_comments`] for a path that starts an expression,
    /// taken out before the expression. Comments in generic arguments stay
    /// there: the arguments are printed as written.
    fn leading_path_comments(&mut self, p: &Path, start: usize) -> Doc {
        for seg in &p.segs {
            if let Some(r) = seg.generics.as_ref().and_then(stream_range) {
                self.mark_done(r);
            }
        }
        self.path_comments(start, path_end(p))
    }

    /// The comments before the path from `start` to `end`, then those
    /// between its tokens (`Vec::<u8> (* c *) ::new`), which go before it:
    /// on lines of their own if they end theirs (as they are read back),
    /// else on its line.
    fn path_comments(&mut self, start: usize, end: usize) -> Doc {
        let before = self.comments_before(start, false);
        let mut inside = Vec::new();
        for i in 0..self.src.comments.len() {
            let c = &self.src.comments[i];
            if !self.done[i] && c.range.start < end {
                self.done[i] = true;
                let (text, line_comment) = self.comment_text(i);
                inside.push(if line_comment || text.contains('\n') {
                    Doc::OwnLine {
                        blank_before: 0,
                        src_col: self.src.column(c.range.start),
                        text,
                    }
                } else {
                    Doc::Inline(text)
                });
            }
        }
        concat(vec![before, concat(inside)])
    }

    fn paren(&mut self, inner: &Expr, sp: Span) -> Doc {
        let r = range(sp);
        let before = self.comments_before(r.start, false);
        let mut after_open = self.trailing_after(r.start + 1);
        // The comments inside a path the body starts with go before it
        // (see `expr`): after the `(`, where they are read back.
        let start = self.start(inner);
        if let Some(p) = leading_path(inner)
            && self.path_start(p) == start
        {
            let inside = self.leading_path_comments(p, start);
            after_open = concat(vec![after_open, inside]);
        }
        // `(begin // c ..)` with `begin_end = "parens"`: after the `(`, as
        // the `begin` is not printed.
        if let Expr::Surface(s) = inner
            && let Surface::Begin {
                begin,
                body: Some(_),
                ..
            } = &**s
            && self.cfg.begin_end == BeginEnd::Parens
        {
            let after_begin = self.trailing_after(end_of(*begin));
            after_open = concat(vec![after_open, after_begin]);
        }
        // A `//` comment after the `(` ends its line: the body starts the
        // next one, where it would start after the `(`.
        let (open, after_open) = if after_open.has_hard() {
            (
                text_doc(self.open_paren(inner)),
                concat(vec![after_open, Doc::HardLine]),
            )
        } else {
            (
                concat(vec![text_doc(self.open_paren(inner)), after_open]),
                Doc::Nil,
            )
        };
        let open = concat(vec![before, open]);
        let body = match inner {
            Expr::Constraint(e, t) => {
                let e = self.expr(e);
                let t = self.ty(t);
                concat(vec![e, text_doc(self.colon()), t])
            }
            // `(begin e end)` with `begin_end = "parens"` is `(e)`, not `((e))`.
            Expr::Surface(s)
                if self.cfg.begin_end == BeginEnd::Parens
                    && matches!(&**s, Surface::Begin { body: Some(_), .. }) =>
            {
                let Surface::Begin {
                    body: Some(b), end, ..
                } = &**s
                else {
                    unreachable!()
                };
                let b = self.expr(b);
                let before_end = self.comments_before(start_of(*end), false);
                concat(vec![b, before_end])
            }
            e => self.expr(e),
        };
        let before_close = self.comments_before(r.end - 1, false);
        let after = self.trailing_after(r.end);
        let body = concat(vec![after_open, body]);
        // A lambda's body is indented from the line, not from the `(`.
        let body = if self.is_lambda_as_printed(inner) {
            body
        } else {
            align(body)
        };
        concat(vec![open, body, before_close, text_doc(")"), after])
    }

    fn op_section(&mut self, op: &Op) -> Doc {
        let t = &op.text;
        let spaced = self.cfg.op_section_spaces == OpSectionSpaces::Always
            || t.starts_with('*')
            || t.ends_with('*')
            || t.chars().all(|c| c.is_ascii_alphabetic());
        let (start, end) = (start_of(op.span()), end_of(op.last_span()));
        // A comment after the operator stays inside: `( + (* c *) )`.
        let Some(close) = self.src.enclosing_close(end) else {
            let s = if spaced {
                format!("( {} )", t)
            } else {
                format!("({})", t)
            };
            return self.leaf(start..end, s);
        };
        let before = self.comments_before(start, false);
        let inside = self.comments_before(close, false);
        let after = self.trailing_after(close + 1);
        let (open, close) = if spaced {
            (format!("( {}", t), " )")
        } else {
            (format!("({}", t), ")")
        };
        concat(vec![before, text_doc(open), inside, text_doc(close), after])
    }

    fn assign(&mut self, lhs: &Expr, op: &str, rhs: &Expr) -> Doc {
        let l = self.expr(lhs);
        let r = self.expr(rhs);
        group(concat(vec![
            l,
            text_doc(op),
            nest(self.cfg.indent, concat(vec![Doc::Line, r])),
        ]))
    }

    // ----- application ---------------------------------------------------

    /// Whether `body`, after `then`, `else`, `->` or `do`, keeps its
    /// opening (`begin`, `unsafe {`) on that keyword's line (see
    /// `begin_end_hug`). Not after a comment, which has a line to itself.
    fn hugs(&self, body: &Expr) -> bool {
        let block = match body {
            Expr::Surface(s) => {
                matches!(&**s, Surface::Begin { body: Some(_), .. })
                    && self.cfg.begin_end == BeginEnd::Preserve
            }
            Expr::RustBlock { body: Some(_), .. } => true,
            _ => false,
        };
        block && self.cfg.begin_end_hug && !self.comment_pending_before(self.start(body))
    }

    /// Whether a comment not printed yet starts before `pos`.
    fn comment_pending_before(&self, pos: usize) -> bool {
        self.src
            .comments
            .iter()
            .zip(&self.done)
            .any(|(c, done)| !done && c.range.start < pos)
    }

    /// The first character of the expression's text, when it is not a
    /// letter, digit or bracket.
    fn first_char(&self, e: &Expr) -> Option<char> {
        match e {
            // `::std::x`
            Expr::Path(p)
            | Expr::Record { ctor: p, .. }
            | Expr::Macro { path: p, .. }
            | Expr::StdMacro { path: p, .. }
                if p.leading_colon =>
            {
                Some(':')
            }
            Expr::Neg(..) => Some('-'),
            Expr::Fun {
                bars: Some(_),
                is_move: false,
                ..
            } => Some('|'),
            Expr::Deref(_, sp) => self.src.text[start_of(*sp)..].chars().next(),
            Expr::Borrow { .. } => Some('&'),
            Expr::MethodRef { .. } | Expr::FieldRef { .. } | Expr::Range { lo: None, .. } => {
                Some('.')
            }
            Expr::Surface(s) => matches!(**s, Surface::Label { .. }).then_some('~'),
            Expr::Range { lo: Some(e), .. }
            | Expr::App(e, _)
            | Expr::Call { func: e, .. }
            | Expr::Seq(e, _)
            | Expr::Binary { lhs: e, .. }
            | Expr::Compose { first: e, .. }
            | Expr::Field(e, _)
            | Expr::TupleIndex(e, _)
            | Expr::Try(e, _)
            | Expr::Await(e, _)
            | Expr::Cast(e, _)
            | Expr::Constraint(e, _)
            | Expr::Index(e, _, _)
            | Expr::StrIndex(e, _)
            | Expr::Assign(e, _, _)
            | Expr::RefAssign(e, _, _)
            | Expr::Method { recv: e, .. } => self.first_char(e),
            Expr::Tuple(es) => self.first_char(&es[0]),
            Expr::Section { body, .. } => self.first_char(body),
            _ => None,
        }
    }

    /// Whether `e`, written right after a prefix `!`, `*` or `&`, would lex
    /// together with it into something else (`!-`, `&.`). Runs of `!`,
    /// `*` and `&` read as one operator each.
    fn glues(&self, e: &Expr) -> bool {
        self.first_char(e)
            .is_some_and(|c| is_op_char(c) && !matches!(c, '!' | '*' | '&'))
    }

    /// `(`, or `( ` before a `*`: `(*` would read as an OCaml comment.
    fn open_paren(&self, first: &Expr) -> &'static str {
        if self.first_char(first) == Some('*') {
            "( "
        } else {
            "("
        }
    }

    /// Whether a bracket starts right at `pos`, as in `Some(1, 2)`: the
    /// constructor's argument was written against it. Both spellings mean
    /// the same, so the formatter keeps the one written.
    fn touches(&self, pos: usize) -> bool {
        self.src.text[pos..].starts_with(['(', '['])
    }

    fn app(&mut self, head: &Expr, args: &[Expr]) -> Doc {
        // Comments among the tokens of a head path go before the whole
        // application, as before a path, which starts its next line if they
        // end theirs.
        let pre = match head {
            Expr::Path(p) => {
                // Those in its generic arguments are printed with them.
                for seg in &p.segs {
                    if let Some(r) = seg.generics.as_ref().and_then(stream_range) {
                        self.mark_done(r);
                    }
                }
                self.path_comments(self.path_start(p), path_end(p))
            }
            _ => Doc::Nil,
        };
        let d = self.app_inner(head, args);
        if pre.has_hard() {
            return concat(vec![pre, Doc::HardLine, d]);
        }
        concat(vec![pre, d])
    }

    fn app_inner(&mut self, head: &Expr, args: &[Expr]) -> Doc {
        // `Some(1, 2)`: an argument written against a constructor stays there.
        let mut docs: Vec<Doc> = Vec::new();
        let mut prev: Option<&Expr> = None;
        for (k, x) in std::iter::once(head).chain(args).enumerate() {
            if k > 0
                && let Expr::Borrow { span, .. } = leftmost(x)
            {
                self.arg_amp = Some(start_of(*span));
            }
            let d = self.expr(x);
            self.arg_amp = None;
            match docs.last_mut() {
                Some(last) if prev.and_then(ctor_end).is_some_and(|end| self.touches(end)) => {
                    *last = concat(vec![std::mem::replace(last, Doc::Nil), d]);
                }
                _ => docs.push(d),
            }
            prev = Some(x);
        }
        let h = docs.remove(0);
        if docs.is_empty() {
            return h;
        }
        let a = docs;
        let ind = self.cfg.app.indent;
        let broken = match self.cfg.app.args {
            Layout::Wrap => {
                let mut items = vec![h.clone()];
                items.extend(a.iter().cloned());
                nest(ind, Doc::Fill(join(items, Doc::Line)))
            }
            Layout::FitOrVertical => {
                let mut rest = Vec::new();
                for x in &a {
                    rest.push(Doc::Line);
                    rest.push(x.clone());
                }
                group(concat(vec![h.clone(), nest(ind, concat(rest))]))
            }
        };
        let n = a.len();
        let hug = self.cfg.fun.hug_last_arg
            && args.last().is_some_and(|a| self.is_paren_lambda(a))
            && !h.has_hard()
            && !a[..n - 1].iter().any(Doc::has_hard);
        if !hug {
            return broken;
        }
        let mut pre = vec![h];
        for x in &a[..n - 1] {
            pre.push(text_doc(" "));
            pre.push(x.clone());
        }
        let last = a[n - 1].clone();
        let all = concat(vec![concat(pre.clone()), text_doc(" "), last.clone()]);
        let hugged = concat(vec![flat(concat(pre)), text_doc(" "), last]);
        choice(vec![all, hugged, broken])
    }

    fn lambda(
        &mut self,
        params: &[Pat],
        body: &Expr,
        span: Span,
        is_move: bool,
        bars: Option<(Span, Span)>,
    ) -> Doc {
        let mut head = Vec::new();
        match bars {
            None => {
                head.push(self.keyword(span, "fun"));
                head.push(text_doc(" "));
                if is_move {
                    head.push(text_doc("move "));
                }
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        head.push(text_doc(" "));
                    }
                    head.push(self.pat(p));
                }
                head.push(text_doc(" ->"));
            }
            // `|x, y: t|`, typed Rust's way, or `||`.
            Some((open, close)) => {
                if is_move {
                    head.push(self.keyword(span, "move"));
                    head.push(text_doc(" "));
                }
                let o = start_of(open);
                if matches!(params, [Pat::Unit(sp)] if start_of(*sp) == o) {
                    head.push(self.leaf(o..end_of(close).max(o + 2), "||"));
                } else {
                    head.push(self.leaf(o..o + 1, "|"));
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 {
                            head.push(text_doc(", "));
                        }
                        head.push(match p {
                            Pat::Constraint(p, t) => {
                                let p = self.pat(p);
                                let t = self.ty(t);
                                concat(vec![p, text_doc(": "), t])
                            }
                            p => self.pat(p),
                        });
                    }
                    let c = start_of(close);
                    head.push(self.leaf(c..c + 1, "|"));
                }
            }
        }
        let hug = self.hugs(body);
        let b = self.expr(body);
        let body = if hug {
            concat(vec![text_doc(" "), b])
        } else {
            nest(self.cfg.fun.body_indent, concat(vec![Doc::Line, b]))
        };
        group(concat(vec![concat(head), body]))
    }

    /// `recv.m(..).n(..)`: breaks before each `.call` after the first when
    /// the chain does not fit.
    fn postfix(&mut self, e: &Expr) -> Doc {
        let mut links = Vec::new();
        let mut root = e;
        while let Expr::Field(r, _)
        | Expr::TupleIndex(r, _)
        | Expr::Method { recv: r, .. }
        | Expr::Index(r, _, _)
        | Expr::StrIndex(r, _)
        | Expr::Try(r, _)
        | Expr::Await(r, _) = root
        {
            links.push(root);
            root = r;
        }
        links.reverse();
        let mut r = self.expr(root);
        // `1 .2`, `0. .len()` and `.0 .1`: glued, the `.` would make a
        // float (`1.2`), a range (`0..len()`) or one field section (`.0.1`).
        // So would `!1 .2`, which is `(!1).2`.
        let mut tip = root;
        while let Expr::Deref(e, _) = tip {
            tip = e;
        }
        let ends_in_digit = match tip {
            Expr::Lit(l) => l.to_string().starts_with(|c: char| c.is_ascii_digit()),
            Expr::FieldRef { .. } => true,
            _ => false,
        };
        let root_text = match tip {
            Expr::Lit(l) => l.to_string(),
            _ => String::new(),
        };
        let first_link = links.first();
        let digit_link = matches!(
            first_link,
            Some(Expr::TupleIndex(..) | Expr::Index(_, _, false) | Expr::StrIndex(..))
        );
        let integer = root_text.chars().all(|c| c.is_ascii_digit() || c == '_');
        if ends_in_digit
            && first_link.is_some_and(|l| !matches!(l, Expr::Try(..) | Expr::Index(_, _, true)))
            && (digit_link && (integer || matches!(tip, Expr::FieldRef { .. }))
                || root_text.ends_with('.'))
        {
            r = concat(vec![r, text_doc(" ")]);
        }
        let calls = links
            .iter()
            .filter(|l| matches!(l, Expr::Method { .. }))
            .count();
        let mut head = vec![r];
        let mut rest = Vec::new();
        let mut seen_call = false;
        let mut prev: Option<&Expr> = None;
        for link in links {
            let is_call = matches!(link, Expr::Method { .. });
            let d = self.link(link);
            // `t.0 .(i)`: glued, `0.` would be a float.
            let d = if matches!(prev, Some(Expr::TupleIndex(..)))
                && matches!(link, Expr::Index(_, _, false) | Expr::StrIndex(..))
            {
                concat(vec![text_doc(" "), d])
            } else {
                d
            };
            prev = Some(link);
            if is_call && seen_call {
                rest.push(Doc::SoftLine);
            }
            seen_call |= is_call;
            if rest.is_empty() {
                head.push(d);
            } else {
                rest.push(d);
            }
        }
        if calls < 2 || rest.is_empty() {
            head.extend(rest);
            return concat(head);
        }
        group(concat(vec![
            concat(head),
            nest(self.cfg.indent, concat(rest)),
        ]))
    }

    fn link(&mut self, e: &Expr) -> Doc {
        match e {
            Expr::Field(_, name) => {
                let dot = self.dot_before(name);
                concat(vec![dot, self.ident(name)])
            }
            Expr::TupleIndex(_, idx) => {
                let s: String = idx.iter().map(|(n, _)| format!(".{}", n)).collect();
                let r = range(idx[0].1);
                self.leaf(r, s)
            }
            Expr::Method {
                name,
                turbofish,
                args,
                ..
            } => {
                let dot = self.dot_before(name);
                let n = self.ident(name);
                let tf = match turbofish.as_ref().and_then(stream_range) {
                    Some(r) => concat(vec![text_doc("::"), self.verbatim(r, true)]),
                    None => Doc::Nil,
                };
                let close = self.args_close(name, turbofish);
                let a = self.call_args(args, close);
                concat(vec![dot, n, tf, a])
            }
            Expr::Index(_, ie, rust) => {
                let close = self.src.enclosing_close(expr_start(ie));
                let i = self.expr(ie);
                let c = close.map_or(Doc::Nil, |p| self.comments_before(p, false));
                let i = concat(vec![i, c]);
                if *rust {
                    concat(vec![text_doc("["), i, text_doc("]")])
                } else {
                    let open = if self.open_paren(ie) == "( " {
                        ".( "
                    } else {
                        ".("
                    };
                    concat(vec![text_doc(open), i, text_doc(")")])
                }
            }
            Expr::StrIndex(_, i) => {
                let i = self.expr(i);
                concat(vec![text_doc(".["), i, text_doc("]")])
            }
            Expr::Try(_, sp) => self.leaf(range(*sp).start..range(*sp).start + 1, "?"),
            Expr::Await(_, sp) => concat(vec![text_doc("."), self.leaf(range(*sp), "await")]),
            _ => unreachable!(),
        }
    }

    /// Where the `)` of a method's arguments is (after its name and
    /// turbofish).
    fn args_close(
        &self,
        name: &Ident,
        turbofish: &Option<proc_macro2::TokenStream>,
    ) -> Option<usize> {
        let end = turbofish
            .as_ref()
            .and_then(stream_range)
            .map_or(end_of(name.span()), |r| r.end);
        let open = self.src.token_at_or_after(end)?;
        self.src.matching_close(open)
    }

    /// Rust-style `(a, b)` arguments of a call; `close` is where its `)`
    /// is, so that comments before it stay inside.
    fn call_args(&mut self, args: &[Expr], close: Option<usize>) -> Doc {
        if args.is_empty() {
            let c = close.map_or(Doc::Nil, |p| self.comments_before(p, false));
            return concat(vec![text_doc("("), c, text_doc(")")]);
        }
        let open = self.open_paren(&args[0]);
        let a: Vec<Doc> = args.iter().map(|x| self.expr(x)).collect();
        let last_lambda = args.last().is_some_and(is_lambda);
        self.arg_list(open, ")", ",", a, last_lambda, close)
    }

    /// `(a, b)` that breaks Rust style, one item per line with a trailing
    /// separator, or hugs a last lambda (`f(a, fun x ->` ..). Comments
    /// before the closing bracket at `close_at` stay inside.
    fn arg_list(
        &mut self,
        open: &str,
        close: &str,
        sep: &str,
        a: Vec<Doc>,
        last_lambda: bool,
        close_at: Option<usize>,
    ) -> Doc {
        let before_close = close_at.map_or(Doc::Nil, |p| self.comments_before(p, false));
        // A comment after a last lambda: no hugging.
        let last_lambda = last_lambda && before_close.flat_width() == Some(0);
        let n = a.len();
        let comma = concat(vec![text_doc(sep), Doc::Line]);
        let broken = group(concat(vec![
            text_doc(open),
            nest(
                self.cfg.indent,
                concat(vec![
                    Doc::SoftLine,
                    concat(join(a.clone(), comma)),
                    if sep == "," {
                        if_break(text_doc(","), Doc::Nil)
                    } else {
                        Doc::Nil
                    },
                    before_close.clone(),
                ]),
            ),
            Doc::SoftLine,
            text_doc(close),
        ]));
        let hug = self.cfg.fun.hug_last_arg && last_lambda && !a[..n - 1].iter().any(Doc::has_hard);
        if !hug {
            return broken;
        }
        let mut pre = Vec::new();
        for x in &a[..n - 1] {
            pre.push(x.clone());
            pre.push(text_doc(format!("{} ", sep)));
        }
        let last = a[n - 1].clone();
        let all = concat(vec![
            text_doc(open),
            concat(pre.clone()),
            last.clone(),
            before_close.clone(),
            text_doc(close),
        ]);
        let hugged = concat(vec![
            text_doc(open),
            flat(concat(pre)),
            last,
            before_close,
            text_doc(close),
        ]);
        choice(vec![all, hugged, broken])
    }

    /// `println!("{}", f x)`: a std macro whose arguments are OCaml, laid
    /// out like a call's. A `{ .. }` macro is kept as written.
    fn std_macro(&mut self, e: &Expr) -> Doc {
        let Expr::StdMacro {
            path,
            group: g,
            args,
            repeat,
            ..
        } = e
        else {
            unreachable!()
        };
        let (open, close) = match g.delimiter() {
            proc_macro2::Delimiter::Parenthesis => ("(", ")"),
            proc_macro2::Delimiter::Bracket => ("[", "]"),
            _ => return self.verbatim_macro(path, g),
        };
        let p = self.path(path);
        let head = concat(vec![p, text_doc("!")]);
        let close_at = Some(start_of(g.span_close()));
        if args.is_empty() {
            let c = self.comments_before(start_of(g.span_close()), false);
            return concat(vec![head, text_doc(open), c, text_doc(close)]);
        }
        let open = if open == "(" {
            self.open_paren(args[0].expr())
        } else {
            open
        };
        let mut docs = Vec::new();
        for a in args {
            docs.push(match a {
                MacroArg::Expr(x) => self.expr(x),
                MacroArg::Named(n, x) => {
                    let n = self.ident(n);
                    let v = self.expr(x);
                    concat(vec![n, text_doc(" = "), v])
                }
            });
        }
        let last_lambda = args.last().is_some_and(|a| is_lambda(a.expr()));
        let sep = if *repeat { ";" } else { "," };
        let list = self.arg_list(open, close, sep, docs, last_lambda, close_at);
        concat(vec![head, list])
    }

    fn verbatim_macro(&mut self, path: &Path, group: &proc_macro2::Group) -> Doc {
        let start = start_of(path.segs[0].ident.span());
        let start = if path.leading_colon {
            self.src.text[..start].trim_end().len() - 2
        } else {
            start
        };
        self.verbatim(start..end_of(group.span()), true)
    }

    // ----- operators -----------------------------------------------------

    fn chain(&mut self, e: &Expr) -> Doc {
        let Expr::Binary { op, .. } = e else {
            unreachable!()
        };
        let class = OpClass::of(&op.text);
        let level = infix_level(&op.text);
        let same = |o: &Op| OpClass::of(&o.text) == class && infix_level(&o.text) == level;
        let right = level.is_some_and(|(_, r)| r);
        let mut operands = Vec::new();
        let mut ops = Vec::new();
        flatten(e, right, &same, &mut operands, &mut ops);

        let style = self.cfg.infix(class).clone();
        let leading = style.position == OperatorPosition::Leading;
        let first = self.expr(operands[0]);
        // Leading: `op operand` units; trailing: `operand op` units.
        let mut units = Vec::new();
        let mut pending = first;
        for (op, operand) in ops.iter().zip(&operands[1..]) {
            let o = self.leaf(start_of(op.span())..end_of(op.last_span()), op.text.clone());
            let d = self.expr(operand);
            if leading {
                units.push(pending);
                // A lambda's body is indented from the line, not the operand.
                let d = if is_lambda(operand) { d } else { align(d) };
                pending = concat(vec![o, text_doc(" "), d]);
            } else {
                units.push(concat(vec![pending, text_doc(" "), o]));
                pending = d;
            }
        }
        units.push(pending);

        let force = style.force_break_at > 0 && ops.len() >= style.force_break_at;
        let sep = if force { Doc::HardLine } else { Doc::Line };
        let ind = self.cfg.indent;
        let wrap = style.layout == InfixLayout::Wrap;
        let lines = |items: Vec<Doc>| -> Doc {
            if wrap {
                Doc::Fill(join(items, sep.clone()))
            } else {
                group(concat(join(items, sep.clone())))
            }
        };
        let rest = units.split_off(1);
        let first = units.pop().unwrap();
        let broken = match style.indent {
            InfixIndent::None => {
                let mut all = vec![first.clone()];
                all.extend(rest.iter().cloned());
                lines(all)
            }
            InfixIndent::Indent => {
                let mut all = vec![first.clone()];
                all.extend(rest.iter().cloned());
                nest(ind, lines(all))
            }
            InfixIndent::Align if leading => concat(vec![
                first.clone(),
                text_doc(" "),
                align(lines(rest.clone())),
            ]),
            InfixIndent::Align => {
                let mut all = vec![first.clone()];
                all.extend(rest.iter().cloned());
                align(lines(all))
            }
        };
        let hug = !force
            && !wrap
            && self.cfg.fun.hug_last_arg
            && operands.last().is_some_and(|x| is_lambda(x))
            && !first.has_hard()
            && !rest[..rest.len() - 1].iter().any(Doc::has_hard);
        if !hug {
            return broken;
        }
        let mut pre = vec![first];
        for u in &rest[..rest.len() - 1] {
            pre.push(text_doc(" "));
            pre.push(u.clone());
        }
        let last = rest.last().unwrap().clone();
        let all = concat(vec![concat(pre.clone()), text_doc(" "), last.clone()]);
        let hugged = concat(vec![flat(concat(pre)), text_doc(" "), last]);
        choice(vec![all, hugged, broken])
    }

    // ----- control -------------------------------------------------------

    fn if_(&mut self, e: &Expr) -> Doc {
        let mut branches = Vec::new();
        let mut els = None;
        let mut cur = e;
        loop {
            let Expr::If {
                cond,
                then,
                els: next,
                span,
            } = cur
            else {
                unreachable!()
            };
            let kw = self.keyword(*span, "if");
            let c = align(self.expr(cond));
            let hug = self.hugs(then);
            let t = self.expr(then);
            branches.push((kw, c, t, hug));
            match next {
                Some(x) if matches!(**x, Expr::If { .. }) => cur = x,
                Some(x) => {
                    let hug = self.hugs(x);
                    els = Some((self.expr(x), hug));
                    break;
                }
                None => break,
            }
        }
        let ind = self.cfg.indent;
        let style = self.cfg.if_.style;
        let mut parts = Vec::new();
        for (n, (kw, c, t, hug)) in branches.into_iter().enumerate() {
            if n > 0 {
                parts.push(Doc::Line);
                parts.push(text_doc("else "));
            }
            match style {
                IfStyle::FitOrVertical | IfStyle::Vertical => parts.extend([
                    kw,
                    text_doc(" "),
                    c,
                    text_doc(" then"),
                    if hug {
                        concat(vec![text_doc(" "), t])
                    } else {
                        nest(ind, concat(vec![Doc::Line, t]))
                    },
                ]),
                IfStyle::Compact => parts.extend([kw, text_doc(" "), c, text_doc(" then "), t]),
                IfStyle::KeywordFirst => {
                    parts.extend([kw, text_doc(" "), c, Doc::Line, text_doc("then "), t])
                }
            }
        }
        if let Some((e, hug)) = els {
            parts.push(Doc::Line);
            match style {
                IfStyle::FitOrVertical | IfStyle::Vertical if hug => {
                    parts.extend([text_doc("else "), e])
                }
                IfStyle::FitOrVertical | IfStyle::Vertical => {
                    parts.extend([text_doc("else"), nest(ind, concat(vec![Doc::Line, e]))])
                }
                IfStyle::Compact | IfStyle::KeywordFirst => parts.extend([text_doc("else "), e]),
            }
        }
        if style == IfStyle::Vertical {
            return Doc::Group {
                doc: Box::new(concat(parts)),
                id: None,
                hard: true,
            };
        }
        group(concat(parts))
    }

    fn cases(&mut self, head: Doc, arms: &[Arm], vertical: bool) -> Doc {
        let m = self.cfg.match_.clone();
        let id = self.id();
        // A case ending in a `match` with no parentheses, before another
        // case: one case per line, in both, so that the indentation shows
        // which `match` each case is of.
        let nested = match arms.split_last() {
            Some((_, init)) => {
                let tails: Vec<usize> = init
                    .iter()
                    .filter_map(|a| tail_cases(&a.body))
                    .map(expr_start)
                    .collect();
                self.vertical.extend(&tails);
                !tails.is_empty()
            }
            None => false,
        };
        let mut lhs = Vec::new();
        let mut bodies = Vec::new();
        let mut hugs = Vec::new();
        let mut before = Vec::new();
        for arm in arms {
            // Comments before a case go before its `|`.
            before.push(match pat_start(&arm.pat) {
                Some(pos) => self.comments_before(pos, false),
                None => Doc::Nil,
            });
            let p = self.pat(&arm.pat);
            let l = match &arm.guard {
                Some(g) => {
                    let kw = if arm.guard_if.is_some() {
                        " if "
                    } else {
                        " when "
                    };
                    let g = self.expr(g);
                    concat(vec![p, text_doc(kw), g])
                }
                None => p,
            };
            lhs.push(l);
            hugs.push(self.hugs(&arm.body));
            bodies.push(self.expr(&arm.body));
        }
        // A comment after a case's `|`, on the line before, ends the
        // previous case's body (as it does once printed there).
        for i in 1..bodies.len() {
            if is_trailing(&before[i]) {
                let c = std::mem::replace(&mut before[i], Doc::Nil);
                let b = std::mem::replace(&mut bodies[i - 1], Doc::Nil);
                bodies[i - 1] = attach_trailing(b, c);
            }
        }
        let widths: Option<Vec<usize>> = if m.align_arrows {
            lhs.iter().map(Doc::flat_width).collect()
        } else {
            None
        };
        let max = widths.as_ref().and_then(|w| w.iter().max().copied());
        let sep = match m.cases {
            _ if nested || vertical => Doc::HardLine,
            Cases::Vertical => Doc::HardLine,
            Cases::FitOrVertical => Doc::Line,
        };
        let mut items = Vec::new();
        for (i, ((l, body), c)) in lhs.into_iter().zip(bodies).zip(before).enumerate() {
            let bar = if i > 0 || m.leading_bar == LeadingBar::Always {
                text_doc("| ")
            } else {
                if_group_breaks(id, text_doc("| "), Doc::Nil)
            };
            let pad = match (&widths, max) {
                (Some(w), Some(max)) => text_doc(" ".repeat(max - w[i])),
                _ => Doc::Nil,
            };
            let body = match m.body {
                // `-> begin` with the block's lines under the case's body
                // and `end` between the two, as ocamlformat lays it out.
                _ if hugs[i] => concat(vec![
                    text_doc(" "),
                    nest(m.body_indent.saturating_sub(self.cfg.indent), body),
                ]),
                CaseBody::Fit => group(nest(m.body_indent, concat(vec![Doc::Line, body]))),
                CaseBody::NextLine => nest(m.body_indent, concat(vec![Doc::HardLine, body])),
            };
            items.push(sep.clone());
            items.push(concat(vec![c, bar, l, pad, text_doc(" ->"), body]));
        }
        group_with_id(
            concat(vec![head, nest(m.case_indent, concat(items))]),
            Some(id),
        )
    }

    fn seq(&mut self, e: &Expr) -> Doc {
        let mut items = Vec::new();
        let mut cur = e;
        while let Expr::Seq(a, b) = cur {
            items.push(&**a);
            cur = b;
        }
        items.push(cur);
        let s = self.cfg.sequence.clone();
        let always = s.break_ == Break::Always;
        let sep = if always { Doc::HardLine } else { Doc::Line };
        let mut parts = Vec::new();
        let n = items.len();
        for i in 0..n {
            let item = items[i];
            if i > 0 {
                let c = self.comments_before(self.start(item), always);
                match s.style {
                    SequenceStyle::Terminator => parts.extend([text_doc(";"), sep.clone(), c]),
                    SequenceStyle::Separator => parts.extend([sep.clone(), c, text_doc("; ")]),
                }
            }
            // With `;` leading, a comment before it (`a // c` then `; b`)
            // goes after the item, as one after it (`a; // c`) does: not
            // into the item, where a `//` would break its last group.
            let before_semi = match s.style {
                SequenceStyle::Separator if i + 1 < n => {
                    self.comments_before_semi(self.start(items[i + 1]))
                }
                _ => Doc::Nil,
            };
            parts.push(self.expr(item));
            parts.push(before_semi);
        }
        if always {
            concat(parts)
        } else {
            group(concat(parts))
        }
    }

    /// Where `e` starts: [`expr_start`], but at the `::` of a path written
    /// with one (`::std::f x`, which a comment may follow).
    fn start(&self, e: &Expr) -> usize {
        let pos = expr_start(e);
        if !matches!(leftmost(e), Expr::Path(p) if p.leading_colon) {
            return pos;
        }
        let colon = |r: &Range<usize>| &self.src.text[r.clone()] == ":";
        match self.src.token_before(pos).filter(colon) {
            Some(second) => self
                .src
                .token_before(second.start)
                .filter(colon)
                .map_or(pos, |r| r.start),
            None => pos,
        }
    }

    /// The source range of the `{ .. }` right after `pos` (a record's
    /// constructor), or `0..0`.
    fn braces_after(&self, pos: usize) -> Range<usize> {
        let open = self.src.token_at_or_after(pos);
        match open.filter(|&o| self.src.text[o..].starts_with('{')) {
            Some(o) => self.src.matching_close(o).map_or(0..0, |c| o..c + 1),
            None => 0..0,
        }
    }

    /// The comments between a sequence item and the `;` before `next`,
    /// marked as printed.
    fn comments_before_semi(&mut self, next: usize) -> Doc {
        let Some(semi) = self.src.token_before(next) else {
            return Doc::Nil;
        };
        if !self.src.text[semi.clone()].starts_with(';') {
            return Doc::Nil;
        }
        let item_end = self.src.code_end_before(semi.start);
        let mut out = Vec::new();
        for i in 0..self.src.comments.len() {
            let r = self.src.comments[i].range.clone();
            if !self.done[i] && r.start >= item_end && r.start < semi.start {
                self.done[i] = true;
                out.push(self.comment(i, false));
            }
        }
        concat(out)
    }

    // ----- collections ---------------------------------------------------

    /// A list, array or record body. `r` is the bracketed source range,
    /// when known, for the comments before the closing bracket.
    fn collection(
        &mut self,
        r: Range<usize>,
        open: &str,
        close: &str,
        items: Vec<Doc>,
        space: bool,
    ) -> Doc {
        // Comments before the opening bracket stay before it (a pattern
        // prints none of its own).
        let before_open = if r.end > 0 {
            self.comments_before(r.start, false)
        } else {
            Doc::Nil
        };
        let before_close = if r.end > 0 {
            self.comments_before(r.end - 1, false)
        } else {
            Doc::Nil
        };
        if items.is_empty() {
            return concat(vec![
                before_open,
                text_doc(open),
                before_close,
                text_doc(close),
            ]);
        }
        let c = self.cfg.collections.clone();
        let sp = if space { " " } else { "" };
        let trailing = match c.trailing_semicolon {
            TrailingSemicolon::Never => Doc::Nil,
            TrailingSemicolon::Vertical => if_break(text_doc(";"), Doc::Nil),
            TrailingSemicolon::Always => text_doc(";"),
        };
        let wrap = c.layout == Layout::Wrap;
        let body = match c.separator_position {
            SeparatorPosition::After => {
                let n = items.len();
                let units: Vec<Doc> = items
                    .into_iter()
                    .enumerate()
                    .map(|(i, d)| {
                        if i + 1 < n {
                            concat(vec![d, text_doc(";")])
                        } else {
                            d
                        }
                    })
                    .collect();
                if wrap {
                    Doc::Fill(join(units, Doc::Line))
                } else {
                    concat(join(units, Doc::Line))
                }
            }
            SeparatorPosition::Before => {
                let units: Vec<Doc> = items
                    .into_iter()
                    .enumerate()
                    .map(|(i, d)| {
                        if i > 0 {
                            concat(vec![text_doc("; "), d])
                        } else {
                            d
                        }
                    })
                    .collect();
                // `a; b` when flat, `; b` starting the line when broken.
                if wrap {
                    Doc::Fill(join(units, Doc::SoftLine))
                } else {
                    concat(join(units, Doc::SoftLine))
                }
            }
        };
        // A `//` after the last item is its, once read back.
        let body = if matches!(trailing, Doc::Nil) && is_trailing(&before_close) {
            attach_trailing(body, before_close)
        } else {
            concat(vec![body, trailing, before_close])
        };
        let list = match (c.style, c.separator_position) {
            (CollectionStyle::Aligned, SeparatorPosition::After) => group(concat(vec![
                text_doc(open),
                text_doc(sp),
                align(body),
                text_doc(sp),
                text_doc(close),
            ])),
            (CollectionStyle::Aligned, SeparatorPosition::Before) => group(align(concat(vec![
                text_doc(open),
                text_doc(sp),
                body,
                text_doc(sp),
                text_doc(close),
            ]))),
            (CollectionStyle::Block, _) => {
                let edge = if space { Doc::Line } else { Doc::SoftLine };
                group(concat(vec![
                    text_doc(open),
                    nest(self.cfg.indent, concat(vec![edge.clone(), body])),
                    edge,
                    text_doc(close),
                ]))
            }
        };
        concat(vec![before_open, list])
    }

    // ----- patterns ------------------------------------------------------

    fn pat(&mut self, p: &Pat) -> Doc {
        match p {
            Pat::Wild(sp) => self.leaf(range(*sp), "_"),
            Pat::Var(i) => self.ident(i),
            Pat::Mut(i) => concat(vec![text_doc("mut "), self.ident(i)]),
            Pat::Lit { lit, neg } => {
                let l = self.verbatim(range(lit.span()), true);
                if *neg {
                    concat(vec![text_doc("-"), l])
                } else {
                    l
                }
            }
            Pat::Bool(b, sp) => self.leaf(range(*sp), if *b { "true" } else { "false" }),
            Pat::Unit(sp) => self.unit(range(*sp)),
            Pat::Paren(inner) => {
                let i = match &**inner {
                    Pat::Constraint(p, t) => {
                        let p = self.pat(p);
                        let t = self.ty(t);
                        concat(vec![p, text_doc(self.colon()), t])
                    }
                    p => self.pat(p),
                };
                concat(vec![text_doc("("), i, text_doc(")")])
            }
            Pat::Tuple(ps) => {
                let ps = ps.iter().map(|p| self.pat(p)).collect();
                concat(join(ps, text_doc(", ")))
            }
            Pat::List(ps, sp) => {
                let space = self.cfg.collections.space_inside_lists;
                let ps = ps.iter().map(|p| self.pat(p)).collect();
                self.collection(range(*sp), "[", "]", ps, space)
            }
            Pat::Array(ps, sp) => {
                let space = self.cfg.collections.space_inside_arrays
                    || matches!(ps.first(), Some(Pat::Ref(..) | Pat::Lit { neg: true, .. }));
                let ps = ps.iter().map(|p| self.pat(p)).collect();
                self.collection(range(*sp), "[|", "|]", ps, space)
            }
            Pat::Cons(h, t, _) => {
                let h = self.pat(h);
                let t = self.pat(t);
                concat(vec![h, text_doc(" :: "), t])
            }
            Pat::Ctor(path, arg) => {
                let p = self.path(path);
                match arg {
                    Some(a) => {
                        let space = if self.touches(path_end(path)) {
                            ""
                        } else {
                            " "
                        };
                        let a = self.pat(a);
                        concat(vec![p, text_doc(space), a])
                    }
                    None => p,
                }
            }
            Pat::CtorRecord(path, fields, wild) => {
                let p = self.path(path);
                let mut items = Vec::new();
                for (f, v) in fields {
                    let name = self.ident(f);
                    items.push(match v {
                        Some(v) => {
                            let v = self.pat(v);
                            concat(vec![name, text_doc(" = "), v])
                        }
                        None => name,
                    });
                }
                if *wild {
                    items.push(text_doc("_"));
                }
                let space = self.cfg.collections.space_inside_records;
                let braces = self.braces_after(path_end(path));
                let body = self.collection(braces, "{", "}", items, space);
                concat(vec![p, text_doc(" "), body])
            }
            Pat::Or(ps) => {
                let ps = ps.iter().map(|p| self.pat(p)).collect();
                concat(join(ps, text_doc(" | ")))
            }
            Pat::Alias(p, name) => {
                let p = self.pat(p);
                let n = self.ident(name);
                concat(vec![p, text_doc(" as "), n])
            }
            Pat::Range(lo, hi, inclusive) => {
                // `0. ..=1.`, not `0...=1.`.
                let float_dot =
                    matches!(&**lo, Pat::Lit { lit, .. } if lit.to_string().ends_with('.'));
                let lo = self.pat(lo);
                let hi = self.pat(hi);
                let op = if *inclusive { "..=" } else { ".." };
                let op = if float_dot {
                    format!(" {}", op)
                } else {
                    op.to_string()
                };
                concat(vec![lo, text_doc(&op), hi])
            }
            Pat::Constraint(p, t) => {
                let p = self.pat(p);
                let t = self.ty(t);
                concat(vec![p, text_doc(self.colon()), t])
            }
            Pat::Ref(p, sp) => {
                let amp = self.leaf(start_of(*sp)..start_of(*sp) + 1, "&");
                // `& -5`, not `&-5` (one operator to the lexer).
                let space = pat_first_char(p).is_some_and(|c| c != '&' && is_op_char(c));
                let p = self.pat(p);
                concat(vec![amp, text_doc(if space { " " } else { "" }), p])
            }
            Pat::Label { label, pat, span } => {
                let tilde = self.leaf(start_of(*span)..start_of(*span) + 1, "~");
                let mut parts = vec![tilde];
                if let Some(l) = label {
                    parts.push(self.ident(l));
                    if let Some(p) = pat {
                        // `~y: &p`, not `~y:&p`.
                        let space = pat_first_char(p).is_some_and(is_op_char);
                        parts.push(text_doc(if space { ": " } else { ":" }));
                    }
                }
                if let Some(p) = pat {
                    parts.push(self.pat(p));
                }
                concat(parts)
            }
        }
    }

    // ----- types ---------------------------------------------------------

    fn ty(&mut self, t: &Type) -> Doc {
        match t {
            Type::Arrow(a, b) => {
                let a = self.ty(a);
                let b = self.ty(b);
                concat(vec![a, text_doc(" -> "), b])
            }
            Type::Tuple(ts) => {
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                concat(join(ts, text_doc(" * ")))
            }
            Type::ParenTuple(ts) => {
                let ts = ts.iter().map(|t| self.ty(t)).collect();
                concat(vec![
                    text_doc("("),
                    concat(join(ts, text_doc(", "))),
                    text_doc(")"),
                ])
            }
            // One token: comments before it go before the `'`.
            Type::Var(i) => {
                let r = range(i.span());
                self.leaf(r.start.saturating_sub(1)..r.end, format!("'{}", i))
            }
            Type::Paren(t) => {
                let t = self.ty(t);
                concat(vec![text_doc("("), t, text_doc(")")])
            }
            Type::Infer(sp) => self.leaf(range(*sp), "_"),
            Type::Unit(sp) => self.leaf(range(*sp), "()"),
            Type::Ref {
                lifetime,
                mutable,
                inner,
                span,
            } => {
                let amp = self.leaf(start_of(*span)..start_of(*span) + 1, "&");
                let mut parts = vec![amp];
                if let Some(l) = lifetime {
                    parts.push(text_doc(format!("'{} ", l)));
                }
                if *mutable {
                    parts.push(text_doc("mut "));
                }
                parts.push(self.ty(inner));
                concat(parts)
            }
            Type::Slice { elem, len } => {
                let e = self.ty(elem);
                let mut parts = vec![text_doc("["), e];
                if let Some(r) = len.as_ref().and_then(stream_range) {
                    parts.push(text_doc("; "));
                    parts.push(self.verbatim(r, true));
                }
                parts.push(text_doc("]"));
                concat(parts)
            }
            Type::Path { tokens, args } => {
                let path = self.tokens(tokens);
                let Some(args) = args else {
                    return path;
                };
                let mut docs = Vec::new();
                for a in args {
                    docs.push(match a {
                        GenericArg::Type(t) => self.ty(t),
                        GenericArg::Binding(name, t) => {
                            let n = self.ident(name);
                            let t = self.ty(t);
                            concat(vec![n, text_doc(" = "), t])
                        }
                        GenericArg::Lifetime(l, _) => concat(vec![text_doc("'"), self.ident(l)]),
                    });
                }
                concat(vec![
                    path,
                    text_doc("<"),
                    concat(join(docs, text_doc(", "))),
                    text_doc(">"),
                ])
            }
            Type::Rust(tokens) => self.tokens(tokens),
        }
    }
}

impl Printer<'_> {
    /// Rust tokens, printed as written.
    fn tokens(&mut self, ts: &proc_macro2::TokenStream) -> Doc {
        match stream_range(ts) {
            Some(r) => self.verbatim(r, true),
            None => text_doc(ts.to_string()),
        }
    }
}

/// The comments at the start of `d` that end a line of code, and the rest.
fn split_trailing(d: Doc) -> (Doc, Doc) {
    match d {
        Doc::Trailing { .. } => (d, Doc::Nil),
        Doc::Concat(ds) => {
            let n = ds
                .iter()
                .take_while(|d| matches!(d, Doc::Trailing { .. }))
                .count();
            let mut ds = ds;
            let rest = ds.split_off(n);
            let trailing = if ds.is_empty() { Doc::Nil } else { concat(ds) };
            (trailing, concat(rest))
        }
        d => (Doc::Nil, d),
    }
}

/// Whether `d` is only comments at the end of a line of code.
fn is_trailing(d: &Doc) -> bool {
    match d {
        Doc::Trailing { .. } => true,
        Doc::Concat(ds) => !ds.is_empty() && ds.iter().all(is_trailing),
        _ => false,
    }
}

/// A loop's body and `done`: `do begin .. end done` when it hugs.
fn loop_body(ind: usize, hug: bool, b: Doc) -> Doc {
    if hug {
        concat(vec![text_doc(" "), b, text_doc(" done")])
    } else {
        concat(vec![
            nest(ind, concat(vec![Doc::Line, b])),
            Doc::Line,
            text_doc("done"),
        ])
    }
}

fn text_doc(s: impl Into<String>) -> Doc {
    text(s)
}

/// Collects a chain of same-class operators: `a + b - c` is `[a, b, c]`.
fn flatten<'e>(
    e: &'e Expr,
    right: bool,
    same: &dyn Fn(&Op) -> bool,
    operands: &mut Vec<&'e Expr>,
    ops: &mut Vec<&'e Op>,
) {
    match e {
        Expr::Binary { op, lhs, rhs } if same(op) => {
            if right {
                operands.push(lhs);
                ops.push(op);
                flatten(rhs, right, same, operands, ops);
            } else {
                flatten(lhs, right, same, operands, ops);
                ops.push(op);
                operands.push(rhs);
            }
        }
        _ => operands.push(e),
    }
}

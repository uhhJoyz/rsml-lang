//! Turns a Rust `TokenStream` into OCaml-flavoured tokens.
//!
//! The Rust lexer has already run by the time we see the input, so the job here
//! is to undo the places where Rust and OCaml tokenize differently:
//!
//! * runs of joint punctuation are merged into one operator (`|`, `>` becomes `|>`),
//!   following OCaml's operator-character rules so `;` and `,` never merge;
//! * `'a` (which Rust lexes as a lifetime) becomes a type variable token;
//! * `(* comments *)` are dropped.

use proc_macro2::{Delimiter, Group, Ident, Literal, Spacing, Span, TokenStream, TokenTree};

#[derive(Clone)]
pub enum Tok {
    Ident(Ident),
    Op(Op),
    Lit(Literal),
    /// `'a`: an OCaml type variable, or a Rust lifetime inside a type.
    TyVar(Ident, Span),
    Group(Group),
}

/// An operator symbol such as `|>`, `::` or `+.`, with one span per character.
#[derive(Clone)]
pub struct Op {
    pub text: String,
    pub spans: Vec<Span>,
}

impl Op {
    pub fn new(text: &str, span: Span) -> Op {
        Op {
            text: text.to_string(),
            spans: text.chars().map(|_| span).collect(),
        }
    }

    pub fn span(&self) -> Span {
        self.spans[0]
    }

    pub fn last_span(&self) -> Span {
        *self.spans.last().unwrap()
    }

    /// Splits off the first `n` characters, returning `(front, rest)`.
    pub fn split(&self, n: usize) -> (Op, Op) {
        let front: String = self.text.chars().take(n).collect();
        let rest: String = self.text.chars().skip(n).collect();
        (
            Op {
                text: front,
                spans: self.spans[..n].to_vec(),
            },
            Op {
                text: rest,
                spans: self.spans[n..].to_vec(),
            },
        )
    }
}

impl Tok {
    pub fn span(&self) -> Span {
        match self {
            Tok::Ident(i) => i.span(),
            Tok::Op(o) => o.span(),
            Tok::Lit(l) => l.span(),
            Tok::TyVar(_, s) => *s,
            Tok::Group(g) => g.span_open(),
        }
    }
}

/// Characters that may continue an OCaml operator (plus `#`).
fn is_op_char(c: char) -> bool {
    "!$%&*+-./:<=>?@^|~#".contains(c)
}

pub fn lex(ts: TokenStream) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut iter = ts.into_iter().peekable();
    while let Some(tt) = iter.next() {
        match tt {
            TokenTree::Group(g) => {
                if !is_comment(&g) {
                    out.push(Tok::Group(g));
                }
            }
            TokenTree::Ident(i) => out.push(Tok::Ident(i)),
            TokenTree::Literal(l) => out.push(Tok::Lit(l)),
            TokenTree::Punct(p) => {
                let c = p.as_char();
                if c == '\''
                    && let Some(TokenTree::Ident(_)) = iter.peek()
                {
                    let Some(TokenTree::Ident(i)) = iter.next() else {
                        unreachable!()
                    };
                    out.push(Tok::TyVar(i, p.span()));
                    continue;
                }
                let mut op = Op {
                    text: c.to_string(),
                    spans: vec![p.span()],
                };
                let mut joint = p.spacing() == Spacing::Joint;
                // `;` only ever merges into `;;`, and `,` / `'` never merge.
                let mergeable = |first: char, next: char| {
                    if first == ';' {
                        next == ';'
                    } else {
                        is_op_char(first) && is_op_char(next)
                    }
                };
                while joint {
                    match iter.peek() {
                        Some(TokenTree::Punct(q))
                            if mergeable(c, q.as_char()) && op.text != ";;" =>
                        {
                            let Some(TokenTree::Punct(q)) = iter.next() else {
                                unreachable!()
                            };
                            op.text.push(q.as_char());
                            op.spans.push(q.span());
                            joint = q.spacing() == Spacing::Joint;
                        }
                        _ => break,
                    }
                }
                out.push(Tok::Op(op));
            }
        }
    }
    out
}

/// `(* ... *)` arrives as a parenthesized group starting and ending with `*`.
/// `( ** )` (the power operator) looks the same, so a lone joint `**` is kept.
pub fn is_comment(g: &Group) -> bool {
    if g.delimiter() != Delimiter::Parenthesis {
        return false;
    }
    let toks: Vec<TokenTree> = g.stream().into_iter().collect();
    if toks.len() < 2 {
        return false;
    }
    let star = |t: &TokenTree| matches!(t, TokenTree::Punct(p) if p.as_char() == '*');
    if !star(&toks[0]) || !star(toks.last().unwrap()) {
        return false;
    }
    if toks.len() == 2
        && let TokenTree::Punct(p) = &toks[0]
    {
        return p.spacing() != Spacing::Joint;
    }
    true
}

/// Whether `b` starts exactly where `a` ends (no whitespace in between).
/// `None` when the compiler does not expose source locations.
pub fn adjacent(a_end: Span, b_start: Span) -> Option<bool> {
    let e = a_end.end();
    let s = b_start.start();
    // A token has width; rust-analyzer reports every position as 1:1.
    if e.line != 0 && s.line != 0 && a_end.start() != e {
        return Some(e.line == s.line && e.column == s.column);
    }
    let (a_anchor, _, a_hi) = debug_range(a_end)?;
    let (b_anchor, b_lo, _) = debug_range(b_start)?;
    (a_anchor == b_anchor).then_some(a_hi == b_lo)
}

/// rust-analyzer's spans print as `SpanData { range: 12..14, anchor: .., ctx:
/// .. }`, a byte range relative to an anchor shared by the tokens of one
/// macro call. Its proc-macro server has no other way to tell whether two
/// tokens touch.
fn debug_range(span: Span) -> Option<(String, usize, usize)> {
    let s = format!("{:?}", span);
    let rest = s.strip_prefix("SpanData { range: ")?;
    let (range, rest) = rest.split_once(", anchor: ")?;
    let (anchor, _) = rest.split_once(", ctx: ")?;
    let (lo, hi) = range.split_once("..")?;
    Some((anchor.to_string(), lo.parse().ok()?, hi.parse().ok()?))
}

/// Turns tokens back into a Rust token stream (used for raw Rust fragments
/// such as generic arguments).
pub fn to_stream(toks: &[Tok]) -> TokenStream {
    let mut ts = TokenStream::new();
    for t in toks {
        match t {
            Tok::Ident(i) => ts.extend([TokenTree::Ident(i.clone())]),
            Tok::Lit(l) => ts.extend([TokenTree::Literal(l.clone())]),
            Tok::Group(g) => ts.extend([TokenTree::Group(g.clone())]),
            Tok::TyVar(i, s) => {
                let mut p = proc_macro2::Punct::new('\'', Spacing::Joint);
                p.set_span(*s);
                ts.extend([TokenTree::Punct(p), TokenTree::Ident(i.clone())]);
            }
            Tok::Op(o) => ts.extend(op_tokens(&o.text, &o.spans)),
        }
    }
    ts
}

pub fn op_tokens(text: &str, spans: &[Span]) -> TokenStream {
    let chars: Vec<char> = text.chars().collect();
    let mut ts = TokenStream::new();
    for (i, c) in chars.iter().enumerate() {
        let spacing = if i + 1 < chars.len() {
            Spacing::Joint
        } else {
            Spacing::Alone
        };
        let mut p = proc_macro2::Punct::new(*c, spacing);
        p.set_span(spans.get(i).copied().unwrap_or_else(|| spans[0]));
        ts.extend([TokenTree::Punct(p)]);
    }
    ts
}

/// The identifier as written, `r#` included. Keyword checks compare this,
/// so a raw identifier (`r#match`) is always a name.
pub fn word(i: &Ident) -> String {
    i.to_string()
}

/// The identifier's text without any `r#` prefix: the name it binds or
/// refers to (`r#x` and `x` are the same name).
pub fn ident_str(i: &Ident) -> String {
    let s = i.to_string();
    match s.strip_prefix("r#") {
        Some(rest) => rest.to_string(),
        None => s,
    }
}

pub fn is_capitalized(i: &Ident) -> bool {
    ident_str(i)
        .chars()
        .next()
        .is_some_and(|c| c.is_uppercase())
}

//! What the parser does not keep: comments and the exact positions of
//! tokens, so comments and blank lines can be put back.
//!
//! Spans of a `TokenStream` parsed outside a proc macro carry byte offsets
//! into the parsed text (proc-macro2's `span-locations`).

use std::ops::Range;

use proc_macro2::{Group, Span, TokenStream, TokenTree};
use rsml_compiler::lexer::is_comment;

#[derive(Clone, Debug)]
pub struct Comment {
    pub range: Range<usize>,
    /// A `//` comment, which must be followed by a newline.
    pub line_comment: bool,
}

pub struct Source<'a> {
    pub text: &'a str,
    pub comments: Vec<Comment>,
    /// Byte ranges of tokens (and of brackets), sorted, comments excluded.
    tokens: Vec<Range<usize>>,
}

pub fn range(span: Span) -> Range<usize> {
    span.byte_range()
}

impl<'a> Source<'a> {
    /// The contents of `group`, a block in `text`.
    pub fn new(text: &'a str, group: &Group) -> Source<'a> {
        let mut tokens = vec![range(group.span_open()), range(group.span_close())];
        let mut comments = Vec::new();
        collect(group.stream(), &mut tokens, &mut comments);
        tokens.sort_by_key(|r| r.start);
        // `//` and `/* */` comments are the gaps between tokens that are
        // not whitespace.
        for pair in tokens.windows(2) {
            scan_gap(text, pair[0].end, pair[1].start, &mut comments);
        }
        // A `//` inside a `(* .. *)` (which Rust reads as tokens) is part
        // of that comment.
        let groups: Vec<Range<usize>> = comments
            .iter()
            .filter(|c| !c.line_comment && text[c.range.clone()].starts_with("(*"))
            .map(|c| c.range.clone())
            .collect();
        comments.retain(|c| {
            !groups
                .iter()
                .any(|g| *g != c.range && g.start <= c.range.start && c.range.end <= g.end)
        });
        comments.sort_by_key(|c| c.range.start);
        Source {
            text,
            comments,
            tokens,
        }
    }

    /// Where the code or comment before `pos` ends.
    pub fn prev_end(&self, pos: usize) -> usize {
        let t = self
            .tokens
            .iter()
            .map(|r| r.end)
            .filter(|&e| e <= pos)
            .max()
            .unwrap_or(0);
        let c = self
            .comments
            .iter()
            .map(|c| c.range.end)
            .filter(|&e| e <= pos)
            .max()
            .unwrap_or(0);
        t.max(c)
    }

    /// Where the block's first token (not comment) starts.
    pub fn first_code(&self) -> usize {
        self.tokens.get(1).map_or(self.text.len(), |r| r.start)
    }

    /// Where the block's closing bracket starts.
    pub fn block_close(&self) -> usize {
        self.tokens.last().map_or(self.text.len(), |r| r.start)
    }

    /// The start of the first token at or after `pos`.
    pub fn token_at_or_after(&self, pos: usize) -> Option<usize> {
        self.tokens
            .iter()
            .map(|r| r.start)
            .filter(|&s| s >= pos)
            .min()
    }

    /// The last token that starts before `pos`.
    pub fn token_before(&self, pos: usize) -> Option<Range<usize>> {
        self.tokens
            .iter()
            .filter(|r| r.start < pos)
            .max_by_key(|r| r.start)
            .cloned()
    }

    /// Where the last token that ends at or before `pos` ends.
    pub fn code_end_before(&self, pos: usize) -> usize {
        self.tokens
            .iter()
            .map(|r| r.end)
            .filter(|&e| e <= pos)
            .max()
            .unwrap_or(0)
    }

    /// Starts of the tokens at or after `pos`, in order.
    fn starts_from(&self, pos: usize) -> impl Iterator<Item = usize> + '_ {
        let mut last = None;
        self.tokens.iter().map(|r| r.start).filter(move |&s| {
            let new = s >= pos && last != Some(s);
            last = Some(s);
            new
        })
    }

    /// The closing bracket matching the opening one at `open`.
    pub fn matching_close(&self, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        for s in self.starts_from(open) {
            match self.text[s..].chars().next() {
                Some('(' | '[' | '{') => depth += 1,
                Some(')' | ']' | '}') => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(s);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The closing bracket of the group holding the token at `pos`.
    pub fn enclosing_close(&self, pos: usize) -> Option<usize> {
        let mut depth = 0usize;
        for s in self.starts_from(pos) {
            match self.text[s..].chars().next() {
                Some('(' | '[' | '{') => depth += 1,
                Some(')' | ']' | '}') => match depth.checked_sub(1) {
                    Some(d) => depth = d,
                    None => return Some(s),
                },
                _ => {}
            }
        }
        None
    }

    /// Whether `pos` is right after the block's opening bracket.
    pub fn is_block_start(&self, pos: usize) -> bool {
        self.tokens.first().is_some_and(|open| open.end == pos)
    }

    /// Newlines in `a..b`, not counting those inside comments.
    pub fn newlines(&self, a: usize, b: usize) -> usize {
        if a >= b {
            return 0;
        }
        let mut n = self.text[a..b].matches('\n').count();
        for c in &self.comments {
            let s = c.range.start.max(a);
            let e = c.range.end.min(b);
            if s < e {
                n -= self.text[s..e].matches('\n').count();
            }
        }
        n
    }

    /// The column (in characters) of byte offset `pos`.
    pub fn column(&self, pos: usize) -> usize {
        let line_start = self.text[..pos].rfind('\n').map_or(0, |i| i + 1);
        self.text[line_start..pos].chars().count()
    }

    pub fn slice(&self, r: Range<usize>) -> &'a str {
        &self.text[r]
    }
}

fn collect(ts: TokenStream, tokens: &mut Vec<Range<usize>>, comments: &mut Vec<Comment>) {
    for tt in ts {
        match tt {
            TokenTree::Group(g) if is_comment(&g) => comments.push(Comment {
                range: range(g.span()),
                line_comment: false,
            }),
            TokenTree::Group(g) => {
                tokens.push(range(g.span_open()));
                tokens.push(range(g.span_close()));
                collect(g.stream(), tokens, comments);
            }
            t => tokens.push(range(t.span())),
        }
    }
}

fn scan_gap(text: &str, a: usize, b: usize, out: &mut Vec<Comment>) {
    let mut i = a;
    while i < b {
        let rest = &text[i..b];
        if rest.starts_with("//") {
            let end = rest.find('\n').map_or(b, |k| i + k);
            out.push(Comment {
                range: i..end,
                line_comment: true,
            });
            i = end;
        } else if rest.starts_with("/*") {
            let mut depth = 0;
            let mut j = i;
            while j < b {
                if text[j..].starts_with("/*") {
                    depth += 1;
                    j += 2;
                } else if text[j..].starts_with("*/") {
                    depth -= 1;
                    j += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    j += text[j..].chars().next().map_or(1, char::len_utf8);
                }
            }
            out.push(Comment {
                range: i..j,
                line_comment: false,
            });
            i = j;
        } else {
            i += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
}

/// Byte range covered by a token stream (for verbatim Rust fragments).
pub fn stream_range(ts: &TokenStream) -> Option<Range<usize>> {
    let mut out: Option<Range<usize>> = None;
    for t in ts.clone() {
        let r = range(t.span());
        out = Some(match out {
            None => r,
            Some(o) => o.start.min(r.start)..o.end.max(r.end),
        });
    }
    out
}

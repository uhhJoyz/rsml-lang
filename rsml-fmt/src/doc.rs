//! A Wadler/Prettier-style document: text plus places where lines may
//! break. A group is printed flat (its breaks are spaces) when it fits in
//! the remaining width, otherwise broken (its breaks are newlines).
//!
//! Newlines are written lazily: a break records a pending newline, which is
//! only emitted with the next text. That keeps lines free of trailing
//! whitespace, lets breaks collapse (two breaks in a row are one newline),
//! and lets a comment that followed code in the source be appended to the
//! line that code ended, even when the break after it was already decided.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Doc {
    Nil,
    /// Text without newlines.
    Text(String),
    /// Source text that may span lines (comments, string literals, Rust
    /// macro calls). With `src_col` (the column of the first line in the
    /// source), continuation lines move along with the first line; without,
    /// they are kept exactly (string contents).
    Verbatim {
        text: String,
        src_col: Option<usize>,
    },
    /// A space, or a newline when broken.
    Line,
    /// Nothing, or a newline when broken.
    SoftLine,
    /// Always a newline (breaks every enclosing group).
    HardLine,
    /// This many blank lines (breaks every enclosing group).
    BlankLines(usize),
    /// Indents the lines that start inside by `n` more columns.
    Nest(usize, Box<Doc>),
    /// Indents the lines that start inside to the current column.
    Align(Box<Doc>),
    Group {
        doc: Box<Doc>,
        id: Option<usize>,
        /// Contains a hard break, so it can never be flat.
        hard: bool,
    },
    /// The first alternative printed flat if it fits, else the first of the
    /// others whose first line fits, else the last one.
    Choice {
        alts: Vec<Doc>,
        hard: bool,
    },
    /// Depends on whether the enclosing group (or group `group`) is broken.
    IfBreak {
        broken: Box<Doc>,
        flat: Box<Doc>,
        group: Option<usize>,
    },
    /// `[item, sep, item, sep, item]`: each separator breaks only if the
    /// item after it does not fit on the line.
    Fill(Vec<Doc>),
    /// Always printed flat.
    Flat(Box<Doc>),
    Concat(Vec<Doc>),
    /// A comment that followed code on its source line.
    Trailing {
        text: String,
        line_comment: bool,
    },
    /// A comment before code on its source line, after other code: spaced
    /// from what it follows (`f (* c *) .0`), else at the start of its line.
    Inline(String),
    /// A comment on a line of its own.
    OwnLine {
        text: String,
        /// Blank lines before it.
        blank_before: usize,
        src_col: usize,
    },
}

pub fn text(s: impl Into<String>) -> Doc {
    Doc::Text(s.into())
}

pub fn concat(docs: Vec<Doc>) -> Doc {
    Doc::Concat(docs)
}

pub fn nest(n: usize, d: Doc) -> Doc {
    Doc::Nest(n, Box::new(d))
}

pub fn align(d: Doc) -> Doc {
    Doc::Align(Box::new(d))
}

pub fn flat(d: Doc) -> Doc {
    Doc::Flat(Box::new(d))
}

pub fn group(d: Doc) -> Doc {
    group_with_id(d, None)
}

pub fn group_with_id(d: Doc, id: Option<usize>) -> Doc {
    let hard = d.has_hard();
    Doc::Group {
        doc: Box::new(d),
        id,
        hard,
    }
}

pub fn choice(alts: Vec<Doc>) -> Doc {
    let hard = alts[0].has_hard();
    Doc::Choice { alts, hard }
}

pub fn if_break(broken: Doc, flat: Doc) -> Doc {
    Doc::IfBreak {
        broken: Box::new(broken),
        flat: Box::new(flat),
        group: None,
    }
}

pub fn if_group_breaks(id: usize, broken: Doc, flat: Doc) -> Doc {
    Doc::IfBreak {
        broken: Box::new(broken),
        flat: Box::new(flat),
        group: Some(id),
    }
}

/// `d` followed by `trailing`, comments at the end of its last line, put
/// inside the groups `d` ends with. That is where they are once printed
/// and read back (after its last token), so a `//` breaks those groups
/// the first time as it will the next.
pub fn attach_trailing(d: Doc, trailing: Doc) -> Doc {
    fn empty(d: &Doc) -> bool {
        match d {
            Doc::Nil => true,
            Doc::Text(s) => s.is_empty(),
            Doc::Concat(ds) => ds.iter().all(empty),
            _ => false,
        }
    }
    match d {
        Doc::Group { doc, id, hard } => {
            let doc = attach_trailing(*doc, trailing);
            let hard = hard || doc.has_hard();
            Doc::Group {
                doc: Box::new(doc),
                id,
                hard,
            }
        }
        Doc::Choice { alts, hard } => {
            let alts: Vec<Doc> = alts
                .into_iter()
                .map(|a| attach_trailing(a, trailing.clone()))
                .collect();
            let hard = hard || alts[0].has_hard();
            Doc::Choice { alts, hard }
        }
        Doc::Concat(mut ds) => match ds.iter().rposition(|x| !empty(x)) {
            Some(k)
                if !matches!(
                    ds[k],
                    Doc::Line | Doc::SoftLine | Doc::HardLine | Doc::BlankLines(_)
                ) =>
            {
                let last = std::mem::replace(&mut ds[k], Doc::Nil);
                ds[k] = attach_trailing(last, trailing);
                Doc::Concat(ds)
            }
            _ => Doc::Concat(vec![Doc::Concat(ds), trailing]),
        },
        Doc::Fill(mut items) if !items.is_empty() => {
            let last = items.pop().unwrap();
            items.push(attach_trailing(last, trailing));
            Doc::Fill(items)
        }
        Doc::Nest(n, d) => Doc::Nest(n, Box::new(attach_trailing(*d, trailing))),
        Doc::Align(d) => Doc::Align(Box::new(attach_trailing(*d, trailing))),
        d => Doc::Concat(vec![d, trailing]),
    }
}

/// `docs` with `sep` between them.
pub fn join(docs: Vec<Doc>, sep: Doc) -> Vec<Doc> {
    let mut out = Vec::with_capacity(docs.len() * 2);
    for (i, d) in docs.into_iter().enumerate() {
        if i > 0 {
            out.push(sep.clone());
        }
        out.push(d);
    }
    out
}

fn width(s: &str) -> usize {
    s.chars().count()
}

impl Doc {
    /// Whether the document always contains a newline.
    pub fn has_hard(&self) -> bool {
        match self {
            Doc::Nil | Doc::Text(_) | Doc::Inline(_) | Doc::Line | Doc::SoftLine => false,
            Doc::Verbatim { text, .. } => text.contains('\n'),
            Doc::HardLine | Doc::BlankLines(_) | Doc::OwnLine { .. } => true,
            Doc::Trailing { line_comment, .. } => *line_comment,
            Doc::Nest(_, d) | Doc::Align(d) | Doc::Flat(d) => d.has_hard(),
            Doc::Group { hard, .. } | Doc::Choice { hard, .. } => *hard,
            Doc::IfBreak { flat, .. } => flat.has_hard(),
            Doc::Fill(ds) | Doc::Concat(ds) => ds.iter().any(Doc::has_hard),
        }
    }

    /// The width of the document printed on one line (`None` if it cannot be).
    pub fn flat_width(&self) -> Option<usize> {
        Some(match self {
            Doc::Nil | Doc::SoftLine => 0,
            Doc::Text(s) => width(s),
            Doc::Line => 1,
            Doc::Verbatim { text, .. } if !text.contains('\n') => width(text),
            Doc::Trailing {
                text,
                line_comment: false,
            } => 1 + width(text),
            Doc::Inline(text) => 1 + width(text),
            Doc::Nest(_, d) | Doc::Align(d) | Doc::Flat(d) | Doc::Group { doc: d, .. } => {
                d.flat_width()?
            }
            Doc::Choice { alts, .. } => alts[0].flat_width()?,
            Doc::IfBreak { flat, .. } => flat.flat_width()?,
            Doc::Fill(ds) | Doc::Concat(ds) => {
                let mut n = 0;
                for d in ds {
                    n += d.flat_width()?;
                }
                n
            }
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Flat,
    Break,
}

#[derive(Clone, Copy)]
enum Item<'a> {
    Doc(&'a Doc),
    /// The remaining items of a [`Doc::Fill`].
    Fill(&'a [Doc]),
}

#[derive(Clone, Copy)]
struct Cmd<'a> {
    indent: usize,
    mode: Mode,
    item: Item<'a>,
}

pub struct Opts {
    pub width: usize,
    /// Indentation of the document's own lines.
    pub indent: usize,
    /// The column the first line starts at.
    pub start_col: usize,
    pub max_blank_lines: usize,
}

/// Prints `doc`; with `flat`, everything is on one line (only meaningful if
/// the document has no hard breaks).
pub fn render(doc: &Doc, opts: &Opts, flat: bool) -> String {
    let mut r = Renderer {
        out: String::new(),
        col: opts.start_col,
        pending: None,
        pending_space: false,
        after_line_comment: false,
        width: opts.width,
        max_newlines: opts.max_blank_lines + 1,
        modes: HashMap::new(),
    };
    let mode = if flat { Mode::Flat } else { Mode::Break };
    let mut stack = vec![Cmd {
        indent: opts.indent,
        mode,
        item: Item::Doc(doc),
    }];
    while let Some(cmd) = stack.pop() {
        r.step(cmd, &mut stack);
    }
    r.out
}

struct Renderer {
    out: String,
    col: usize,
    /// Newlines (count, indentation of the next line) not written yet.
    pending: Option<(usize, usize)>,
    /// A space not written yet (dropped if a newline comes first).
    pending_space: bool,
    /// A `//` comment ended the line: the next text starts a new one, at
    /// that text's own indentation.
    after_line_comment: bool,
    width: usize,
    max_newlines: usize,
    modes: HashMap<usize, Mode>,
}

impl Renderer {
    fn flush(&mut self, next: &str) {
        if let Some((n, indent)) = self.pending.take() {
            // A separator before a line break (`x + ` when a comment has
            // the next line) is not kept at the end of the line.
            while self.out.ends_with(' ') {
                self.out.pop();
            }
            for _ in 0..n {
                self.out.push('\n');
            }
            self.out.extend(std::iter::repeat_n(' ', indent));
            self.col = indent;
        } else if self.pending_space && !next.starts_with([' ', ')', ']', ';', ',']) {
            self.out.push(' ');
            self.col += 1;
        }
        self.pending_space = false;
    }

    fn write(&mut self, s: &str) {
        self.flush(s);
        self.out.push_str(s);
        self.col += width(s);
    }

    fn newline(&mut self, n: usize, indent: usize) {
        self.after_line_comment = false;
        let n = n.min(self.max_newlines);
        let prev = self.pending.map_or(0, |(p, _)| p);
        self.pending = Some((n.max(prev), indent));
    }

    fn cur_col(&self) -> usize {
        self.pending
            .map_or(self.col + self.pending_space as usize, |(_, indent)| indent)
    }

    fn verbatim(&mut self, text: &str, src_col: Option<usize>) {
        self.flush(text);
        let shift = src_col.map_or(0, |c| self.col as isize - c as isize);
        for (i, line) in text.split('\n').enumerate() {
            if i == 0 {
                self.out.push_str(line);
                self.col += width(line);
                continue;
            }
            self.out.push('\n');
            let body = if src_col.is_some() {
                line.trim_start_matches(' ')
            } else {
                line
            };
            let lead = if body.is_empty() || src_col.is_none() {
                0
            } else {
                (((line.len() - body.len()) as isize) + shift).max(0) as usize
            };
            self.out.extend(std::iter::repeat_n(' ', lead));
            self.out.push_str(body);
            self.col = lead + width(body);
        }
    }

    fn step<'a>(&mut self, cmd: Cmd<'a>, stack: &mut Vec<Cmd<'a>>) {
        let Cmd { indent, mode, item } = cmd;
        let doc = match item {
            Item::Doc(d) => d,
            Item::Fill(items) => return self.fill(items, indent, mode, stack),
        };
        let push = |stack: &mut Vec<Cmd<'a>>, indent, mode, d: &'a Doc| {
            stack.push(Cmd {
                indent,
                mode,
                item: Item::Doc(d),
            })
        };
        match doc {
            Doc::Nil => {}
            Doc::Text(s) => {
                if self.after_line_comment {
                    // A separator after a `//` comment: the line break is it.
                    if s.trim().is_empty() {
                        return;
                    }
                    self.newline(1, indent);
                }
                self.write(s)
            }
            Doc::Verbatim { text, src_col } => {
                if self.after_line_comment {
                    self.newline(1, indent);
                }
                self.verbatim(text, *src_col)
            }
            Doc::Line => match mode {
                Mode::Flat => {
                    if self.pending.is_none() {
                        self.pending_space = true;
                    }
                }
                Mode::Break => self.newline(1, indent),
            },
            Doc::SoftLine => {
                if mode == Mode::Break {
                    self.newline(1, indent)
                }
            }
            Doc::HardLine => self.newline(1, indent),
            Doc::BlankLines(n) => self.newline(n + 1, indent),
            Doc::Nest(n, d) => push(stack, indent + n, mode, d),
            Doc::Align(d) => push(stack, self.cur_col(), mode, d),
            Doc::Group { doc, id, hard } => {
                let m = if *hard {
                    Mode::Break
                } else if mode == Mode::Flat || self.fits(&[(Mode::Flat, doc)], stack, false) {
                    Mode::Flat
                } else {
                    Mode::Break
                };
                if let Some(id) = id {
                    self.modes.insert(*id, m);
                }
                push(stack, indent, m, doc);
            }
            Doc::Choice { alts, .. } => {
                if mode == Mode::Flat
                    || (!alts[0].has_hard() && self.fits(&[(Mode::Flat, &alts[0])], stack, false))
                {
                    return push(stack, indent, Mode::Flat, &alts[0]);
                }
                let last = alts.len() - 1;
                for alt in &alts[1..last] {
                    if self.fits(&[(Mode::Break, alt)], stack, false) {
                        return push(stack, indent, Mode::Break, alt);
                    }
                }
                push(stack, indent, Mode::Break, &alts[last]);
            }
            Doc::IfBreak {
                broken,
                flat,
                group,
            } => {
                let m = group
                    .and_then(|g| self.modes.get(&g).copied())
                    .unwrap_or(mode);
                let d = if m == Mode::Break { broken } else { flat };
                push(stack, indent, mode, d);
            }
            Doc::Fill(items) => stack.push(Cmd {
                indent,
                mode,
                item: Item::Fill(items),
            }),
            Doc::Flat(d) => push(stack, indent, Mode::Flat, d),
            Doc::Concat(ds) => {
                for d in ds.iter().rev() {
                    push(stack, indent, mode, d);
                }
            }
            Doc::Trailing { text, line_comment } => {
                if self.pending.is_some() {
                    // Goes at the end of the line the code was on.
                    self.out.push(' ');
                    self.out.push_str(text);
                } else {
                    // `f((* c *))`, not `f( (* c *))`.
                    self.pending_space =
                        !self.out.is_empty() && !self.out.ends_with(['(', '[', ' ']);
                    self.write(text);
                    if *line_comment {
                        self.after_line_comment = true;
                    } else {
                        self.pending_space = true;
                    }
                }
            }
            Doc::Inline(text) => {
                if self.after_line_comment {
                    self.newline(1, indent);
                }
                if self.pending.is_none() {
                    // `f((* c *) x)`, not `f( (* c *) x)`.
                    self.pending_space =
                        !self.out.is_empty() && !self.out.ends_with(['(', '[', ' ']);
                }
                self.write(text);
                self.pending_space = true;
            }
            Doc::OwnLine {
                text,
                blank_before,
                src_col,
            } => {
                if !self.out.is_empty() {
                    self.newline(blank_before + 1, indent);
                }
                self.verbatim(text, Some(*src_col));
                self.newline(1, indent);
            }
        }
    }

    fn fill<'a>(&mut self, items: &'a [Doc], indent: usize, mode: Mode, stack: &mut Vec<Cmd<'a>>) {
        let Some(content) = items.first() else {
            return;
        };
        let mode_of = |fits: bool| if fits { Mode::Flat } else { Mode::Break };
        let content_fits = self.fits(&[(Mode::Flat, content)], &[], true);
        if items.len() >= 3 {
            let pair_fits = self.fits(
                &[
                    (Mode::Flat, content),
                    (Mode::Flat, &items[1]),
                    (Mode::Flat, &items[2]),
                ],
                &[],
                true,
            );
            stack.push(Cmd {
                indent,
                mode,
                item: Item::Fill(&items[2..]),
            });
            stack.push(Cmd {
                indent,
                mode: mode_of(pair_fits),
                item: Item::Doc(&items[1]),
            });
        }
        stack.push(Cmd {
            indent,
            mode: mode_of(content_fits),
            item: Item::Doc(content),
        });
    }

    /// Whether `docs`, followed by the rest of the line from `rest`, fit in
    /// the remaining width. Measuring stops at the first line break.
    fn fits(&self, docs: &[(Mode, &Doc)], rest: &[Cmd], must_be_flat: bool) -> bool {
        let mut rem = self.width as isize - self.cur_col() as isize;
        let mut stack: Vec<(Mode, &Doc)> = docs.iter().rev().copied().collect();
        let mut rest_i = rest.len();
        loop {
            let Some((mode, doc)) = stack.pop() else {
                if rest_i == 0 {
                    return true;
                }
                rest_i -= 1;
                let cmd = rest[rest_i];
                match cmd.item {
                    Item::Doc(d) => stack.push((cmd.mode, d)),
                    Item::Fill(items) => stack.extend(items.iter().rev().map(|d| (cmd.mode, d))),
                }
                continue;
            };
            match doc {
                Doc::Nil => {}
                Doc::Text(s) => rem -= width(s) as isize,
                Doc::Verbatim { text, .. } => {
                    let first = text.split('\n').next().unwrap_or("");
                    rem -= width(first) as isize;
                    if text.contains('\n') {
                        return rem >= 0;
                    }
                }
                Doc::Line => match mode {
                    Mode::Break => return true,
                    Mode::Flat => rem -= 1,
                },
                Doc::SoftLine => {
                    if mode == Mode::Break {
                        return true;
                    }
                }
                Doc::HardLine | Doc::BlankLines(_) | Doc::OwnLine { .. } => return true,
                Doc::Inline(text) => rem -= 1 + width(text) as isize,
                Doc::Trailing { text, line_comment } => {
                    rem -= 1 + width(text) as isize;
                    if *line_comment {
                        return rem >= 0;
                    }
                }
                Doc::Nest(_, d) | Doc::Align(d) => stack.push((mode, d)),
                Doc::Flat(d) => stack.push((Mode::Flat, d)),
                Doc::Group { doc, hard, .. } => {
                    if must_be_flat && *hard {
                        return false;
                    }
                    stack.push((if *hard { Mode::Break } else { mode }, doc));
                }
                Doc::Choice { alts, hard } => {
                    let m = if *hard { Mode::Break } else { mode };
                    let alt = if m == Mode::Break {
                        alts.last().unwrap()
                    } else {
                        &alts[0]
                    };
                    stack.push((m, alt));
                }
                Doc::IfBreak {
                    broken,
                    flat,
                    group,
                } => {
                    let m = group
                        .and_then(|g| self.modes.get(&g).copied())
                        .unwrap_or(mode);
                    stack.push((mode, if m == Mode::Break { broken } else { flat }));
                }
                Doc::Fill(ds) | Doc::Concat(ds) => stack.extend(ds.iter().rev().map(|d| (mode, d))),
            }
            if rem < 0 {
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(d: &Doc, width: usize) -> String {
        render(
            d,
            &Opts {
                width,
                indent: 0,
                start_col: 0,
                max_blank_lines: 1,
            },
            false,
        )
    }

    #[test]
    fn groups_break_when_too_wide() {
        let d = group(concat(vec![
            text("f"),
            nest(
                2,
                concat(vec![Doc::Line, text("aaaa"), Doc::Line, text("bbbb")]),
            ),
        ]));
        assert_eq!(show(&d, 20), "f aaaa bbbb");
        assert_eq!(show(&d, 8), "f\n  aaaa\n  bbbb");
    }

    #[test]
    fn fill_wraps() {
        let d = nest(
            2,
            Doc::Fill(vec![
                text("f"),
                Doc::Line,
                text("aaaa"),
                Doc::Line,
                text("bbbb"),
                Doc::Line,
                text("cc"),
            ]),
        );
        assert_eq!(show(&d, 11), "f aaaa bbbb\n  cc");
    }

    #[test]
    fn trailing_comments_stay_on_their_line() {
        let d = concat(vec![
            text("a"),
            Doc::HardLine,
            Doc::Trailing {
                text: "(* c *)".into(),
                line_comment: false,
            },
            text("b"),
        ]);
        assert_eq!(show(&d, 80), "a (* c *)\nb");
    }

    #[test]
    fn choice_takes_the_first_fitting_alternative() {
        let body = group(nest(2, concat(vec![Doc::Line, text("body_body_body")])));
        let d = choice(vec![
            concat(vec![text("f"), body.clone()]),
            concat(vec![text("f"), body.clone()]),
            concat(vec![text("f"), nest(2, concat(vec![Doc::HardLine, body]))]),
        ]);
        assert_eq!(show(&d, 80), "f body_body_body");
        assert_eq!(show(&d, 10), "f\n  body_body_body");
    }
}

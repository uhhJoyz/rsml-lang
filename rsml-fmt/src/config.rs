//! Formatting options, read from `rsmlfmt.toml`.
//!
//! Every option has a default, so a config file only lists what it changes.
//! Partial tables are merged onto the defaults: `[apply] layout = "wrap"`
//! keeps the other `apply` defaults. `DEFAULT_TOML` documents every option.

use serde::{Deserialize, Serialize};
use toml::{Table, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Maximum line width, counted from the start of the line (Rust
    /// indentation included).
    pub max_width: usize,
    /// Spaces per nesting level inside rsml code.
    pub indent: usize,
    /// Indentation of an `ml! { .. }` block's body relative to the line
    /// holding its `{` (`0`: the body and the `}` line up with that line).
    pub block_indent: usize,
    /// Indentation of a function's body relative to its line: an `rsml!`
    /// function's, and a Rust function's that is one `ml!` block (see
    /// `hug_fn_body`).
    pub fn_body_indent: usize,
    /// A Rust function whose body is one `ml! { .. }` block gets
    /// `{ ml! {` on its line and `}}` to close: the block reads as the body.
    pub hug_fn_body: bool,
    /// Indent the items of an `rsml! { .. }` one level from the line with
    /// `rsml! {` (`false`: at that line's indentation).
    pub indent_rsml_items: bool,
    /// When an `ml! { .. }` block is kept on one line.
    pub single_line_blocks: SingleLineBlocks,
    /// Keep a short `rsml!` function body on the signature's line.
    pub fn_single_line: bool,
    /// Keep blank lines between toplevel phrases, `let .. in` steps and
    /// sequence items.
    pub preserve_blank_lines: bool,
    /// The most consecutive blank lines kept.
    pub max_blank_lines: usize,
    /// `( + )` or `(+)`.
    pub op_section_spaces: OpSectionSpaces,
    /// `(x : int)` or `(x: int)`.
    pub colon_spacing: ColonSpacing,
    /// Keep `begin .. end` or turn it into parentheses.
    pub begin_end: BeginEnd,
    /// Keep `begin`, `unsafe {` and `async {` on the line of the `then`,
    /// `else`, `->` or `do` before them when the block breaks.
    pub begin_end_hug: bool,
    /// Keep `(* .. *)` comments or write them as Rust comments.
    pub comments: Comments,
    /// How the Rust inside `rsml! { .. }` (signatures, types, `use`s,
    /// `#[rust]` bodies) is formatted.
    pub rust_items: RustItems,
    /// The rustfmt to run.
    pub rustfmt_path: String,
    pub toplevel: Toplevel,
    #[serde(rename = "let")]
    pub let_: Let,
    pub fun: Fun,
    pub app: App,
    #[serde(rename = "match")]
    pub match_: Match,
    #[serde(rename = "if")]
    pub if_: If,
    pub sequence: Sequence,
    pub collections: Collections,
    /// `|>`
    pub pipe: Infix,
    /// `@@`
    pub apply: Infix,
    /// `>>`, `<<`
    pub compose: Infix,
    /// `&&`, `||`
    pub boolean: Infix,
    /// `::`, `@`, `^`
    pub cons: Infix,
    /// `+`, `-`, `*`, `/`, `**`, `mod`, `land`, `lsl`, ..
    pub arithmetic: Infix,
    /// `=`, `<>`, `<`, `>`, `<=`, `>=`, `==`, `!=`
    pub comparison: Infix,
    /// User-defined operators (`>>=`, `+!`, ..).
    pub custom_operators: Infix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SingleLineBlocks {
    /// One line whenever the formatted body fits.
    WhenFits,
    /// One line only if the block already was on one line (and fits).
    Preserve,
    /// Always put the body on its own lines.
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpSectionSpaces {
    /// `( + )`
    Always,
    /// `(+)`, but `( * )` (`(*` would start a comment).
    Minimal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColonSpacing {
    /// `x : int` (OCaml)
    Both,
    /// `x: int` (Rust)
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeginEnd {
    Preserve,
    /// `begin a; b end` becomes `(a; b)`.
    Parens,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustItems {
    /// By rustfmt, with the OCaml bodies put back.
    Rustfmt,
    /// As written.
    Preserve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comments {
    Preserve,
    /// `(* c *)` becomes `// c` when it has a line to itself, and
    /// `/* c */` otherwise.
    Rust,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toplevel {
    /// Which phrases get `;;`.
    pub double_semi: DoubleSemi,
    /// Where `;;` goes.
    pub double_semi_position: DoubleSemiPosition,
    /// Blank lines between toplevel phrases.
    pub blank_lines: ToplevelBlankLines,
    /// Whether phrases may share a line (`external f : _ -> _ ;; f 1`).
    #[serde(rename = "break")]
    pub break_: Break,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoubleSemi {
    /// Only where it is required: between an expression and another phrase.
    Minimal,
    /// After every phrase but the last.
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoubleSemiPosition {
    /// `let x = 1 ;;`
    EndOfLine,
    /// `;;` on a line of its own.
    OwnLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToplevelBlankLines {
    /// As in the source (see `preserve_blank_lines`).
    Preserve,
    /// One blank line between phrases.
    Always,
    /// None.
    Never,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Let {
    /// Where `in` goes.
    pub in_position: InPosition,
    /// The body after `in`.
    pub body: LetBody,
    /// Indentation of a binding's value when it goes below `=`.
    pub binding_indent: usize,
    /// Keep `fun` / `function` on the `=` line: `let f = fun x ->`.
    pub fun_on_eq_line: bool,
    /// A blank line before each `and`.
    pub and_blank_line: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InPosition {
    /// At the end of the binding, or on its own line when the binding
    /// spans several lines.
    Auto,
    /// Always at the end of the binding's last line.
    EndOfLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LetBody {
    /// The body always starts a new line.
    AlwaysBreak,
    /// `let x = 1 in x + 1` stays on one line when it fits.
    Fit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fun {
    /// Indentation of a lambda body that goes below `->`.
    pub body_indent: usize,
    /// Keep the line open for a lambda that is the last argument:
    /// `Iterator::map _ (fun x ->` with the body below.
    pub hug_last_arg: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
    /// Arguments of an application that does not fit.
    pub args: Layout,
    /// Indentation of arguments that go to the next line.
    pub indent: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// All on one line, or one per line.
    FitOrVertical,
    /// As many per line as fit.
    Wrap,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Match {
    /// The `|` before the first case (`match` and `function`).
    pub leading_bar: LeadingBar,
    /// Whether a short `match` may stay on one line.
    pub cases: Cases,
    /// Indentation of the `| case` lines relative to `match`.
    pub case_indent: usize,
    /// Where a case's body goes.
    pub body: CaseBody,
    /// Indentation of a case body that goes below its pattern.
    pub body_indent: usize,
    /// Align the `->` of all cases (when every pattern is one line).
    pub align_arrows: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadingBar {
    /// Even on one line: `match x with | A -> 1 | B -> 2`.
    Always,
    /// Only when the cases are on separate lines.
    WhenBroken,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cases {
    /// One line if the whole `match` fits, else one case per line.
    FitOrVertical,
    /// Always one case per line.
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseBody {
    /// After `->` when it fits, else below.
    Fit,
    /// Always below `->`.
    NextLine,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct If {
    pub style: IfStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IfStyle {
    /// One line if it fits, else `then` / `else` end lines and the
    /// branches are indented below them.
    FitOrVertical,
    /// Like `fit_or_vertical`, but never on one line.
    Vertical,
    /// One line if it fits, else `if c then a` / `else b`.
    Compact,
    /// One line if it fits, else `if c` / `then a` / `else b`.
    KeywordFirst,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    /// Where `;` goes.
    pub style: SequenceStyle,
    /// Whether `a; b` may stay on one line.
    #[serde(rename = "break")]
    pub break_: Break,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SequenceStyle {
    /// `a;` at the end of the line.
    Terminator,
    /// `; b` at the start of the next line.
    Separator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Break {
    /// One item per line.
    Always,
    /// One line when it fits.
    Fit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collections {
    /// How lists, arrays and records break.
    pub style: CollectionStyle,
    /// Items of a broken collection.
    pub layout: Layout,
    /// `;` at the end of an item, or at the start of the next one.
    pub separator_position: SeparatorPosition,
    /// A `;` after the last item.
    pub trailing_semicolon: TrailingSemicolon,
    /// `[ 1; 2 ]` instead of `[1; 2]`.
    pub space_inside_lists: bool,
    /// `[| 1; 2 |]` instead of `[|1; 2|]`.
    pub space_inside_arrays: bool,
    /// `{ x = 1 }` instead of `{x = 1}`.
    pub space_inside_records: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionStyle {
    /// Items aligned after the bracket: `[ a;` / `  b ]`.
    Aligned,
    /// Brackets on their own lines, items indented.
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeparatorPosition {
    After,
    Before,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailingSemicolon {
    Never,
    /// Only when the collection spans several lines.
    Vertical,
    Always,
}

/// Layout of a chain of one class of infix operators (`a |> f |> g`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Infix {
    /// `fit`: one line, or one operator per line. `wrap`: as many per line as fit.
    pub layout: InfixLayout,
    /// The operator ends a line (`trailing`) or starts the next (`leading`).
    pub position: OperatorPosition,
    /// Continuation lines: at the chain's indentation (`none`), one level
    /// deeper (`indent`), or under the first operator (`align`, leading;
    /// the first operand stays on the first line) / first operand (trailing).
    pub indent: InfixIndent,
    /// Chains with at least this many operators always break (`0`: never forced).
    pub force_break_at: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfixLayout {
    Fit,
    Wrap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorPosition {
    Leading,
    Trailing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfixIndent {
    None,
    Indent,
    Align,
}

fn infix(layout: InfixLayout, position: OperatorPosition, indent: InfixIndent) -> Infix {
    Infix {
        layout,
        position,
        indent,
        force_break_at: 0,
    }
}

impl Default for Config {
    fn default() -> Config {
        use InfixIndent as I;
        use InfixLayout as L;
        use OperatorPosition as P;
        Config {
            max_width: 100,
            indent: 2,
            block_indent: 0,
            fn_body_indent: 4,
            hug_fn_body: true,
            indent_rsml_items: false,
            single_line_blocks: SingleLineBlocks::WhenFits,
            fn_single_line: false,
            preserve_blank_lines: true,
            max_blank_lines: 1,
            op_section_spaces: OpSectionSpaces::Always,
            colon_spacing: ColonSpacing::Both,
            begin_end: BeginEnd::Preserve,
            begin_end_hug: true,
            comments: Comments::Preserve,
            rust_items: RustItems::Rustfmt,
            rustfmt_path: "rustfmt".to_string(),
            toplevel: Toplevel {
                double_semi: DoubleSemi::Minimal,
                double_semi_position: DoubleSemiPosition::EndOfLine,
                blank_lines: ToplevelBlankLines::Preserve,
                break_: Break::Always,
            },
            let_: Let {
                in_position: InPosition::Auto,
                body: LetBody::AlwaysBreak,
                binding_indent: 2,
                fun_on_eq_line: true,
                and_blank_line: false,
            },
            fun: Fun {
                body_indent: 2,
                hug_last_arg: true,
            },
            app: App {
                args: Layout::Wrap,
                indent: 2,
            },
            match_: Match {
                leading_bar: LeadingBar::WhenBroken,
                cases: Cases::FitOrVertical,
                case_indent: 0,
                body: CaseBody::Fit,
                body_indent: 4,
                align_arrows: false,
            },
            if_: If {
                style: IfStyle::FitOrVertical,
            },
            sequence: Sequence {
                style: SequenceStyle::Terminator,
                break_: Break::Always,
            },
            collections: Collections {
                style: CollectionStyle::Aligned,
                layout: Layout::FitOrVertical,
                separator_position: SeparatorPosition::After,
                trailing_semicolon: TrailingSemicolon::Never,
                space_inside_lists: false,
                space_inside_arrays: false,
                space_inside_records: true,
            },
            pipe: infix(L::Fit, P::Leading, I::None),
            apply: infix(L::Fit, P::Trailing, I::Indent),
            compose: infix(L::Fit, P::Leading, I::Indent),
            boolean: infix(L::Fit, P::Leading, I::None),
            cons: infix(L::Fit, P::Trailing, I::Indent),
            arithmetic: infix(L::Wrap, P::Leading, I::Indent),
            comparison: infix(L::Fit, P::Leading, I::Indent),
            custom_operators: infix(L::Fit, P::Leading, I::None),
        }
    }
}

impl Config {
    /// Reads a config file's contents: the options it sets, merged onto the defaults.
    pub fn from_toml(text: &str) -> Result<Config, String> {
        let table: Table = text.parse().map_err(|e| format!("{e}"))?;
        Config::default().merged(table)
    }

    /// This config with the options in `overrides` replaced. Keys may be
    /// dotted (`pipe.indent`); unknown keys and bad values are errors.
    pub fn merged(&self, overrides: Table) -> Result<Config, String> {
        let mut base = Table::try_from(self).map_err(|e| e.to_string())?;
        merge(&mut base, overrides);
        Value::Table(base)
            .try_into()
            .map_err(|e: toml::de::Error| e.message().to_string())
    }

    /// Applies `key = value` overrides written as TOML lines, or separated
    /// by `;` (as in a `(* rsmlfmt: .. *)` comment).
    pub fn with_overrides(&self, text: &str) -> Result<Config, String> {
        let toml_text: String = text
            .split([';', '\n'])
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|l| format!("{}\n", bare_word_value(l)))
            .collect();
        let table: Table = toml_text.parse().map_err(|e| format!("{e}"))?;
        self.merged(table)
    }

    /// The options as TOML.
    pub fn to_toml(&self) -> String {
        toml::to_string(self).expect("config serializes")
    }

    pub(crate) fn infix(&self, class: OpClass) -> &Infix {
        match class {
            OpClass::Pipe => &self.pipe,
            OpClass::Apply => &self.apply,
            OpClass::Compose => &self.compose,
            OpClass::Boolean => &self.boolean,
            OpClass::Cons => &self.cons,
            OpClass::Arithmetic => &self.arithmetic,
            OpClass::Comparison => &self.comparison,
            OpClass::Custom => &self.custom_operators,
        }
    }
}

/// `key = word` becomes `key = "word"`, so command-line and comment
/// overrides need no quotes around enum values.
fn bare_word_value(line: &str) -> String {
    if let Some((k, v)) = line.split_once('=') {
        let v = v.trim();
        let is_word = !v.is_empty() && v.chars().all(|c| c.is_ascii_alphabetic() || c == '_');
        if is_word && v != "true" && v != "false" {
            return format!("{} = \"{}\"", k.trim(), v);
        }
    }
    line.to_string()
}

fn merge(base: &mut Table, overrides: Table) {
    for (k, v) in overrides {
        match (base.get_mut(&k), v) {
            (Some(Value::Table(b)), Value::Table(o)) => merge(b, o),
            (_, v) => {
                base.insert(k, v);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum OpClass {
    Pipe,
    Apply,
    Compose,
    Boolean,
    Cons,
    Arithmetic,
    Comparison,
    Custom,
}

impl OpClass {
    pub(crate) fn of(op: &str) -> OpClass {
        match op {
            "|>" => OpClass::Pipe,
            "@@" => OpClass::Apply,
            ">>" | "<<" => OpClass::Compose,
            "&&" | "||" | "&" | "or" => OpClass::Boolean,
            "::" | "@" | "^" | "^^" => OpClass::Cons,
            "+" | "-" | "*" | "/" | "+." | "-." | "*." | "/." | "**" | "%" | "mod" | "land"
            | "lor" | "lxor" | "lsl" | "lsr" | "asr" => OpClass::Arithmetic,
            "=" | "<>" | "<" | ">" | "<=" | ">=" | "==" | "!=" => OpClass::Comparison,
            _ => OpClass::Custom,
        }
    }
}

/// Every option with its default value, as a commented `rsmlfmt.toml`.
pub const DEFAULT_TOML: &str = r#"# rsmlfmt.toml: options of rsml-fmt. Every key is optional; this file
# lists the defaults. Tables can be partial: `[pipe]` with only
# `indent = "align"` keeps the other pipe defaults.

# Maximum line width, counted from the start of the line (Rust indentation included).
max_width = 100
# Spaces per nesting level inside rsml code.
indent = 2
# Indentation of an `ml! { .. }` block's body relative to the line holding its `{` (0: the
# body and the `}` line up with that line).
block_indent = 0
# Indentation of a function's body relative to its line: an `rsml!` function's, and a Rust
# function's that is one `ml!` block (see `hug_fn_body`).
fn_body_indent = 4
# A Rust function whose body is one `ml! { .. }` block gets `{ ml! {` on its line and `}}`
# to close, so that the block reads as the body (rustfmt alone undoes this: format with
# `rsml-fmt --rustfmt`).
hug_fn_body = true
# Indent the items of an `rsml! { .. }` one level from the line with `rsml! {` (false: at
# that line's indentation).
indent_rsml_items = false
# When `ml! { .. }` stays on one line: "when_fits" | "preserve" (only if it
# already was) | "never".
single_line_blocks = "when_fits"
# Keep a short `rsml!` function body on the signature's line.
fn_single_line = false
# Keep blank lines between toplevel phrases, `let .. in` steps and sequence items.
preserve_blank_lines = true
# The most consecutive blank lines kept.
max_blank_lines = 1
# "always": `( + )` | "minimal": `(+)` (but `( * )`, since `(*` opens a comment).
op_section_spaces = "always"
# "both": `(x : int)` | "after": `(x: int)`.
colon_spacing = "both"
# "preserve" | "parens": `begin a; b end` becomes `(a; b)`.
begin_end = "preserve"
# Keep `begin` (and `unsafe {`, `async {`) on the line of the `then`, `else`, `->` or `do`
# before it when the block breaks, with `end` lined up with that line (`false`: the block on
# lines of its own, indented).
begin_end_hug = true
# "preserve" | "rust": `(* c *)` becomes `// c` when it has a line to itself, else `/* c */`
# (Rust comments, which editors color as comments; the layout stays the same).
comments = "preserve"
# "rustfmt": the Rust inside `rsml! { .. }` (items, signatures, `#[rust]` bodies) is formatted
# by rustfmt, honouring the nearest rustfmt.toml and the crate's edition | "preserve": as written.
rust_items = "rustfmt"
# The rustfmt to run (a path, or a name looked up on the PATH).
rustfmt_path = "rustfmt"

[toplevel]
# Which phrases get `;;`: "minimal" (where it is required: between an expression and
# another phrase) | "always" (after every phrase but the last).
double_semi = "minimal"
# "end_of_line": `let x = 1 ;;` | "own_line": `;;` alone on a line.
double_semi_position = "end_of_line"
# Blank lines between phrases: "preserve" | "always" | "never".
blank_lines = "preserve"
# "always": one phrase per line | "fit": `external f : _ -> _ ;; f 1` stays on
# one line when it fits.
break = "always"

[let]
# "auto": `in` ends the binding, or goes on its own line when the binding spans
# several lines | "end_of_line": always at the end of the binding.
in_position = "auto"
# The body after `in`: "always_break" (new line) | "fit" (`let x = 1 in x + 1`
# stays on one line when it fits).
body = "always_break"
# Indentation of a binding's value when it goes below `=`.
binding_indent = 2
# Keep `fun` / `function` on the `=` line: `let f = fun x ->`.
fun_on_eq_line = true
# A blank line before each `and`.
and_blank_line = false

[fun]
# Indentation of a lambda body that goes below `->`.
body_indent = 2
# Keep the line open for a lambda that is the last argument:
# `Iterator::map _ (fun x ->` with the body below.
hug_last_arg = true

[app]
# Arguments of an application that does not fit: "wrap" (as many per line as
# fit) | "fit_or_vertical" (one per line).
args = "wrap"
# Indentation of arguments that go to the next line.
indent = 2

[match]
# `|` before the first case: "when_broken" | "always" (even on one line).
leading_bar = "when_broken"
# "fit_or_vertical": a short match may stay on one line | "vertical": one case per line.
cases = "fit_or_vertical"
# Indentation of `| case` lines relative to `match` / `function`.
case_indent = 0
# A case's body: "fit" (after `->` when it fits) | "next_line" (always below).
body = "fit"
# Indentation of a case body that goes below its pattern.
body_indent = 4
# Align the `->` of all cases (when every pattern fits on one line).
align_arrows = false

[if]
# "fit_or_vertical": one line, or branches indented below `then` / `else`
# "vertical": like fit_or_vertical, never on one line
# "compact": one line, or `if c then a` / `else b`
# "keyword_first": one line, or `if c` / `then a` / `else b`
style = "fit_or_vertical"

[sequence]
# "terminator": `a;` ends the line | "separator": `; b` starts the next line.
style = "terminator"
# "always": one item per line | "fit": `a; b` stays on one line when it fits.
break = "always"

[collections]
# Lists, arrays and records that do not fit: "aligned" (`[ a;` / `  b ]`) |
# "block" (brackets on their own lines, items indented).
style = "aligned"
# Items of a broken collection: "fit_or_vertical" (one per line) | "wrap".
layout = "fit_or_vertical"
# `;` at the end of an item ("after") or the start of the next one ("before").
separator_position = "after"
# `;` after the last item: "never" | "vertical" (when broken) | "always".
trailing_semicolon = "never"
space_inside_lists = false
space_inside_arrays = false
space_inside_records = true

# Chains of infix operators. Each class has the same four options:
#   layout         "fit": one line, or one operator per line | "wrap": as many
#                  per line as fit
#   position       "leading": operators start lines | "trailing": they end lines
#   indent         continuation lines at the chain's indentation ("none"), one
#                  level deeper ("indent"), or "align"ed: leading operators line
#                  up under the first one (the first operand stays on the first
#                  line); trailing chains line up under the first operand
#   force_break_at chains with at least this many operators always break (0: never)

[pipe]  # |>
layout = "fit"
position = "leading"
indent = "none"
force_break_at = 0

[apply]  # @@
layout = "fit"
position = "trailing"
indent = "indent"
force_break_at = 0

[compose]  # >> <<
layout = "fit"
position = "leading"
indent = "indent"
force_break_at = 0

[boolean]  # && ||
layout = "fit"
position = "leading"
indent = "none"
force_break_at = 0

[cons]  # :: @ ^
layout = "fit"
position = "trailing"
indent = "indent"
force_break_at = 0

[arithmetic]  # + - * / ** mod land lor lxor lsl lsr asr
layout = "wrap"
position = "leading"
indent = "indent"
force_break_at = 0

[comparison]  # = <> < > <= >= == !=
layout = "fit"
position = "leading"
indent = "indent"
force_break_at = 0

[custom_operators]  # operators defined with `let ( op ) a b = ..`
layout = "fit"
position = "leading"
indent = "none"
force_break_at = 0
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_toml_is_the_default() {
        assert_eq!(Config::from_toml(DEFAULT_TOML).unwrap(), Config::default());
        assert_eq!(
            Config::from_toml(&Config::default().to_toml()).unwrap(),
            Config::default()
        );
    }

    #[test]
    fn partial_tables_keep_other_defaults() {
        let c = Config::from_toml("max_width = 80\n[apply]\nlayout = \"wrap\"").unwrap();
        assert_eq!(c.max_width, 80);
        assert_eq!(c.apply.layout, InfixLayout::Wrap);
        assert_eq!(c.apply.position, OperatorPosition::Trailing);
    }

    #[test]
    fn overrides() {
        let c = Config::default()
            .with_overrides("pipe.indent = align; max_width = 60; let.fun_on_eq_line = false")
            .unwrap();
        assert_eq!(c.pipe.indent, InfixIndent::Align);
        assert_eq!(c.max_width, 60);
        assert!(!c.let_.fun_on_eq_line);
    }

    #[test]
    fn mistakes_are_errors() {
        assert!(Config::from_toml("max_widht = 80").is_err());
        assert!(Config::from_toml("[pipe]\nindent = \"sideways\"").is_err());
        assert!(
            Config::default()
                .with_overrides("pipe.indnet = none")
                .is_err()
        );
    }
}

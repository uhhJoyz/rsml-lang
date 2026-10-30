//! Lowering of the OCaml syntax tree to plain Rust tokens.
//!
//! Nothing here generates runtime machinery: pipelines become nested calls,
//! known functions applied to all of their arguments become direct calls, and
//! partial applications / compositions become ordinary `move` closures. The
//! output is the Rust a person would write by hand for the same computation.
//!
//! The one thing a macro cannot see is the arity of Rust functions defined
//! elsewhere. Unknown functions are therefore called with every argument they
//! are given (`f a b` is `f(a, b)`); arities of ml-local functions,
//! closure-typed parameters of `rsml!` functions, `external` declarations and
//! sections (`f _ 1`) are tracked as a [`Shape`].

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use proc_macro2::{Delimiter, Group, Ident, Literal, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};

use crate::ast::*;
use crate::lexer::{Op, ident_str, is_capitalized, lex, op_tokens, word};
use crate::parser::{
    Error, MISSING, Mark, Parser, R, RESERVED, assign_op_error, bang_op_error, glued_section_error,
};
use crate::types::{Ctx, Type};

pub fn expand(input: TokenStream) -> TokenStream {
    expand_with(input, &[])
}

/// Like [`expand`], with Rust locals already in scope (the parameters of a
/// function in `rsml!`), each with its shape if it is a known closure.
pub fn expand_with(input: TokenStream, locals: &[(String, Option<Shape>)]) -> TokenStream {
    expand_fn(input, locals, &[])
}

/// Like [`expand_with`], for a function body of `rsml!` that also knows
/// the shapes of the functions it can call by name (`add`, `Self::new`).
pub fn expand_fn(
    input: TokenStream,
    locals: &[(String, Option<Shape>)],
    known: &[(String, Shape)],
) -> TokenStream {
    expand_for(input, locals, known, in_rust_analyzer())
}

/// Like [`expand_fn`], with the functions known by name in a [`Scope`]
/// shared by the bodies of one level of an `rsml!`.
pub(crate) fn expand_in(
    input: TokenStream,
    locals: &[(String, Option<Shape>)],
    known: Rc<Scope>,
) -> TokenStream {
    expand_scoped(input, locals, known, in_rust_analyzer())
}

/// The functions that the bodies of one level of an `rsml!` (the whole
/// block, a `mod`, an `impl` or `trait`) can call by name, with their
/// shapes. It is built once per level and shared by its bodies, and the
/// names of the levels around it are looked up through those, not copied:
/// a block of n functions costs O(n), not O(n²).
#[derive(Default)]
pub struct Scope {
    /// Defined at this level (`f`, `Type::f`, `Self::f`), later ones winning.
    own: HashMap<String, Bind>,
    /// The level around this one.
    parent: Option<Rc<Scope>>,
    kind: ScopeKind,
}

enum ScopeKind {
    /// A module, with its `use super::name as local`s (in order: `local`
    /// and `local::..` are the parent's `name` and `name::..`) and, for
    /// `use super::*`, the first path segments that other `use`s shadow.
    Module {
        aliases: Vec<(String, String)>,
        glob: Option<HashSet<String>>,
    },
    /// An `impl` or `trait` block, for the type of this name: it sees all
    /// that the level around it does, and `Type::f` as `Self::f`.
    Assoc(Option<String>),
}

impl Default for ScopeKind {
    fn default() -> ScopeKind {
        ScopeKind::Module {
            aliases: Vec::new(),
            glob: None,
        }
    }
}

impl Scope {
    fn defining(fns: Vec<(String, Shape)>) -> HashMap<String, Bind> {
        let mut own = HashMap::with_capacity(fns.len());
        for (name, shape) in fns {
            own.insert(name, Bind::Local(Some(shape)));
        }
        own
    }

    /// Just these functions, later ones winning.
    pub fn new(fns: Vec<(String, Shape)>) -> Scope {
        Scope {
            own: Scope::defining(fns),
            ..Scope::default()
        }
    }

    /// A module's functions (`own`, later ones winning), with `parent`'s
    /// names as its `use super::..`s bring them in.
    pub(crate) fn module(
        own: Vec<(String, Shape)>,
        parent: &Rc<Scope>,
        aliases: Vec<(String, String)>,
        glob: Option<HashSet<String>>,
    ) -> Scope {
        Scope {
            own: Scope::defining(own),
            parent: Some(parent.clone()),
            kind: ScopeKind::Module { aliases, glob },
        }
    }

    /// The associated functions of an `impl` or `trait` block, already
    /// named `Self::f`.
    pub(crate) fn assoc(
        own: Vec<(String, Shape)>,
        parent: &Rc<Scope>,
        ty: Option<String>,
    ) -> Scope {
        Scope {
            own: Scope::defining(own),
            parent: Some(parent.clone()),
            kind: ScopeKind::Assoc(ty),
        }
    }

    fn get(&self, name: &str) -> Option<&Bind> {
        if let Some(b) = self.own.get(name) {
            return Some(b);
        }
        let parent = self.parent.as_deref()?;
        match &self.kind {
            ScopeKind::Assoc(ty) => {
                if let (Some(ty), Some(rest)) = (ty, name.strip_prefix("Self::"))
                    && let Some(b) = parent.get(&format!("{}::{}", ty, rest))
                {
                    return Some(b);
                }
                parent.get(name)
            }
            ScopeKind::Module { aliases, glob } => {
                for (local, target) in aliases.iter().rev() {
                    let key = if name == local {
                        target.clone()
                    } else if let Some(rest) = name
                        .strip_prefix(local.as_str())
                        .and_then(|r| r.strip_prefix("::"))
                    {
                        format!("{}::{}", target, rest)
                    } else {
                        continue;
                    };
                    if let Some(b) = parent.get(&key) {
                        return Some(b);
                    }
                }
                let first = name.split("::").next().unwrap_or(name);
                match glob {
                    Some(shadowed) if !shadowed.contains(first) => parent.get(name),
                    _ => None,
                }
            }
        }
    }
}

thread_local! {
    static FORCE_EDITOR: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// Makes the macros expand on this thread as for rust-analyzer
/// (`Some(true)`) or for rustc (`Some(false)`), whatever program runs them;
/// `None` goes back to finding out. For tests such as the fuzzer.
#[doc(hidden)]
pub fn force_editor_mode(mode: Option<bool>) {
    FORCE_EDITOR.with(|f| f.set(mode));
}

/// Whether the macro is running inside rust-analyzer's proc-macro server
/// rather than rustc.
fn in_rust_analyzer() -> bool {
    if let Some(mode) = FORCE_EDITOR.with(|f| f.get()) {
        return mode;
    }
    static IN_RA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *IN_RA.get_or_init(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| Some(p.file_name()?.to_str()?.contains("rust-analyzer")))
            .unwrap_or(false)
    })
}

/// For an editor, unfinished code is recovered from rather than turned
/// into a lone `compile_error!`. rust-analyzer completes inside a macro
/// call by expanding it twice, as written and with a made-up identifier at
/// the cursor, and needs the two expansions to read the same up to the
/// cursor. So the recovered parse puts a stand-in where the identifier
/// would go, the expansion is always a block, and the errors come last.
pub(crate) fn expand_for(
    input: TokenStream,
    locals: &[(String, Option<Shape>)],
    known: &[(String, Shape)],
    editor: bool,
) -> TokenStream {
    expand_scoped(input, locals, Rc::new(Scope::new(known.to_vec())), editor)
}

fn expand_scoped(
    input: TokenStream,
    locals: &[(String, Option<Shape>)],
    known: Rc<Scope>,
    editor: bool,
) -> TokenStream {
    let locate = LOCATE.with(|l| l.replace(!editor));
    let out = expand_located(input, locals, known, editor);
    LOCATE.with(|l| l.set(locate));
    out
}

fn expand_located(
    input: TokenStream,
    locals: &[(String, Option<Shape>)],
    known: Rc<Scope>,
    editor: bool,
) -> TokenStream {
    let toks = lex(input);
    let anchor = toks.first().map_or_else(Span::call_site, |t| t.span());
    let mut parser = Parser::new(toks, Span::call_site());
    parser.bind_names(locals.iter().map(|(n, _)| n.clone()));
    if editor {
        parser.set_recover();
    }
    let ast = match parser.parse_toplevel() {
        Ok(e) => e,
        Err(e) => return e.to_compile_error(),
    };
    // Parameters shadow the functions around them (`lookup` tries `known`
    // last).
    let mut cx = Cx {
        editor,
        known,
        ..Cx::default()
    };
    for (name, shape) in locals {
        cx.bind(name.clone(), Bind::Local(shape.clone()));
    }
    match cx.expr(&ast, true) {
        Ok(out) => {
            let mut unmapped = parser.unmapped_operators();
            unmapped.extend(cx.unmapped);
            let marks = parser.marks();
            let ts = out.ts;
            let mut marker = TokenStream::new();
            if !unmapped.is_empty() {
                marker.extend(operator_marker(&unmapped, anchor));
            }
            if !marks.is_empty() {
                marker.extend(keyword_marker(&marks, anchor));
            }
            let marker = (!marker.is_empty() && MARKERS.with(|m| m.get())).then_some(marker);
            if editor {
                let errors = parser.take_recovered();
                let errors = errors.iter().map(|e| e.to_compile_error());
                return quote!({ #marker #ts #(; #errors)* });
            }
            let Some(marker) = marker else {
                return ts;
            };
            // A block takes the markers as its first items.
            let mut it = ts.clone().into_iter();
            if let (Some(TokenTree::Group(g)), None) = (it.next(), it.next())
                && g.delimiter() == Delimiter::Brace
            {
                let inner = g.stream();
                let mut b = Group::new(Delimiter::Brace, quote!(#marker #inner));
                b.set_span(g.span());
                return TokenStream::from(TokenTree::Group(b));
            }
            quote!({ #marker #ts })
        }
        Err(e) => e.to_compile_error(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// An identifier or path: free to re-evaluate and callable as `f(..)`.
    Name,
    /// Ends in a call or index: callable as `e(..)`.
    Call,
    /// Usable as an operand without parentheses, but not callable directly.
    Atomic,
    /// Needs parentheses when used as an operand, receiver or callee.
    Compound,
}

#[derive(Clone)]
struct Out {
    ts: TokenStream,
    shape: Option<Shape>,
    kind: Kind,
    /// The source parentheses the expression was written in, if any.
    paren: Option<Span>,
}

impl Out {
    fn new(ts: TokenStream, kind: Kind) -> Out {
        Out {
            ts,
            shape: None,
            kind,
            paren: None,
        }
    }

    fn shaped(mut self, shape: Option<Shape>) -> Out {
        self.shape = shape;
        self
    }

    fn operand(&self) -> TokenStream {
        if self.kind == Kind::Compound {
            self.parens()
        } else {
            self.ts.clone()
        }
    }

    fn callee(&self) -> TokenStream {
        match self.kind {
            Kind::Name | Kind::Call => self.ts.clone(),
            _ => self.parens(),
        }
    }

    /// `(e)`, located at the source parentheses, or else at the start of `e`.
    fn parens(&self) -> TokenStream {
        let sp = self.paren.or_else(|| first_span(&self.ts));
        group(
            Delimiter::Parenthesis,
            self.ts.clone(),
            sp.unwrap_or_else(Span::call_site),
        )
    }
}

#[derive(Clone)]
enum Bind {
    /// An ml-bound value; `Some` shape for known functions.
    Local(Option<Shape>),
    /// `let r = ref e`, compiled to a `let mut` local.
    RefCell,
    /// A user-defined or `external` operator, called as `path(a, b)`.
    Op(TokenStream, Shape),
    /// An `external` declaration.
    External(TokenStream, Option<Shape>),
}

#[derive(Default)]
struct Cx {
    env: Vec<(String, Bind)>,
    /// The functions of the same `rsml!` the body can call by name.
    known: Rc<Scope>,
    counter: usize,
    /// How many of each numbered local (`value1`, `function1`, ..) there are.
    numbered: Vec<(&'static str, usize)>,
    /// Operator characters lowered to something else (see [`operator_marker`]).
    unmapped: Vec<Span>,
    /// Expanding for rust-analyzer (see [`expand_for`]).
    editor: bool,
}

/// Rust spelling of an OCaml identifier: as written, but raw for a word
/// Rust reserves (`gen` is `r#gen`; see [`RESERVED`]). A raw identifier
/// (`r#fn`, the only way a Rust keyword gets through the parser as a name)
/// stays raw.
fn rust_ident(i: &Ident) -> Ident {
    let s = word(i);
    if RESERVED.binary_search(&s.as_str()).is_ok() {
        Ident::new_raw(&s, i.span())
    } else {
        i.clone()
    }
}

fn ident_ts(i: &Ident) -> TokenStream {
    let i = rust_ident(i);
    quote!(#i)
}

/// Literal tokens, translating OCaml's `1l` / `1L` / `1n` integer suffixes.
fn lit_tokens(l: &Literal) -> TokenStream {
    let s = l.to_string();
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        for (suffix, rust) in [("L", "i64"), ("l", "i32"), ("n", "isize")] {
            if let Some(num) = s.strip_suffix(suffix)
                && let Ok(mut lit) = format!("{}{}", num.replace('_', ""), rust).parse::<Literal>()
            {
                lit.set_span(l.span());
                return quote!(#lit);
            }
        }
    }
    quote!(#l)
}

fn path_tokens(p: &Path) -> TokenStream {
    let mut ts = TokenStream::new();
    if p.leading_colon {
        ts.extend(quote!(::));
    }
    for (i, seg) in p.segs.iter().enumerate() {
        if i > 0 {
            ts.extend(quote_spanned!(seg.ident.span()=> ::));
        }
        ts.extend(ident_ts(&seg.ident));
        if let Some(g) = &seg.generics {
            ts.extend(quote!(:: #g));
        }
    }
    ts
}

/// `[]`, `[a; b]` and `x :: xs` patterns are slice patterns in Rust; string
/// literal patterns need a `&str`.
#[derive(PartialEq, Clone, Copy)]
enum Class {
    Slice,
    Str,
    Other,
}

fn pat_class(p: &Pat) -> Class {
    match p {
        Pat::Paren(p) | Pat::Constraint(p, _) | Pat::Alias(p, _) => pat_class(p),
        Pat::Or(ps) => {
            let classes: Vec<Class> = ps.iter().map(pat_class).collect();
            if classes.contains(&Class::Slice) {
                Class::Slice
            } else if classes.contains(&Class::Str) {
                Class::Str
            } else {
                Class::Other
            }
        }
        Pat::List(..) | Pat::Array(..) | Pat::Cons(..) => Class::Slice,
        Pat::Lit { lit, .. } => {
            let s = lit.to_string();
            if s.starts_with('"') || s.starts_with("r\"") || s.starts_with("r#") {
                Class::Str
            } else {
                Class::Other
            }
        }
        _ => Class::Other,
    }
}

fn combine(classes: impl IntoIterator<Item = Class>) -> Class {
    let mut out = Class::Other;
    for c in classes {
        if c == Class::Slice {
            return Class::Slice;
        }
        if c == Class::Str {
            out = Class::Str;
        }
    }
    out
}

fn refutable(p: &Pat) -> bool {
    match p {
        Pat::Wild(_) | Pat::Var(_) | Pat::Mut(_) | Pat::Unit(_) => false,
        Pat::Lit { .. } | Pat::Bool(..) | Pat::Range(..) | Pat::Or(_) => true,
        Pat::List(..) | Pat::Array(..) | Pat::Cons(..) => true,
        Pat::Paren(p) | Pat::Constraint(p, _) | Pat::Alias(p, _) | Pat::Ref(p, _) => refutable(p),
        Pat::Tuple(ps) => ps.iter().any(refutable),
        Pat::Ctor(path, arg) => {
            matches!(
                ident_str(path.last()).as_str(),
                "Some" | "None" | "Ok" | "Err"
            ) || arg.as_ref().is_some_and(|a| refutable(a))
        }
        Pat::CtorRecord(_, fields, _) => fields
            .iter()
            .any(|(_, p)| p.as_ref().is_some_and(refutable)),
        Pat::Label { .. } => unreachable!("labels are only kept by a surface parse"),
    }
}

fn is_unit_pat(p: &Pat) -> bool {
    matches!(p.strip(), Pat::Unit(_))
}

/// Constructors whose single field commonly holds a tuple; `Some (a, b)` is
/// `Some((a, b))`, while `Pair (a, b)` is `Pair(a, b)`.
fn single_field_ctor(name: &str) -> bool {
    matches!(name, "Some" | "Ok" | "Err")
}

fn ctor_name(e: &Expr) -> Option<String> {
    match e.strip() {
        Expr::Path(p) if is_capitalized(p.last()) => Some(ident_str(p.last())),
        _ => None,
    }
}

fn is_special(name: &str) -> bool {
    matches!(name, "not" | "ref" | "incr" | "decr" | "raise")
}

/// A literal or a path (a constant, a unit variant, a function), which a
/// closure can use without capturing it.
fn is_constant(e: &Expr) -> bool {
    match e.strip() {
        Expr::Lit(_) | Expr::Bool(..) | Expr::Path(_) => true,
        Expr::Neg(e, _) => matches!(e.strip(), Expr::Lit(_)),
        _ => false,
    }
}

/// Whether `e` is written as a closure (`fun`, `function`, a section, a
/// method or field reference, an operator section, a composition), whose
/// parameters take their types from where it is used.
fn is_closure(e: &Expr) -> bool {
    matches!(
        e.strip(),
        Expr::Fun { .. }
            | Expr::Function { .. }
            | Expr::Section { .. }
            | Expr::MethodRef { .. }
            | Expr::FieldRef { .. }
            | Expr::OpSection(_)
            | Expr::Compose { .. }
    )
}

/// Where the arguments of an application split into calls when the
/// function's arity is unknown. Juxtaposed arguments are one call (`f a b`
/// is `f(a, b)`, and `(f a) b` too), and so is each Rust-style call:
/// `f(a)(b) c` is `f(a)(b)(c)`. With a known arity the split does not
/// matter: `add(1)(2)` is `add 1 2`.
#[derive(Clone, Default)]
struct Groups {
    /// Arguments that start a call of their own. At `0`: the arguments do
    /// not continue the head's own application (`(fun x -> g x)(1)(2)` is
    /// `g(1)(2)`, while `(fun x -> g x) 1 2` is `g(1, 2)`).
    starts: Vec<usize>,
    /// Ends of calls written with a trailing comma (kept for an editor).
    commas: Vec<usize>,
}

impl Groups {
    /// The groups of `args[n..]`.
    fn after(&self, n: usize) -> Groups {
        Groups {
            starts: self
                .starts
                .iter()
                .filter(|s| **s >= n)
                .map(|s| s - n)
                .collect(),
            commas: self
                .commas
                .iter()
                .filter(|c| **c > n)
                .map(|c| c - n)
                .collect(),
        }
    }

    /// The end of the first call made by `len` arguments.
    fn first_end(&self, len: usize) -> usize {
        self.starts
            .iter()
            .copied()
            .filter(|s| *s > 0 && *s < len)
            .min()
            .unwrap_or(len)
    }

    /// The number of arguments of each call made by `len` arguments.
    fn lens(&self, len: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut at = 0;
        while at < len {
            let end = at + self.after(at).first_end(len - at);
            out.push(end - at);
            at = end;
        }
        out
    }
}

/// Flattens `((f a) b) c` into `(f, [a, b, c])`, and Rust-style calls
/// likewise, keeping where each call starts: `f(a)(b) c` is `(f, [a, b,
/// c])` in three groups.
fn collect_app(e: &Expr) -> (&Expr, Vec<&Expr>, Groups) {
    match e.strip() {
        Expr::App(h, args) => {
            let (head, mut all, mut groups) = collect_app(h);
            if matches!(h.strip(), Expr::Call { .. }) {
                groups.starts.push(all.len());
            }
            all.extend(args.iter());
            (head, all, groups)
        }
        Expr::Call {
            func,
            args,
            trailing_comma,
            ..
        } => {
            let (head, mut all, mut groups) = collect_app(func);
            if !all.is_empty() {
                groups.starts.push(all.len());
            }
            all.extend(args.iter());
            if *trailing_comma {
                groups.commas.push(all.len());
            }
            (head, all, groups)
        }
        other => (other, Vec::new(), Groups::default()),
    }
}

/// The shape a function type annotation gives a call of unknown arity with
/// arguments in calls of `lens` arguments: the last call is partial.
fn hinted(mut lens: Vec<usize>, h: Shape) -> Shape {
    let last = lens.pop().unwrap();
    if h[0] == 0 {
        lens.push(last);
        lens.extend(h);
    } else {
        lens.push(last + h[0]);
        lens.extend(&h[1..]);
    }
    lens
}

/// `x <- x + e` as Rust's `x = x + e`, which clippy would have written
/// `x += e`. OCaml has no `+=`, so the lint is allowed on the statement
/// (attributes on expressions are not stable; on statements they are).
fn assignment(place: TokenStream, value: TokenStream, allow: bool, sp: Span) -> Out {
    let eq = quote_spanned!(sp=> =);
    if allow {
        return Out::new(
            group(
                Delimiter::Brace,
                quote!(#[allow(clippy::assign_op_pattern)] { #place #eq #value; }),
                sp,
            ),
            Kind::Compound,
        );
    }
    Out::new(quote!(#place #eq #value), Kind::Compound)
}

/// Whether two place expressions name the same place: `x`, `p.x`,
/// `t.0`, `a.(i)` with a name or literal index, `!r`.
fn same_place(a: &Expr, b: &Expr) -> bool {
    match (a.strip(), b.strip()) {
        (Expr::Var(x), Expr::Var(y)) => ident_str(x) == ident_str(y),
        (Expr::Field(e, x), Expr::Field(f, y)) => ident_str(x) == ident_str(y) && same_place(e, f),
        (Expr::TupleIndex(e, x), Expr::TupleIndex(f, y)) => {
            x.iter().map(|i| i.0).eq(y.iter().map(|i| i.0)) && same_place(e, f)
        }
        (Expr::Index(e, i, _), Expr::Index(f, j, _)) => {
            let same_index = match (i.strip(), j.strip()) {
                (Expr::Lit(l), Expr::Lit(m)) => l.to_string() == m.to_string(),
                (i, j) => same_place(i, j),
            };
            same_index && same_place(e, f)
        }
        (Expr::Deref(e, _), Expr::Deref(f, _)) => same_place(e, f),
        _ => false,
    }
}

/// `move |params| body`, located at `sp`, after the `hoists`.
fn closure(hoists: Vec<TokenStream>, params: &[Ident], body: TokenStream, sp: Span) -> TokenStream {
    let ps: Vec<TokenStream> = params.iter().map(|p| quote!(#p)).collect();
    let bars = closure_bars(&ps, sp);
    let mv = keyword("move", sp);
    let c = quote!(#mv #bars #body);
    if hoists.is_empty() {
        c
    } else {
        block(quote!(#(#hoists)* #c))
    }
}

/// `const _: usize = 1 + 1 + ..;` with one `+` per operator character
/// that the expansion lowers to something else (`|>` to a call).
///
/// Editors such as rust-analyzer color a token inside a macro call by the
/// token with its span in the expansion: `a + b` keeps its `+`, so `+` is
/// colored as an operator, but `|>` would have nothing and stay plain. In
/// this constant each of those characters is a `+`, so they are colored
/// like `+`. It is evaluated at compile time and generates no code.
///
/// The rest of the constant is located at `anchor`, the first token of the
/// code (see [`at`]). With the macro call's own span, the constant would
/// seem to contain all of the code, and lints that skip constants (such
/// as `clippy::arithmetic_side_effects`) would skip it too.
fn operator_marker(spans: &[Span], anchor: Span) -> TokenStream {
    let mut t = MarkerToks::new(anchor);
    t.ident("const")
        .ident("_")
        .punct(':')
        .ident("usize")
        .punct('=')
        .one();
    for sp in spans {
        let mut plus = proc_macro2::Punct::new('+', proc_macro2::Spacing::Alone);
        plus.set_span(*sp);
        t.out.push(TokenTree::Punct(plus));
        t.one();
    }
    t.punct(';');
    t.out.into_iter().collect()
}

/// The tokens of a marker, all at `anchor` as [`located`] would put them.
/// They are made directly, as the markers are made for every body: each
/// `quote!` is a round trip to the compiler (or rust-analyzer), and so is
/// each literal in one, which it parses from text.
struct MarkerToks {
    sp: Span,
    out: Vec<TokenTree>,
}

impl MarkerToks {
    fn new(anchor: Span) -> MarkerToks {
        MarkerToks {
            sp: at(anchor),
            out: Vec::new(),
        }
    }

    fn ident(&mut self, s: &str) -> &mut MarkerToks {
        self.out.push(TokenTree::Ident(Ident::new(s, self.sp)));
        self
    }

    fn punct(&mut self, c: char) -> &mut MarkerToks {
        let mut p = proc_macro2::Punct::new(c, proc_macro2::Spacing::Alone);
        p.set_span(self.sp);
        self.out.push(TokenTree::Punct(p));
        self
    }

    fn lit(&mut self, n: usize) -> &mut MarkerToks {
        let mut l = Literal::usize_unsuffixed(n);
        l.set_span(self.sp);
        self.out.push(TokenTree::Literal(l));
        self
    }

    fn punct_joint(&mut self, c: char) -> &mut MarkerToks {
        let mut p = proc_macro2::Punct::new(c, proc_macro2::Spacing::Joint);
        p.set_span(self.sp);
        self.out.push(TokenTree::Punct(p));
        self
    }

    fn one(&mut self) -> &mut MarkerToks {
        self.lit(1)
    }

    fn group(&mut self, delim: Delimiter, inner: Vec<TokenTree>) -> &mut MarkerToks {
        let mut g = Group::new(delim, inner.into_iter().collect());
        g.set_span(self.sp);
        self.out.push(TokenTree::Group(g));
        self
    }

    /// The tokens added by `f`, taken back out (for a group's contents).
    fn inner(&mut self, f: impl FnOnce(&mut MarkerToks)) -> Vec<TokenTree> {
        let outer = std::mem::take(&mut self.out);
        f(self);
        std::mem::replace(&mut self.out, outer)
    }
}

/// `const _: () = { .. };`, with a statement for each OCaml keyword or
/// symbol that has no Rust counterpart in the expansion ([`Parser::marks`]).
///
/// Like the [`operator_marker`], this is for editors: in `if true {}` the
/// `if` is located at a `then`, so `then` is colored like `if`. `with` gets
/// a `match`, `done` a `break`, `fun` a `let`, the `|` of a match arm a
/// `|` operator and `->` a `=>`. It is evaluated at compile time and
/// generates no code. Its other tokens are located at `anchor`, as in the
/// operator marker.
fn keyword_marker(marks: &[Mark], anchor: Span) -> TokenStream {
    let mut t = MarkerToks::new(anchor);
    let stmts = t.inner(|t| {
        for m in marks {
            let sp = mapped(m.spans[0]);
            match m.like {
                "|" => {
                    let mut bar = proc_macro2::Punct::new('|', proc_macro2::Spacing::Alone);
                    bar.set_span(sp);
                    t.ident("let").ident("_").punct('=').lit(0);
                    t.out.push(TokenTree::Punct(bar));
                    t.lit(0).punct(';');
                }
                "=>" => {
                    let arm = t.inner(|t| {
                        t.ident("_");
                        t.out.extend(fat_arrow(&m.spans));
                        t.group(Delimiter::Brace, Vec::new());
                    });
                    t.ident("match").group(Delimiter::Parenthesis, Vec::new());
                    t.group(Delimiter::Brace, arm);
                }
                kw => {
                    let k = TokenTree::Ident(Ident::new(kw, sp));
                    match kw {
                        "if" => {
                            t.out.push(k);
                            t.ident("true").group(Delimiter::Brace, Vec::new());
                        }
                        "match" => {
                            t.out.push(k);
                            let arm = t.inner(|t| {
                                t.ident("_").punct_joint('=').punct('>');
                                t.group(Delimiter::Brace, Vec::new());
                            });
                            t.group(Delimiter::Parenthesis, Vec::new());
                            t.group(Delimiter::Brace, arm);
                        }
                        "break" => {
                            t.ident("loop");
                            t.group(Delimiter::Brace, vec![k]);
                        }
                        "loop" => {
                            t.out.push(k);
                            let body = t.inner(|t| {
                                t.ident("break");
                            });
                            t.group(Delimiter::Brace, body);
                        }
                        _ => {
                            t.out.push(k);
                            t.ident("_").punct('=').lit(0).punct(';');
                        }
                    }
                }
            }
        }
    });
    t.ident("const")
        .ident("_")
        .punct(':')
        .group(Delimiter::Parenthesis, Vec::new())
        .punct('=');
    t.group(Delimiter::Brace, stmts);
    t.punct(';');
    t.out.into_iter().collect()
}

/// `recv.0.1`
fn tuple_fields(mut ts: TokenStream, idx: &[(u32, Span)]) -> TokenStream {
    for (n, sp) in idx {
        let mut l = Literal::u32_unsuffixed(*n);
        l.set_span(*sp);
        ts = quote!(#ts.#l);
    }
    ts
}

/// `recv.name::<T>(args)`; `.await` takes no parentheses.
fn method_call(
    recv: TokenStream,
    name: &Ident,
    turbofish: &Option<TokenStream>,
    args: &[TokenStream],
    trailing_comma: bool,
) -> TokenStream {
    if name == "await" && args.is_empty() {
        return quote!(#recv.#name);
    }
    let n = rust_ident(name);
    let tf = turbofish.as_ref().map(|g| quote!(:: #g));
    let comma = trailing_comma.then(|| quote!(,));
    let g = group(
        Delimiter::Parenthesis,
        quote!(#(#args),* #comma),
        name.span(),
    );
    quote!(#recv.#n #tf #g)
}

/// `{ let a = ..; out }`, or `out` itself when nothing was hoisted.
fn with_hoists(hoists: Vec<TokenStream>, out: Out) -> Out {
    if hoists.is_empty() {
        return out;
    }
    let ts = out.ts;
    Out::new(block(quote!(#(#hoists)* #ts)), Kind::Compound).shaped(out.shape)
}

/// `|a, b|`, or `||` when there are no parameters, located at `sp`.
fn closure_bars(params: &[TokenStream], sp: Span) -> TokenStream {
    let bar = |spacing| {
        let mut p = proc_macro2::Punct::new('|', spacing);
        p.set_span(at(sp));
        TokenTree::Punct(p)
    };
    if params.is_empty() {
        let mut ts = TokenStream::new();
        ts.extend([
            bar(proc_macro2::Spacing::Joint),
            bar(proc_macro2::Spacing::Alone),
        ]);
        return ts;
    }
    let open = bar(proc_macro2::Spacing::Alone);
    let close = bar(proc_macro2::Spacing::Alone);
    quote!(#open #(#params),* #close)
}

/// A keyword (or other generated identifier) located at `sp`.
fn keyword(kw: &str, sp: Span) -> Ident {
    Ident::new(kw, at(sp))
}

/// Wraps tokens in braces unless they already are a block.
fn braced(ts: TokenStream) -> TokenStream {
    let mut iter = ts.clone().into_iter();
    if let (Some(TokenTree::Group(g)), None) = (iter.next(), iter.next())
        && g.delimiter() == Delimiter::Brace
    {
        return ts;
    }
    block(ts)
}

/// `{ ts }`, located at the end of `ts` (its value).
fn block(ts: TokenStream) -> TokenStream {
    let sp = last_span(&ts).unwrap_or_else(Span::call_site);
    group(Delimiter::Brace, ts, sp)
}

impl Cx {
    /// A generated local variable, named `name` for rustc and
    /// `__rsml_{prefix}{n}` for an editor.
    ///
    /// rustc prints a local's name in its errors ("`receiver` is a `&`
    /// reference, so it cannot be borrowed as mutable"), so there it is
    /// named for what it stands for: `_0` for a placeholder, `receiver`,
    /// `arg2`, `value1`, .. Its span is `Span::mixed_site()`, so macro
    /// hygiene keeps it apart from the user's names, as in `macro_rules!`.
    /// rust-analyzer offers the locals in scope as completions, generated
    /// ones included, so for it the name keeps the prefix that no user name
    /// has.
    fn local(&mut self, prefix: &str, name: &str) -> Ident {
        self.counter += 1;
        let text = if self.editor {
            format!("__rsml_{}{}", prefix, self.counter)
        } else {
            name.to_string()
        };
        Ident::new(&text, Span::mixed_site())
    }

    /// Like [`Cx::local`], located at `sp` for error messages (see [`at`]).
    fn local_at(&mut self, prefix: &str, name: &str, sp: Span) -> Ident {
        let mut i = self.local(prefix, name);
        if LOCATE.with(|l| l.get()) {
            i.set_span(Span::mixed_site().located_at(sp));
        }
        i
    }

    /// A local that can be in scope together with others of its kind,
    /// numbered: `value1`, `value2`, .. (see [`Cx::local`]).
    fn numbered_at(&mut self, prefix: &str, stem: &'static str, sp: Span) -> Ident {
        let n = match self.numbered.iter_mut().find(|(s, _)| *s == stem) {
            Some((_, n)) => {
                *n += 1;
                *n
            }
            None => {
                self.numbered.push((stem, 1));
                1
            }
        };
        self.local_at(prefix, &format!("{}{}", stem, n), sp)
    }

    fn lookup(&self, name: &str) -> Option<&Bind> {
        self.env
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, b)| b)
            .or_else(|| self.known.get(name))
    }

    /// The shape of an associated function of the same `rsml!`, known as
    /// `Self::name` or `Type::name` (see [`expand_fn`]).
    fn path_shape(&self, p: &Path) -> Option<Shape> {
        if p.leading_colon || p.segs.len() < 2 {
            return None;
        }
        let key: Vec<String> = p.segs.iter().map(|s| ident_str(&s.ident)).collect();
        match self.lookup(&key.join("::")) {
            Some(Bind::Local(shape)) => shape.clone(),
            _ => None,
        }
    }

    fn is_refcell(&self, e: &Expr) -> Option<Ident> {
        match e.strip() {
            Expr::Var(i) if matches!(self.lookup(&ident_str(i)), Some(Bind::RefCell)) => {
                Some(rust_ident(i))
            }
            _ => None,
        }
    }

    fn bind(&mut self, name: String, b: Bind) {
        self.env.push((name, b));
    }

    fn bind_pat(&mut self, p: &Pat) {
        let mut names = Vec::new();
        p.names(&mut names);
        for n in names {
            self.bind(ident_str(&n), Bind::Local(None));
        }
    }

    /// Whether evaluating `e` has no effects and costs nothing, so it can be
    /// moved into a closure body instead of being evaluated up front.
    fn is_trivial(&self, e: &Expr) -> bool {
        match e {
            Expr::Var(_) => self.is_refcell(e).is_none(),
            Expr::Tmp(..)
            | Expr::Path(_)
            | Expr::Lit(_)
            | Expr::Bool(..)
            | Expr::Unit(_)
            | Expr::OpSection(_)
            | Expr::Fun { .. }
            | Expr::Function { .. }
            | Expr::MethodRef { .. }
            | Expr::FieldRef { .. } => true,
            Expr::Paren(e, _) => self.is_trivial(e),
            Expr::Neg(e, _) => matches!(e.strip(), Expr::Lit(_)),
            Expr::Compose { first, second, .. } => {
                self.is_trivial(first) && self.is_trivial(second)
            }
            // Creating the closure has no effects if nothing is hoisted.
            Expr::Section { body, .. } => self
                .operands(body)
                .iter()
                .all(|o| o.has_hole() || self.is_trivial(o)),
            _ => false,
        }
    }

    /// Whether a function's shape is known without translating it: a local
    /// or `external` function, a closure, a section, `not`, ...
    fn known_fn(&self, e: &Expr) -> bool {
        match e.strip() {
            Expr::Var(i) => match self.lookup(&ident_str(i)) {
                Some(Bind::Local(s) | Bind::External(_, s)) => s.is_some(),
                Some(Bind::RefCell | Bind::Op(..)) => false,
                None => is_special(&word(i)),
            },
            Expr::Tmp(_, s) => s.is_some(),
            Expr::Path(p) => self.path_shape(p).is_some(),
            Expr::Fun { .. }
            | Expr::Function { .. }
            | Expr::Compose { .. }
            | Expr::OpSection(_)
            | Expr::Section { .. }
            | Expr::MethodRef { .. }
            | Expr::FieldRef { .. } => true,
            _ => false,
        }
    }

    /// Whether `rhs` is `a op b` with an operator that Rust has an
    /// assignment form of (`+=`), and `a` (or `b` of a commutative one) is
    /// the place assigned: clippy's `assign_op_pattern`.
    fn assign_op(&self, rhs: &Expr, is_place: impl Fn(&Expr) -> bool) -> bool {
        let Expr::Binary { op, lhs, rhs } = rhs.strip() else {
            return false;
        };
        if self.lookup(&format!("op:{}", op.text)).is_some() {
            return false;
        }
        match rust_binop(&op.text) {
            Some("+" | "*" | "&" | "|" | "^") => is_place(lhs) || is_place(rhs),
            Some("-" | "/" | "%" | "<<" | ">>") => is_place(lhs),
            _ => false,
        }
    }

    /// Whether `e` plainly is a `bool`, which `!` cannot dereference: a
    /// comparison, `&&`, `||`, `not e` or a boolean literal.
    fn is_bool(&self, e: &Expr) -> bool {
        match e.strip() {
            Expr::Bool(..) => true,
            Expr::Binary { op, .. } => {
                self.lookup(&format!("op:{}", op.text)).is_none()
                    && matches!(
                        rust_binop(&op.text),
                        Some("==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||")
                    )
            }
            Expr::App(h, _) => {
                matches!(h.strip(), Expr::Var(i) if i == "not" && self.lookup("not").is_none())
            }
            _ => false,
        }
    }

    /// `!x.is_empty()` in an assertion: Rust's `!` negates the call, but
    /// OCaml's dereferences `x` and binds tighter than `.`, so it would
    /// assert the opposite (`(!x).is_empty ()`). `(!x).is_empty()` says
    /// which is meant.
    fn deref_link(&self, e: &Expr) -> Option<Error> {
        let mut e = e;
        let mut links = 0;
        while let Expr::Method { recv: r, .. }
        | Expr::Field(r, _)
        | Expr::TupleIndex(r, _)
        | Expr::Index(r, _, _)
        | Expr::Call { func: r, .. }
        | Expr::Try(r, _)
        | Expr::Await(r, _) = e
        {
            e = r;
            links += 1;
        }
        match e {
            Expr::Deref(inner, sp) if links > 0 && self.is_refcell(inner).is_none() => {
                Some(Error::new(
                    *sp,
                    "`!` dereferences in OCaml and binds tighter than `.`: `!x.f()` is \
                     `(!x).f()`. To negate, write `not (x.f())`; to dereference, `(!x).f()`",
                ))
            }
            _ => None,
        }
    }

    /// Evaluates `e` into a hoisted temporary unless it is trivial.
    fn hoist(&mut self, e: &Expr, hoists: &mut Vec<TokenStream>) -> R<Expr> {
        self.hoist_value(e, false, hoists)
    }

    /// Like [`Cx::hoist`], for the receiver of a method call or a function
    /// that is called: the temporary is `mut`, as the method may take
    /// `&mut self` and the function may be a closure that mutates what it
    /// captures (`FnMut`). The user could not add the `mut`.
    fn hoist_mut(&mut self, e: &Expr, hoists: &mut Vec<TokenStream>) -> R<Expr> {
        self.hoist_value(e, true, hoists)
    }

    fn hoist_value(&mut self, e: &Expr, mutable: bool, hoists: &mut Vec<TokenStream>) -> R<Expr> {
        if self.is_trivial(e) {
            return Ok(e.clone());
        }
        self.temporary(e, mutable, hoists)
    }

    /// Like [`Cx::hoist_value`], but for rust-analyzer every operand except
    /// a closure or a constant goes into a temporary, trivial or not, so that
    /// the operands come before the closure that uses them (see
    /// [`Cx::partial`]).
    fn bind_value(&mut self, e: &Expr, mutable: bool, hoists: &mut Vec<TokenStream>) -> R<Expr> {
        if self.editor && !is_closure(e) && !is_constant(e) {
            self.temporary(e, mutable, hoists)
        } else {
            self.hoist_value(e, mutable, hoists)
        }
    }

    fn temporary(&mut self, e: &Expr, mutable: bool, hoists: &mut Vec<TokenStream>) -> R<Expr> {
        let out = self.expr(e, false)?;
        let ts = out.ts;
        // Located at the expression, which errors about it then show.
        let sp = first_span(&ts).unwrap_or_else(|| span_of(e));
        let t = self.numbered_at("a", "value", sp);
        let m = mutable.then(|| quote!(mut));
        hoists.push(quote!(let #m #t = #ts;));
        Ok(Expr::Tmp(t, out.shape))
    }

    // ----- expressions ---------------------------------------------------

    /// Translates `e`. `tail` is true when the value escapes the `ml!` block
    /// or the enclosing closure (it is the result, or part of the result).
    fn expr(&mut self, e: &Expr, tail: bool) -> R<Out> {
        match e {
            Expr::Var(_) | Expr::Path(_) => {
                let out = self.callee(e)?;
                Ok(self.eta(out))
            }
            Expr::Tmp(i, shape) => Ok(Out::new(quote!(#i), Kind::Name).shaped(shape.clone())),
            Expr::Lit(l) => Ok(Out::new(lit_tokens(l), Kind::Atomic)),
            Expr::Bool(b, sp) => {
                let ts = if *b {
                    quote_spanned!(*sp=> true)
                } else {
                    quote_spanned!(*sp=> false)
                };
                Ok(Out::new(ts, Kind::Atomic))
            }
            Expr::Unit(sp) => Ok(Out::new(quote_spanned!(*sp=> ()), Kind::Atomic)),
            Expr::Paren(inner, sp) => {
                if let Expr::Tuple(items) = &**inner {
                    let ts = self.exprs(items.iter(), tail)?;
                    let g = group(Delimiter::Parenthesis, quote!(#(#ts),*), *sp);
                    return Ok(Out::new(g, Kind::Atomic));
                }
                let mut o = self.expr(inner, tail)?;
                o.paren.get_or_insert(*sp);
                Ok(o)
            }
            Expr::Tuple(items) => {
                let ts = self.exprs(items.iter(), tail)?;
                let sp = ts
                    .first()
                    .and_then(first_span)
                    .unwrap_or_else(Span::call_site);
                Ok(Out::new(
                    group(Delimiter::Parenthesis, quote!(#(#ts),*), sp),
                    Kind::Atomic,
                ))
            }
            // The list's own span is kept off `vec!`, whose `::` and `!`
            // would otherwise give the brackets an operator's color. For an
            // editor a list is `Vec::from([..])`: rust-analyzer follows a
            // completion into `vec!` only when `[]` and `[x]` take the same
            // macro arm, which they do not (see `expand_for`).
            Expr::List(items, sp) => {
                let ts = self.exprs(items.iter(), tail)?;
                let ts = if self.editor {
                    quote!(::std::vec::Vec::from([#(#ts),*]))
                } else if ts.is_empty() {
                    located(quote!(::std::vec::Vec::new()), *sp)
                } else {
                    // Errors point at the list (rust-analyzer, which would
                    // color the brackets like `::` and `!`, never sees this).
                    let mac = located(quote!(::std::vec!), *sp);
                    let items = group(Delimiter::Bracket, quote!(#(#ts),*), *sp);
                    quote!(#mac #items)
                };
                Ok(Out::new(ts, Kind::Call))
            }
            Expr::Array(items, sp) => {
                let ts = self.exprs(items.iter(), tail)?;
                Ok(Out::new(
                    group(Delimiter::Bracket, quote!(#(#ts),*), *sp),
                    Kind::Atomic,
                ))
            }
            Expr::App(..) | Expr::Call { .. } => self.app(e, None, tail),
            Expr::Fun {
                params,
                body,
                is_move,
                span,
            } => self.fun_value(params, body, *is_move || (tail && !body.mutates()), *span),
            Expr::Function { arms, span } => {
                let x = self.local_at("arg", "argument", *span);
                let m = self.match_tokens(&Expr::Tmp(x.clone(), None), arms, true, *span)?;
                let mv = (tail && !e.mutates()).then(|| keyword("move", *span));
                let bars = closure_bars(&[quote!(#x)], *span);
                Ok(Out::new(quote!(#mv #bars #m), Kind::Compound).shaped(Some(vec![1])))
            }
            Expr::Let { .. } | Expr::Seq(..) | Expr::External { .. } | Expr::Open { .. } => {
                self.block_expr(e, tail)
            }
            Expr::If {
                cond,
                then,
                els,
                span,
            } => {
                let c = self.expr(cond, false)?.ts;
                let len = self.env.len();
                let t = self.expr(then, tail)?;
                self.env.truncate(len);
                let tb = braced(t.ts.clone());
                match els {
                    Some(els) => {
                        let e = self.expr(els, tail)?;
                        let eb = if matches!(els.strip(), Expr::If { .. }) {
                            e.ts.clone()
                        } else {
                            braced(e.ts.clone())
                        };
                        let shape = if t.shape == e.shape { t.shape } else { None };
                        let kw = mapped_keyword("if", *span);
                        Ok(Out::new(quote!(#kw #c #tb else #eb), Kind::Compound).shaped(shape))
                    }
                    None => {
                        let kw = mapped_keyword("if", *span);
                        Ok(Out::new(quote!(#kw #c #tb), Kind::Compound))
                    }
                }
            }
            Expr::Match { scrut, arms, span } => {
                let ts = self.match_tokens(scrut, arms, tail, *span)?;
                Ok(Out::new(ts, Kind::Compound))
            }
            Expr::Binary { op, lhs, rhs } => self.binary(op, lhs, rhs),
            Expr::Compose { .. } => self.compose_value(e),
            Expr::Neg(inner, op) => {
                let o = self.expr(inner, false)?.operand();
                Ok(Out::new(quote_spanned!(op.span()=> -#o), Kind::Compound))
            }
            Expr::Deref(inner, sp) => {
                if let Some(r) = self.is_refcell(inner) {
                    self.unmapped.push(*sp);
                    return Ok(Out::new(quote!(#r), Kind::Name));
                }
                if !self.editor && self.is_bool(inner) {
                    return Err(Error::new(
                        *sp,
                        "a `bool` cannot be dereferenced: in OCaml `!r` is the value of `r`, and \
                         negation is `not e`",
                    ));
                }
                let o = self.expr(inner, false)?.operand();
                Ok(Out::new(quote_spanned!(*sp=> *#o), Kind::Compound))
            }
            Expr::Borrow {
                mutable,
                expr,
                span,
            } => {
                let m = mutable.then(|| keyword("mut", *span));
                let o = match self.is_refcell(expr) {
                    Some(r) => quote!(#r),
                    None => self.expr(expr, false)?.operand(),
                };
                Ok(Out::new(quote_spanned!(*span=> & #m #o), Kind::Compound))
            }
            Expr::Field(inner, name) => {
                let b = self.expr(inner, false)?.operand();
                let n = rust_ident(name);
                Ok(Out::new(quote!(#b.#n), Kind::Atomic))
            }
            Expr::TupleIndex(inner, idx) => {
                let ts = self.expr(inner, false)?.operand();
                Ok(Out::new(tuple_fields(ts, idx), Kind::Atomic))
            }
            Expr::Method {
                recv,
                name,
                turbofish,
                args,
                trailing_comma,
            } => {
                // Adapters like `.map(f)` keep their receiver, so it escapes too.
                let r = self.expr(recv, tail)?.operand();
                let a = self.exprs(args.iter(), false)?;
                // A trailing comma is kept for an editor only, as in a call.
                let comma = *trailing_comma && self.editor;
                Ok(Out::new(
                    method_call(r, name, turbofish, &a, comma),
                    Kind::Call,
                ))
            }
            Expr::Index(a, i, _) => {
                let a = self.expr(a, false)?.operand();
                let i = self.expr(i, false)?.ts;
                let sp = last_span(&i).unwrap_or_else(Span::call_site);
                let g = group(Delimiter::Bracket, i, sp);
                Ok(Out::new(quote!(#a #g), Kind::Call))
            }
            Expr::Range { lo, hi, op } => {
                let lo = match lo {
                    Some(e) => Some(self.expr(e, false)?.operand()),
                    None => None,
                };
                let hi = match hi {
                    Some(e) => Some(self.expr(e, false)?.operand()),
                    None => None,
                };
                let o = op_tokens(&op.text, &op.spans);
                Ok(Out::new(quote!(#lo #o #hi), Kind::Compound))
            }
            Expr::Jump { kind, value, span } => {
                let kw = Ident::new(kind.keyword(), *span);
                // A returned value leaves the function, like a block's result.
                let v = match value {
                    Some(v) => Some(self.expr(v, *kind == Jump::Return)?.ts),
                    None => None,
                };
                Ok(Out::new(quote!(#kw #v), Kind::Compound))
            }
            Expr::RustBlock {
                kind,
                body,
                span,
                brace,
            } => {
                let kw = match kind {
                    RustBlock::Unsafe => quote_spanned!(*span=> unsafe),
                    RustBlock::Async => quote_spanned!(*span=> async),
                    RustBlock::AsyncMove => quote_spanned!(*span=> async move),
                };
                // An async block's value is the future's output: it escapes.
                let tail = tail || *kind != RustBlock::Unsafe;
                let b = match body {
                    Some(b) => braced(self.expr(b, tail)?.ts),
                    None => quote_spanned!(*brace=> {}),
                };
                Ok(Out::new(quote!(#kw #b), Kind::Compound))
            }
            Expr::StrIndex(a, i) => {
                let a = self.expr(a, false)?.operand();
                let i = self.expr(i, false)?.ts;
                let sp = last_span(&i).unwrap_or_else(Span::call_site);
                let g = group(
                    Delimiter::Parenthesis,
                    quote!(#a.as_bytes()[#i] as char),
                    sp,
                );
                Ok(Out::new(g, Kind::Atomic))
            }
            Expr::Assign(lhs, rhs, sp) => {
                let place = self.place(lhs)?;
                let v = self.expr(rhs, false)?.ts;
                let allow = self.assign_op(rhs, |operand| same_place(lhs, operand));
                Ok(assignment(place, v, allow, *sp))
            }
            Expr::RefAssign(lhs, rhs, sp) => {
                let v = self.expr(rhs, false)?.ts;
                let allow = self.assign_op(
                    rhs,
                    |operand| matches!(operand.strip(), Expr::Deref(r, _) if same_place(lhs, r)),
                );
                let place = match self.is_refcell(lhs) {
                    Some(r) => quote!(#r),
                    None => {
                        let l = self.expr(lhs, false)?.operand();
                        quote_spanned!(*sp=> *#l)
                    }
                };
                Ok(assignment(place, v, allow, *sp))
            }
            Expr::Try(inner, sp) => {
                let o = self.expr(inner, false)?.operand();
                Ok(Out::new(quote_spanned!(*sp=> #o?), Kind::Atomic))
            }
            Expr::Await(inner, sp) => {
                let o = self.expr(inner, false)?.operand();
                Ok(Out::new(quote_spanned!(*sp=> #o.await), Kind::Atomic))
            }
            Expr::OpSection(op) => self.op_section_value(op),
            Expr::Constraint(inner, ty) => self.constraint(inner, ty, tail),
            Expr::Cast(inner, ty) => {
                let o = self.expr(inner, false)?.operand();
                let t = ty.emit(Ctx::Local, &mut Vec::new());
                Ok(Out::new(quote!(#o as #t), Kind::Compound))
            }
            Expr::Record {
                ctor,
                fields,
                trailing_semi,
            } => {
                let p = path_tokens(ctor);
                let mut fs = Vec::new();
                for (f, v) in fields {
                    let f = rust_ident(f);
                    fs.push(match v {
                        Some(v) => {
                            let v = self.expr(v, tail)?.ts;
                            quote!(#f: #v)
                        }
                        None => quote!(#f),
                    });
                }
                // For an editor, a `;` after the last field stays as `,`, so
                // that a field being added reads the same up to it (see
                // `expand_for`). rustc gets the same record either way.
                let comma = (*trailing_semi && self.editor).then(|| quote!(,));
                let sp = fs
                    .last()
                    .and_then(last_span)
                    .unwrap_or_else(|| ctor.last().span());
                let b = group(Delimiter::Brace, quote!(#(#fs),* #comma), sp);
                Ok(Out::new(quote!(#p #b), Kind::Atomic))
            }
            Expr::RecordUpdate { base, fields, span } => {
                // For rustc, the new values come first, as in Rust's
                // `S { x: v, ..p }`: they may read the old record (`{ p with
                // x = p.y }`), which the copy then moves. An editor gets
                // them in place, so that a value being typed is where it
                // was (rust-analyzer does not check moves).
                let mut hoists = Vec::new();
                let mut vals = Vec::new();
                for (_, v) in fields {
                    vals.push(if self.editor {
                        v.clone()
                    } else {
                        self.hoist(v, &mut hoists)?
                    });
                }
                let b = self.expr(base, false)?;
                let r = if self.editor {
                    match base.strip() {
                        // The copy of a variable is named after it, with
                        // macro hygiene: the user's code still sees the
                        // variable, and rust-analyzer, which offers the
                        // locals in scope by name, hygiene or not, offers it
                        // once (the variable's name and type) instead of a
                        // generated name.
                        Expr::Var(i) if b.kind == Kind::Name && i != MISSING => {
                            let mut r = rust_ident(i);
                            r.set_span(Span::mixed_site());
                            r
                        }
                        _ => Ident::new("__rsml_record", Span::mixed_site()),
                    }
                } else {
                    let sp = first_span(&b.ts).unwrap_or(*span);
                    Ident::new("record", Span::mixed_site().located_at(sp))
                };
                let b = b.ts;
                let mut sets = Vec::new();
                for ((f, _), v) in fields.iter().zip(&vals) {
                    let f = rust_ident(f);
                    let v = self.expr(v, false)?.ts;
                    sets.push(quote!(#r.#f = #v;));
                }
                Ok(Out::new(
                    quote_spanned!(*span=> { #(#hoists)* let mut #r = #b; #(#sets)* #r }),
                    Kind::Compound,
                ))
            }
            Expr::For {
                var,
                from,
                to,
                down,
                body,
                span,
            } => {
                let from = self.expr(from, false)?.operand();
                let to = self.expr(to, false)?.operand();
                let len = self.env.len();
                self.bind(ident_str(var), Bind::Local(None));
                let body = self.stmt_block(body)?;
                self.env.truncate(len);
                let v = rust_ident(var);
                let range = if *down {
                    quote!((#to..=#from).rev())
                } else {
                    quote!(#from..=#to)
                };
                let kw = mapped_keyword("for", *span);
                Ok(Out::new(quote!(#kw #v in #range #body), Kind::Compound))
            }
            Expr::ForIn {
                pat,
                iter,
                body,
                span,
            } => {
                let it = self.expr(iter, false)?.ts;
                let len = self.env.len();
                let p = self.pat(pat)?;
                self.bind_pat(pat);
                let body = self.stmt_block(body);
                self.env.truncate(len);
                let body = body?;
                let kw = mapped_keyword("for", *span);
                Ok(Out::new(quote!(#kw #p in #it #body), Kind::Compound))
            }
            Expr::While { cond, body, span } => {
                // `while true` is OCaml's endless loop; Rust's is `loop`.
                if let Expr::Bool(true, _) = cond.strip() {
                    let body = self.stmt_block(body)?;
                    return Ok(Out::new(quote_spanned!(*span=> loop #body), Kind::Compound));
                }
                let c = self.expr(cond, false)?.ts;
                let body = self.stmt_block(body)?;
                let kw = mapped_keyword("while", *span);
                Ok(Out::new(quote!(#kw #c #body), Kind::Compound))
            }
            Expr::Macro { path, bang, group } => {
                let p = path_tokens(path);
                let b = quote_spanned!(*bang=> !);
                Ok(Out::new(quote!(#p #b #group), Kind::Call))
            }
            Expr::StdMacro {
                path,
                bang,
                group,
                args,
                repeat,
                trailing_comma,
            } => {
                let name = path.last().to_string();
                if !self.editor
                    && (name == "assert" || name == "debug_assert")
                    && let Some(err) = args.first().and_then(|c| self.deref_link(c.expr()))
                {
                    return Err(err);
                }
                // `vec!`'s items and `dbg!`'s argument are the macro's value.
                let escapes = tail && (name == "vec" || name == "dbg");
                let mut items = Vec::new();
                for a in args {
                    items.push(match a {
                        MacroArg::Expr(e) => self.expr(e, escapes)?.ts,
                        MacroArg::Named(n, e) => {
                            let v = self.expr(e, false)?.ts;
                            quote!(#n = #v)
                        }
                    });
                }
                let mut inner = if *repeat {
                    quote!(#(#items);*)
                } else {
                    quote!(#(#items),*)
                };
                // A trailing comma is kept for an editor only, as in a call:
                // either way is the same macro call.
                if *trailing_comma && self.editor {
                    inner.extend(quote!(,));
                }
                let mut g = Group::new(group.delimiter(), inner);
                g.set_span(group.span());
                let p = path_tokens(path);
                let b = quote_spanned!(*bang=> !);
                Ok(Out::new(quote!(#p #b #g), Kind::Call))
            }
            Expr::Rust(group) => Ok(Out::new(quote!(#group), Kind::Call)),
            Expr::Assert(inner, sp) => {
                if let Expr::Bool(false, _) = inner.strip() {
                    return Ok(Out::new(
                        quote_spanned!(*sp=> ::core::unreachable!("assert false")),
                        Kind::Call,
                    ));
                }
                let o = self.expr(inner, false)?.ts;
                Ok(Out::new(
                    quote_spanned!(*sp=> ::core::assert!(#o)),
                    Kind::Call,
                ))
            }
            Expr::Section { body, params, .. } => self.section_value(body, params, tail),
            Expr::MethodRef {
                name,
                turbofish,
                args,
                span,
            } => {
                let r = self.local_at("r", "receiver", *span);
                let (a, mv) = match args {
                    // Like `fun r -> r.name(args)`.
                    Some(args) => {
                        let captures = !args.is_empty() && !args.iter().any(Expr::mutates);
                        (self.exprs(args.iter(), false)?, tail && captures)
                    }
                    None => (Vec::new(), false),
                };
                let call = method_call(quote!(#r), name, turbofish, &a, false);
                let mv = mv.then(|| keyword("move", *span));
                let bars = closure_bars(&[quote!(#r)], *span);
                Ok(Out::new(quote!(#mv #bars #call), Kind::Compound).shaped(Some(vec![1])))
            }
            Expr::FieldRef { idx, span } => {
                let r = self.local_at("r", "tuple", *span);
                let f = tuple_fields(quote!(#r), idx);
                let bars = closure_bars(&[quote!(#r)], *span);
                Ok(Out::new(quote!(#bars #f), Kind::Compound).shaped(Some(vec![1])))
            }
            Expr::Hole { span, .. } => Err(Error::new(
                *span,
                "`_` stands for a parameter of the expression around it, but it is alone here",
            )),
            Expr::Surface(_) => unreachable!("surface syntax is only kept for the formatter"),
        }
    }

    fn exprs<'a>(&mut self, es: impl Iterator<Item = &'a Expr>, tail: bool) -> R<Vec<TokenStream>> {
        es.map(|e| self.expr(e, tail).map(|o| o.ts)).collect()
    }

    /// A loop body: always a unit-typed block.
    fn stmt_block(&mut self, body: &Expr) -> R<TokenStream> {
        let o = self.expr(body, false)?;
        let ts = o.ts;
        let sp = last_span(&ts).unwrap_or_else(Span::call_site);
        Ok(group(Delimiter::Brace, quote!(#ts;), sp))
    }

    fn place(&mut self, e: &Expr) -> R<TokenStream> {
        match e.strip() {
            Expr::Var(i) => Ok(ident_ts(i)),
            Expr::Field(..) | Expr::TupleIndex(..) | Expr::Index(..) | Expr::Deref(..) => {
                Ok(self.expr(e.strip(), false)?.ts)
            }
            _ => Err(Error::new(span_of(e), "cannot assign to this expression")),
        }
    }

    fn var(&mut self, i: &Ident) -> R<Out> {
        let name = ident_str(i);
        match self.lookup(&name).cloned() {
            Some(Bind::Local(shape)) => return Ok(Out::new(ident_ts(i), Kind::Name).shaped(shape)),
            Some(Bind::RefCell) => {
                let r = rust_ident(i);
                let m = keyword("mut", i.span());
                let amp = located(quote!(&), i.span());
                return Ok(Out::new(quote!(#amp #m #r), Kind::Compound));
            }
            Some(Bind::External(path, shape)) => {
                return Ok(Out::new(path, Kind::Name).shaped(shape));
            }
            Some(Bind::Op(..)) | None => {}
        }
        let sp = i.span();
        // `r#not` is a Rust function, not OCaml's `not`.
        match word(i).as_str() {
            "not" => {
                let a = self.local_at("a", "operand", sp);
                return Ok(
                    Out::new(quote_spanned!(sp=> |#a| !#a), Kind::Compound).shaped(Some(vec![1]))
                );
            }
            "incr" | "decr" => {
                let a = self.local_at("r", "operand", sp);
                let op = if name == "incr" {
                    quote!(+=)
                } else {
                    quote!(-=)
                };
                let ts = quote_spanned!(sp=> |#a: &mut _| *#a #op 1);
                return Ok(Out::new(ts, Kind::Compound).shaped(Some(vec![1])));
            }
            "ref" => {
                return Err(Error::new(
                    sp,
                    "`ref` is only supported directly in `let r = ref e`",
                ));
            }
            "raise" => {
                return Err(Error::new(
                    sp,
                    "exceptions are not supported; use `panic!` or `Result`",
                ));
            }
            _ => {}
        }
        Ok(Out::new(ident_ts(i), Kind::Name))
    }

    /// A function about to be called.
    fn callee(&mut self, head: &Expr) -> R<Out> {
        match head {
            Expr::Var(i) => self.var(i),
            Expr::Path(p) => Ok(Out::new(path_tokens(p), Kind::Name).shaped(self.path_shape(p))),
            _ => self.expr(head, false),
        }
    }

    /// A function of known arity used as a value, for rust-analyzer: `plus`
    /// (two parameters) is `{ move |p1, p2| plus(p1, p2) }`, which lines up
    /// with `plus a` (see [`Cx::partial`]). rustc gets `plus`.
    ///
    /// rust-analyzer completes `plus ‸` in the expansion of `plus marker`
    /// (`{ let a1 = marker; move |p2| plus(a1, p2) }`), at the marker's
    /// offset, and takes the scope and the expected type from the token at
    /// the same offset in the expansion of `plus`. Bare `plus` is shorter
    /// than that offset, so nothing completed. Here the offset falls on
    /// `p1`, the first parameter: in scope are the locals around `plus`
    /// (not `p1` or `p2`), and the expected type is `p1`'s, the type of
    /// `plus`'s first parameter.
    ///
    /// Likewise `f` taking `()` is `{ move || f() }`, against the
    /// `{ let () = marker; f() }` of `f marker`: the marker falls on its
    /// `||`, with no expected type. A function of one parameter is left
    /// alone: `inc marker` is the call `inc(marker)`, whose marker falls
    /// after `inc`, on the `}` or `;` that follows it.
    fn eta(&mut self, out: Out) -> Out {
        let k = match out.shape.as_deref() {
            Some(&[k, ..]) if self.editor && out.kind == Kind::Name && k != 1 => k,
            _ => return out,
        };
        let sp = first_span(&out.ts).unwrap_or_else(Span::call_site);
        let params: Vec<Ident> = (1..=k)
            .map(|j| self.local_at("p", &format!("arg{}", j), sp))
            .collect();
        let f = &out.ts;
        let call = call_parens(f, quote!(#(#params),*));
        let c = closure(Vec::new(), &params, quote!(#f #call), sp);
        Out::new(block(c), Kind::Compound).shaped(out.shape)
    }

    // ----- application ---------------------------------------------------

    fn app(&mut self, e: &Expr, hint: Option<Shape>, tail: bool) -> R<Out> {
        let (head, args, groups) = collect_app(e);
        self.apply(head, &args, &groups, hint, tail)
    }

    fn apply(
        &mut self,
        head: &Expr,
        args: &[&Expr],
        groups: &Groups,
        hint: Option<Shape>,
        tail: bool,
    ) -> R<Out> {
        if args.is_empty() {
            return self.expr(head, tail);
        }
        let head = head.strip();
        match head {
            Expr::App(..) | Expr::Call { .. } => {
                let (h, mut all, mut g) = collect_app(head);
                let n = all.len();
                if matches!(head, Expr::Call { .. }) || groups.starts.contains(&0) {
                    g.starts.push(n);
                }
                g.starts
                    .extend(groups.starts.iter().filter(|s| **s > 0).map(|s| s + n));
                g.commas.extend(groups.commas.iter().map(|c| c + n));
                all.extend(args.iter().copied());
                return self.apply(h, &all, &g, hint, tail);
            }
            Expr::Fun { params, body, .. } if args.len() >= params.len() => {
                return self.beta(params, body, args, groups, tail);
            }
            Expr::Compose { first, second, .. } => {
                // (f >> g) x rest = g (f x) rest
                let inner = Expr::App(first.clone(), vec![args[0].clone()]);
                let mut all = vec![&inner];
                all.extend(args[1..].iter().copied());
                return self.apply(second, &all, groups, hint, tail);
            }
            Expr::OpSection(op) => return self.apply_op_section(op, args, groups, tail),
            Expr::Section { body, params, .. } => {
                return self.apply_section(body, params, args, groups, tail);
            }
            Expr::MethodRef {
                name,
                turbofish,
                args: margs,
                ..
            } => {
                let recv = self.expr(args[0], false)?.operand();
                let (a, used) = match margs {
                    // `.m(a) r` is `r.m(a)`.
                    Some(margs) => (self.exprs(margs.iter(), false)?, 1),
                    // `.m r a b` is `r.m(a, b)`: every argument after the
                    // receiver goes to the method (up to the end of a
                    // Rust-style call). The receiver alone is a call with no
                    // arguments (`.iter xs`), so `()` after it is a real
                    // unit argument: `.send tx ()` is `tx.send(())`.
                    None => {
                        let end = groups.first_end(args.len());
                        (self.exprs(args[1..end].iter().copied(), false)?, end)
                    }
                };
                let out = Out::new(method_call(recv, name, turbofish, &a, false), Kind::Call);
                if used == args.len() {
                    return Ok(out);
                }
                return self.apply_call(out, None, &args[used..], &groups.after(used), None, tail);
            }
            // `.0 t` is `t.0`.
            Expr::FieldRef { idx, .. } => {
                let recv = self.expr(args[0], false)?.operand();
                let out = Out::new(tuple_fields(recv, idx), Kind::Atomic);
                if args.len() == 1 {
                    return Ok(out);
                }
                return self.apply_call(out, None, &args[1..], &groups.after(1), None, tail);
            }
            Expr::Var(i) if self.lookup(&ident_str(i)).is_none() && is_special(&word(i)) => {
                return self.apply_special(i, args, groups, tail);
            }
            _ => {}
        }
        let callee = self.callee(head)?;
        let shape = callee
            .shape
            .clone()
            .or_else(|| hint.map(|h| hinted(groups.lens(args.len()), h)));
        self.apply_call(callee, shape, args, groups, ctor_name(head), tail)
    }

    fn apply_call(
        &mut self,
        callee: Out,
        shape: Option<Shape>,
        args: &[&Expr],
        groups: &Groups,
        ctor: Option<String>,
        tail: bool,
    ) -> R<Out> {
        let Some(gs) = shape.filter(|s| !s.is_empty()) else {
            // Unknown arity: one Rust call per group of arguments. Closures
            // passed to a constructor are part of the result; passed to a
            // function, they are not.
            let end = groups.first_end(args.len());
            let a = self.call_args(&args[..end], ctor.as_deref(), tail && ctor.is_some())?;
            // A trailing comma reads the same as an argument being added.
            let comma = (self.editor && groups.commas.contains(&end)).then(|| quote!(,));
            let f = callee.callee();
            let g = call_parens(&f, quote!(#(#a),* #comma));
            let out = Out::new(quote!(#f #g), Kind::Call);
            if end == args.len() {
                return Ok(out);
            }
            return self.apply_call(out, None, &args[end..], &groups.after(end), None, tail);
        };
        let g = gs[0];
        let rest = if gs.len() > 1 {
            Some(gs[1..].to_vec())
        } else {
            None
        };
        if g == 0 {
            let f = callee.callee();
            let unit = call_parens(&f, quote!());
            let call = if matches!(args[0].strip(), Expr::Unit(_)) {
                quote!(#f #unit)
            } else {
                let a = self.expr(args[0], false)?.ts;
                block(quote!(let () = #a; #f #unit))
            };
            let out = Out::new(call, Kind::Call).shaped(rest.clone());
            if args.len() == 1 {
                return Ok(out);
            }
            return self.apply_call(out, rest, &args[1..], &groups.after(1), None, tail);
        }
        if args.len() >= g {
            let a = self.exprs(args[..g].iter().copied(), false)?;
            let f = callee.callee();
            let call = call_parens(&f, quote!(#(#a),*));
            let out = Out::new(quote!(#f #call), Kind::Call).shaped(rest.clone());
            if args.len() == g {
                return Ok(out);
            }
            return self.apply_call(out, rest, &args[g..], &groups.after(g), None, tail);
        }
        self.partial(callee, &gs, args)
    }

    /// Arguments of a call to a function of unknown arity.
    fn call_args(&mut self, args: &[&Expr], ctor: Option<&str>, tail: bool) -> R<Vec<TokenStream>> {
        if args.len() == 1 {
            match args[0] {
                // `f ()` calls a zero-argument function; `Some ()` holds a unit.
                Expr::Unit(_) if ctor.is_none() => return Ok(Vec::new()),
                Expr::Paren(inner, _) if ctor.is_some_and(|c| !single_field_ctor(c)) => {
                    if let Expr::Tuple(items) = &**inner {
                        return self.exprs(items.iter(), tail);
                    }
                }
                _ => {}
            }
        }
        self.exprs(args.iter().copied(), tail)
    }

    /// `f a` where `f` takes more arguments: `move |b| f(a, b)`, with `a`
    /// evaluated once, up front, as OCaml does.
    ///
    /// For rust-analyzer every argument but a closure or a constant is
    /// bound first: `plus a` is `{ let a1 = a; move |p2| plus(a1, p2) }`.
    /// Nothing before such an argument then depends on the arguments after
    /// it, and completion lines up (see [`Cx::eta`]): in `plus a ‸`, for a
    /// function of three parameters, the marker's `let a2 = marker` falls on
    /// the `p2` of `plus a`'s `move |p2, p3|`. No generated name is in scope
    /// in the first argument either. A closure stays in place, where the
    /// call gives its parameters their types, and so does a constant, which
    /// the closure would otherwise capture (hovers show closure sizes).
    fn partial(&mut self, callee: Out, groups: &[usize], args: &[&Expr]) -> R<Out> {
        let mut hoists = Vec::new();
        let f = if callee.kind == Kind::Name {
            callee.ts.clone()
        } else {
            // `mut`, for a closure that mutates what it captures.
            let ts = &callee.ts;
            let sp = first_span(ts).unwrap_or_else(Span::call_site);
            let t = self.numbered_at("f", "function", sp);
            hoists.push(quote!(let mut #t = #ts;));
            quote!(#t)
        };
        let mut arg_ts = Vec::new();
        for a in args {
            let h = self.bind_value(a, false, &mut hoists)?;
            arg_ts.push(self.expr(&h, false)?.ts);
        }
        let remaining = groups[0] - args.len();
        let sp = first_span(&callee.ts).unwrap_or_else(Span::call_site);
        // Named for the argument each stands for: `add 1` is
        // `move |arg2| add(1, arg2)`.
        let params: Vec<Ident> = (0..remaining)
            .map(|k| self.local_at("p", &format!("arg{}", args.len() + k + 1), sp))
            .collect();
        arg_ts.extend(params.iter().map(|p| quote!(#p)));
        // Located at the function, like the call it stands for.
        let call = call_parens(&callee.ts, quote!(#(#arg_ts),*));
        let ts = closure(hoists, &params, quote!(#f #call), sp);
        let mut shape = vec![remaining];
        shape.extend(&groups[1..]);
        Ok(Out::new(ts, Kind::Compound).shaped(Some(shape)))
    }

    /// `(fun x y -> body) a b` is `{ let (x, y) = (a, b); body }`.
    fn beta(
        &mut self,
        params: &[Pat],
        body: &Expr,
        args: &[&Expr],
        groups: &Groups,
        tail: bool,
    ) -> R<Out> {
        let n = params.len();
        let mut stmts = Vec::new();
        // Extra arguments are evaluated before the parameters come into scope.
        let mut rest = Vec::new();
        for a in &args[n..] {
            let out = self.expr(a, false)?;
            let ts = out.ts;
            let sp = first_span(&ts).unwrap_or_else(|| span_of(a));
            let t = self.numbered_at("a", "value", sp);
            stmts.push(quote!(let #t = #ts;));
            rest.push(Expr::Tmp(t, out.shape));
        }
        let vals = self.exprs(args[..n].iter().copied(), false)?;
        let len = self.env.len();
        if n == 1 {
            let (p, ty) = self.typed_pat(&params[0])?;
            let v = &vals[0];
            stmts.push(quote!(let #p #ty = #v;));
        } else {
            let mut ps = Vec::new();
            let mut tys = Vec::new();
            let mut typed = false;
            for p in params {
                let (pt, ty) = self.typed_pat(p)?;
                ps.push(pt);
                typed |= !ty.is_empty();
                tys.push(if ty.is_empty() {
                    quote!(_)
                } else {
                    strip_colon(ty)
                });
            }
            let ann = if typed {
                quote!(: (#(#tys),*))
            } else {
                quote!()
            };
            stmts.push(quote!(let (#(#ps),*) #ann = (#(#vals),*);));
        }
        for p in params {
            self.bind_pat(p);
        }
        let rest_refs: Vec<&Expr> = rest.iter().collect();
        let out = self.apply(body, &rest_refs, &groups.after(n), None, tail)?;
        self.env.truncate(len);
        let ts = out.ts;
        Ok(Out::new(block(quote!(#(#stmts)* #ts)), Kind::Compound).shaped(out.shape))
    }

    fn apply_special(&mut self, i: &Ident, args: &[&Expr], groups: &Groups, tail: bool) -> R<Out> {
        let name = word(i);
        let sp = i.span();
        let out = match name.as_str() {
            "not" => {
                let a = self.expr(args[0], false)?.operand();
                Out::new(quote_spanned!(sp=> !#a), Kind::Compound)
            }
            "incr" | "decr" => {
                let op = if name == "incr" {
                    quote_spanned!(sp=> +=)
                } else {
                    quote_spanned!(sp=> -=)
                };
                let one = Literal::usize_unsuffixed(1);
                let ts = match self.is_refcell(args[0]) {
                    Some(r) => quote!(#r #op #one),
                    None => {
                        let a = self.expr(args[0], false)?.operand();
                        let star = located(quote!(*), sp);
                        quote!(#star #a #op #one)
                    }
                };
                Out::new(ts, Kind::Compound)
            }
            "ref" => {
                return Err(Error::new(
                    sp,
                    "`ref` is only supported directly in `let r = ref e`",
                ));
            }
            _ => {
                return Err(Error::new(
                    sp,
                    "exceptions are not supported; use `panic!` or `Result`",
                ));
            }
        };
        if args.len() == 1 {
            return Ok(out);
        }
        self.apply_call(out, None, &args[1..], &groups.after(1), None, tail)
    }

    // ----- sections ------------------------------------------------------

    /// The operands of a section's body that are evaluated once, when the
    /// section is created (as the arguments of a partial application are):
    /// those of a call, method call or (strict) operator, and of tuples and
    /// lists. Everything else is part of the function body.
    fn operands<'e>(&self, body: &'e Expr) -> Vec<&'e Expr> {
        match body {
            Expr::App(h, args) | Expr::Call { func: h, args, .. } => {
                std::iter::once(&**h).chain(args).collect()
            }
            Expr::Method { recv, args, .. } => std::iter::once(&**recv).chain(args).collect(),
            Expr::Binary { op, lhs, rhs } if !matches!(op.text.as_str(), "&&" | "||" | "or") => {
                vec![lhs, rhs]
            }
            Expr::Tuple(es) | Expr::List(es, _) | Expr::Array(es, _) => es.iter().collect(),
            _ => Vec::new(),
        }
    }

    /// `body` with its non-trivial operands without holes evaluated into
    /// hoisted temporaries (for rust-analyzer, every operand but closures,
    /// constants and the function called: see [`Cx::bind_value`]), and
    /// whether rustc's expansion hoists anything.
    fn hoist_operands(&mut self, body: &Expr, hoists: &mut Vec<TokenStream>) -> R<(Expr, bool)> {
        let mut out = body.clone();
        // The slots of a function that is called and of a method's
        // receiver (see [`Cx::hoist_mut`]).
        let (slots, called): (Vec<&mut Expr>, &[usize]) = match &mut out {
            Expr::App(h, args) | Expr::Call { func: h, args, .. } => {
                let method = matches!(h.strip(), Expr::MethodRef { .. });
                (
                    std::iter::once(&mut **h).chain(args.iter_mut()).collect(),
                    if method { &[1] } else { &[0] },
                )
            }
            Expr::Method { recv, args, .. } => (
                std::iter::once(&mut **recv)
                    .chain(args.iter_mut())
                    .collect(),
                &[0],
            ),
            Expr::Binary { op, lhs, rhs } if !matches!(op.text.as_str(), "&&" | "||" | "or") => {
                (vec![&mut **lhs, &mut **rhs], &[])
            }
            Expr::Tuple(es) | Expr::List(es, _) | Expr::Array(es, _) => {
                (es.iter_mut().collect(), &[])
            }
            _ => (Vec::new(), &[]),
        };
        let mut hoisted = false;
        for (k, slot) in slots.into_iter().enumerate() {
            if slot.has_hole() {
                continue;
            }
            hoisted |= !self.is_trivial(slot);
            *slot = if called.contains(&k) {
                self.hoist_value(slot, true, hoists)?
            } else {
                self.bind_value(slot, false, hoists)?
            };
        }
        Ok((out, hoisted))
    }

    /// A section as a value: `f _ 1` is `move |a| f(a, 1)`.
    fn section_value(&mut self, body: &Expr, params: &[Option<Type>], tail: bool) -> R<Out> {
        let mut hoists = Vec::new();
        let (body, hoisted) = self.hoist_operands(body, &mut hoists)?;
        let ps = self.hole_params(&body, 0..params.len());
        let vals: Vec<Expr> = ps.iter().map(|p| Expr::Tmp(p.clone(), None)).collect();
        self.section_closure(&body, params, hoists, &ps, &vals, tail || hoisted)
    }

    /// The closure parameters for placeholders `range` of a section's
    /// `body`: `_0`, `_1`, .., each located at its first placeholder (see
    /// [`Cx::local`]).
    fn hole_params(&mut self, body: &Expr, range: std::ops::Range<usize>) -> Vec<Ident> {
        let mut spans = Vec::new();
        body.clone()
            .visit_holes(&mut |_, h| spans.extend(h.index.map(|i| (i, h.span))));
        range
            .map(|i| {
                let sp = spans
                    .iter()
                    .find(|(j, _)| *j == i)
                    .map_or_else(|| span_of(body), |s| s.1);
                self.local_at("p", &format!("_{}", i), sp)
            })
            .collect()
    }

    /// `move |ps..| body` with the holes of `body` replaced by `vals`; the
    /// last `ps.len()` holes are the closure's parameters. `escapes`: the
    /// closure is the result (`tail`) or captures values that rustc's
    /// expansion hoists (rust-analyzer's may hoist more).
    fn section_closure(
        &mut self,
        body: &Expr,
        params: &[Option<Type>],
        hoists: Vec<TokenStream>,
        ps: &[Ident],
        vals: &[Expr],
        escapes: bool,
    ) -> R<Out> {
        let filled = fill_holes(body, vals);
        let out = self.expr(&filled, true)?;
        let first = params.len() - ps.len();
        let used = hole_indices(body);
        let mut pts = Vec::new();
        for (k, p) in ps.iter().enumerate() {
            let i = first + k;
            let pat = if used.contains(&i) {
                quote!(#p)
            } else {
                quote!(_)
            };
            pts.push(match &params[i] {
                Some(t) => {
                    let t = t.emit(Ctx::Local, &mut Vec::new());
                    quote!(#pat: #t)
                }
                None => pat,
            });
        }
        // Like `fun`: `move` when it escapes (or captures hoisted values),
        // never when it assigns to something.
        let sp = span_of(body);
        let mv = (escapes && !body.mutates()).then(|| keyword("move", sp));
        let bars = closure_bars(&pts, sp);
        let bt = out.ts;
        let mut shape = vec![ps.len()];
        if let Some(s) = out.shape {
            shape.extend(s);
        }
        let c = Out::new(quote!(#mv #bars #bt), Kind::Compound).shaped(Some(shape));
        Ok(with_hoists(hoists, c))
    }

    /// A section applied to arguments: `(f _1 _0) a b` is `f(b, a)`, with no
    /// closure. Arguments are hoisted when the holes would evaluate them
    /// more than once, out of order, or inside a binding.
    fn apply_section(
        &mut self,
        body: &Expr,
        params: &[Option<Type>],
        args: &[&Expr],
        groups: &Groups,
        tail: bool,
    ) -> R<Out> {
        let n = params.len();
        let mut hoists = Vec::new();
        if args.len() < n {
            let (body, mut hoisted) = self.hoist_operands(body, &mut hoists)?;
            let called = called_holes(&body);
            let mut vals = Vec::new();
            for (i, a) in args.iter().enumerate() {
                hoisted |= !self.is_trivial(a);
                vals.push(self.bind_value(a, called.contains(&i), &mut hoists)?);
            }
            let ps = self.hole_params(&body, args.len()..n);
            vals.extend(ps.iter().map(|p| Expr::Tmp(p.clone(), None)));
            return self.section_closure(&body, params, hoists, &ps, &vals, tail || hoisted);
        }
        let in_order =
            hole_indices(body) == (0..n).collect::<Vec<_>>() && !holes_under_binder(body);
        let called = called_holes(body);
        let mut vals = Vec::new();
        for (i, a) in args[..n].iter().enumerate() {
            vals.push(if in_order || self.is_trivial(a) {
                (*a).clone()
            } else {
                self.hoist_value(a, called.contains(&i), &mut hoists)?
            });
        }
        let filled = fill_holes(body, &vals);
        let out = self.expr(&filled, tail)?;
        let out = with_hoists(hoists, out);
        if args.len() == n {
            return Ok(out);
        }
        let shape = out.shape.clone();
        self.apply_call(out, shape, &args[n..], &groups.after(n), None, tail)
    }

    // ----- functions -----------------------------------------------------

    fn typed_pat(&mut self, p: &Pat) -> R<(TokenStream, TokenStream)> {
        match p {
            Pat::Paren(inner) => self.typed_pat(inner),
            Pat::Constraint(inner, ty) => {
                let t = ty.emit(Ctx::Local, &mut Vec::new());
                Ok((self.pat(inner)?, quote!(: #t)))
            }
            _ => Ok((self.pat(p)?, quote!())),
        }
    }

    fn closure_params(&mut self, params: &[Pat]) -> R<Vec<TokenStream>> {
        if params.len() == 1 && is_unit_pat(&params[0]) {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for p in params {
            let (pt, ty) = self.typed_pat(p)?;
            out.push(quote!(#pt #ty));
        }
        Ok(out)
    }

    fn arity(params: &[Pat]) -> usize {
        if params.len() == 1 && is_unit_pat(&params[0]) {
            0
        } else {
            params.len()
        }
    }

    fn fun_value(&mut self, params: &[Pat], body: &Expr, is_move: bool, sp: Span) -> R<Out> {
        let len = self.env.len();
        let ps = self.closure_params(params)?;
        for p in params {
            self.bind_pat(p);
        }
        let b = self.expr(body, true)?;
        self.env.truncate(len);
        let mv = is_move.then(|| keyword("move", sp));
        let mut shape = vec![Self::arity(params)];
        if let Some(s) = b.shape {
            shape.extend(s);
        }
        let bt = b.ts;
        let bars = closure_bars(&ps, sp);
        Ok(Out::new(quote!(#mv #bars #bt), Kind::Compound).shaped(Some(shape)))
    }

    /// `f >> g` as a value: `move |x| g(f(x))`, with the non-trivial parts of
    /// `f` and `g` evaluated once, up front.
    fn compose_value(&mut self, e: &Expr) -> R<Out> {
        let mut hoists = Vec::new();
        let prepared = self.prepare_fn(e, &mut hoists)?;
        let start = match e.strip() {
            Expr::Compose { first, .. } => span_of(first),
            e => span_of(e),
        };
        let x = self.local_at("x", "argument", start);
        let arg = Expr::Tmp(x.clone(), None);
        let body = self.apply(&prepared, &[&arg], &Groups::default(), None, false)?;
        let mut shape = vec![1];
        if let Some(s) = body.shape {
            shape.extend(s);
        }
        let ts = closure(hoists, &[x], body.ts, start);
        Ok(Out::new(ts, Kind::Compound).shaped(Some(shape)))
    }

    fn prepare_fn(&mut self, e: &Expr, hoists: &mut Vec<TokenStream>) -> R<Expr> {
        match e.strip() {
            Expr::Compose {
                first,
                second,
                span,
            } => Ok(Expr::Compose {
                first: Box::new(self.prepare_part(first, hoists)?),
                second: Box::new(self.prepare_part(second, hoists)?),
                span: *span,
            }),
            Expr::App(h, args) => {
                let head = self.prepare_fn(h, hoists)?;
                // `.push (Vec::new ()) >> f`: the receiver of a method.
                let method = matches!(h.strip(), Expr::MethodRef { .. });
                let mut prepared = Vec::new();
                for (k, a) in args.iter().enumerate() {
                    prepared.push(self.bind_value(a, method && k == 0, hoists)?);
                }
                Ok(Expr::App(Box::new(head), prepared))
            }
            // With a known shape, the arguments are evaluated once and the
            // call is part of the composition, as for `add 1 >> ..`; a Rust
            // function's call is evaluated once, up front.
            Expr::Call {
                func,
                args,
                trailing_comma,
                span,
            } if self.known_fn(collect_app(e).0) => {
                let func = self.prepare_fn(func, hoists)?;
                let mut prepared = Vec::new();
                for a in args {
                    prepared.push(self.bind_value(a, false, hoists)?);
                }
                Ok(Expr::Call {
                    func: Box::new(func),
                    args: prepared,
                    trailing_comma: *trailing_comma,
                    span: *span,
                })
            }
            // A function the composition calls.
            other => self.hoist_mut(other, hoists),
        }
    }

    /// A function composed with `>>` or `<<`: for rust-analyzer it goes
    /// into a temporary even if it is a name (see [`Cx::bind_value`]).
    fn prepare_part(&mut self, e: &Expr, hoists: &mut Vec<TokenStream>) -> R<Expr> {
        match e.strip() {
            Expr::Compose { .. } | Expr::App(..) | Expr::Call { .. } => self.prepare_fn(e, hoists),
            _ => self.bind_value(e, true, hoists),
        }
    }

    fn op_section_value(&mut self, op: &Op) -> R<Out> {
        if let Some(Bind::Op(f, shape)) = self.lookup(&format!("op:{}", op.text)).cloned() {
            self.unmapped.extend(&op.spans);
            return Ok(Out::new(f, Kind::Name).shaped(Some(shape)));
        }
        let sp = op.span();
        // The parameters end the closure's body, so errors about it point
        // at the operator.
        let a = if is_unary_section(&op.text) {
            self.local_at("a", "operand", sp)
        } else {
            self.local_at("a", "lhs", sp)
        };
        if is_unary_section(&op.text) {
            let body = self.unary_section(op, &Expr::Tmp(a.clone(), None))?.ts;
            return Ok(
                Out::new(quote_spanned!(sp=> |#a| #body), Kind::Compound).shaped(Some(vec![1]))
            );
        }
        let b = self.local_at("b", "rhs", sp);
        let body = self
            .op_exprs(op, &Expr::Tmp(a.clone(), None), &Expr::Tmp(b.clone(), None))?
            .ts;
        Ok(Out::new(quote_spanned!(sp=> |#a, #b| #body), Kind::Compound).shaped(Some(vec![2])))
    }

    fn unary_section(&mut self, op: &Op, a: &Expr) -> R<Out> {
        let sp = op.span();
        let o = self.expr(a, false)?.operand();
        match op.text.as_str() {
            "!" => Ok(Out::new(quote_spanned!(sp=> *#o), Kind::Compound)),
            _ => Ok(Out::new(quote_spanned!(sp=> -#o), Kind::Compound)),
        }
    }

    /// `a op b` for an operator used as a function.
    fn op_exprs(&mut self, op: &Op, a: &Expr, b: &Expr) -> R<Out> {
        let span = op.span();
        if matches!(op.text.as_str(), "|>" | "@@" | ">>" | "<<") {
            self.unmapped.extend(&op.spans);
        }
        match op.text.as_str() {
            "|>" => self.apply(b, &[a], &Groups::default(), None, false),
            "@@" => self.apply(a, &[b], &Groups::default(), None, false),
            ">>" => self.compose_value(&Expr::Compose {
                first: Box::new(a.clone()),
                second: Box::new(b.clone()),
                span,
            }),
            "<<" => self.compose_value(&Expr::Compose {
                first: Box::new(b.clone()),
                second: Box::new(a.clone()),
                span,
            }),
            _ => self.binary(op, a, b),
        }
    }

    fn apply_op_section(&mut self, op: &Op, args: &[&Expr], groups: &Groups, tail: bool) -> R<Out> {
        if let Some(Bind::Op(f, shape)) = self.lookup(&format!("op:{}", op.text)).cloned() {
            self.unmapped.extend(&op.spans);
            return self.apply_call(
                Out::new(f, Kind::Name),
                Some(shape),
                args,
                groups,
                None,
                tail,
            );
        }
        let arity = if is_unary_section(&op.text) { 1 } else { 2 };
        if args.len() >= arity {
            let out = if arity == 1 {
                self.unary_section(op, args[0])?
            } else {
                self.op_exprs(op, args[0], args[1])?
            };
            if args.len() == arity {
                return Ok(out);
            }
            let shape = out.shape.clone();
            return self.apply_call(out, shape, &args[arity..], &groups.after(arity), None, tail);
        }
        let mut hoists = Vec::new();
        let a = self.hoist(args[0], &mut hoists)?;
        let p = self.local_at("p", "rhs", op.span());
        let body = self.op_exprs(op, &a, &Expr::Tmp(p.clone(), None))?;
        let ts = closure(hoists, std::slice::from_ref(&p), body.ts, op.span());
        let mut shape = vec![1];
        if let Some(s) = body.shape {
            shape.extend(s);
        }
        Ok(Out::new(ts, Kind::Compound).shaped(Some(shape)))
    }

    // ----- operators -----------------------------------------------------

    fn binary(&mut self, op: &Op, lhs: &Expr, rhs: &Expr) -> R<Out> {
        let sp = op.span();
        if let Some(Bind::Op(f, _)) = self.lookup(&format!("op:{}", op.text)).cloned() {
            self.unmapped.extend(&op.spans);
            let a = self.expr(lhs, false)?.ts;
            let b = self.expr(rhs, false)?.ts;
            let args = group(Delimiter::Parenthesis, quote!(#a, #b), sp);
            return Ok(Out::new(quote!(#f #args), Kind::Call));
        }
        if matches!(op.text.as_str(), "::" | "^" | "^^" | "@" | "**") {
            self.unmapped.extend(&op.spans);
        }
        match op.text.as_str() {
            "::" => return self.cons(lhs, rhs, op),
            "^" | "^^" => {
                let a = self.expr(lhs, false)?.ts;
                let b = self.expr(rhs, false)?.ts;
                return Ok(Out::new(
                    quote_spanned!(sp=> ::rsml::__rt::concat(#a, #b)),
                    Kind::Call,
                ));
            }
            "@" => {
                let a = self.expr(lhs, false)?.ts;
                let b = self.expr(rhs, false)?.ts;
                return Ok(Out::new(
                    quote_spanned!(sp=> ::rsml::__rt::append(#a, #b)),
                    Kind::Call,
                ));
            }
            "**" => {
                let a = self.expr(lhs, false)?.ts;
                let b = self.expr(rhs, false)?.ts;
                return Ok(Out::new(
                    quote_spanned!(sp=> ::core::primitive::f64::powf(#a, #b)),
                    Kind::Call,
                ));
            }
            _ => {}
        }
        let Some(rust_op) = rust_binop(&op.text) else {
            if let Some(e) = assign_op_error(op)
                .or_else(|| bang_op_error(op))
                .or_else(|| glued_section_error(op))
            {
                return Err(e);
            }
            return Err(Error::new(
                sp,
                format!(
                    "unknown operator `{0}`; define it with `let ( {0} ) a b = ..`",
                    op.text
                ),
            ));
        };
        let a = self.expr(lhs, false)?.operand();
        let b = self.expr(rhs, false)?.operand();
        // Each character keeps its own span; `=` becoming `==` repeats it,
        // and the characters dropped (`.` of `+.`) need a stand-in.
        let n = rust_op.len();
        let spans: Vec<Span> = (0..n)
            .map(|i| op.spans[i.min(op.spans.len() - 1)])
            .collect();
        if op.spans.len() > n && op.text.len() == op.spans.len() {
            self.unmapped.extend(&op.spans[n..]);
        }
        // `=` becoming `==`: two tokens from one, which editors do not map back.
        if n > op.spans.len() {
            self.unmapped.extend(&op.spans);
        }
        let o = op_tokens(rust_op, &spans);
        Ok(Out::new(quote!(#a #o #b), Kind::Compound))
    }

    /// `a :: b :: [c]` is `vec![a, b, c]`; with any other tail each head is
    /// pushed onto the front at runtime.
    fn cons(&mut self, lhs: &Expr, rhs: &Expr, op: &Op) -> R<Out> {
        // `vec![..]` / `cons(..)` at the first `:`, its brackets at the second.
        let (sp, end) = (op.span(), op.last_span());
        let mut heads = vec![lhs];
        let mut cur = rhs;
        while let Expr::Binary { op, lhs, rhs } = cur.strip() {
            if op.text != "::" || self.lookup("op:::").is_some() {
                break;
            }
            heads.push(lhs);
            cur = rhs;
        }
        if let Expr::List(items, _) = cur.strip() {
            let mut all = heads;
            all.extend(items.iter());
            let ts = self.exprs(all.into_iter(), false)?;
            let mut items = Group::new(Delimiter::Bracket, quote!(#(#ts),*));
            items.set_span(end);
            return Ok(Out::new(quote_spanned!(sp=> ::std::vec!#items), Kind::Call));
        }
        let hs = self.exprs(heads.into_iter(), false)?;
        let mut acc = self.expr(cur, false)?.ts;
        for h in hs.iter().rev() {
            let mut args = Group::new(Delimiter::Parenthesis, quote!(#h, #acc));
            args.set_span(end);
            acc = quote_spanned!(sp=> ::rsml::__rt::cons #args);
        }
        Ok(Out::new(acc, Kind::Call))
    }

    fn constraint(&mut self, inner: &Expr, ty: &Type, tail: bool) -> R<Out> {
        if ty.is_arrow() {
            // `(f x : int -> int)` declares that the result is a function.
            let hint = ty.shape();
            let out = match inner.strip() {
                e @ Expr::App(..) => self.app(e, hint.clone(), tail)?,
                e => self.expr(e, tail)?,
            };
            let shape = out.shape.clone().or(hint);
            return Ok(out.shaped(shape));
        }
        let o = self.expr(inner, tail)?;
        let t = ty.emit(Ctx::Local, &mut Vec::new());
        let ts = o.ts;
        let sp = last_span(&ts).unwrap_or_else(Span::call_site);
        let name = if self.editor { "__rsml_typed" } else { "typed" };
        let v = Ident::new(name, Span::mixed_site().located_at(sp));
        let b = group(Delimiter::Brace, quote!(let #v: #t = #ts; #v), sp);
        Ok(Out::new(b, Kind::Compound).shaped(o.shape))
    }

    // ----- blocks and bindings -------------------------------------------

    fn block_expr(&mut self, e: &Expr, tail: bool) -> R<Out> {
        let len = self.env.len();
        let mut stmts = Vec::new();
        let result = self.block_items(e, tail, &mut stmts);
        self.env.truncate(len);
        let out = result?;
        if stmts.is_empty() {
            return Ok(out);
        }
        let ts = out.ts;
        Ok(Out::new(block(quote!(#(#stmts)* #ts)), Kind::Compound).shaped(out.shape))
    }

    fn block_items(&mut self, e: &Expr, tail: bool, stmts: &mut Vec<TokenStream>) -> R<Out> {
        match e {
            Expr::Let {
                rec_,
                bindings,
                body,
                ..
            } => {
                if *rec_ {
                    self.let_rec(bindings, stmts)?;
                } else {
                    self.let_bindings(bindings, stmts)?;
                }
                self.block_items(body, tail, stmts)
            }
            Expr::Seq(a, b) => {
                let ts = self.expr(a, false)?.ts;
                stmts.push(quote!(#ts;));
                self.block_items(b, tail, stmts)
            }
            Expr::External {
                name,
                ty,
                rust,
                body,
                ..
            } => {
                let path = match (rust, name) {
                    (Some((s, sp)), _) => {
                        let mut ts: TokenStream = s
                            .parse()
                            .map_err(|_| Error::new(*sp, "expected a Rust path"))?;
                        ts = respan(ts, *sp);
                        ts
                    }
                    (None, Name::Ident(i)) => ident_ts(i),
                    (None, Name::Op(o)) => {
                        return Err(Error::new(
                            o.span(),
                            "operator externals need a Rust path: `= \"path\"`",
                        ));
                    }
                };
                let shape = ty.shape();
                match name {
                    Name::Ident(i) => self.bind(ident_str(i), Bind::External(path, shape)),
                    Name::Op(o) => self.bind(
                        format!("op:{}", o.text),
                        Bind::Op(path, shape.unwrap_or(vec![2])),
                    ),
                }
                self.block_items(body, tail, stmts)
            }
            Expr::Open { module, body, .. } => {
                // `open m` is a glob import for the rest of the block.
                let m = path_tokens(module);
                stmts.push(quote_spanned!(module.span()=> use #m::*;));
                self.block_items(body, tail, stmts)
            }
            other => self.expr(other, tail),
        }
    }

    fn let_bindings(&mut self, bindings: &[Binding], stmts: &mut Vec<TokenStream>) -> R<()> {
        let mut parts = Vec::new();
        for b in bindings {
            parts.push(self.binding(b)?);
        }
        if parts.len() == 1 {
            let p = parts.pop().unwrap();
            let (lhs, ty, value, els) = (&p.lhs, &p.ty, &p.value, &p.els);
            stmts.push(quote!(let #lhs #ty = #value #els;));
            for (n, b) in p.binds {
                self.bind(n, b);
            }
            return Ok(());
        }
        // `let a = .. and b = ..`: every value is evaluated before any name is bound.
        if self.editor {
            // As `let (a, b) = (.., ..);`, so that no generated name is in
            // scope in a value: rust-analyzer would offer it there.
            let lhs: Vec<&TokenStream> = parts.iter().map(|p| &p.lhs).collect();
            let values: Vec<&TokenStream> = parts.iter().map(|p| &p.value).collect();
            let ty = if parts.iter().all(|p| p.ty.is_empty()) {
                quote!()
            } else {
                let tys = parts.iter().map(|p| {
                    if p.ty.is_empty() {
                        quote!(_)
                    } else {
                        strip_colon(p.ty.clone())
                    }
                });
                quote!(: (#(#tys),*))
            };
            let els = parts
                .iter()
                .find(|p| !p.els.is_empty())
                .map_or_else(TokenStream::new, |p| p.els.clone());
            stmts.push(quote!(let (#(#lhs),*) #ty = (#(#values),*) #els;));
            for p in parts {
                for (n, b) in p.binds {
                    self.bind(n, b);
                }
            }
            return Ok(());
        }
        let temps: Vec<Ident> = parts
            .iter()
            .map(|p| {
                let sp = first_span(&p.value).unwrap_or_else(Span::call_site);
                self.numbered_at("b", "value", sp)
            })
            .collect();
        for (t, p) in temps.iter().zip(&parts) {
            let v = &p.value;
            stmts.push(quote!(let #t = #v;));
        }
        for (t, p) in temps.iter().zip(parts) {
            let (lhs, ty, els) = (&p.lhs, &p.ty, &p.els);
            stmts.push(quote!(let #lhs #ty = #t #els;));
            for (n, b) in p.binds {
                self.bind(n, b);
            }
        }
        Ok(())
    }

    fn binding(&mut self, b: &Binding) -> R<Parts> {
        match &b.kind {
            BindKind::Func {
                name,
                params,
                mutable,
            } => {
                let len = self.env.len();
                let ps = self.closure_params(params)?;
                for p in params {
                    self.bind_pat(p);
                }
                let body = self.expr(&b.value, true);
                self.env.truncate(len);
                let body = body?;
                let mut shape = vec![Self::arity(params)];
                if let Some(s) = &body.shape {
                    shape.extend(s);
                }
                let bt = body.ts;
                let bars = closure_bars(&ps, b.span);
                let value = match &b.ty {
                    Some(t) if !t.is_arrow() => {
                        let t = t.emit(Ctx::Local, &mut Vec::new());
                        let bt = braced(bt);
                        quote!(#bars -> #t #bt)
                    }
                    _ => quote!(#bars #bt),
                };
                let (lhs, key, bind) = match name {
                    Name::Ident(i) => {
                        let m = mutable.then(|| quote!(mut));
                        let i2 = rust_ident(i);
                        (quote!(#m #i2), ident_str(i), Bind::Local(Some(shape)))
                    }
                    Name::Op(o) => {
                        self.unmapped.extend(&o.spans);
                        // `mut`, for an operator that mutates what it
                        // captures: the user cannot write it.
                        let f = self.numbered_at("op", "operator", o.span());
                        (
                            quote!(mut #f),
                            format!("op:{}", o.text),
                            Bind::Op(quote!(#f), shape),
                        )
                    }
                };
                Ok(Parts {
                    lhs,
                    ty: quote!(),
                    value,
                    els: quote!(),
                    binds: vec![(key, bind)],
                })
            }
            BindKind::Value { pat, mutable } => {
                let (pat, ann) = match (pat, &b.ty) {
                    (Pat::Constraint(p, t), None) => (&**p, Some(t)),
                    (Pat::Paren(inner), None) => match &**inner {
                        Pat::Constraint(p, t) => (&**p, Some(t)),
                        _ => (pat, None),
                    },
                    (p, t) => (p, t.as_ref()),
                };
                // let r = ref e  ->  let mut r = e  (also `ref(e)`)
                if let (Pat::Var(x), Expr::App(f, args) | Expr::Call { func: f, args, .. }) =
                    (pat.strip(), b.value.strip())
                    && let (Expr::Var(r), [v]) = (f.strip(), args.as_slice())
                    && r == "ref"
                    && self.lookup("ref").is_none()
                    && !*mutable
                {
                    let v = self.expr(v, false)?.ts;
                    let x2 = rust_ident(x);
                    let ty = self.local_ann(ann);
                    return Ok(Parts {
                        lhs: quote!(mut #x2),
                        ty,
                        value: v,
                        els: quote!(),
                        binds: vec![(ident_str(x), Bind::RefCell)],
                    });
                }
                let out = self.expr(&b.value, false)?;
                let hint = ann.filter(|t| t.is_arrow()).and_then(|t| t.shape());
                let ty = self.local_ann(ann.filter(|t| !t.is_arrow()));
                if *mutable {
                    let Pat::Var(x) = pat else { unreachable!() };
                    let x2 = rust_ident(x);
                    return Ok(Parts {
                        lhs: quote!(mut #x2),
                        ty,
                        value: out.ts,
                        els: quote!(),
                        binds: vec![(ident_str(x), Bind::Local(None))],
                    });
                }
                let value = match pat_class(pat) {
                    Class::Slice => {
                        let v = out.operand();
                        quote!(::rsml::__rt::as_slice(&#v))
                    }
                    Class::Str => {
                        let v = out.operand();
                        quote!(::rsml::__rt::as_str(&#v))
                    }
                    Class::Other => out.ts.clone(),
                };
                let els = if refutable(pat) {
                    quote!(else { ::core::panic!("Match_failure") })
                } else {
                    quote!()
                };
                let binds = match pat.strip() {
                    Pat::Var(x) => vec![(ident_str(x), Bind::Local(out.shape.clone().or(hint)))],
                    p => {
                        let mut names = Vec::new();
                        p.names(&mut names);
                        names
                            .iter()
                            .map(|n| (ident_str(n), Bind::Local(None)))
                            .collect()
                    }
                };
                Ok(Parts {
                    lhs: self.pat(pat)?,
                    ty,
                    value,
                    els,
                    binds,
                })
            }
        }
    }

    fn local_ann(&self, t: Option<&Type>) -> TokenStream {
        match t {
            Some(t) => {
                let t = t.emit(Ctx::Local, &mut Vec::new());
                quote!(: #t)
            }
            None => quote!(),
        }
    }

    /// `let rec` compiles to nested `fn` items, which need full signatures.
    fn let_rec(&mut self, bindings: &[Binding], stmts: &mut Vec<TokenStream>) -> R<()> {
        let mut sigs = Vec::new();
        for b in bindings {
            let sig = self.rec_signature(b)?;
            self.bind(ident_str(&sig.name), Bind::Local(Some(sig.shape.clone())));
            sigs.push(sig);
        }
        for sig in sigs {
            let len = self.env.len();
            let mut generics = Vec::new();
            let mut params = Vec::new();
            for (p, t) in &sig.params {
                let pt = self.pat(p)?;
                let tt = t.emit(Ctx::Sig, &mut generics);
                params.push(quote!(#pt: #tt));
                self.bind_pat(p);
            }
            let ret = sig.ret.emit(Ctx::Sig, &mut generics);
            let body = match &sig.body {
                RecBody::Expr(e) => self.expr(e, true),
                RecBody::Function(arg, arms) => self
                    .match_tokens(&Expr::Tmp(arg.clone(), None), arms, true, sig.name.span())
                    .map(|ts| Out::new(ts, Kind::Compound)),
            };
            self.env.truncate(len);
            let body = braced(body?.ts);
            let name = rust_ident(&sig.name);
            let generics = if generics.is_empty() {
                quote!()
            } else {
                quote!(<#(#generics),*>)
            };
            stmts.push(quote!(fn #name #generics (#(#params),*) -> #ret #body));
        }
        Ok(())
    }

    fn rec_signature(&mut self, b: &Binding) -> R<RecSig> {
        let need_types = |sp: Span| {
            Error::new(
                sp,
                "`let rec` compiles to a Rust `fn`, which needs explicit types: \
                 `let rec f (x : int) : int = ..` or `let rec f : int -> int = function ..`",
            )
        };
        match &b.kind {
            BindKind::Func {
                name: Name::Ident(name),
                params,
                ..
            } => {
                let Some(ret) = b.ty.clone() else {
                    return Err(need_types(name.span()));
                };
                let mut typed = Vec::new();
                if !(params.len() == 1 && is_unit_pat(&params[0])) {
                    for p in params {
                        match p.clone() {
                            Pat::Paren(inner) => match *inner {
                                Pat::Constraint(p, t) => typed.push((*p, t)),
                                _ => return Err(need_types(name.span())),
                            },
                            Pat::Constraint(p, t) => typed.push((*p, t)),
                            _ => return Err(need_types(name.span())),
                        }
                    }
                }
                let mut shape = vec![typed.len()];
                if let Some(s) = ret.shape() {
                    shape.extend(s);
                }
                let (body, more) = self.rec_body(&b.value, None);
                debug_assert!(more.is_empty());
                Ok(RecSig {
                    name: name.clone(),
                    params: typed,
                    ret,
                    body,
                    shape,
                })
            }
            BindKind::Value {
                pat,
                mutable: false,
            } => {
                let constraint = match pat {
                    Pat::Constraint(p, t) => Some((p, t)),
                    Pat::Paren(inner) => match &**inner {
                        Pat::Constraint(p, t) => Some((p, t)),
                        _ => None,
                    },
                    _ => None,
                };
                let (name, ty) = match (pat.strip(), &b.ty, constraint) {
                    (Pat::Var(n), Some(t), _) => (n.clone(), t.clone()),
                    (_, None, Some((p, t))) => match p.strip() {
                        Pat::Var(n) => (n.clone(), t.clone()),
                        _ => return Err(need_types(b.span)),
                    },
                    _ => return Err(need_types(b.span)),
                };
                if !ty.is_arrow() {
                    return Err(Error::new(
                        name.span(),
                        "`let rec` is only supported for functions",
                    ));
                }
                let (tparams, tret) = ty.arrow_parts();
                let unit = tparams.len() == 1 && tparams[0].is_unit();
                let (body, fun_params) = self.rec_body(&b.value, Some(tparams.len()));
                let k = fun_params.len();
                if k == 0 && !matches!(body, RecBody::Function(..)) {
                    return Err(Error::new(
                        name.span(),
                        "expected `fun` or `function` after `let rec f : t =`",
                    ));
                }
                let mut params = Vec::new();
                if !unit {
                    for (p, t) in fun_params.into_iter().zip(tparams.iter()) {
                        params.push((p, (*t).clone()));
                    }
                }
                let n_params = if matches!(body, RecBody::Function(..)) {
                    1
                } else {
                    k
                };
                if let RecBody::Function(arg, _) = &body {
                    params.push((Pat::Var(arg.clone()), tparams[0].clone()));
                }
                let used = n_params.min(tparams.len());
                let ret = rebuild_arrow(&tparams[used..], tret);
                let mut shape = vec![if unit { 0 } else { used }];
                if let Some(s) = ret.shape() {
                    shape.extend(s);
                }
                Ok(RecSig {
                    name,
                    params,
                    ret,
                    body,
                    shape,
                })
            }
            _ => Err(need_types(b.span)),
        }
    }

    /// Splits `fun a b -> e` / `function ..` off the value of a `let rec`.
    fn rec_body(&mut self, value: &Expr, max: Option<usize>) -> (RecBody, Vec<Pat>) {
        match (value.strip(), max) {
            (Expr::Fun { params, body, .. }, Some(n)) if params.len() <= n => {
                (RecBody::Expr((**body).clone()), params.clone())
            }
            (Expr::Function { arms, .. }, Some(_)) => (
                RecBody::Function(self.local("arg", "argument"), arms.clone()),
                Vec::new(),
            ),
            (v, _) => (RecBody::Expr(v.clone()), Vec::new()),
        }
    }

    // ----- pattern matching ----------------------------------------------

    fn match_tokens(&mut self, scrut: &Expr, arms: &[Arm], tail: bool, sp: Span) -> R<TokenStream> {
        let s = self.scrutinee(scrut, arms)?;
        let mut out = Vec::new();
        let mut end = sp;
        for arm in arms {
            let p = self.pat(&arm.pat)?;
            let len = self.env.len();
            self.bind_pat(&arm.pat);
            let guard = match &arm.guard {
                Some(g) => {
                    let g = self.expr(g, false)?.ts;
                    let kw = keyword("if", first_span(&g).unwrap_or(sp));
                    quote!(#kw #g)
                }
                None => quote!(),
            };
            let body = self.expr(&arm.body, tail);
            self.env.truncate(len);
            let body = body?.ts;
            end = last_span(&body).unwrap_or(end);
            let arrow = fat_arrow(&arm.arrow);
            out.push(quote!(#p #guard #arrow #body,));
        }
        let kw = mapped_keyword("match", sp);
        let arms = group(Delimiter::Brace, quote!(#(#out)*), end);
        Ok(quote!(#kw #s #arms))
    }

    fn scrutinee(&mut self, scrut: &Expr, arms: &[Arm]) -> R<TokenStream> {
        let class = combine(arms.iter().map(|a| pat_class(&a.pat)));
        if class != Class::Other {
            return self.convert_scrutinee(scrut, class);
        }
        // `match (xs, ys) with ([], _) -> ..` converts tuple components.
        if let Expr::Tuple(items) = scrut.strip() {
            let mut classes = vec![Class::Other; items.len()];
            for arm in arms {
                tuple_classes(&arm.pat, &mut classes);
            }
            if classes.iter().any(|c| *c != Class::Other) {
                let mut ts = Vec::new();
                for (item, class) in items.iter().zip(classes) {
                    ts.push(self.convert_scrutinee(item, class)?);
                }
                return Ok(group(
                    Delimiter::Parenthesis,
                    quote!(#(#ts),*),
                    span_of(scrut),
                ));
            }
        }
        Ok(self.expr(scrut, false)?.ts)
    }

    fn convert_scrutinee(&mut self, e: &Expr, class: Class) -> R<TokenStream> {
        let o = self.expr(e, false)?;
        let v = o.operand();
        let sp = span_of(e);
        let f = match class {
            Class::Slice => quote!(::rsml::__rt::as_slice),
            Class::Str => quote!(::rsml::__rt::as_str),
            Class::Other => return Ok(o.ts),
        };
        let f = located(f, sp);
        let amp = located(quote!(&), sp);
        let args = group(Delimiter::Parenthesis, quote!(#amp #v), sp);
        Ok(quote!(#f #args))
    }

    fn pat(&mut self, p: &Pat) -> R<TokenStream> {
        Ok(match p {
            Pat::Wild(sp) => quote_spanned!(*sp=> _),
            Pat::Var(i) => ident_ts(i),
            Pat::Mut(i) => {
                let i = rust_ident(i);
                quote!(mut #i)
            }
            Pat::Lit { lit, neg } => {
                let l = lit_tokens(lit);
                if *neg { quote!(-#l) } else { l }
            }
            Pat::Bool(b, sp) => {
                if *b {
                    quote_spanned!(*sp=> true)
                } else {
                    quote_spanned!(*sp=> false)
                }
            }
            Pat::Unit(sp) => quote_spanned!(*sp=> ()),
            Pat::Paren(inner) => {
                let i = self.pat(inner)?;
                if matches!(inner.strip(), Pat::Or(_)) {
                    let sp = first_span(&i).unwrap_or_else(Span::call_site);
                    group(Delimiter::Parenthesis, i, sp)
                } else {
                    i
                }
            }
            Pat::Tuple(ps) => {
                let ps = self.pats(ps)?;
                let sp = ps
                    .first()
                    .and_then(first_span)
                    .unwrap_or_else(Span::call_site);
                group(Delimiter::Parenthesis, quote!(#(#ps),*), sp)
            }
            Pat::List(ps, sp) | Pat::Array(ps, sp) => {
                let ps = self.pats(ps)?;
                quote_spanned!(*sp=> [#(#ps),*])
            }
            Pat::Cons(..) => self.cons_pat(p)?,
            Pat::Ctor(path, None) => path_tokens(path),
            Pat::Ctor(path, Some(arg)) => {
                let pt = path_tokens(path);
                // Located at the constructor, like the pattern.
                let fields = |ts| group(Delimiter::Parenthesis, ts, path.last().span());
                match &**arg {
                    Pat::Wild(_) => {
                        let f = fields(quote!(..));
                        quote!(#pt #f)
                    }
                    Pat::Paren(inner) if !single_field_ctor(&ident_str(path.last())) => {
                        match &**inner {
                            Pat::Tuple(ps) => {
                                let ps = self.pats(ps)?;
                                let f = fields(quote!(#(#ps),*));
                                quote!(#pt #f)
                            }
                            _ => {
                                let f = fields(self.pat(arg)?);
                                quote!(#pt #f)
                            }
                        }
                    }
                    _ => {
                        let f = fields(self.pat(arg)?);
                        quote!(#pt #f)
                    }
                }
            }
            Pat::CtorRecord(path, fields, _) => {
                let pt = path_tokens(path);
                let mut fs = Vec::new();
                for (f, p) in fields {
                    let f2 = rust_ident(f);
                    fs.push(match p {
                        Some(p) => {
                            let p = self.pat(p)?;
                            quote!(#f2: #p)
                        }
                        None => quote!(#f2),
                    });
                }
                let b = group(Delimiter::Brace, quote!(#(#fs,)* ..), path.last().span());
                quote!(#pt #b)
            }
            Pat::Or(ps) => {
                let ps = self.pats(ps)?;
                quote!(#(#ps)|*)
            }
            Pat::Alias(inner, name) => {
                let n = rust_ident(name);
                let i = self.pat(inner)?;
                if matches!(inner.strip(), Pat::Or(_)) {
                    quote!(#n @ (#i))
                } else {
                    quote!(#n @ #i)
                }
            }
            Pat::Range(lo, hi, _) => {
                let lo = self.pat(lo)?;
                let hi = self.pat(hi)?;
                quote!(#lo..=#hi)
            }
            Pat::Constraint(inner, _) => self.pat(inner)?,
            Pat::Ref(inner, sp) => {
                let i = self.pat(inner)?;
                quote_spanned!(*sp=> &#i)
            }
            Pat::Label { .. } => unreachable!("labels are only kept by a surface parse"),
        })
    }

    fn pats(&mut self, ps: &[Pat]) -> R<Vec<TokenStream>> {
        ps.iter().map(|p| self.pat(p)).collect()
    }

    /// `x :: y :: rest` is the slice pattern `[x, y, rest @ ..]`.
    fn cons_pat(&mut self, p: &Pat) -> R<TokenStream> {
        let mut items = Vec::new();
        let mut cur = p;
        let mut span = Span::call_site();
        while let Pat::Cons(h, t, sp) = cur.strip() {
            items.push(self.pat(h)?);
            span = *sp;
            cur = t;
        }
        match cur.strip() {
            Pat::List(ps, _) => items.extend(self.pats(ps)?),
            Pat::Wild(sp) => items.push(quote_spanned!(*sp=> ..)),
            Pat::Var(i) => {
                let i = rust_ident(i);
                items.push(quote!(#i @ ..));
            }
            Pat::Mut(i) => {
                let i = rust_ident(i);
                items.push(quote!(mut #i @ ..));
            }
            Pat::Alias(inner, n) if matches!(inner.strip(), Pat::Wild(_)) => {
                let n = rust_ident(n);
                items.push(quote!(#n @ ..));
            }
            _ => {
                return Err(Error::new(
                    span,
                    "the tail of a `::` pattern must be a name, `_` or a list literal",
                ));
            }
        }
        Ok(group(Delimiter::Bracket, quote!(#(#items),*), span))
    }
}

struct Parts {
    lhs: TokenStream,
    ty: TokenStream,
    value: TokenStream,
    els: TokenStream,
    binds: Vec<(String, Bind)>,
}

enum RecBody {
    Expr(Expr),
    /// `function` arms matched on the given parameter.
    Function(Ident, Vec<Arm>),
}

struct RecSig {
    name: Ident,
    params: Vec<(Pat, Type)>,
    ret: Type,
    body: RecBody,
    shape: Shape,
}

/// `body` with each hole (with its annotation) replaced by `vals[index]`.
fn fill_holes(body: &Expr, vals: &[Expr]) -> Expr {
    let mut out = body.clone();
    out.visit_holes(&mut |e, h| {
        if let Some(i) = h.index {
            *e = vals[i].clone();
        }
    });
    out
}

/// The parameters of a section body that are called (`_ 1`) or the
/// receiver of a method call (`_.push(1)`, `.push _ 1`): a value hoisted for
/// one is `mut` (see [`Cx::hoist_mut`]).
fn called_holes(body: &Expr) -> Vec<usize> {
    fn hole(e: &Expr) -> Option<usize> {
        match e {
            Expr::Hole { index, .. } => *index,
            Expr::Paren(e, _) | Expr::Constraint(e, _) => hole(e),
            _ => None,
        }
    }
    fn walk(e: &mut Expr, out: &mut Vec<usize>) {
        match e {
            // Its holes are its own.
            Expr::Section { .. } => return,
            Expr::Method { recv, .. } => out.extend(hole(recv)),
            Expr::App(h, args) if matches!(h.strip(), Expr::MethodRef { .. }) => {
                out.extend(args.first().and_then(hole))
            }
            Expr::App(h, _) | Expr::Call { func: h, .. } => out.extend(hole(h)),
            _ => {}
        }
        for c in e.children_mut() {
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    walk(&mut body.clone(), &mut out);
    out
}

/// The parameters that the holes of a section body stand for, in order.
fn hole_indices(body: &Expr) -> Vec<usize> {
    let mut out = Vec::new();
    body.clone().visit_holes(&mut |_, h| out.extend(h.index));
    out
}

/// Whether a hole is where names are bound (a `let` body, a `fun`, a match
/// case), so that an argument put in its place could be captured.
fn holes_under_binder(e: &Expr) -> bool {
    fn walk(e: &mut Expr, under: bool) -> bool {
        match e {
            Expr::Hole { .. } => under,
            Expr::Constraint(inner, _) if matches!(**inner, Expr::Hole { .. }) => under,
            Expr::Let {
                rec_,
                bindings,
                body,
                ..
            } => {
                let rec_ = *rec_;
                bindings
                    .iter_mut()
                    .any(|b| walk(&mut b.value, under || rec_))
                    || walk(body, true)
            }
            Expr::Match { scrut, arms, .. } => {
                walk(scrut, under)
                    || arms.iter_mut().any(|a| {
                        a.guard.as_mut().is_some_and(|g| walk(g, true)) || walk(&mut a.body, true)
                    })
            }
            Expr::For { from, to, body, .. } => {
                walk(from, under) || walk(to, under) || walk(body, true)
            }
            Expr::ForIn { iter, body, .. } => walk(iter, under) || walk(body, true),
            Expr::Fun { body, .. } | Expr::External { body, .. } | Expr::Open { body, .. } => {
                walk(body, true)
            }
            Expr::Function { .. } => e.children_mut().into_iter().any(|c| walk(c, true)),
            e => e.children_mut().into_iter().any(|c| walk(c, under)),
        }
    }
    walk(&mut e.clone(), false)
}

fn rebuild_arrow(params: &[&Type], ret: &Type) -> Type {
    match params.split_first() {
        None => ret.clone(),
        Some((p, rest)) => Type::Arrow(Box::new((*p).clone()), Box::new(rebuild_arrow(rest, ret))),
    }
}

fn tuple_classes(p: &Pat, classes: &mut [Class]) {
    match p.strip() {
        Pat::Tuple(ps) if ps.len() == classes.len() => {
            for (c, p) in classes.iter_mut().zip(ps) {
                let pc = pat_class(p);
                if pc == Class::Slice || (pc == Class::Str && *c == Class::Other) {
                    *c = pc;
                }
            }
        }
        Pat::Or(ps) => ps.iter().for_each(|p| tuple_classes(p, classes)),
        Pat::Alias(p, _) => tuple_classes(p, classes),
        _ => {}
    }
}

fn is_unary_section(op: &str) -> bool {
    matches!(op, "~-" | "~-." | "!")
}

fn rust_binop(op: &str) -> Option<&'static str> {
    Some(match op {
        "+" | "+." => "+",
        "-" | "-." => "-",
        "*" | "*." => "*",
        "/" | "/." => "/",
        "mod" | "%" => "%",
        "land" => "&",
        "lor" => "|",
        "lxor" => "^",
        "lsl" => "<<",
        "lsr" | "asr" => ">>",
        "=" | "==" => "==",
        "<>" | "!=" => "!=",
        "<" => "<",
        ">" => ">",
        "<=" => "<=",
        ">=" => ">=",
        "&&" => "&&",
        "||" | "or" => "||",
        _ => return None,
    })
}

fn strip_colon(ty: TokenStream) -> TokenStream {
    ty.into_iter().skip(1).collect()
}

/// The span of the first token of `ts` (a group's whole span).
fn first_span(ts: &TokenStream) -> Option<Span> {
    ts.clone().into_iter().next().map(|t| t.span())
}

/// The span of the last token of `ts` (a group's whole span).
fn last_span(ts: &TokenStream) -> Option<Span> {
    ts.clone().into_iter().last().map(|t| t.span())
}

/// Generated tokens (a path such as `::rsml::__rt::cons`) located at `sp`
/// (see [`at`]).
fn located(ts: TokenStream, sp: Span) -> TokenStream {
    if !LOCATE.with(|l| l.get()) {
        return ts;
    }
    ts.into_iter()
        .map(|t| match t {
            TokenTree::Group(g) => {
                let mut n = Group::new(g.delimiter(), located(g.stream(), sp));
                n.set_span(at(sp));
                TokenTree::Group(n)
            }
            mut t => {
                t.set_span(t.span().located_at(sp));
                t
            }
        })
        .collect()
}

/// The parentheses of a call of `f`, located at the function: an error
/// about the call points at it.
fn call_parens(f: &TokenStream, args: TokenStream) -> TokenStream {
    group(
        Delimiter::Parenthesis,
        args,
        first_span(f).unwrap_or_else(Span::call_site),
    )
}

/// A group whose delimiters are located at `sp`, a span from the source.
fn group(delim: Delimiter, ts: TokenStream, sp: Span) -> TokenStream {
    let mut g = Group::new(delim, ts);
    g.set_span(at(sp));
    TokenStream::from(TokenTree::Group(g))
}

thread_local! {
    /// Whether generated code is located in the source (see [`at`]).
    static LOCATE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    /// Whether expansions get editor markers (only tests turn them off).
    static MARKERS: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Runs `f` with expansions that leave out the editor markers
/// (`operator_marker`, `keyword_marker`), for tests that compare
/// expansions with hand-written Rust (and `scripts/check_docs.py`, which
/// compares them with the documentation).
pub fn without_markers<T>(f: impl FnOnce() -> T) -> T {
    let before = MARKERS.with(|m| m.replace(false));
    let out = f();
    MARKERS.with(|m| m.set(before));
    out
}

/// A span for generated code located at `sp` in the source, so that
/// rustc's errors about it point there (lints still see code produced by
/// the macro). Not for an editor: rust-analyzer colors and resolves a
/// source token by the expansion's tokens at its location, and a
/// generated `(` or temporary there would compete with the token itself.
fn at(sp: Span) -> Span {
    if LOCATE.with(|l| l.get()) {
        Span::call_site().located_at(sp)
    } else {
        Span::call_site()
    }
}

/// A span for a generated keyword or operator that stands for the source
/// token at `sp` (`match` for `function`, `=>` for `->`): editors color
/// that token like this one.
fn mapped(sp: Span) -> Span {
    Span::call_site().located_at(sp)
}

/// `=>`, standing for the `->` with these spans (see [`mapped`]).
fn fat_arrow(arrow: &[Span]) -> TokenStream {
    let mut eq = proc_macro2::Punct::new('=', proc_macro2::Spacing::Joint);
    let mut gt = proc_macro2::Punct::new('>', proc_macro2::Spacing::Alone);
    if let [a, b] = arrow {
        eq.set_span(mapped(*a));
        gt.set_span(mapped(*b));
    }
    quote!(#eq #gt)
}

/// A keyword standing for the source token at `sp` (see [`mapped`]).
fn mapped_keyword(kw: &str, sp: Span) -> Ident {
    Ident::new(kw, mapped(sp))
}

fn respan(ts: TokenStream, sp: Span) -> TokenStream {
    ts.into_iter()
        .map(|mut t| {
            t.set_span(sp);
            t
        })
        .collect()
}

fn span_of(e: &Expr) -> Span {
    match e {
        Expr::Var(i) | Expr::Tmp(i, _) => i.span(),
        Expr::Path(p) => p.span(),
        Expr::Lit(l) => l.span(),
        Expr::Bool(_, s) | Expr::Unit(s) | Expr::List(_, s) | Expr::Array(_, s) => *s,
        Expr::Paren(e, _) => span_of(e),
        Expr::App(h, _) | Expr::Call { func: h, .. } => span_of(h),
        Expr::Index(e, _, _) => span_of(e),
        Expr::Range { op, .. } => op.span(),
        Expr::Jump { span, .. } | Expr::RustBlock { span, .. } | Expr::ForIn { span, .. } => *span,
        Expr::Fun { span, .. }
        | Expr::Function { span, .. }
        | Expr::Compose { span, .. }
        | Expr::Section { span, .. }
        | Expr::MethodRef { span, .. }
        | Expr::FieldRef { span, .. }
        | Expr::Hole { span, .. } => *span,
        Expr::Binary { op, .. } | Expr::OpSection(op) => op.span(),
        Expr::Field(_, i) | Expr::Method { name: i, .. } => i.span(),
        _ => Span::call_site(),
    }
}

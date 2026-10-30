//! Syntax tree for the OCaml subset accepted by `ml!`.

use proc_macro2::{Group, Ident, Literal, Span, TokenStream};

use crate::lexer::Op;
use crate::types::Type;

/// How a known function consumes arguments, as a list of Rust call groups.
///
/// `let add x y = ..` has shape `[2]`: one Rust call taking two arguments.
/// `let adder x = fun y -> ..` has shape `[1, 1]`: a call returning a closure.
/// A group of `0` is a function taking `()`, compiled to a zero-argument call.
pub type Shape = Vec<usize>;

/// The labels a call may give a known function's arguments by (`f ~x:1`),
/// one per parameter of its first call group: the names of OCaml's `~x`
/// parameters, or of every parameter of a Rust signature in `rsml!`.
pub type Labels = Option<std::rc::Rc<[Option<String>]>>;

#[derive(Clone)]
pub struct PathSeg {
    pub ident: Ident,
    /// Rust generic arguments, `<...>` included (from `Vec::<i32>::new`).
    pub generics: Option<TokenStream>,
}

#[derive(Clone)]
pub struct Path {
    pub leading_colon: bool,
    /// Written OCaml style (`Color.Red`) rather than Rust style (`Vec::new`).
    pub dotted: bool,
    pub segs: Vec<PathSeg>,
}

impl Path {
    pub fn last(&self) -> &Ident {
        &self.segs.last().unwrap().ident
    }

    pub fn span(&self) -> Span {
        self.segs[0].ident.span()
    }
}

#[derive(Clone)]
pub enum Expr {
    Var(Ident),
    /// A hygienic temporary introduced by codegen, with the shape of its value.
    Tmp(Ident, Option<Shape>),
    Path(Path),
    Lit(Literal),
    Bool(bool, Span),
    Unit(Span),
    /// `(e)`, with the span of the parentheses (or of `begin`).
    Paren(Box<Expr>, Span),
    Tuple(Vec<Expr>),
    List(Vec<Expr>, Span),
    Array(Vec<Expr>, Span),
    App(Box<Expr>, Vec<Expr>),
    /// `f(a, b)`, with `(` written right after the function: a Rust-style
    /// call, each item one argument. `f(a)(b)` calls the result of `f(a)`.
    Call {
        func: Box<Expr>,
        /// The items. A macro parse (not a [surface](crate::parser::Parser::set_surface)
        /// one) makes each one an argument as written OCaml style: `f()` has
        /// the single argument `()`, as `f ()` has, and an item `()` is
        /// `(())`.
        args: Vec<Expr>,
        /// Written with a `,` after the last item.
        trailing_comma: bool,
        /// The parentheses.
        span: Span,
    },
    /// `fun p.. -> body`; `fun move p.. -> body` forces a `move` closure.
    Fun {
        params: Vec<Pat>,
        /// As in [`BindKind::Func`]: kept for `let f = fun ~x -> ..`.
        labels: Vec<Option<Ident>>,
        body: Box<Expr>,
        span: Span,
        is_move: bool,
    },
    Function {
        arms: Vec<Arm>,
        span: Span,
    },
    // The `span` of keyword-led expressions is the keyword's.
    Let {
        rec_: bool,
        bindings: Vec<Binding>,
        body: Box<Expr>,
        span: Span,
    },
    External {
        name: Name,
        ty: Type,
        rust: Option<(String, Span)>,
        body: Box<Expr>,
        span: Span,
    },
    /// `open path`: a Rust glob import (`use path::*;`) for `body`.
    Open {
        module: Path,
        body: Box<Expr>,
        span: Span,
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        els: Option<Box<Expr>>,
        span: Span,
    },
    Match {
        scrut: Box<Expr>,
        arms: Vec<Arm>,
        span: Span,
    },
    Seq(Box<Expr>, Box<Expr>),
    Binary {
        op: Op,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `first >> second`, i.e. `fun x -> second (first x)`.
    Compose {
        first: Box<Expr>,
        second: Box<Expr>,
        span: Span,
    },
    /// `-e` or `-.e`
    Neg(Box<Expr>, Op),
    Deref(Box<Expr>, Span),
    Borrow {
        mutable: bool,
        expr: Box<Expr>,
        span: Span,
    },
    Field(Box<Expr>, Ident),
    TupleIndex(Box<Expr>, Vec<(u32, Span)>),
    Method {
        recv: Box<Expr>,
        name: Ident,
        turbofish: Option<TokenStream>,
        args: Vec<Expr>,
        /// Written with a `,` after the last argument.
        trailing_comma: bool,
    },
    /// `a.(i)`, or `a[i]` written Rust style (`true`).
    Index(Box<Expr>, Box<Expr>, bool),
    StrIndex(Box<Expr>, Box<Expr>),
    /// `a..b`, `a..=b`, `a..`, `..b` or `..`: a Rust range.
    Range {
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        /// `..` or `..=`
        op: Op,
    },
    /// `return e`, `break e` (the value is optional) or `continue`.
    Jump {
        kind: Jump,
        value: Option<Box<Expr>>,
        /// The keyword.
        span: Span,
    },
    /// `unsafe { e }`, `async { e }` or `async move { e }`, with OCaml
    /// inside (`None` for `{}`).
    RustBlock {
        kind: RustBlock,
        body: Option<Box<Expr>>,
        /// The (first) keyword.
        span: Span,
        /// The braces.
        brace: Span,
    },
    /// `place <- value`
    Assign(Box<Expr>, Box<Expr>, Span),
    /// `r := value`
    RefAssign(Box<Expr>, Box<Expr>, Span),
    Try(Box<Expr>, Span),
    Await(Box<Expr>, Span),
    OpSection(Op),
    Constraint(Box<Expr>, Type),
    /// `e as t` (Rust extension)
    Cast(Box<Expr>, Type),
    Record {
        ctor: Path,
        fields: Vec<(Ident, Option<Expr>)>,
        /// Written with a `;` after the last field.
        trailing_semi: bool,
    },
    RecordUpdate {
        base: Box<Expr>,
        fields: Vec<(Ident, Expr)>,
        span: Span,
    },
    For {
        var: Ident,
        from: Box<Expr>,
        to: Box<Expr>,
        dir: ForDir,
        body: Box<Expr>,
        span: Span,
    },
    /// `for pat in iter do body done`
    ForIn {
        pat: Pat,
        iter: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    While {
        cond: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Macro {
        path: Path,
        bang: Span,
        group: Group,
    },
    /// A std macro whose arguments are OCaml: `println!("{}", f x)`,
    /// `assert_eq!(add 1 2, 3)`, `vec![a; n]`. Every other macro keeps Rust
    /// arguments ([`Expr::Macro`]).
    StdMacro {
        path: Path,
        bang: Span,
        /// The delimiters, as written (the tokens are parsed into `args`).
        group: Group,
        args: Vec<MacroArg>,
        /// `vec![e; n]`: two arguments separated by `;`.
        repeat: bool,
        /// Written with a `,` after the last argument.
        trailing_comma: bool,
    },
    /// An already-parsed Rust expression substituted by `macro_rules!`
    /// (`$e:expr` arrives as an invisible group).
    Rust(Group),
    Assert(Box<Expr>, Span),
    /// `.name`: a method as a function, `.push xs 1` being `xs.push(1)`.
    /// With Rust-style arguments, `.push(1)` is the method with those
    /// arguments, waiting for its receiver: `.push(1) xs` is `xs.push(1)`.
    MethodRef {
        name: Ident,
        turbofish: Option<TokenStream>,
        args: Option<Vec<Expr>>,
        /// The `.`
        span: Span,
    },
    /// `.0` (or `.0.1`): a tuple field as a function, `.0 t` being `t.0`.
    FieldRef {
        idx: Vec<(u32, Span)>,
        /// The `.`
        span: Span,
    },
    /// `_` or `_2`: a parameter of the enclosing [`Expr::Section`].
    Hole {
        /// The number written, or `None` for `_` (the lowest one not written).
        written: Option<usize>,
        /// The parameter it stands for, set when its section is formed.
        index: Option<usize>,
        span: Span,
    },
    /// An expression with holes, which is a function of them: `f _ 1` is
    /// `fun a -> f a 1`, `g _1 _0` is `fun a b -> g b a`.
    Section {
        body: Box<Expr>,
        /// One per parameter: its type, if a hole was annotated (`(_ : i64)`).
        params: Vec<Option<Type>>,
        span: Span,
    },
    /// `~x:e`, or `~x` for `~x:x`: an argument given by label, matched with
    /// the callee's parameter of that name. (A surface parse keeps
    /// [`Surface::Label`] instead.)
    Labeled {
        label: Ident,
        value: Box<Expr>,
        span: Span,
    },
    /// A `let rec` function's call of itself whose value is the function's
    /// (made by the code generator): a jump back to its start with these
    /// arguments, each with the index of its parameter, in the order
    /// written.
    TailCall {
        args: Vec<(usize, Expr)>,
        span: Span,
    },
    /// Only produced by a [surface](crate::parser::Parser::set_surface) parse.
    Surface(Box<Surface>),
}

/// An argument of an [`Expr::StdMacro`].
#[derive(Clone)]
pub enum MacroArg {
    Expr(Expr),
    /// `name = e` after a format string: a named format argument.
    Named(Ident, Expr),
}

impl MacroArg {
    pub fn expr(&self) -> &Expr {
        match self {
            MacroArg::Expr(e) | MacroArg::Named(_, e) => e,
        }
    }

    pub fn expr_mut(&mut self) -> &mut Expr {
        match self {
            MacroArg::Expr(e) | MacroArg::Named(_, e) => e,
        }
    }
}

/// How `for i = a .. b` counts: `to` and `downto` include `b`, as in
/// OCaml; `til` leaves it out, as Rust's `a..b` does.
#[derive(Clone, Copy, PartialEq)]
pub enum ForDir {
    To,
    Downto,
    Til,
}

impl ForDir {
    pub fn keyword(self) -> &'static str {
        match self {
            ForDir::To => "to",
            ForDir::Downto => "downto",
            ForDir::Til => "til",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Jump {
    Return,
    Break,
    Continue,
}

impl Jump {
    pub fn keyword(self) -> &'static str {
        match self {
            Jump::Return => "return",
            Jump::Break => "break",
            Jump::Continue => "continue",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RustBlock {
    Unsafe,
    Async,
    AsyncMove,
}

impl RustBlock {
    pub fn keywords(self) -> &'static str {
        match self {
            RustBlock::Unsafe => "unsafe",
            RustBlock::Async => "async",
            RustBlock::AsyncMove => "async move",
        }
    }
}

/// A hole as seen by [`Expr::visit_holes`].
#[derive(Clone)]
pub struct HoleInfo {
    pub written: Option<usize>,
    pub index: Option<usize>,
    pub span: Span,
    pub ty: Option<Type>,
}

/// Syntax that the macro parser desugars on the spot but the formatter must
/// reproduce. A surface parse also keeps `|>`, `@@`, `>>` and `<<` as
/// [`Expr::Binary`], and returns toplevel [`Phrase`]s.
#[derive(Clone)]
pub enum Surface {
    /// `begin e end`, or `begin end` (`body` is `None`).
    Begin {
        body: Option<Expr>,
        begin: Span,
        end: Span,
    },
    /// `~label:e`, `~label` (`value` is `None`) or `~(e)` (`label` is `None`).
    Label {
        label: Option<Ident>,
        value: Option<Expr>,
        span: Span,
    },
}

/// A toplevel phrase of a surface parse.
#[derive(Clone)]
pub enum Phrase {
    /// `let` without `in`
    Let {
        rec_: bool,
        bindings: Vec<Binding>,
        span: Span,
    },
    External {
        name: Name,
        ty: Type,
        rust: Option<(String, Span)>,
        span: Span,
    },
    Open {
        module: Path,
        span: Span,
    },
    Expr(Expr),
}

#[derive(Clone)]
pub enum Name {
    Ident(Ident),
    Op(Op),
}

#[derive(Clone)]
pub enum BindKind {
    Value {
        pat: Pat,
        mutable: bool,
    },
    /// `let f x = ..`, or `let mut f x = ..` for a closure that mutates
    /// what it captures (Rust's `FnMut`, called through a `mut` binding).
    Func {
        name: Name,
        params: Vec<Pat>,
        /// The label of each parameter (`~x`, `~x:p`, `~(x : t)`); empty
        /// in a surface parse, which keeps [`Pat::Label`] in `params`.
        labels: Vec<Option<Ident>>,
        mutable: bool,
    },
}

#[derive(Clone)]
pub struct Binding {
    pub kind: BindKind,
    /// Return type for functions, type annotation for values.
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone)]
pub struct Arm {
    pub pat: Pat,
    pub guard: Option<Expr>,
    pub body: Expr,
    /// The spans of the `->` (none in unfinished code).
    pub arrow: Vec<Span>,
}

#[derive(Clone)]
pub enum Pat {
    Wild(Span),
    Var(Ident),
    /// `mut x` (Rust extension)
    Mut(Ident),
    Lit {
        lit: Literal,
        neg: bool,
    },
    Bool(bool, Span),
    Unit(Span),
    Paren(Box<Pat>),
    Tuple(Vec<Pat>),
    List(Vec<Pat>, Span),
    Array(Vec<Pat>, Span),
    Cons(Box<Pat>, Box<Pat>, Span),
    Ctor(Path, Option<Box<Pat>>),
    /// `P { x; y = p }`; the flag records a trailing `; _`.
    CtorRecord(Path, Vec<(Ident, Option<Pat>)>, bool),
    Or(Vec<Pat>),
    Alias(Box<Pat>, Ident),
    /// `'a'..'z'`, Rust's exclusive range, or `1..=3`, inclusive (`true`).
    Range(Box<Pat>, Box<Pat>, bool),
    Constraint(Box<Pat>, Type),
    /// `&p` (Rust extension)
    Ref(Box<Pat>, Span),
    /// A labelled parameter, like [`Surface::Label`]. Surface parse only.
    Label {
        label: Option<Ident>,
        pat: Option<Box<Pat>>,
        span: Span,
    },
}

impl Pat {
    /// Names bound by the pattern.
    pub fn names(&self, out: &mut Vec<Ident>) {
        match self {
            Pat::Var(i) | Pat::Mut(i) => out.push(i.clone()),
            Pat::Wild(_) | Pat::Lit { .. } | Pat::Bool(..) | Pat::Unit(_) => {}
            Pat::Paren(p) | Pat::Constraint(p, _) | Pat::Ref(p, _) => p.names(out),
            Pat::Tuple(ps) | Pat::List(ps, _) | Pat::Array(ps, _) => {
                ps.iter().for_each(|p| p.names(out))
            }
            Pat::Or(ps) => {
                if let Some(p) = ps.first() {
                    p.names(out)
                }
            }
            Pat::Cons(h, t, _) => {
                h.names(out);
                t.names(out);
            }
            Pat::Ctor(_, arg) => {
                if let Some(p) = arg {
                    p.names(out)
                }
            }
            Pat::CtorRecord(_, fields, _) => {
                for (f, p) in fields {
                    match p {
                        Some(p) => p.names(out),
                        None => out.push(f.clone()),
                    }
                }
            }
            Pat::Alias(p, i) => {
                p.names(out);
                out.push(i.clone());
            }
            Pat::Range(..) => {}
            Pat::Label { label, pat, .. } => match (pat, label) {
                (Some(p), _) => p.names(out),
                (None, Some(l)) => out.push(l.clone()),
                (None, None) => {}
            },
        }
    }

    pub fn strip(&self) -> &Pat {
        match self {
            Pat::Paren(p) | Pat::Constraint(p, _) => p.strip(),
            p => p,
        }
    }
}

impl Expr {
    /// Looks through grouping parentheses.
    pub fn strip(&self) -> &Expr {
        match self {
            Expr::Paren(e, _) => e.strip(),
            e => e,
        }
    }

    /// Whether the expression assigns to or mutably borrows anything. A
    /// closure that does must not be made `move`: it would silently mutate
    /// its own copy of a captured `Copy` variable.
    pub fn mutates(&self) -> bool {
        let any = |es: &[Expr]| es.iter().any(Expr::mutates);
        match self {
            Expr::Assign(..) | Expr::RefAssign(..) => true,
            Expr::Borrow { mutable: true, .. } => true,
            Expr::App(h, args) | Expr::Call { func: h, args, .. } => {
                matches!(h.strip(), Expr::Var(i) if i == "incr" || i == "decr")
                    || h.mutates()
                    || any(args)
            }
            Expr::Var(_)
            | Expr::Tmp(..)
            | Expr::Path(_)
            | Expr::Lit(_)
            | Expr::Bool(..)
            | Expr::Unit(_) => false,
            Expr::OpSection(_)
            | Expr::Macro { .. }
            | Expr::Rust(_)
            | Expr::Hole { .. }
            | Expr::FieldRef { .. } => false,
            Expr::MethodRef { args, .. } => args.as_deref().is_some_and(any),
            Expr::StdMacro { args, .. } => args.iter().any(|a| a.expr().mutates()),
            Expr::TailCall { .. } => true,
            Expr::Section { body, .. } => body.mutates(),
            Expr::Range { lo, hi, .. } => {
                lo.as_ref().is_some_and(|e| e.mutates()) || hi.as_ref().is_some_and(|e| e.mutates())
            }
            Expr::Jump { value: e, .. } | Expr::RustBlock { body: e, .. } => {
                e.as_ref().is_some_and(|e| e.mutates())
            }
            Expr::Paren(e, _)
            | Expr::Neg(e, _)
            | Expr::Deref(e, _)
            | Expr::Borrow { expr: e, .. }
            | Expr::Field(e, _)
            | Expr::TupleIndex(e, _)
            | Expr::Try(e, _)
            | Expr::Await(e, _)
            | Expr::Constraint(e, _)
            | Expr::Cast(e, _)
            | Expr::Assert(e, _)
            | Expr::Labeled { value: e, .. }
            | Expr::Fun { body: e, .. }
            | Expr::External { body: e, .. }
            | Expr::Open { body: e, .. } => e.mutates(),
            Expr::Tuple(es) | Expr::List(es, _) | Expr::Array(es, _) => any(es),
            Expr::Function { arms, .. } => arms_mutate(arms),
            Expr::Match { scrut, arms, .. } => scrut.mutates() || arms_mutate(arms),
            Expr::Let { bindings, body, .. } => {
                body.mutates() || bindings.iter().any(|b| b.value.mutates())
            }
            Expr::If {
                cond, then, els, ..
            } => cond.mutates() || then.mutates() || els.as_ref().is_some_and(|e| e.mutates()),
            Expr::Seq(a, b)
            | Expr::Binary { lhs: a, rhs: b, .. }
            | Expr::Compose {
                first: a,
                second: b,
                ..
            }
            | Expr::Index(a, b, _)
            | Expr::StrIndex(a, b)
            | Expr::While {
                cond: a, body: b, ..
            }
            | Expr::ForIn {
                iter: a, body: b, ..
            } => a.mutates() || b.mutates(),
            Expr::For { from, to, body, .. } => from.mutates() || to.mutates() || body.mutates(),
            Expr::Method { recv, args, .. } => recv.mutates() || any(args),
            Expr::Record { fields, .. } => fields
                .iter()
                .any(|(_, v)| v.as_ref().is_some_and(Expr::mutates)),
            Expr::RecordUpdate { base, fields, .. } => {
                base.mutates() || fields.iter().any(|(_, v)| v.mutates())
            }
            Expr::Surface(s) => match &**s {
                Surface::Begin { body, .. } => body.as_ref().is_some_and(Expr::mutates),
                Surface::Label { value, .. } => value.as_ref().is_some_and(Expr::mutates),
            },
        }
    }
}

fn arms_mutate(arms: &[Arm]) -> bool {
    arms.iter()
        .any(|a| a.body.mutates() || a.guard.as_ref().is_some_and(Expr::mutates))
}

impl Expr {
    /// The direct subexpressions, in source order. Sections are opaque:
    /// their holes are theirs.
    pub fn children_mut(&mut self) -> Vec<&mut Expr> {
        fn arms(arms: &mut [Arm]) -> Vec<&mut Expr> {
            let mut out = Vec::new();
            for a in arms {
                if let Some(g) = &mut a.guard {
                    out.push(g);
                }
                out.push(&mut a.body);
            }
            out
        }
        match self {
            Expr::Var(_)
            | Expr::Tmp(..)
            | Expr::Path(_)
            | Expr::Lit(_)
            | Expr::Bool(..)
            | Expr::Unit(_)
            | Expr::OpSection(_)
            | Expr::Macro { .. }
            | Expr::Rust(_)
            | Expr::Hole { .. }
            | Expr::FieldRef { .. }
            | Expr::Section { .. } => Vec::new(),
            Expr::MethodRef { args, .. } => args.iter_mut().flatten().collect(),
            Expr::StdMacro { args, .. } => args.iter_mut().map(MacroArg::expr_mut).collect(),
            Expr::TailCall { args, .. } => args.iter_mut().map(|(_, e)| e).collect(),
            Expr::Range { lo, hi, .. } => lo
                .iter_mut()
                .chain(hi.iter_mut())
                .map(|e| &mut **e)
                .collect(),
            Expr::Jump { value: e, .. } | Expr::RustBlock { body: e, .. } => {
                e.iter_mut().map(|e| &mut **e).collect()
            }
            Expr::Paren(e, _)
            | Expr::Neg(e, _)
            | Expr::Deref(e, _)
            | Expr::Borrow { expr: e, .. }
            | Expr::Field(e, _)
            | Expr::TupleIndex(e, _)
            | Expr::Try(e, _)
            | Expr::Await(e, _)
            | Expr::Constraint(e, _)
            | Expr::Cast(e, _)
            | Expr::Assert(e, _)
            | Expr::Labeled { value: e, .. }
            | Expr::Fun { body: e, .. }
            | Expr::External { body: e, .. }
            | Expr::Open { body: e, .. } => vec![&mut **e],
            Expr::Tuple(es) | Expr::List(es, _) | Expr::Array(es, _) => es.iter_mut().collect(),
            Expr::App(h, args) | Expr::Call { func: h, args, .. } => {
                std::iter::once(&mut **h).chain(args.iter_mut()).collect()
            }
            Expr::Function { arms: a, .. } => arms(a),
            Expr::Match { scrut, arms: a, .. } => {
                let mut out = vec![&mut **scrut];
                out.extend(arms(a));
                out
            }
            Expr::Let { bindings, body, .. } => {
                let mut out: Vec<&mut Expr> = bindings.iter_mut().map(|b| &mut b.value).collect();
                out.push(body);
                out
            }
            Expr::If {
                cond, then, els, ..
            } => {
                let mut out = vec![&mut **cond, &mut **then];
                if let Some(e) = els {
                    out.push(e);
                }
                out
            }
            Expr::Seq(a, b)
            | Expr::Binary { lhs: a, rhs: b, .. }
            | Expr::Compose {
                first: a,
                second: b,
                ..
            }
            | Expr::Index(a, b, _)
            | Expr::StrIndex(a, b)
            | Expr::Assign(a, b, _)
            | Expr::RefAssign(a, b, _)
            | Expr::While {
                cond: a, body: b, ..
            }
            | Expr::ForIn {
                iter: a, body: b, ..
            } => vec![&mut **a, &mut **b],
            Expr::For { from, to, body, .. } => vec![&mut **from, &mut **to, &mut **body],
            Expr::Method { recv, args, .. } => std::iter::once(&mut **recv)
                .chain(args.iter_mut())
                .collect(),
            Expr::Record { fields, .. } => {
                fields.iter_mut().filter_map(|(_, v)| v.as_mut()).collect()
            }
            Expr::RecordUpdate { base, fields, .. } => std::iter::once(&mut **base)
                .chain(fields.iter_mut().map(|(_, v)| v))
                .collect(),
            Expr::Surface(s) => match &mut **s {
                Surface::Begin { body, .. } => body.iter_mut().collect(),
                Surface::Label { value, .. } => value.iter_mut().collect(),
            },
        }
    }

    /// Calls `f` on every hole that is not inside a section, in source
    /// order. `f` gets the hole's expression, including its type annotation
    /// (`(_ : t)`) if it has one, so that it can replace it.
    pub fn visit_holes(&mut self, f: &mut dyn FnMut(&mut Expr, HoleInfo)) {
        let info = match self {
            Expr::Constraint(inner, ty) => match **inner {
                Expr::Hole {
                    written,
                    index,
                    span,
                } => Some(HoleInfo {
                    written,
                    index,
                    span,
                    ty: Some(ty.clone()),
                }),
                _ => None,
            },
            Expr::Hole {
                written,
                index,
                span,
            } => Some(HoleInfo {
                written: *written,
                index: *index,
                span: *span,
                ty: None,
            }),
            _ => None,
        };
        match info {
            Some(info) => f(self, info),
            None => {
                for c in self.children_mut() {
                    c.visit_holes(f);
                }
            }
        }
    }

    /// Whether the expression contains a hole outside any section.
    pub fn has_hole(&self) -> bool {
        let mut found = false;
        self.clone().visit_holes(&mut |_, _| found = true);
        found
    }

    /// Sets the parameter of a hole (or annotated hole) found by
    /// [`Expr::visit_holes`].
    pub fn set_hole_index(&mut self, i: usize) {
        match self {
            Expr::Hole { index, .. } => *index = Some(i),
            Expr::Constraint(inner, _) => inner.set_hole_index(i),
            _ => {}
        }
    }

    /// A hole alone, perhaps in parentheses or annotated: it belongs to
    /// the expression around it rather than forming a section.
    pub fn is_bare_hole(&self) -> bool {
        match self {
            Expr::Hole { .. } => true,
            Expr::Paren(e, _) => e.is_bare_hole(),
            Expr::Constraint(e, _) => matches!(**e, Expr::Hole { .. }),
            _ => false,
        }
    }
}

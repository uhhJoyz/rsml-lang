//! Type expressions and their Rust translation.
//!
//! Types appear in annotations (`(x : i64)`, `let f (x : i64) : i64 = ..`),
//! in `external` declarations and in `(e : t)` constraints. Names are Rust's
//! (`i64`, `Vec<String>`, `&str`); the syntax adds OCaml's arrows
//! (`i64 -> i64`), tuples (`a * b`) and type variables (`'a`).

use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};

use crate::ast::Shape;
use crate::lexer::ident_str;

#[derive(Clone)]
pub enum Type {
    Arrow(Box<Type>, Box<Type>),
    /// `a * b`
    Tuple(Vec<Type>),
    /// `(a, b)`
    ParenTuple(Vec<Type>),
    /// `'a`, a type variable: a generic parameter of a `let rec` function.
    Var(Ident),
    Paren(Box<Type>),
    Infer(Span),
    Unit(Span),
    Ref {
        lifetime: Option<Ident>,
        mutable: bool,
        inner: Box<Type>,
        span: Span,
    },
    /// `[t]` or `[t; N]`
    Slice {
        elem: Box<Type>,
        len: Option<TokenStream>,
    },
    /// A path such as `i64` or `std::string::String`, with its generic
    /// arguments when they parse as types (`Vec<'a>`). Otherwise `tokens`
    /// holds the arguments too, as written.
    Path {
        tokens: TokenStream,
        args: Option<Vec<GenericArg>>,
    },
    /// Rust syntax passed through untouched (`!`).
    Rust(TokenStream),
}

#[derive(Clone)]
pub enum GenericArg {
    Type(Type),
    /// `Item = t`
    Binding(Ident, Type),
    /// `'static`
    Lifetime(Ident, Span),
}

/// Where a type is emitted. Rust only allows `impl Trait` in function
/// signatures and has no way to name type variables in `let`s or closures.
#[derive(Clone, Copy, PartialEq)]
pub enum Ctx {
    Sig,
    Local,
}

fn lifetime(name: &Ident, span: Span) -> TokenStream {
    let mut q = Punct::new('\'', Spacing::Joint);
    q.set_span(span);
    quote!(#q #name)
}

impl Type {
    pub fn is_arrow(&self) -> bool {
        match self {
            Type::Arrow(..) => true,
            Type::Paren(t) => t.is_arrow(),
            _ => false,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            Type::Unit(_) => true,
            Type::Paren(t) => t.is_unit(),
            _ => false,
        }
    }

    fn strip(&self) -> &Type {
        match self {
            Type::Paren(t) => t.strip(),
            t => t,
        }
    }

    /// Splits `a -> b -> c` into `([a, b], c)`. Parentheses stop the
    /// flattening: `a -> (b -> c)` is `([a], (b -> c))`.
    pub fn arrow_parts(&self) -> (Vec<&Type>, &Type) {
        let mut params = Vec::new();
        let mut cur = self.strip();
        while let Type::Arrow(a, b) = cur {
            params.push(&**a);
            cur = b;
        }
        (params, cur)
    }

    /// The call shape a value of this type has (see [`Shape`]).
    pub fn shape(&self) -> Option<Shape> {
        let (params, ret) = self.arrow_parts();
        if params.is_empty() {
            return None;
        }
        let n = if params.len() == 1 && params[0].is_unit() {
            0
        } else {
            params.len()
        };
        let mut shape = vec![n];
        if let Type::Paren(inner) = ret
            && let Some(rest) = inner.shape()
        {
            shape.extend(rest);
        }
        Some(shape)
    }

    /// Emits the Rust type. In `Sig` position type variables become generic
    /// parameters (collected into `generics`); in `Local` position they and
    /// function types become `_`.
    pub fn emit(&self, ctx: Ctx, generics: &mut Vec<Ident>) -> TokenStream {
        match self {
            Type::Arrow(..) => {
                if ctx == Ctx::Local {
                    return quote!(_);
                }
                let (params, ret) = self.arrow_parts();
                let params: Vec<TokenStream> = if params.len() == 1 && params[0].is_unit() {
                    Vec::new()
                } else {
                    params.iter().map(|p| p.emit(ctx, generics)).collect()
                };
                let ret = ret.emit(ctx, generics);
                quote!(impl ::core::ops::Fn(#(#params),*) -> #ret)
            }
            Type::Tuple(ts) | Type::ParenTuple(ts) => {
                let ts = ts.iter().map(|t| t.emit(ctx, generics));
                quote!((#(#ts),*))
            }
            Type::Var(i) => {
                if ctx == Ctx::Local {
                    return quote!(_);
                }
                let name = ident_str(i);
                let mut chars = name.chars();
                let first = chars
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>())
                    .unwrap_or_default();
                let g = format_ident!("{}{}", first, chars.as_str(), span = i.span());
                if !generics.contains(&g) {
                    generics.push(g.clone());
                }
                quote!(#g)
            }
            Type::Paren(t) => t.emit(ctx, generics),
            Type::Slice { elem, len } => {
                let e = elem.emit(ctx, generics);
                match len {
                    Some(n) => quote!([#e; #n]),
                    None => quote!([#e]),
                }
            }
            Type::Infer(sp) => quote_spanned!(*sp=> _),
            Type::Unit(sp) => quote_spanned!(*sp=> ()),
            Type::Ref {
                lifetime: lt,
                mutable,
                inner,
                span,
            } => {
                let inner = inner.emit(ctx, generics);
                let lt = lt.as_ref().map(|l| lifetime(l, *span));
                let m = if *mutable { quote!(mut) } else { quote!() };
                quote_spanned!(*span=> & #lt #m #inner)
            }
            Type::Path { tokens, args } => match args {
                None => tokens.clone(),
                Some(args) => {
                    let args = args.iter().map(|a| match a {
                        GenericArg::Type(t) => t.emit(ctx, generics),
                        GenericArg::Binding(name, t) => {
                            let t = t.emit(ctx, generics);
                            quote!(#name = #t)
                        }
                        GenericArg::Lifetime(l, sp) => lifetime(l, *sp),
                    });
                    quote!(#tokens<#(#args),*>)
                }
            },
            Type::Rust(ts) => ts.clone(),
        }
    }
}

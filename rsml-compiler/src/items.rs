//! `rsml! { .. }`: Rust items whose function bodies are OCaml.
//!
//! An attribute (`#[rsml] fn f() { .. }`) cannot do this: rustc parses the
//! item an attribute is attached to as Rust before the attribute runs, and an
//! OCaml body is not Rust. A function-like macro receives raw tokens, so
//! `rsml!` wraps the items instead. Every `fn` with a body inside it,
//! including those in `impl`, `trait` and `mod` blocks, has its body
//! translated like `ml!`; everything else passes through untouched, and so
//! does every body under a `#[rust]` item.
//!
//! The macro sees each signature, so parameters are in scope as Rust locals
//! (with a parameter `h`, `h::t` is a cons, not a path), and parameters whose
//! type is a closure (`f: impl Fn(i64, i64) -> i64`, or a generic `F` with
//! such a bound) have a known shape: `f 1` is a partial application. So do
//! the functions of the same `rsml!` that a body can name (see
//! `module_scope`).

use std::collections::{HashMap, HashSet};

use std::rc::Rc;

use proc_macro2::{Delimiter, Group, Spacing, Span, TokenStream, TokenTree};

use crate::ast::{Labels, Shape};
use crate::codegen::Scope;
use crate::parser::Error;

/// Shapes of functions known by name (`add`, `Self::new`, `Point::new`),
/// with the names of their parameters, by which a call can label them.
pub type Known = Vec<(String, Shape, Labels)>;

/// A function with an OCaml body found inside `rsml! { .. }`.
#[derive(Clone)]
pub struct FnBody {
    /// Parameters bound to a plain name, with the shape of closure-typed ones.
    pub params: Vec<(String, Option<Shape>)>,
    /// The functions of the same `rsml!` the body can call by name, shared
    /// with the other bodies of its level.
    pub known: Rc<Scope>,
    /// The `{ .. }` body.
    pub body: Group,
}

pub fn expand_items(input: TokenStream) -> TokenStream {
    rewrite(input, Level::Module, &Rc::default(), &mut |f| {
        let ts = crate::codegen::expand_in(f.body.stream(), &f.params, f.known);
        // An expansion that already is a block becomes the body itself.
        let mut iter = ts.clone().into_iter();
        let stream = match (iter.next(), iter.next()) {
            (Some(TokenTree::Group(g)), None) if g.delimiter() == Delimiter::Brace => g.stream(),
            _ => ts,
        };
        let mut g = Group::new(Delimiter::Brace, stream);
        g.set_span(f.body.span());
        g
    })
}

/// Every OCaml function body in `rsml! { .. }` input, in source order
/// (bodies under `#[rust]` are Rust, and not among them).
pub fn fn_bodies(input: TokenStream) -> Vec<FnBody> {
    let mut out = Vec::new();
    rewrite(input, Level::Module, &Rc::default(), &mut |f| {
        let g = f.body.clone();
        out.push(f);
        g
    });
    out
}

/// Where a list of items is.
#[derive(Clone, PartialEq)]
enum Level {
    /// A module: the whole `rsml!`, or a `mod m { .. }` in it.
    Module,
    /// The inside of an `impl` or `trait` block, with the type the impl
    /// is for when it is a plain name.
    Assoc(Option<String>),
}

fn rewrite(
    input: TokenStream,
    level: Level,
    outer: &Rc<Scope>,
    f: &mut dyn FnMut(FnBody) -> Group,
) -> TokenStream {
    let toks: Vec<TokenTree> = input.into_iter().collect();
    let known = Rc::new(match &level {
        Level::Module => module_scope(&toks, outer),
        // `Self::name` for the type's associated functions (`Type::name`
        // around the block), those of this block first.
        Level::Assoc(ty) => {
            let own = own_fns(&toks)
                .into_iter()
                .map(|(n, s, l)| (format!("Self::{}", n), s, l))
                .collect();
            Scope::assoc(own, outer, ty.clone())
        }
    });
    let mut out = TokenStream::new();
    let mut i = 0;
    while i < toks.len() {
        // `#[rust]`: the item goes through as written, without the attribute.
        if let Some(attr) = rust_attr(&toks, i) {
            match rust_item_end(&toks, i + 2) {
                Some(end) => {
                    out.extend(toks[i + 2..=end].iter().cloned());
                    i = end + 1;
                }
                None => {
                    let msg = "`#[rust]` goes on a `fn`, `impl`, `trait` or `mod`, and leaves the \
                               function bodies under it as Rust";
                    out.extend(Error::new(attr, msg).to_compile_error());
                    out.extend(quote::quote!(;));
                    i += 2;
                }
            }
            continue;
        }
        if let Some(b) = fn_body_index(&toks, i) {
            // The signature passes through; the body is OCaml.
            out.extend(toks[i..b].iter().cloned());
            let TokenTree::Group(body) = &toks[b] else {
                unreachable!()
            };
            let params = params(&toks[i + 2..b]);
            let body = body.clone();
            let known = known.clone();
            out.extend([TokenTree::Group(f(FnBody {
                params,
                known,
                body,
            }))]);
            i = b + 1;
            continue;
        }
        match &toks[i] {
            // `impl`, `trait` and `mod` blocks. Macro invocations are left alone.
            TokenTree::Group(g)
                if g.delimiter() == Delimiter::Brace && !(i > 0 && is_punct(&toks[i - 1], '!')) =>
            {
                let item = &toks[item_start(&toks, i)..i];
                let inner = match item_keyword(item) {
                    Some(kw) if kw == "mod" => rewrite(g.stream(), Level::Module, &known, f),
                    Some(kw) if kw == "impl" => rewrite(
                        g.stream(),
                        Level::Assoc(impl_type(item).map(|(t, _)| t)),
                        &known,
                        f,
                    ),
                    Some(kw) if kw == "trait" => rewrite(g.stream(), Level::Assoc(None), &known, f),
                    _ => rewrite(g.stream(), level.clone(), &known, f),
                };
                let mut inner = Group::new(Delimiter::Brace, inner);
                inner.set_span(g.span());
                out.extend([TokenTree::Group(inner)]);
            }
            t => out.extend([t.clone()]),
        }
        i += 1;
    }
    out
}

/// The shapes known in the function bodies of a module (see the `rsml`
/// docs): its functions, by name, and the associated functions of its
/// inherent `impl`s as `Type::name`, plus the parent module's that
/// `use super::*` or `use super::name` imports. `parent` is the parent's.
fn module_scope(toks: &[TokenTree], parent: &Rc<Scope>) -> Scope {
    let mut glob = false;
    let mut aliases = Vec::new();
    let mut shadowed = HashSet::new();
    for (path, local) in module_uses(toks) {
        match (path.as_slice(), local.as_str()) {
            ([sup], "*") if sup == "super" => glob = true,
            (_, "*") => {}
            ([sup, name], _) if sup == "super" => {
                aliases.push((local.clone(), name.clone()));
                shadowed.insert(local);
            }
            _ => {
                shadowed.insert(local);
            }
        }
    }
    let mut known: Known = own_fns(toks);
    // Inherent `impl Type { .. }` blocks: `Type::name`.
    let mut assoc = Vec::new();
    for (k, t) in toks.iter().enumerate() {
        let TokenTree::Group(g) = t else { continue };
        if g.delimiter() != Delimiter::Brace || (k > 0 && is_punct(&toks[k - 1], '!')) {
            continue;
        }
        let item = &toks[item_start(toks, k)..k];
        if let Some((ty, false)) = impl_type(item) {
            let body: Vec<TokenTree> = g.stream().into_iter().collect();
            for (n, s, l) in own_fns(&body) {
                assoc.push((format!("{}::{}", ty, n), s, l));
            }
        }
    }
    known.extend(without_conflicts(assoc));
    Scope::module(known, parent, aliases, glob.then_some(shadowed))
}

/// The functions defined in a list of items that take no `self`, with
/// their shapes. A name defined twice with different shapes (under
/// different `#[cfg]`s) is left out.
fn own_fns(toks: &[TokenTree]) -> Known {
    let mut out = Vec::new();
    for i in 0..toks.len() {
        let (TokenTree::Ident(kw), Some(TokenTree::Ident(name))) = (&toks[i], toks.get(i + 1))
        else {
            continue;
        };
        if kw != "fn" {
            continue;
        }
        let end = (i + 2..toks.len())
            .find(|&j| match &toks[j] {
                TokenTree::Punct(p) => p.as_char() == ';',
                TokenTree::Group(g) => g.delimiter() == Delimiter::Brace,
                _ => false,
            })
            .unwrap_or(toks.len());
        let sig = &toks[i + 2..end];
        if let Some(shape) = fn_shape(sig, is_async(&toks[..i])) {
            out.push((crate::lexer::ident_str(name), shape, fn_labels(sig)));
        }
    }
    without_conflicts(out)
}

/// Whether the qualifiers before a `fn` (the end of `before`) include `async`.
fn is_async(before: &[TokenTree]) -> bool {
    for t in before.iter().rev() {
        match t {
            TokenTree::Ident(q) if q == "async" => return true,
            TokenTree::Ident(q)
                if ["unsafe", "const", "extern", "pub", "default", "safe"]
                    .contains(&q.to_string().as_str()) => {}
            TokenTree::Literal(_) => {}
            TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => {}
            _ => return false,
        }
    }
    false
}

/// Names defined more than once (under different `#[cfg]`s) keep their
/// shape if every definition has the same one, and their labels likewise.
fn without_conflicts(entries: Known) -> Known {
    let mut seen: HashMap<String, Option<(Shape, Labels)>> = HashMap::new();
    for (n, s, l) in &entries {
        seen.entry(n.clone())
            .and_modify(|old| match old {
                Some((s0, _)) if s0 != s => *old = None,
                Some((_, l0)) if l0.as_deref() != l.as_deref() => *l0 = None,
                _ => {}
            })
            .or_insert_with(|| Some((s.clone(), l.clone())));
    }
    let mut out = Vec::new();
    for (n, _, _) in entries {
        if let Some(Some((s, l))) = seen.remove(&n) {
            out.push((n, s, l));
        }
    }
    out
}

/// The shape of a function with signature `sig` (the tokens after its
/// name): one group of its parameters (`[0]` for none), and one more for
/// each closure it returns (`-> impl Fn(i64) -> i64`). `None` for a method
/// taking `self`.
fn fn_shape(sig: &[TokenTree], is_async: bool) -> Option<Shape> {
    let mut bounds = HashMap::new();
    let mut rest = sig;
    if rest.first().is_some_and(|t| is_punct(t, '<')) {
        let close = angle_close(rest);
        generic_bounds(&rest[1..close], &mut bounds);
        rest = &rest[(close + 1).min(rest.len())..];
    }
    let Some(TokenTree::Group(list)) = rest.first() else {
        return None;
    };
    let ret = &rest[1..];
    let where_at = ret
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(i) if i == "where"))
        .unwrap_or(ret.len());
    generic_bounds(&ret[(where_at + 1).min(ret.len())..], &mut bounds);
    let toks: Vec<TokenTree> = list.stream().into_iter().collect();
    let params = split_top(&toks, ',');
    if params
        .first()
        .is_some_and(|p| is_receiver(skip_attributes(p)))
    {
        return None;
    }
    let mut shape = vec![params.len()];
    let ret = &ret[..where_at];
    if !is_async && ret.len() > 2 && is_punct(&ret[0], '-') && is_punct(&ret[1], '>') {
        shape.extend(fn_groups(&ret[2..], &bounds));
    }
    Some(shape)
}

/// The names of the parameters of a signature (the tokens after the
/// function's name), by which a call can label its arguments: `None` for
/// a parameter that is a pattern rather than a name.
fn fn_labels(sig: &[TokenTree]) -> Labels {
    let mut rest = sig;
    if rest.first().is_some_and(|t| is_punct(t, '<')) {
        rest = &rest[(angle_close(rest) + 1).min(rest.len())..];
    }
    let Some(TokenTree::Group(list)) = rest.first() else {
        return None;
    };
    let toks: Vec<TokenTree> = list.stream().into_iter().collect();
    let labels: Vec<Option<String>> = split_top(&toks, ',')
        .into_iter()
        .map(|param| {
            let param = skip_attributes(param);
            let colon = (0..param.len()).find(|&k| is_lone_colon(param, k))?;
            match &param[..colon] {
                [TokenTree::Ident(n)] => Some(n),
                [TokenTree::Ident(m), TokenTree::Ident(n)] if m == "mut" => Some(n),
                _ => None,
            }
            .map(crate::lexer::ident_str)
            .filter(|n| n != "_")
        })
        .collect();
    labels.iter().any(Option::is_some).then(|| labels.into())
}

/// `self`, `mut self`, `&self`, `&'a mut self`, `self: Box<Self>`.
fn is_receiver(p: &[TokenTree]) -> bool {
    let mut k = 0;
    while k < p.len() {
        match &p[k] {
            TokenTree::Punct(q) if q.as_char() == '&' => k += 1,
            TokenTree::Punct(q) if q.as_char() == '\'' => k += 2,
            TokenTree::Ident(i) if i == "mut" => k += 1,
            _ => break,
        }
    }
    matches!(p.get(k), Some(TokenTree::Ident(i)) if i == "self")
}

/// The call groups of a returned closure type: `impl Fn(a) -> impl Fn(b)
/// -> c` is `[1, 1]`; `Box<dyn Fn(..)>`, `&dyn Fn(..)` and a generic with
/// an `Fn` bound count too.
fn fn_groups(ty: &[TokenTree], bounds: &HashMap<String, usize>) -> Vec<usize> {
    let mut k = 0;
    while k < ty.len() {
        match &ty[k] {
            TokenTree::Punct(p) if p.as_char() == '&' => k += 1,
            TokenTree::Punct(p) if p.as_char() == '\'' => k += 2,
            TokenTree::Ident(i) if i == "impl" || i == "dyn" || i == "mut" => k += 1,
            _ => break,
        }
    }
    match (ty.get(k), ty.get(k + 1)) {
        (Some(TokenTree::Ident(i)), Some(t))
            if (i == "Box" || i == "Rc" || i == "Arc") && is_punct(t, '<') =>
        {
            let inner = &ty[k + 1..];
            let close = angle_close(inner);
            fn_groups(&inner[1..close.min(inner.len())], bounds)
        }
        (Some(TokenTree::Ident(i)), Some(TokenTree::Group(g)))
            if (i == "Fn" || i == "FnMut" || i == "FnOnce")
                && g.delimiter() == Delimiter::Parenthesis =>
        {
            let args: Vec<TokenTree> = g.stream().into_iter().collect();
            let mut out = vec![split_top(&args, ',').len()];
            let rest = &ty[k + 2..];
            if rest.len() > 2 && is_punct(&rest[0], '-') && is_punct(&rest[1], '>') {
                // The return type ends at a `+` of the bound list.
                let ret = &rest[2..];
                let end = split_top(ret, '+').first().map_or(0, |r| r.len());
                out.extend(fn_groups(&ret[..end], bounds));
            }
            out
        }
        (Some(TokenTree::Ident(g)), next) if next.is_none_or(|t| is_punct(t, '+')) => bounds
            .get(&g.to_string())
            .map(|&n| vec![n])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The type an `impl` is for (`item` is everything before its braces),
/// when it is a plain name (`impl<T> P<T>`, `impl Clone for P`), and
/// whether the impl is a trait's.
fn impl_type(item: &[TokenTree]) -> Option<(String, bool)> {
    if item_keyword(item).is_none_or(|kw| kw != "impl") {
        return None;
    }
    let k = item
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(i) if i == "impl"))?;
    let mut rest = &item[k + 1..];
    if rest.first().is_some_and(|t| is_punct(t, '<')) {
        let close = angle_close(rest);
        rest = &rest[(close + 1).min(rest.len())..];
    }
    let for_at = rest
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(i) if i == "for"));
    let ty = match for_at {
        Some(f) => &rest[f + 1..],
        None => rest,
    };
    match ty {
        [TokenTree::Ident(name), tail @ ..]
            if tail.is_empty()
                || is_punct(&tail[0], '<')
                || matches!(&tail[0], TokenTree::Ident(w) if w == "where") =>
        {
            Some((name.to_string(), for_at.is_some()))
        }
        _ => None,
    }
}

/// The names a module's `use` items bring in, as `(path, name)`: `use
/// super::add;` is `(["super", "add"], "add")`, `use super::*` is
/// `(["super"], "*")`.
fn module_uses(toks: &[TokenTree]) -> Vec<(Vec<String>, String)> {
    let mut out = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if matches!(t, TokenTree::Ident(u) if u == "use") {
            let end = (i..toks.len())
                .find(|&j| is_punct(&toks[j], ';'))
                .unwrap_or(toks.len());
            use_tree(&toks[i + 1..end], Vec::new(), &mut out);
        }
    }
    out
}

fn use_tree(toks: &[TokenTree], mut path: Vec<String>, out: &mut Vec<(Vec<String>, String)>) {
    let mut k = 0;
    if toks.len() >= 2 && is_punct(&toks[0], ':') && is_punct(&toks[1], ':') {
        k = 2;
    }
    loop {
        match toks.get(k) {
            Some(TokenTree::Ident(i)) => {
                let name = crate::lexer::ident_str(i);
                k += 1;
                if toks.len() >= k + 2 && is_punct(&toks[k], ':') && is_punct(&toks[k + 1], ':') {
                    path.push(name);
                    k += 2;
                    continue;
                }
                let local = match (toks.get(k), toks.get(k + 1)) {
                    (Some(TokenTree::Ident(a)), Some(TokenTree::Ident(l))) if a == "as" => {
                        crate::lexer::ident_str(l)
                    }
                    _ => name.clone(),
                };
                let mut full = path.clone();
                full.push(name);
                // `use super::{self}` names the module itself.
                if full.last().is_some_and(|n| n != "self") {
                    out.push((full, local));
                }
                return;
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '*' => {
                out.push((path, "*".to_string()));
                return;
            }
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                for sub in split_top(&inner, ',') {
                    use_tree(sub, path.clone(), out);
                }
                return;
            }
            _ => return,
        }
    }
}

/// Where the item that the token at `i` belongs to starts: after the
/// previous `;` or `{ .. }` of the list.
fn item_start(toks: &[TokenTree], i: usize) -> usize {
    (0..i)
        .rev()
        .find(|&j| match &toks[j] {
            TokenTree::Punct(p) => p.as_char() == ';',
            TokenTree::Group(g) => g.delimiter() == Delimiter::Brace,
            _ => false,
        })
        .map_or(0, |j| j + 1)
}

/// The keyword an item starts with, after its attributes, visibility and
/// qualifiers (`unsafe`, `async`, `const fn`, `extern "C"`, `default`).
fn item_keyword(item: &[TokenTree]) -> Option<String> {
    let mut k = 0;
    loop {
        match (item.get(k), item.get(k + 1)) {
            (Some(TokenTree::Punct(p)), Some(TokenTree::Group(g)))
                if p.as_char() == '#' && g.delimiter() == Delimiter::Bracket =>
            {
                k += 2
            }
            // Inner attributes, `#![..]`.
            (Some(TokenTree::Punct(p)), Some(TokenTree::Punct(q)))
                if p.as_char() == '#' && q.as_char() == '!' =>
            {
                k += 3
            }
            (Some(TokenTree::Ident(i)), next) if i == "pub" => {
                k += 1;
                if matches!(next, Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis)
                {
                    k += 1;
                }
            }
            (Some(TokenTree::Ident(i)), Some(TokenTree::Ident(next)))
                if (i == "unsafe" || i == "async" || i == "default" || i == "safe")
                    || (i == "const"
                        && (next == "fn"
                            || next == "unsafe"
                            || next == "async"
                            || next == "extern")) =>
            {
                k += 1
            }
            (Some(TokenTree::Ident(i)), _) if i == "extern" => {
                k += 1;
                if matches!(item.get(k), Some(TokenTree::Literal(_))) {
                    k += 1;
                }
                if !matches!(item.get(k), Some(TokenTree::Ident(n)) if n == "fn") {
                    return Some("extern".to_string());
                }
            }
            (Some(TokenTree::Ident(i)), _) => return Some(i.to_string()),
            _ => return None,
        }
    }
}

/// The span of a `#[rust]` attribute starting at `i`.
fn rust_attr(toks: &[TokenTree], i: usize) -> Option<Span> {
    let (TokenTree::Punct(hash), Some(TokenTree::Group(g))) = (&toks[i], toks.get(i + 1)) else {
        return None;
    };
    let inner: Vec<TokenTree> = g.stream().into_iter().collect();
    match &inner[..] {
        [TokenTree::Ident(r)]
            if hash.as_char() == '#' && g.delimiter() == Delimiter::Bracket && r == "rust" =>
        {
            Some(hash.span())
        }
        _ => None,
    }
}

/// The last token of the `fn`, `impl`, `trait` or `mod` item starting at
/// `start` (after a `#[rust]`), or `None` for any other item.
fn rust_item_end(toks: &[TokenTree], start: usize) -> Option<usize> {
    let ends = |from: usize| {
        (from..toks.len()).find(|&j| match &toks[j] {
            TokenTree::Punct(p) => p.as_char() == ';',
            TokenTree::Group(g) => g.delimiter() == Delimiter::Brace,
            _ => false,
        })
    };
    let item_end = ends(start)?;
    let kw = item_keyword(&toks[start..item_end])?;
    match kw.as_str() {
        "fn" | "impl" | "trait" | "mod" => Some(item_end),
        _ => None,
    }
}

/// For `fn name` at `i`, the index of its `{ .. }` body. `None` for anything
/// else, including declarations without a body and `fn(..)` pointer types.
fn fn_body_index(toks: &[TokenTree], i: usize) -> Option<usize> {
    match (&toks[i], toks.get(i + 1)) {
        (TokenTree::Ident(kw), Some(TokenTree::Ident(_))) if kw == "fn" => {}
        _ => return None,
    }
    for (j, t) in toks.iter().enumerate().skip(i + 2) {
        match t {
            TokenTree::Punct(p) if p.as_char() == ';' => return None,
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => return Some(j),
            _ => {}
        }
    }
    None
}

fn is_punct(t: &TokenTree, c: char) -> bool {
    matches!(t, TokenTree::Punct(p) if p.as_char() == c)
}

/// The parameters of a signature (the tokens between the name and the body).
fn params(sig: &[TokenTree]) -> Vec<(String, Option<Shape>)> {
    let mut bounds = HashMap::new();
    let mut rest = sig;
    if rest.first().is_some_and(|t| is_punct(t, '<')) {
        let close = angle_close(rest);
        generic_bounds(&rest[1..close], &mut bounds);
        rest = &rest[(close + 1).min(rest.len())..];
    }
    let Some(TokenTree::Group(list)) = rest.first() else {
        return Vec::new();
    };
    if let Some(w) = rest
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(i) if i == "where"))
    {
        generic_bounds(&rest[w + 1..], &mut bounds);
    }
    let toks: Vec<TokenTree> = list.stream().into_iter().collect();
    let mut out = Vec::new();
    for param in split_top(&toks, ',') {
        let param = skip_attributes(param);
        let Some(colon) = (0..param.len()).find(|&k| is_lone_colon(param, k)) else {
            continue; // `self`, `&self`, ..
        };
        let name = match &param[..colon] {
            [TokenTree::Ident(n)] | [TokenTree::Ident(_), TokenTree::Ident(n)] => n.to_string(),
            _ => continue,
        };
        if name == "_" || name == "self" {
            continue;
        }
        let ty = &param[colon + 1..];
        let arity = fn_arity(ty).or_else(|| match ty {
            [TokenTree::Ident(g)] => bounds.get(&g.to_string()).copied(),
            _ => None,
        });
        out.push((name, arity.map(|n| vec![n])));
    }
    out
}

/// The index of the `>` closing the `<` at index 0 (or the length if unclosed).
fn angle_close(toks: &[TokenTree]) -> usize {
    let mut depth = 0i32;
    for k in 0..toks.len() {
        depth += angle_delta(toks, k);
        if depth == 0 {
            return k;
        }
    }
    toks.len()
}

/// `+1` for `<`, `-1` for a `>` that is not part of `->`.
fn angle_delta(toks: &[TokenTree], k: usize) -> i32 {
    match &toks[k] {
        TokenTree::Punct(p) if p.as_char() == '<' => 1,
        TokenTree::Punct(p) if p.as_char() == '>' => {
            let arrow = k > 0
                && matches!(&toks[k - 1], TokenTree::Punct(q)
                    if q.as_char() == '-' && q.spacing() == Spacing::Joint);
            if arrow { 0 } else { -1 }
        }
        _ => 0,
    }
}

/// Splits on `sep` outside of angle brackets (other brackets are groups).
fn split_top(toks: &[TokenTree], sep: char) -> Vec<&[TokenTree]> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for k in 0..toks.len() {
        depth += angle_delta(toks, k);
        if depth == 0 && is_punct(&toks[k], sep) {
            out.push(&toks[start..k]);
            start = k + 1;
        }
    }
    if start < toks.len() {
        out.push(&toks[start..]);
    }
    out
}

/// A `:` that is not half of `::`.
fn is_lone_colon(toks: &[TokenTree], k: usize) -> bool {
    let TokenTree::Punct(p) = &toks[k] else {
        return false;
    };
    let after_colon = k > 0
        && matches!(&toks[k - 1], TokenTree::Punct(q)
            if q.as_char() == ':' && q.spacing() == Spacing::Joint);
    p.as_char() == ':' && p.spacing() == Spacing::Alone && !after_colon
}

fn skip_attributes(mut toks: &[TokenTree]) -> &[TokenTree] {
    while let [TokenTree::Punct(p), TokenTree::Group(_), rest @ ..] = toks
        && p.as_char() == '#'
    {
        toks = rest;
    }
    toks
}

/// Records `F: Fn(..)` bounds from generics or a `where` clause.
fn generic_bounds(toks: &[TokenTree], bounds: &mut HashMap<String, usize>) {
    for item in split_top(toks, ',') {
        let [TokenTree::Ident(name), ..] = item else {
            continue;
        };
        if item.len() < 2 || !is_lone_colon(item, 1) {
            continue;
        }
        for bound in split_top(&item[2..], '+') {
            if let Some(n) = fn_arity(bound) {
                bounds.insert(name.to_string(), n);
            }
        }
    }
}

/// `Fn(a, b) -> c`, possibly behind `impl`, `dyn`, `&` and lifetimes: `Some(2)`.
fn fn_arity(ty: &[TokenTree]) -> Option<usize> {
    let mut k = 0;
    while k < ty.len() {
        match &ty[k] {
            TokenTree::Punct(p) if p.as_char() == '&' => k += 1,
            TokenTree::Punct(p) if p.as_char() == '\'' => k += 2,
            TokenTree::Ident(i) if i == "impl" || i == "dyn" || i == "mut" => k += 1,
            _ => break,
        }
    }
    match (ty.get(k), ty.get(k + 1)) {
        (Some(TokenTree::Ident(i)), Some(TokenTree::Group(g)))
            if (i == "Fn" || i == "FnMut" || i == "FnOnce")
                && g.delimiter() == Delimiter::Parenthesis =>
        {
            let args: Vec<TokenTree> = g.stream().into_iter().collect();
            Some(split_top(&args, ',').len())
        }
        _ => None,
    }
}

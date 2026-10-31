//! Which `match` a case belongs to when matches nest without parentheses.
//!
//! In `match a with A::X -> match b with B::P -> 1 | A::Y -> 2`, the
//! inner `match` would take every case after it, `A::Y` included, which
//! is a type error. The macro runs before types are known, but patterns
//! show part of their type: a path's type segment (`A` in `A::Y`), `Some`
//! and `None` (`Option`), `Ok` and `Err` (`Result`), the kind of a
//! literal, a tuple's length, a list. A case that the inner `match`
//! cannot take, because its pattern is of another type than the inner
//! cases' or a case before it already matches everything it does, goes
//! to the nearest enclosing `match` that can take it. Without such
//! evidence (`_`, a variable, or the same type at both levels) it stays
//! with the inner `match`, as in OCaml.
//!
//! Only cases reachable without crossing brackets, `begin .. end` or a
//! keyword that ends an expression (`in`, `then`, `with`, `do`, `done`)
//! can move: those are barriers.

use crate::ast::{Pat, Path};
use crate::lexer::{ident_str, is_capitalized};

/// The cases of one `match` or `function` parsed so far.
#[derive(Default)]
pub struct Level {
    /// The types the patterns show (see [`type_key`]).
    keys: Vec<String>,
    /// The patterns of the cases without a guard.
    unguarded: Vec<Pat>,
}

impl Level {
    /// Records a case's pattern, before its guard and body are parsed.
    pub fn add(&mut self, pat: &Pat) {
        if let Some(k) = type_key(pat)
            && !self.keys.contains(&k)
        {
            self.keys.push(k);
        }
    }

    /// Records that the case with this pattern has no guard.
    pub fn add_unguarded(&mut self, pat: &Pat) {
        self.unguarded.push(pat.clone());
    }

    /// Whether this `match` can take a case with pattern `p`: it is not
    /// of another type than the cases so far, nor matched by one of them.
    fn takes(&self, p: &Pat) -> bool {
        let other_type = match type_key(p) {
            Some(k) => !self.keys.is_empty() && !self.keys.contains(&k),
            None => false,
        };
        !other_type && !self.unguarded.iter().any(|q| covers(q, p))
    }
}

/// Whether the case with pattern `p`, coming after the cases of the
/// innermost level of `levels`, belongs to an enclosing one. `None`
/// entries are barriers.
pub fn belongs_outside(levels: &[Option<Level>], p: &Pat) -> bool {
    let Some((Some(inner), outer)) = levels.split_last() else {
        return false;
    };
    if inner.takes(p) {
        return false;
    }
    for level in outer.iter().rev() {
        match level {
            None => return false,
            Some(l) if l.takes(p) => return true,
            Some(_) => {}
        }
    }
    false
}

/// The type a pattern shows, as a name only compared with others: the
/// type segment of a path (`Shape` for `Shape::Circle r`), `Option`,
/// `Result`, a literal's kind, `tuple2`, `slice`. `None` when the pattern
/// says nothing (`_`, `x`, a constructor without its type, `Self::A`).
pub fn type_key(p: &Pat) -> Option<String> {
    match p {
        Pat::Paren(p) | Pat::Constraint(p, _) | Pat::Alias(p, _) | Pat::Ref(p, _) => type_key(p),
        Pat::Range(lo, _, _) => type_key(lo),
        Pat::Or(ps) => ps.iter().find_map(type_key),
        Pat::Ctor(path, _) | Pat::CtorRecord(path, _, _) => path_key(path),
        Pat::Lit { lit, .. } => {
            let s = lit.to_string();
            Some(
                if s.starts_with('"') || s.starts_with("r\"") || s.starts_with("r#") {
                    "str"
                } else if s.starts_with("b\"") || s.starts_with("br") {
                    "bytes"
                } else if s.starts_with('\'') {
                    "char"
                } else if is_float(&s) {
                    "float"
                } else {
                    // Integers and byte literals (`b'a'` is a `u8`).
                    "integer"
                }
                .to_string(),
            )
        }
        Pat::Bool(..) => Some("bool".to_string()),
        Pat::Unit(_) => Some("unit".to_string()),
        Pat::Tuple(ps) => Some(format!("tuple{}", ps.len())),
        Pat::List(..) | Pat::Array(..) | Pat::Cons(..) => Some("slice".to_string()),
        Pat::Wild(_) | Pat::Var(_) | Pat::Mut(_) | Pat::Label { .. } => None,
    }
}

fn path_key(path: &Path) -> Option<String> {
    let segs = &path.segs;
    if let [only] = &segs[..] {
        return match ident_str(&only.ident).as_str() {
            "Some" | "None" => Some("Option".to_string()),
            "Ok" | "Err" => Some("Result".to_string()),
            _ => None,
        };
    }
    // `Shape::Circle`, `std::cmp::Ordering::Less`: the type is the segment
    // before the variant. `Self` could be any type, and a lowercase one
    // (`i32::MAX`, `consts::PI`) is a module or a primitive.
    let ty = &segs[segs.len() - 2].ident;
    (is_capitalized(ty) && ty != "Self").then(|| ident_str(ty))
}

fn is_float(lit: &str) -> bool {
    if lit.starts_with("0x") || lit.starts_with("0o") || lit.starts_with("0b") {
        return false;
    }
    lit.contains('.')
        || lit.ends_with("f32")
        || lit.ends_with("f64")
        || (lit.contains(['e', 'E']) && lit.starts_with(|c: char| c.is_ascii_digit()))
}

/// Whether a pattern matches every value of its type.
fn irrefutable(p: &Pat) -> bool {
    match p {
        Pat::Wild(_) | Pat::Var(_) | Pat::Mut(_) | Pat::Unit(_) => true,
        Pat::Paren(p) | Pat::Constraint(p, _) | Pat::Alias(p, _) | Pat::Ref(p, _) => irrefutable(p),
        Pat::Tuple(ps) => ps.iter().all(irrefutable),
        _ => false,
    }
}

/// Whether every value `next` matches is matched by `prev` already (so a
/// case with `next` after one with `prev` would never be taken). Only the
/// plain cases: `prev` irrefutable, or the same constructor with
/// irrefutable fields, or the same literal.
fn covers(prev: &Pat, next: &Pat) -> bool {
    if irrefutable(prev) {
        return true;
    }
    match (strip(prev), strip(next)) {
        (Pat::Or(ps), _) => ps.iter().any(|p| covers(p, next)),
        (_, Pat::Or(ns)) => ns.iter().all(|n| covers(prev, n)),
        (Pat::Ctor(a, arg), Pat::Ctor(b, _) | Pat::CtorRecord(b, _, _)) => {
            same_path(a, b) && arg.as_deref().is_none_or(irrefutable)
        }
        (Pat::CtorRecord(a, fields, _), Pat::Ctor(b, _) | Pat::CtorRecord(b, _, _)) => {
            same_path(a, b)
                && fields
                    .iter()
                    .all(|(_, p)| p.as_ref().is_none_or(irrefutable))
        }
        (Pat::Lit { lit: a, neg: na }, Pat::Lit { lit: b, neg: nb }) => {
            na == nb && a.to_string() == b.to_string()
        }
        (Pat::Bool(a, _), Pat::Bool(b, _)) => a == b,
        _ => false,
    }
}

fn strip(p: &Pat) -> &Pat {
    match p {
        Pat::Paren(p) | Pat::Constraint(p, _) | Pat::Alias(p, _) => strip(p),
        p => p,
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.segs.len() == b.segs.len()
        && a.segs
            .iter()
            .zip(&b.segs)
            .all(|(x, y)| ident_str(&x.ident) == ident_str(&y.ident))
}

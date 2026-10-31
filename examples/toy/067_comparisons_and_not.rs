// Toy 067: comparisons `=` and `<>`, and `not`.
//
// What it is: rsml compares with `=` and `<>`; they are Rust's `==` and
// `!=`: `a = b` is `a == b` and `a <> b` is `a != b`. `==` and `!=` are
// accepted as themselves. `not e` negates, `&&` and `||` are as in Rust.
//
// When to use it: in every condition.
//
// How to write it:
// * `!` is not negation: `!r` reads a reference (`!r` is `*r`, toy 092).
//   Negation is `not`: `not (a = b)` is `!(a == b)`.
// * `not` takes the one atom after it, as a function would, so it binds
//   tighter than `=`: `not a = b` is `(!a) == b`. Put a comparison in
//   parentheses. A method call after a name is part of the atom:
//   `not s.is_empty()` is `!s.is_empty()`.
// * The gotcha: `!s.is_empty()` is `(*s).is_empty()`, which does not
//   negate anything (in `assert!` it is an error, toy 106).
// * A `=` inside a `let` is the binding; in an expression it compares:
//   `let same = a = b in ..` binds the comparison.
//
// Reference: [Operators that become calls](../../docs/notation.md#operators-that-become-calls)

use rsml::ml;

pub const SUMMARY: &str = "`a = b`, `a <> b`, `not (a = b)`; `!` is not negation";

/// Whether a password and its confirmation match, and are long enough.
pub fn valid(password: &str, again: &str) -> bool { ml! {
    // `password = again && password.len() >= 8` is
    // `(password == again) && (password.len() >= 8)`.
    password = again && password.len() >= 8
}}

/// Why a password is refused, if it is.
pub fn problem(password: &str, again: &str) -> Option<&'static str> { ml! {
    if password <> again then
      Some "they differ"
    else if not (password.len() >= 8) then
      Some "too short"
    else
      None
}}

/// Whether an answer is neither empty nor a default.
pub fn answered(answer: &str) -> bool { ml! {
    // `let is_default = answer = "n/a" in`: the first `=` binds, the
    // second compares.
    let is_default = answer = "n/a" in
    not answer.is_empty() && not is_default
}}

pub fn run() {
    println!(
        "valid \"secret12\" \"secret12\" = {}",
        valid("secret12", "secret12")
    );
    println!("problem \"abc\" \"abd\" = {:?}", problem("abc", "abd"));
    println!("problem \"abc\" \"abc\" = {:?}", problem("abc", "abc"));
    println!(
        "answered \"n/a\" = {}, answered \"yes\" = {}",
        answered("n/a"),
        answered("yes")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparisons() {
        assert!(valid("secret12", "secret12"));
        assert!(!valid("secret12", "secret13"));
        assert!(!valid("short", "short"));
        assert_eq!(problem("abc", "abd"), Some("they differ"));
        assert_eq!(problem("abc", "abc"), Some("too short"));
        assert_eq!(problem("long enough", "long enough"), None);
    }

    #[test]
    fn negation() {
        assert!(!answered("n/a"));
        assert!(!answered(""));
        assert!(answered("yes"));
    }
}

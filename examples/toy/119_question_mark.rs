// Toy 119: `?`, Rust's early return on errors.
//
// What it is: Rust's `?` operator, written right after an expression as
// in Rust: on an `Err` (or a `None`) it returns that from the function;
// on an `Ok` (or a `Some`) its value is the one inside.
//
// When to use it: in a function that returns a `Result` or an `Option`,
// to stop at the first failure instead of matching on every step. rsml has
// no exceptions (`try .. with` and `raise` are errors), so this is how
// rsml code fails.
//
// How to write it: `?` goes with the atom right before it.
// * `s.parse::<u32>()?` applies to the call, as in Rust.
// * `parse s?` is `parse(s?)`: the `?` goes with the argument `s`. For
//   the result of the call, write `(parse s)?` or `parse(s)?`.
// * In a pipeline, `?` after the last stage goes with that stage, a
//   method section, so a whole pipeline needs parentheses:
//   `(b |> .trim |> .parse::<u32>)?` is `b.trim().parse::<u32>()?`.
//
// Reference: [Rust-style methods, fields and calls](../../docs/notation.md#rust-style-methods-fields-and-calls),
// [Rust inside rsml code](../../README.md#rust-inside-rsml-code)

use rsml::rsml;
use std::num::ParseIntError;

pub const SUMMARY: &str = "Rust's `?`: `s.parse::<u32>()?`, `(parse s)?`, `words.first()?`";

rsml! {
/// A number from text, spaces around it allowed.
fn parse(s: &str) -> Result<u32, ParseIntError> {
    s.trim().parse()
}

/// The sum of two numbers written as text, or the first parse error.
fn add_text(a: &str, b: &str) -> Result<u32, ParseIntError> {
    // If `a` is not a number, `a.parse::<u32>()?` returns its `Err`.
    let x = a.parse::<u32>()? in
    let y = (b |> .trim |> .parse::<u32>)? in
    Ok (x + y)
}

/// Twice the number in `s`.
fn doubled(s: &str) -> Result<u32, ParseIntError> {
    // `2 * (parse s)?` is `2 * parse(s)?`.
    Ok (2 * (parse s)?)
}

/// The length of the first word, plus one; `None` without words.
fn first_len(words: &[&str]) -> Option<usize> {
    // On an `Option`, `?` returns `None` early.
    let w = words.first()? in
    Some (w.len() + 1)
}
}

pub fn run() {
    println!("add_text \"2\" \" 40 \" = {:?}", add_text("2", " 40 "));
    println!(
        "add_text \"2\" \"x\" is an Err: {}",
        add_text("2", "x").is_err()
    );
    println!("doubled \"21\" = {:?}", doubled("21"));
    println!("first_len [\"tea\"] = {:?}", first_len(&["tea"]));
    println!("first_len [] = {:?}", first_len(&[]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results() {
        assert_eq!(add_text("2", " 40 "), Ok(42));
        assert!(add_text("2", "x").is_err());
        assert!(add_text("two", "40").is_err());
        assert_eq!(doubled("21"), Ok(42));
        assert!(doubled("").is_err());
    }

    #[test]
    fn options() {
        assert_eq!(first_len(&["tea"]), Some(4));
        assert_eq!(first_len(&[]), None);
    }
}

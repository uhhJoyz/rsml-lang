// Toy 102: lifetimes, `&'a str` and `'static`.
//
// What it is: a `'name` right after `&` is a Rust lifetime, not a type
// variable: `(s : &'static str)` is `{ let typed: &'static str = s; typed }`.
// Lifetimes are declared where Rust declares them, in the signature of
// an `rsml!` function (`fn first_word<'a>(..)`), and the rsml code
// inside may use them.
//
// When to use it: in annotations inside a function that has lifetime
// parameters, and for `'static` values.
//
// How to write it:
// * As in Rust: `&'a str`, `&'static str`, `Vec<&'a str>`.
// * The gotcha: a `let rec` cannot declare one. In
//   `let rec longest (a : &'a str) (b : &'a str) : &'a str = ..` the
//   nested `fn` uses `'a` without declaring it, and rustc rejects it
//   ("use of undeclared lifetime name `'a`"). Write such a function as
//   an `rsml!` item, with `<'a>` in its signature, as `longest` below.
//
// Reference: [Types](../../docs/notation.md#types)

use rsml::{ml, rsml};

pub const SUMMARY: &str = "Lifetimes: `(s : &'a str)` in a `fn f<'a>`, `&'static str`";

rsml! {
/// The longer of two words; the result borrows from both.
pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() then a else b
}

/// The first word of a text, borrowed from it.
pub fn first_word<'a>(text: &'a str) -> &'a str {
    // `'a` is the signature's: the annotation states what the
    // `split_whitespace` items borrow from.
    let words : Vec<&'a str> = text.split_whitespace().collect() in
    words |> .first() |> .copied() |> .unwrap_or("")
}
}

/// A greeting from a fixed list: `'static` text lives for the whole program.
pub fn greeting(formal: bool) -> &'static str { ml! {
    let hello : &'static str = if formal then "Good day" else "Hi" in
    hello
}}

pub fn run() {
    println!(
        "longest \"tea\" \"coffee\" = {:?}",
        longest("tea", "coffee")
    );
    println!("first_word \"rsml rocks\" = {:?}", first_word("rsml rocks"));
    println!("greeting true = {:?}", greeting(true));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_results() {
        assert_eq!(longest("tea", "coffee"), "coffee");
        assert_eq!(first_word("rsml rocks"), "rsml");
        assert_eq!(first_word(""), "");
        assert_eq!(greeting(true), "Good day");
        assert_eq!(greeting(false), "Hi");
    }
}

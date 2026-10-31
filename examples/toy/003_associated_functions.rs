// Toy 003: associated functions, `Vec::with_capacity 10`, `Vec.new ()`.
//
// What it is: a type's functions (Rust's associated functions, such as
// constructors) are applied like any other function. The path to them is
// Rust's, `Type::name`, or the dotted `Type.name` when the type is
// capitalized (toy 059).
//
// When to use it: to build values (`String.from`, `Vec::with_capacity`)
// and to call a method through its type (`i64::pow 2 10`,
// `str::to_uppercase s`), which is handy in pipelines (toy 025).
//
// How to write it: the path, then the arguments. A function without
// arguments takes `()`: `Vec.new ()` (toy 005 has the rules for `()`).
// The path is written without spaces around `::` (toy 058).
//
// Reference: [Application](../../docs/notation.md#application)

use rsml::ml;

pub const SUMMARY: &str =
    "Type functions: `Vec::with_capacity 10`, `Vec.new ()`, `String.from \"a\"`";

/// A greeting, built with `String`'s functions.
pub fn greeting(name: &str) -> String { ml! {
    // `String.from "Hello, "` is `String::from("Hello, ")`.
    let s = String.from "Hello, " in
    s + name
}}

/// An empty buffer with room for `n` lines.
pub fn buffer(n: usize) -> Vec<String> { ml! {
    // `Vec::with_capacity n` is `Vec::with_capacity(n)`.
    Vec::with_capacity n
}}

/// An empty list: `()` after a function means "no arguments".
pub fn no_lines() -> Vec<String> { ml! {
    // `Vec.new ()` is `Vec::new()`.
    Vec.new ()
}}

/// A method called through its type takes the receiver first.
pub fn kibibytes(k: u32) -> i64 { ml! {
    // `i64::pow 2 10` is `i64::pow(2, 10)`: the method `pow`, called
    // through its type, takes the receiver first.
    k as i64 * i64::pow 2 10
}}

pub fn shout(word: &str) -> String { ml! {
    // `str::to_uppercase word` is `str::to_uppercase(word)`.
    str::to_uppercase word
}}

pub fn run() {
    println!("greeting \"Ada\" = {:?}", greeting("Ada"));
    println!("buffer 8 has room for {} lines", buffer(8).capacity());
    println!("no_lines () = {:?}", no_lines());
    println!("kibibytes 3 = {}", kibibytes(3));
    println!("shout \"hey\" = {:?}", shout("hey"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors() {
        assert_eq!(greeting("Ada"), "Hello, Ada");
        assert!(buffer(8).capacity() >= 8);
        assert!(no_lines().is_empty());
    }

    #[test]
    fn methods_through_their_type() {
        assert_eq!(kibibytes(3), 3072);
        assert_eq!(shout("hey"), "HEY");
    }
}

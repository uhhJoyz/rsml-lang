// Toy 042: typed placeholders, `(_ : &str)`.
//
// What it is: `(_ : t)` is a placeholder whose parameter has the type
// `t`. Rust must know a closure parameter's type before a method is
// called on it; inside `.map(..)` the iterator says it, but a function
// stored with `let`, or returned, has no such context.
//
// When to use it: when rustc says "type annotations needed" about a
// placeholder function that calls methods.
//
// How to write it: `(_ : &str)` where the `_` would go.
// * Alone in its parentheses, it belongs to the expression around it
//   (toy 040): `(_ : &str).len()` is `move |_0: &str| _0.len()`.
// * With a method as a function (toy 047): `.len (_ : &str)` is
//   `move |_0: &str| _0.len()` too.
// * `_N` takes a type the same way: `(_0 : &str)`.
//
// Reference: [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-)

use rsml::ml;

pub const SUMMARY: &str = "`(_ : &str).trim().len()`: a placeholder with a type";

/// Initials of a name, with a stored helper that needs its type.
pub fn initials(first: &str, last: &str) -> String { ml! {
    // `(_ : &str).chars().next()` is
    // `move |_0: &str| _0.chars().next()`. Without the type, rustc
    // could not look up `chars` when the closure is made.
    let initial = (_ : &str).chars().next() in
    format!("{}.{}.", initial first |> .unwrap_or('?'), initial last |> .unwrap_or('?'))
}}

/// A function that measures a text without the spaces around it.
pub fn measure() -> impl Fn(&str) -> usize { ml! {
    // `(_ : &str).trim().len()` is `move |_0: &str| _0.trim().len()`.
    (_ : &str).trim().len()
}}

/// A function that counts the words of a text: a typed `_0`.
pub fn word_counter() -> impl Fn(&str) -> usize { ml! {
    // `(_0 : &str).split_whitespace().count()` is
    // `move |_0: &str| _0.split_whitespace().count()`.
    (_0 : &str).split_whitespace().count()
}}

pub fn run() {
    println!(
        "initials \"Grace\" \"Hopper\" = {:?}",
        initials("Grace", "Hopper")
    );
    println!("measure () \"  tea \" = {}", measure()("  tea "));
    println!(
        "word_counter () \"tea for two\" = {}",
        word_counter()("tea for two")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_parameters() {
        assert_eq!(initials("Grace", "Hopper"), "G.H.");
        assert_eq!(initials("", "Hopper"), "?.H.");
        assert_eq!(measure()("  tea "), 3);
        assert_eq!(word_counter()("tea for two"), 3);
    }
}

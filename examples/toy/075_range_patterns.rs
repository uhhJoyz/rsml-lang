// Toy 075: range patterns, `'a' .. 'z'` and `1..=3`, both inclusive.
//
// What it is: a pattern that matches every value from one literal to
// another, both ends included: OCaml's `'a' .. 'z'`, or Rust's `1..=3`.
//
// When to use it: for classes of characters and for bands of numbers.
//
// How to write it:
// * Both ends are literals: numbers (negative ones too) or characters.
// * In a pattern `..` includes its end, as in OCaml, however it is
//   spaced: `1..3` there matches 3. In an expression `..` is Rust's range,
//   which stops before its end (toy 088): `(1..3).count()` is 2.
// * Both ends are needed:
//   `match n with ..=0 -> 0 | _ -> 1` is an error: "expected a pattern",
//   and `match n with 90.. -> 0 | _ -> 1` is an error: "expected the end
//   of the range". Use a guard there (toy 074).
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str = "Range patterns: `'a' .. 'z'`, `1..=3`, inclusive (unlike `..` in code)";

/// What kind of character `c` is.
pub fn kind(c: char) -> &'static str {
    ml! {
        // `match c with 'a' .. 'z' -> 1 | _ -> 0` is
        // `match c { 'a'..='z' => 1, _ => 0, }`.
        match c with 'a' .. 'z' | 'A' .. 'Z' -> "letter" | '0'..='9' -> "digit" | _ -> "other"
    }
}

/// A band of temperatures, in degrees.
pub fn band(t: i32) -> &'static str {
    ml! {
        // A pattern range includes its end:
        // `match n with 1..3 -> 1 | _ -> 0` is `match n { 1..=3 => 1, _ => 0, }`.
        match t with -273 .. -1 -> "freezing" | 0 .. 15 -> "cold" | 16 .. 25 -> "mild" | _ -> "hot"
    }
}

/// The number of letters from `a` to `e` in an expression range, which
/// stops before its end.
pub fn letters_before_e() -> usize {
    ml! { ('a'..'e').count() }
}

pub fn run() {
    println!(
        "kind 'q' = {}, kind '7' = {}, kind '!' = {}",
        kind('q'),
        kind('7'),
        kind('!')
    );
    println!(
        "band -3 = {}, band 15 = {}, band 25 = {}, band 30 = {}",
        band(-3),
        band(15),
        band(25),
        band(30)
    );
    println!("('a'..'e').count() = {} (a, b, c, d)", letters_before_e());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_of_characters() {
        assert_eq!(kind('q'), "letter");
        assert_eq!(kind('Z'), "letter");
        assert_eq!(kind('7'), "digit");
        assert_eq!(kind('!'), "other");
    }

    #[test]
    fn both_ends_are_included() {
        assert_eq!(band(-273), "freezing");
        assert_eq!(band(-1), "freezing");
        assert_eq!(band(15), "cold");
        assert_eq!(band(25), "mild");
        assert_eq!(band(30), "hot");
    }

    #[test]
    fn expression_ranges_stop_before_the_end() {
        assert_eq!(letters_before_e(), 4);
        assert_eq!(ml! { (1..3).count() }, 2);
    }
}

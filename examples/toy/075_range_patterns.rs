// Toy 075: range patterns, `'a'..='z'` and `0..10`, as in Rust.
//
// What it is: a pattern that matches every value from one literal to
// another: `..=` includes its end and `..` leaves it out, as Rust's range
// patterns do.
//
// When to use it: for classes of characters and for bands of numbers.
//
// How to write it:
// * Both ends are literals: numbers (negative ones too) or characters.
// * `..` stops before its end however it is spaced, in a pattern as in an
//   expression (toy 088): `1..3` matches 1 and 2, as `(1..3).count()` is 2.
//   OCaml's `'a' .. 'z'` would include `'z'`; rsml follows Rust.
// * Both ends are needed:
//   `match n with ..=0 -> 0 | _ -> 1` is an error: "expected a pattern",
//   and `match n with 90.. -> 0 | _ -> 1` is an error: "expected the end
//   of the range". Use a guard there (toy 074).
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str =
    "Range patterns: `'a'..='z'` includes its end, `0..10` leaves it out, as in Rust";

/// What kind of character `c` is.
pub fn kind(c: char) -> &'static str {
    ml! {
        // `match c with 'a'..='z' -> 1 | _ -> 0` is
        // `match c { 'a'..='z' => 1, _ => 0, }`.
        match c with 'a'..='z' | 'A'..='Z' -> "letter" | '0'..='9' -> "digit" | _ -> "other"
    }
}

/// A band of temperatures, in degrees.
pub fn band(t: i32) -> &'static str {
    ml! {
        // A pattern range leaves its end out, spaced or not:
        // `match n with 1 .. 3 -> 1 | _ -> 0` is `match n { 1..3 => 1, _ => 0, }`.
        match t with -273..0 -> "freezing" | 0..16 -> "cold" | 16..26 -> "mild" | _ -> "hot"
    }
}

/// The number of letters from `a` to `e` in an expression range, which
/// stops before its end too.
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
        "band -1 = {}, band 0 = {}, band 16 = {}, band 26 = {}",
        band(-1),
        band(0),
        band(16),
        band(26)
    );
    println!("('a'..'e').count() = {} (a, b, c, d)", letters_before_e());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_of_characters() {
        assert_eq!(kind('q'), "letter");
        assert_eq!(kind('z'), "letter");
        assert_eq!(kind('Z'), "letter");
        assert_eq!(kind('7'), "digit");
        assert_eq!(kind('!'), "other");
    }

    #[test]
    fn the_end_of_a_dotted_range_is_left_out() {
        assert_eq!(band(-273), "freezing");
        assert_eq!(band(-1), "freezing");
        assert_eq!(band(0), "cold");
        assert_eq!(band(15), "cold");
        assert_eq!(band(16), "mild");
        assert_eq!(band(25), "mild");
        assert_eq!(band(26), "hot");
        assert_eq!(ml! { match 3 with 1..3 -> 1 | _ -> 0 }, 0);
    }

    #[test]
    fn expression_ranges_stop_before_the_end() {
        assert_eq!(letters_before_e(), 4);
        assert_eq!(ml! { (1..3).count() }, 2);
    }
}

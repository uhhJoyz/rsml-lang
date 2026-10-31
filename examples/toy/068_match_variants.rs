// Toy 068: matching variants, `match e with C x -> .. | D -> ..`.
//
// What it is: `match` takes a value and a list of cases, `pattern -> result`,
// separated by `|`. A constructor pattern names a variant and the patterns
// of its fields: `Some x`, `None`, `Shape::Circle r`, `Shape::Rect (w, h)`.
//
// When to use it: to take apart an `Option`, a `Result` or your own enum.
// It is Rust's `match`, so every variant must be covered.
//
// How to write it:
// * `match e with` and the cases; a `|` may also come before the first.
// * A variant with fields takes their patterns ML style: one pattern, or
//   a tuple of them for several fields (`Shape::Rect (w, h)`). The path
//   may be Rust's `Shape::Rect` or the dotted `Shape.Rect` (toy 059).
// * `Some _` matches any `Some`, whatever its fields.
// * Matching a reference (`&Shape`) binds references: `r` has type `&f64`,
//   and `!r` reads it (toy 092).
// * A `match` in a case needs no parentheses when a later case cannot be
//   the inner one's: its pattern is of another type (the `A` of `A::Y`,
//   `Option` for `None`, an integer for `0`), or an inner case before it
//   already takes every value it matches (a `_` does). That case goes
//   back to the outer `match`:
//   `match a with A::X -> match b with B::P -> 1 | B::Q -> 2 | A::Y -> 3`
//   is `match a { A::X => match b { B::P => 1, B::Q => 2, }, A::Y => 3, }`.
//   Without such a sign (the same type at both levels, or a `_` that fits
//   either), the case stays with the inner `match`: put that one in
//   parentheses or `begin .. end`.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str = "Variants: `Some x`, `None`, `Shape::Rect (w, h)` in `match .. with`";

#[derive(Debug)]
pub enum Shape {
    Circle(f64),
    Rect(f64, f64),
    Dot,
}

/// The area of a shape, by its variant.
pub fn area(s: &Shape) -> f64 { ml! {
    // `Shape.Rect` is `Shape::Rect`, and `(w, h)` its two fields.
    match s with
    | Shape::Circle r -> 3.0 *. !r *. !r
    | Shape.Rect (w, h) -> !w *. !h
    | Shape::Dot -> 0.0
}}

/// A price, or the default when there is none.
pub fn price_or(o: Option<u32>, default: u32) -> u32 { ml! {
    // `match o with Some p -> p | None -> default` is
    // `match o { Some(p) => p, None => default, }`.
    match o with Some p -> p | None -> default
}}

/// Whether a parse worked, without looking at what it found.
pub fn parsed(text: &str) -> bool { ml! {
    // `match r with Ok _ -> true | Err _ -> false` is
    // `match r { Ok(..) => true, Err(..) => false, }`.
    let r = text.parse::<i64>() in
    match r with Ok _ -> true | Err _ -> false
}}

/// When a parcel arrives, from the days it still needs, if known.
pub fn arrival(days: Option<u32>) -> &'static str { ml! {
    // `None` cannot be a case of the inner `match`, on an integer:
    // `match o with Some d -> match d with 0 -> 1 | _ -> 2 | None -> 3` is
    // `match o { Some(d) => match d { 0 => 1, _ => 2, }, None => 3, }`.
    match days with
    | Some d ->
        match d with
        | 0 -> "today"
        | 1 -> "tomorrow"
        | _ -> "later"
    | None -> "unknown"
}}

pub fn run() {
    for s in [Shape::Circle(1.0), Shape::Rect(2.0, 3.0), Shape::Dot] {
        println!("area {:?} = {}", s, area(&s));
    }
    println!(
        "price_or (Some 5) 9 = {}, price_or None 9 = {}",
        price_or(Some(5), 9),
        price_or(None, 9)
    );
    println!(
        "parsed \"42\" = {}, parsed \"4x\" = {}",
        parsed("42"),
        parsed("4x")
    );
    for days in [Some(0), Some(1), Some(5), None] {
        println!("arrival {:?} = {:?}", days, arrival(days));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes() {
        assert_eq!(area(&Shape::Circle(1.0)), 3.0);
        assert_eq!(area(&Shape::Rect(2.0, 3.0)), 6.0);
        assert_eq!(area(&Shape::Dot), 0.0);
    }

    #[test]
    fn options_and_results() {
        assert_eq!(price_or(Some(5), 9), 5);
        assert_eq!(price_or(None, 9), 9);
        assert!(parsed("42"));
        assert!(!parsed("4x"));
    }

    #[test]
    fn a_nested_match() {
        assert_eq!(arrival(Some(0)), "today");
        assert_eq!(arrival(Some(1)), "tomorrow");
        assert_eq!(arrival(Some(5)), "later");
        assert_eq!(arrival(None), "unknown");
    }
}

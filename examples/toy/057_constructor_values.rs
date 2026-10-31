// Toy 057: a constructor written against its `(` is one value,
// `Some(x).unwrap()`.
//
// What it is: a constructor with its `(` right after it, Rust style, is one
// value, like a Rust call: what follows (`.method()`, `.field`, `.0`, `[i]`,
// `?`) applies to the value it builds, and as an argument it is one
// argument. Spaced, it is an ML-style application, where `.` binds
// tighter.
//
// When to use it: to call a method on a value you just built, or to pass
// a built value without extra parentheses.
//
// How to write it: no space before the `(`.
// * `Some(x).unwrap_or(d)` builds the option, then calls `unwrap_or`.
// * `Some (x).unwrap()` is `Some(x.unwrap())`: spaced, the method goes
//   with `x`.
// * `f Some(x) y` passes two arguments; `f Some (x) y` is
//   `f(Some, x, y)`, three.
// * The argument keeps the rules of toy 056: `Some(1, 2)` is `Some((1, 2))`.
// rsml-fmt keeps the space, or its absence, as written.
//
// Reference: [Rust-style calls and indexing](../../docs/notation.md#rust-style-calls-and-indexing-fa-b-vi)

use rsml::ml;

pub const SUMMARY: &str = "`Some(x).unwrap()` vs `Some (x).unwrap()`; `f Some(x) y`";

pub enum Shape {
    Circle(f64),
    Rect(f64, f64),
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => 3.0 * r * r,
            Shape::Rect(w, h) => w * h,
        }
    }
}

/// Metres, as a tuple struct.
pub struct Meters(f64);

/// A name, or "guest" when it is empty: the methods of an option built
/// on the spot.
pub fn name_or_guest(name: &str) -> String { ml! {
    // `Some(name).filter(..)` builds the option, then calls `filter`.
    Some(name).filter(fun n -> not (n.is_empty())).unwrap_or("guest").to_string()
}}

/// The area of a rectangle and a circle, by methods of values built here.
pub fn areas(w: f64, h: f64) -> f64 { ml! {
    // `Shape::Rect(w, h).area()` is `Shape::Rect(w, h).area()`.
    Shape::Rect(w, h).area() +. Shape::Circle(1.0).area()
}}

/// The field of a tuple struct built on the spot.
pub fn in_metres(v: f64) -> f64 { ml! {
    // `Meters(v).0` is `Meters(v).0`.
    Meters(v).0
}}

fn describe(label: Option<&str>, n: u32) -> String {
    format!("{} x{n}", label.unwrap_or("?"))
}

/// As an argument, `Some(x)` is one argument.
pub fn described(label: &str) -> String { ml! {
    // `describe Some(label) 3` is `describe(Some(label), 3)`.
    describe Some(label) 3
}}

/// Spaced, the method call binds first: it goes with the argument.
pub fn trimmed(word: &str) -> Option<&str> { ml! {
    // `Some (word).map(str::trim)` would be `Some(word.map(str::trim))`;
    // here `Some (word.trim())` is `Some(word.trim())`.
    Some (word.trim())
}}

pub fn run() {
    println!("name_or_guest \"ada\" = {:?}", name_or_guest("ada"));
    println!("name_or_guest \"\" = {:?}", name_or_guest(""));
    println!("areas 2.0 3.0 = {}", areas(2.0, 3.0));
    println!("in_metres 1.5 = {}", in_metres(1.5));
    println!("described \"tea\" = {:?}", described("tea"));
    println!("trimmed \" hi \" = {:?}", trimmed(" hi "));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods_of_built_values() {
        assert_eq!(name_or_guest("ada"), "ada");
        assert_eq!(name_or_guest(""), "guest");
        assert_eq!(areas(2.0, 3.0), 9.0);
        assert_eq!(in_metres(1.5), 1.5);
    }

    #[test]
    fn one_argument() {
        assert_eq!(described("tea"), "tea x3");
        assert_eq!(trimmed(" hi "), Some("hi"));
    }

    #[test]
    fn spaced_the_method_goes_with_x() {
        let x: Option<i64> = Some(4);
        // `Some (x).unwrap()` is `Some(x.unwrap())`.
        assert_eq!(ml! { Some (x).unwrap() }, Some(4));
    }
}

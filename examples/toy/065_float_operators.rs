// Toy 065: float operators, `+.`, `*.`, `**`.
//
// What it is: the dotted operators `+.`, `-.`, `*.` and `/.` are for
// floats. Rust's `+`, `-`, `*` and `/` work on every number type, so the
// dotted ones are the same as the plain ones: `a +. b *. c` is
// `a + b * c`. `**` is the power of two floats, Rust's `f64::powf`.
//
// When to use it: write whichever you like; the dotted ones read like ML,
// the plain ones like Rust.
//
// How to write it:
// * `2.0 ** 3.0` is `::core::primitive::f64::powf(2.0, 3.0)`: both sides
//   are `f64`. For integers, use `i64::pow` (toy 003).
// * Rust does not convert between number types: `n as f64 *. 2.0` is
//   `(n as f64) * 2.0` (toy 123).
// * A float literal needs its dot or exponent: `2.0`, not `2`.
//
// Reference: [Operators that become calls](../../docs/notation.md#operators-that-become-calls)

use rsml::ml;

pub const SUMMARY: &str = "`+.` `-.` `*.` `/.` are `+ - * /`; `**` calls `f64::powf`";

/// The amount after `years` of yearly interest at `rate` percent.
pub fn compound(amount: f64, rate: f64, years: f64) -> f64 { ml! {
    // `amount *. (1.0 +. rate /. 100.0) ** years` is
    // `amount * ::core::primitive::f64::powf(1.0 + (rate / 100.0), years)`.
    amount *. (1.0 +. rate /. 100.0) ** years
}}

/// The distance between two points.
pub fn distance((x1, y1): (f64, f64), (x2, y2): (f64, f64)) -> f64 { ml! {
    ((x2 -. x1) ** 2.0 +. (y2 -. y1) ** 2.0) ** 0.5
}}

/// The average of whole-number scores, as a float.
pub fn average(scores: &[u32]) -> f64 { ml! {
    let total = scores |> .iter() |> .sum::<u32>() in
    total as f64 /. scores.len() as f64
}}

pub fn run() {
    println!(
        "compound 100.0 10.0 2.0 = {:.2}",
        compound(100.0, 10.0, 2.0)
    );
    println!(
        "distance (0, 0) (3, 4) = {}",
        distance((0.0, 0.0), (3.0, 4.0))
    );
    println!("average [3; 4] = {}", average(&[3, 4]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_arithmetic() {
        assert!((compound(100.0, 10.0, 2.0) - 121.0).abs() < 1e-9);
        assert_eq!(distance((0.0, 0.0), (3.0, 4.0)), 5.0);
        assert_eq!(average(&[3, 4]), 3.5);
    }

    #[test]
    fn dotted_and_plain_are_the_same() {
        let (a, b, c) = (1.5, 2.0, 4.0);
        assert_eq!(ml! { a +. b *. c }, ml! { a + b * c });
        assert_eq!(ml! { 2.0 ** 10.0 }, 1024.0);
    }
}

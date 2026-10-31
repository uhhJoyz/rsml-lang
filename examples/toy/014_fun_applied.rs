// Toy 014: a `fun` applied on the spot is a `let`, not a closure.
//
// What it is: `(fun x -> body) value` and `value |> fun x -> body` name
// `value` `x` for the body. The macro writes that as a `let`: no closure
// is made or called.
//
// When to use it: to name a value in the middle of a pipeline, where a
// `let .. in` would break the flow.
//
// How to write it:
// * `(fun x -> x + 1) 2` is `{ let x = 2; x + 1 }`.
// * `(fun x y -> x * y) 2 3` is `{ let (x, y) = (2, 3); x * y }`.
// * With an argument missing, as in `(fun x y -> x * y) 3`, it is a
//   partial application: a closure (toy 018).
// * The body of a `fun` reaches as far right as it can:
//   `x |> fun y -> y * 2 |> g` is `{ let y = x; g(y * 2) }`. Put the
//   `fun` in parentheses to end it: `(x |> fun y -> y * 2) |> g`.
//
// Reference: [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function)

use rsml::ml;

pub const SUMMARY: &str = "`(fun x -> e) v` and `v |> fun x -> e` are `let`s";

/// A price with a 20% tax, the price named on the spot.
pub fn with_tax(price: u32) -> u32 { ml! {
    // `(fun p -> p + p / 5) price` is `let p = price in p + p / 5`.
    (fun p -> p + p / 5) price
}}

/// Celsius to Fahrenheit, rounded: the value is piped into a `fun`.
pub fn fahrenheit(celsius: f64) -> i64 { ml! {
    // `celsius |> fun c -> c * 9.0 / 5.0 + 32.0` is
    // `let c = celsius in c * 9.0 / 5.0 + 32.0`.
    (celsius |> fun c -> c * 9.0 / 5.0 + 32.0).round() as i64
}}

/// Three times `n`, through a partial application of a `fun`.
pub fn tripled(n: u32) -> u32 { ml! {
    let triple = (fun x y -> x * y) 3 in
    triple n
}}

pub fn run() {
    println!("with_tax 100 = {}", with_tax(100));
    println!(
        "fahrenheit 100.0 = {}, fahrenheit 37.0 = {}",
        fahrenheit(100.0),
        fahrenheit(37.0)
    );
    println!("tripled 5 = {}", tripled(5));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applied_on_the_spot() {
        assert_eq!(with_tax(100), 120);
        assert_eq!(fahrenheit(100.0), 212);
        assert_eq!(fahrenheit(37.0), 99);
        assert_eq!(tripled(5), 15);
    }

    #[test]
    fn the_header_examples() {
        assert_eq!(ml! { (fun x -> x + 1) 2 }, 3);
        assert_eq!(ml! { (fun x y -> x * y) 2 3 }, 6);
        let g = |n: i32| n - 1;
        assert_eq!(ml! { 5 |> fun y -> y * 2 |> g }, 9);
        assert_eq!(ml! { (5 |> fun y -> y * 2) |> g }, 9);
    }
}

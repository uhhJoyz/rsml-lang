// Toy 109: whole functions in rsml, `rsml! { fn .. }`.
//
// What it is: `rsml! { .. }` holds Rust items whose function bodies are
// rsml code. The signature, attributes, doc comments and visibility are
// Rust and pass through untouched; only what is between the body's braces
// is rsml code, so the body needs no `ml! { }`.
//
// When to use it: for functions whose bodies are rsml code; `ml! { }`
// inside a Rust function for a few lines of rsml in Rust code.
//
// How to write it:
// * The body is one rsml expression, its value the result:
//   `rsml! { fn area(w: f64, h: f64) -> f64 { w *. h } }` is
//   `fn area(w: f64, h: f64) -> f64 { w * h }`.
// * Parameters are names bound in the body, so with a parameter `h`,
//   `h::t` is a cons, not a path:
//   `rsml! { fn push_front(h: i64, t: Vec<i64>) -> Vec<i64> { h::t } }` is
//   `fn push_front(h: i64, t: Vec<i64>) -> Vec<i64> { ::rsml::__rt::cons(h, t) }`.
// * Items that are not functions (`struct`, `const`, `use`) pass through.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "`rsml! { fn .. }`: Rust signatures, rsml bodies";

rsml! {
/// The smallest price that counts as a sale item, in cents.
pub const SALE_MIN: u64 = 500;

/// A price after a discount in percent, rounded down to a cent.
#[must_use]
pub fn discounted(cents: u64, percent: u64) -> u64 {
    cents - cents * percent / 100
}

/// The prices of at least `SALE_MIN`, 10% off: one pipeline is the body.
pub fn sale(prices: &[u64]) -> Vec<u64> {
    prices
    |> .iter()
    |> .copied()
    // `!p` reads the `&u64` that `filter` lends.
    |> .filter(fun p -> !p >= SALE_MIN)
    |> .map(fun p -> discounted p 10)
    |> .collect()
}

/// A body with `let`s and an `if`, still one expression.
pub fn shipping(total: u64) -> u64 {
    let free_from = 5000 in
    let flat = 450 in
    if total >= free_from then 0 else flat
}
}

pub fn run() {
    println!("discounted 1000 25 = {}", discounted(1000, 25));
    println!("sale [|200; 500; 1999|] = {:?}", sale(&[200, 500, 1999]));
    println!(
        "shipping 4999 = {}, shipping 5000 = {}",
        shipping(4999),
        shipping(5000)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocaml_bodies() {
        assert_eq!(discounted(1000, 25), 750);
        assert_eq!(sale(&[200, 500, 1999]), [450, 1800]);
        assert_eq!(shipping(4999), 450);
        assert_eq!(shipping(5000), 0);
        assert_eq!(SALE_MIN, 500);
    }
}

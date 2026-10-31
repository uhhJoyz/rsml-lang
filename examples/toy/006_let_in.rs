// Toy 006: local bindings, `let x = e in body`.
//
// What it is: `let name = value in body` names a value for the body after
// `in`. It is Rust's `let name = value;` followed by the rest of the
// block, and the whole `let .. in` is an expression: its value is the
// body's.
//
// When to use it: to name intermediate results, as in Rust.
//
// How to write it:
// * `let x = e in` and then the body; each `let` has its own `in`.
// * A new `let` of the same name shadows the old one, as in Rust.
// * The name can be a pattern: `let (w, h) = size in ..`.
// * `let x : t = e in ..` gives the type, `let _ = e in ..` drops a value.
// * The Rust habit `let x = e; more` does not work: after `=`, a `;`
//   makes the rest part of the value, and rsml says so (toy 130).
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)

use rsml::ml;

pub const SUMMARY: &str = "`let x = e in body`: shadowing, patterns, types, `let _`";

/// A bill with a 10% tip.
pub fn with_tip(bill: u32) -> u32 { ml! {
    // `let tip = bill / 10 in bill + tip` is
    // `{ let tip = bill / 10; bill + tip }`.
    let tip = bill / 10 in
    bill + tip
}}

/// A price after a 25% discount, then a 20% tax.
pub fn final_price(price: u32) -> u32 { ml! {
    let price = price - price / 4 in
    // This `price` shadows the one above, which shadows the parameter.
    let price = price + price / 5 in
    price
}}

/// The area of a rectangle given as (width, height).
pub fn area(size: (u32, u32)) -> u32 { ml! {
    // `let (w, h) = size in w * h` is `{ let (w, h) = size; w * h }`.
    let (w, h) = size in
    w * h
}}

/// A number read from text, doubled (0 if it is not a number).
pub fn doubled(text: &str) -> i64 { ml! {
    // `let n : i64 = e in n * 2` is `{ let n: i64 = e; n * 2 }`: the
    // type tells `parse` what to make.
    let n : i64 = text.trim().parse().unwrap_or(0) in
    n * 2
}}

/// The second item from the top of a stack.
pub fn second_from_top(mut stack: Vec<u32>) -> Option<u32> { ml! {
    // `let _ = stack.pop() in stack.pop()` is
    // `{ let _ = stack.pop(); stack.pop() }`.
    let _ = stack.pop() in
    stack.pop()
}}

pub fn run() {
    println!("with_tip 50 = {}", with_tip(50));
    println!("final_price 100 = {}", final_price(100));
    println!("area (3, 4) = {}", area((3, 4)));
    println!(
        "doubled \" 21 \" = {}, doubled \"x\" = {}",
        doubled(" 21 "),
        doubled("x")
    );
    println!(
        "second_from_top [1; 2; 3] = {:?}",
        second_from_top(vec![1, 2, 3])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings() {
        assert_eq!(with_tip(50), 55);
        assert_eq!(final_price(100), 90);
        assert_eq!(area((3, 4)), 12);
    }

    #[test]
    fn typed_and_dropped() {
        assert_eq!(doubled(" 21 "), 42);
        assert_eq!(doubled("x"), 0);
        assert_eq!(second_from_top(vec![1, 2, 3]), Some(2));
    }
}

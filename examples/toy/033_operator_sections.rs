// Toy 033: operator sections, `( + )`, `( * )`, `( - )`.
//
// What it is: an operator in parentheses is a function of its two
// operands. `( + )` is a closure that adds, ready to pass where Rust
// wants a function, such as `fold`.
//
// When to use it: instead of writing `fun a b -> a + b` for an argument.
//
// How to write it: the operator between `(` and `)`, with spaces.
// * `( * )` must keep its spaces: `(*` opens an ML comment.
// * Applied to both operands, a section is just the operation:
//   `( - ) a b` is `a - b`. Applied to one, see toy 034.
//
// Reference: [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators)

use rsml::ml;

pub const SUMMARY: &str = "An operator as a function: `( + )`, `( * )`, `( - )`";

/// The total of the prices.
pub fn total(prices: &[i64]) -> i64 { ml! {
    // `( + )` is `|lhs, rhs| lhs + rhs`; `fold` calls it with the total
    // so far and each `&i64`.
    prices |> .iter() |> .fold(0, ( + ))
}}

/// The product of the factors, 1 for none.
pub fn product(factors: &[i64]) -> i64 { ml! {
    // `( * )` is `|lhs, rhs| lhs * rhs`.
    factors |> .iter() |> .copied() |> .fold(1, ( * ))
}}

/// The change from a payment.
pub fn change(paid: i64, price: i64) -> i64 { ml! {
    // `( - ) paid price` is `paid - price`: no closure at all.
    ( - ) paid price
}}

/// A section is a value: here one of three, picked by a character.
pub fn operation(op: char) -> fn(i64, i64) -> i64 { ml! {
    // None of these closures captures anything, so each one can be the
    // plain function pointer that the return type asks for.
    match op with '+' -> ( + ) | '-' -> ( - ) | _ -> ( * )
}}

/// `a op b`, through the function `operation` picks.
pub fn calculate(op: char, a: i64, b: i64) -> i64 { ml! {
    // `operation op a b` would be `operation(op, a, b)`, three
    // arguments for a function of one: call it first, then its result.
    let f = operation op in
    f a b
}}

pub fn run() {
    println!("total [3; 4; 5] = {}", total(&[3, 4, 5]));
    println!("product [2; 3; 4] = {}", product(&[2, 3, 4]));
    println!("change 20 13 = {}", change(20, 13));
    println!("calculate '-' 9 4 = {}", calculate('-', 9, 4));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_as_arguments() {
        assert_eq!(total(&[3, 4, 5]), 12);
        assert_eq!(total(&[]), 0);
        assert_eq!(product(&[2, 3, 4]), 24);
        assert_eq!(product(&[]), 1);
    }

    #[test]
    fn sections_applied_and_stored() {
        assert_eq!(change(20, 13), 7);
        assert_eq!(calculate('+', 9, 4), 13);
        assert_eq!(calculate('-', 9, 4), 5);
        assert_eq!(calculate('*', 9, 4), 36);
    }
}

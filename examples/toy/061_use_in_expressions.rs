// Toy 061: `use` inside an expression, for that expression only.
//
// What it is: `use path; e` can start any expression (a `let` body, a
// case's body, the inside of parentheses), not only the block: its names
// are usable in `e` alone. It is a Rust block that starts with the
// `use`: `let y = 2 in use std::cmp::max; max y 3` is
// `{ let y = 2; use std::cmp::max; max(y, 3) }`.
//
// When to use it: when only one expression needs the names, so that they
// do not leak into the rest of the code.
//
// How to write it: `use path;` before the expression; the expression
// extends as far as a `let` body does, so put it in parentheses to end it
// early: `(use std::cmp::max; max a b) + 1` is
// `({ use std::cmp::max; max(a, b) }) + 1`.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)
// (and [Paths](../../docs/notation.md#paths))

use rsml::ml;

pub const SUMMARY: &str = "`(use std::cmp::max; max a b) + bonus`: names for one expression";

/// The area of a circle, with `PI` imported for the `let` body.
pub fn area(r: f64) -> f64 { ml! {
    let r2 = r *. r in
    // `use std::f64::consts::PI; PI *. r2` is
    // `{ use std::f64::consts::PI; PI * r2 }`.
    use std::f64::consts::PI;
    PI *. r2
}}

/// The larger of two scores, plus a bonus: the `use` ends at `)`.
pub fn with_bonus(a: u32, b: u32, bonus: u32) -> u32 { ml! {
    // `max` is only known inside the parentheses.
    (use std::cmp::max;
     max a b) + bonus
}}

/// A sign, with `Ordering`'s variants imported in one case.
pub fn sign(x: i32) -> i32 { ml! {
    match x with
    | 0 -> 0
    | _ ->
        use std::cmp::Ordering::*;
        match x.cmp(&0) with Less -> -1 | _ -> 1
}}

pub fn run() {
    println!("area 2.0 = {:.3}", area(2.0));
    println!("with_bonus 7 9 1 = {}", with_bonus(7, 9, 1));
    println!(
        "sign (-4) = {}, sign 0 = {}, sign 9 = {}",
        sign(-4),
        sign(0),
        sign(9)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_for_one_expression() {
        assert_eq!(area(2.0), std::f64::consts::PI * 4.0);
        assert_eq!(with_bonus(7, 9, 1), 10);
        assert_eq!((sign(-4), sign(0), sign(9)), (-1, 0, 1));
    }
}

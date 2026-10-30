// Toy 061: `let open M in e`, a module's names for one expression.
//
// What it is: `let open path in e` opens the module for `e` only, where
// `open path ;;` (toy 060) opens it for the rest of the block. It is a
// Rust block with a glob import:
// `let open std::cmp in max a b` is `{ use std::cmp::*; max(a, b) }`.
//
// When to use it: when only one expression needs the module's names, so
// that they do not leak into the rest of the code.
//
// How to write it: `let open path in` before the expression, like a
// `let`; its body extends as far as a `let` body does, so put it in
// parentheses to end it early: `(let open std::cmp in max a b) + 1` is
// `({ use std::cmp::*; max(a, b) }) + 1`.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)
// (and [Paths](../../docs/notation.md#paths))

use rsml::ml;

pub const SUMMARY: &str = "`let open std::f64::consts in PI *. r *. r`: names for one expression";

/// The area of a circle, with `PI` opened for one expression.
pub fn area(r: f64) -> f64 {
    ml! {
        // `let open std::f64::consts in PI *. r *. r` is
        // `{ use std::f64::consts::*; (PI * r) * r }`.
        let open std::f64::consts in
        PI *. r *. r
    }
}

/// The larger of two scores, plus a bonus: the `open` ends at `)`.
pub fn with_bonus(a: u32, b: u32, bonus: u32) -> u32 {
    ml! {
        // `max` is only known inside the parentheses.
        (let open std::cmp in
         max a b) + bonus
    }
}

/// A clamp written with the two functions of `std::cmp`.
pub fn clamp(low: i32, x: i32, high: i32) -> i32 {
    ml! {
        let open std::cmp in
        min (max low x) high
    }
}

pub fn run() {
    println!("area 2.0 = {:.3}", area(2.0));
    println!("with_bonus 7 9 1 = {}", with_bonus(7, 9, 1));
    println!("clamp 0 42 10 = {}", clamp(0, 42, 10));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opened_for_one_expression() {
        assert_eq!(area(2.0), std::f64::consts::PI * 4.0);
        assert_eq!(with_bonus(7, 9, 1), 10);
        assert_eq!(clamp(0, 42, 10), 10);
        assert_eq!(clamp(0, -1, 10), 0);
    }
}

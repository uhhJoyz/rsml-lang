// Toy 011: the block is a toplevel: `let` without `in`, and `;;`.
//
// What it is: the inside of `ml! { .. }` (and of an `rsml!` function's
// body) is read as a toplevel, a list of phrases. A `let x = e`
// without `in` binds `x` for the rest of the block; `;;` separates
// phrases. The last phrase is the block's value.
//
// When to use it: for a block of several steps, where an `in` after every
// `let` would be noise.
//
// How to write it:
// * One `let` phrase ends where the next `let` starts:
//   `let x = 1 let y = 2 ;; x + y` is `{ let x = 1; let y = 2; x + y }`.
// * An expression after a `let` phrase needs `;;` before it. Without it,
//   the expression continues the value: `let x = 1 x + 1` is
//   `{ let x = (1)(x) + 1; () }`, which rustc rejects.
// * A block that ends with a `let` phrase has the value `()`:
//   `let x = 1` is `{ let x = 1; () }`.
// * `;;` has no space inside: `; ;` is two `;`s, an error (toy 079).
// * `external` (toy 020) and `use` (toy 060) are phrases too.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)

use rsml::ml;

pub const SUMMARY: &str = "Toplevel phrases: `let` without `in`, `;;`, the last is the value";

/// An invoice line, in three named steps.
pub fn invoice(hours: u32, rate: u32) -> String { ml! {
    let subtotal = hours * rate
    let tax = subtotal / 5
    let total = subtotal + tax ;;
    format!("{subtotal} + {tax} tax = {total}")
}}

/// Expressions separated by `;;` run in order; the last is the value.
pub fn chores() -> Vec<&'static str> { ml! {
    let mut finished = Vec.new () ;;
    finished.push("wash") ;;
    finished.push("dry") ;;
    finished
}}

pub fn run() {
    println!("invoice 3 50 = {:?}", invoice(3, 50));
    println!("chores () = {:?}", chores());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrases() {
        assert_eq!(invoice(3, 50), "150 + 30 tax = 180");
        assert_eq!(chores(), ["wash", "dry"]);
    }

    #[test]
    fn the_header_examples() {
        let (x, y) = (5, 6);
        assert_eq!(
            ml! {
            let x = 1
            let y = 2 ;;
            x + y
            },
            3
        );
        assert_eq!((x, y), (5, 6));
    }
}

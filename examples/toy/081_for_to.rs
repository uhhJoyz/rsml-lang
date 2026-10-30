// Toy 081: counting loops, `for i = a to b do .. done`.
//
// What it is: OCaml's counting loop runs its body for `i` from `a` up to
// `b`, both included. It is Rust's `for` over an inclusive range:
// `for i = 0 to n do f i done` is `for i in 0..=n { f(i); }`.
//
// When to use it: to count, when the numbers themselves matter.
//
// How to write it:
// * `b` is included, unlike in a Rust range `a..b` (toy 088): `for i = 1
//   to 3` runs three times, with 1, 2 and 3. When `b < a` it runs no
//   times.
// * The body is a unit block: its value is dropped, and a sequence in it
//   needs no `begin .. end` (it ends at `done`).
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`for i = 1 to n do .. done`: both ends included";

/// A multiplication table row: `n * 1` to `n * 5`.
pub fn row(n: u32) -> Vec<u32> {
    ml! {
        let mut out = Vec.new () in
        for i = 1 to 5 do out.push(n * i) done;
        out
    }
}

/// The sum 1 + 2 + .. + n.
pub fn triangle(n: u32) -> u32 {
    ml! {
        let mut sum = 0 in
        for i = 1 to n do sum <- sum + i done;
        sum
    }
}

pub fn run() {
    println!("row 3 = {:?}", row(3));
    println!("triangle 4 = {}, triangle 0 = {}", triangle(4), triangle(0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ends() {
        assert_eq!(row(3), [3, 6, 9, 12, 15]);
        assert_eq!(triangle(4), 10);
        // `for i = 1 to 0` runs no times.
        assert_eq!(triangle(0), 0);
    }
}

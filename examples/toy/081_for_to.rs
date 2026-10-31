// Toy 081: counting loops, `for i = a to b do .. done` and `til`.
//
// What it is: a counting loop runs its body for `i` from `a` up to `b`,
// both included. It is Rust's `for` over an inclusive range:
// `for i = 0 to n do f i done` is `for i in 0..=n { f(i); }`. With `til`
// instead of `to`, `b` is left out, as in a Rust range `a..b` (toy 088):
// `for i = 0 til n do f i done` is `for i in 0..n { f(i); }`.
//
// When to use it: to count, when the numbers themselves matter; `til` for
// indices, which stop before a length.
//
// How to write it:
// * `to` includes `b`: `for i = 1 to 3` runs three times, with 1, 2 and 3.
//   `til` does not: `for i = 1 til 3` runs with 1 and 2. When there is
//   nothing between `a` and `b` it runs no times.
// * The body is a unit block: its value is dropped, and a sequence in it
//   needs no `begin .. end` (it ends at `done`).
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`for i = 1 to n do .. done` includes `n`, `til` stops before it";

/// A multiplication table row: `n * 1` to `n * 5`.
pub fn row(n: u32) -> Vec<u32> { ml! {
    let mut out = Vec.new () in
    for i = 1 to 5 do out.push(n * i) done;
    out
}}

/// The sum 1 + 2 + .. + n.
pub fn triangle(n: u32) -> u32 { ml! {
    let mut sum = 0 in
    for i = 1 to n do sum <- sum + i done;
    sum
}}

/// Every index of `xs`: `til` stops before the length.
pub fn indices(xs: &[char]) -> Vec<usize> { ml! {
    let mut out = Vec.new () in
    for i = 0 til xs.len() do out.push(i) done;
    out
}}

pub fn run() {
    println!("row 3 = {:?}", row(3));
    println!("triangle 4 = {}, triangle 0 = {}", triangle(4), triangle(0));
    println!("indices ['a', 'b', 'c'] = {:?}", indices(&['a', 'b', 'c']));
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

    #[test]
    fn til_leaves_the_end_out() {
        assert_eq!(indices(&['a', 'b', 'c']), [0, 1, 2]);
        assert_eq!(indices(&[]), Vec::<usize>::new());
    }
}

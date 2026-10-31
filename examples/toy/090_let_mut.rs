// Toy 090: mutable locals, `let mut x = e in ..` and `x <- e`.
//
// What it is: Rust's `let mut`, with the assignment arrow `<-`: `x <- e`
// stores `e` in `x`. It makes the same local as `ref` (toy 089), read by
// its name instead of with `!`.
//
// When to use it: when the code reads better with plain names than with
// `!r` everywhere, as in a loop that keeps a best value so far.
//
// How to write it:
// * The `let` needs `mut`: `x <- 2` after `let x = 1 in` is rustc's
//   "cannot assign twice to immutable variable".
// * rsml has no `+=`; write `x <- x + e`. rsml allows clippy's
//   `assign_op_pattern` lint on such a statement, as nothing else could
//   be written.
// * `<-` is also how you write through `*` (toy 091): `*c <- *c + 1`.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)

use rsml::ml;

pub const SUMMARY: &str = "`let mut n = 0 in n <- n + 1`: assignment with `<-`";

/// A balance after deposits and withdrawals; it never goes below zero.
pub fn balance(moves: &[i64]) -> i64 { ml! {
    // `let mut n = 0 in n <- 10 - n; n` is `{ let mut n = 0; n = 10 - n; n }`.
    let mut money = 0 in
    for m in moves do money <- std::cmp::max 0 (money + !m) done;
    money
}}

/// The first of the longest words.
pub fn longest<'a>(words: &[&'a str]) -> &'a str { ml! {
    let mut best = "" in
    for w in words do if w.len() > best.len() then best <- !w done;
    best
}}

/// How many doublings it takes to reach `target`, starting from 1.
pub fn doublings(target: u64) -> u32 { ml! {
    let mut n = 1 in
    let mut steps = 0 in
    while n < target do
      // `n <- n * 2` is `{ #[allow(clippy::assign_op_pattern)] { n = n * 2; } }`.
      n <- n * 2;
      steps <- steps + 1
    done;
    steps
}}

// `x += 1` is an error: "rsml has no `+=`: write `x <- x + e` (for a
// `ref`: `r := !r + e`, or `incr r`), or define the operator with
// `let ( += ) a b = ..`"

pub fn run() {
    println!("balance [50; -80; 30] = {}", balance(&[50, -80, 30]));
    println!(
        "longest [fig; pear; plum] = {:?}",
        longest(&["fig", "pear", "plum"])
    );
    println!("doublings 1000 = {}", doublings(1000));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignments() {
        assert_eq!(balance(&[50, -80, 30]), 30);
        assert_eq!(balance(&[10, 5]), 15);
        assert_eq!(longest(&["fig", "pear", "plum"]), "pear");
        assert_eq!(doublings(1000), 10);
        assert_eq!(doublings(1), 0);
    }

    #[test]
    fn assignment_from_the_old_value() {
        assert_eq!(
            ml! {
            let mut n = 0 in
            n <- 10 - n;
            n
            },
            10
        );
    }
}

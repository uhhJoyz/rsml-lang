// Toy 076: reference patterns, `&p`.
//
// What it is: Rust's `&p` pattern, which matches a reference and binds
// what it points to: matching `&3` with `&x` binds `x` to 3. rsml takes
// it as written, in `match`, `let`, `for` and `fun`.
//
// When to use it: to copy numbers out of the references that iterators
// over a collection give (`for &x in &xs`), instead of writing `!x`
// (toy 092) at each use.
//
// How to write it:
// * `&x`, `&&x` (two references), `(&a, &b)`, `Some &x`.
// * A `fun` parameter that is more than a name goes in parentheses:
//   `fun (&x) -> ..`, while `xs.iter().map(fun &x -> x * 2)` is an error:
//   "expected a parameter after `fun`".
// * The value must be `Copy` to be bound this way, as in Rust.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str = "Reference patterns: `for &x in ..`, `fun (&x) -> ..`, `let &x = r in`";

/// The total of the prices, copied out of the slice by the pattern.
pub fn total(prices: &[u32]) -> u32 {
    ml! {
        let mut sum = 0 in
        // With `for &p in prices`, `p` has type `u32`; with `for p in
        // prices`, it would have type `&u32`.
        for &p in prices do sum <- sum + p done;
        sum
    }
}

/// The prices above a limit, doubled.
pub fn doubled_above(prices: &[u32], limit: u32) -> Vec<u32> {
    ml! {
        // `xs.iter().map(fun (&p) -> p * 2)` is `xs.iter().map(|&p| p * 2)`.
        // `filter` gives its closure a `&&u32`, so its pattern is `&&p`.
        prices.iter().filter(fun (&&p) -> p > limit).map(fun (&p) -> p * 2).collect()
    }
}

/// A reference taken apart by `let` and by `match`.
pub fn describe(best: &u32) -> String {
    ml! {
        // `let &b = best in b` is `{ let &b = best; b }`.
        let &b = best in
        match best with &0 -> String.from "nothing sold" | &n -> format!("best: {n}, again {b}")
    }
}

pub fn run() {
    println!("total [3; 4; 5] = {}", total(&[3, 4, 5]));
    println!(
        "doubled_above [3; 4; 5] 3 = {:?}",
        doubled_above(&[3, 4, 5], 3)
    );
    println!(
        "describe &0 = {:?}, describe &7 = {:?}",
        describe(&0),
        describe(&7)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copied_out() {
        assert_eq!(total(&[3, 4, 5]), 12);
        assert_eq!(doubled_above(&[3, 4, 5], 3), [8, 10]);
        assert_eq!(describe(&0), "nothing sold");
        assert_eq!(describe(&7), "best: 7, again 7");
    }
}

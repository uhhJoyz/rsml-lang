// Toy 060: `use`, Rust's imports for the rest of the block.
//
// What it is: Rust's `use` item, `use path;`, written where a phrase of
// the block starts: the names it imports are usable in the rest of the
// block. `use std::cmp::*; e` is `{ use std::cmp::*; e }`.
//
// When to use it: when a block calls several functions of one module, or
// names a type without its path.
//
// How to write it: `use` with any Rust use tree and its `;` (`use
// std::cmp::max;`, `use std::cmp::{max, min};`, `use std::cmp::*;`). The
// `;` ends it, so no `;;` is needed after it. Rust's rules apply: a local
// name hides an imported one, and two globs may not both give a name you
// use.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)
// (and [Paths](../../docs/notation.md#paths))

use rsml::ml;

pub const SUMMARY: &str = "`use std::cmp::{max, min};`: Rust's `use` for the rest of the block";

/// A value kept between two bounds, with `std::cmp`'s functions.
pub fn clamped(low: i32, x: i32, high: i32) -> i32 { ml! {
    // `use std::cmp::{max, min}; max low x` is
    // `{ use std::cmp::{max, min}; max(low, x) }`.
    use std::cmp::{max, min};
    min (max low x) high
}}

/// The circumference and area of a circle, with `PI` and `TAU` imported.
pub fn circle(r: f64) -> (f64, f64) { ml! {
    use std::f64::consts::*;
    // `TAU` names `std::f64::consts::TAU`, through the `use`.
    (TAU *. r, PI *. r *. r)
}}

/// A count of the words of each length, with a map from `std::collections`.
pub fn lengths(text: &str) -> Vec<(usize, u32)> { ml! {
    use std::collections::BTreeMap;
    let mut by_len = BTreeMap.new () in
    for w in text.split_whitespace() do
      let n = by_len.entry(w.len()).or_insert(0) in
      *n <- *n + 1
    done;
    by_len.into_iter().collect()
}}

pub fn run() {
    println!(
        "clamped 0 15 10 = {}, clamped 0 (-3) 10 = {}",
        clamped(0, 15, 10),
        clamped(0, -3, 10)
    );
    let (c, a) = circle(1.0);
    println!("circle 1.0 = ({c:.4}, {a:.4})");
    println!("lengths \"a bb cc d\" = {:?}", lengths("a bb cc d"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_names() {
        assert_eq!(clamped(0, 15, 10), 10);
        assert_eq!(clamped(0, -3, 10), 0);
        assert_eq!(clamped(0, 4, 10), 4);
        assert_eq!(circle(1.0), (std::f64::consts::TAU, std::f64::consts::PI));
        assert_eq!(lengths("a bb cc d"), [(1, 2), (2, 2)]);
    }
}

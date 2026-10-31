// Toy 083: `for p in e do .. done`, Rust's `for` over an iterator.
//
// What it is: Rust's `for` loop, ML style: the body runs for each item
// of anything iterable (a range, a `Vec`, a map, an iterator), and the
// item is matched against a pattern:
// `for (k, v) in m do f k v done` is `for (k, v) in m { f(k, v); }`.
//
// When to use it: to walk a collection when the loop does something
// (pushes, prints, updates); to compute a value, a pipeline (toy 046) is
// often shorter.
//
// How to write it:
// * The pattern is any pattern: `x`, `(i, x)`, `&x` (toy 076); a tuple may
//   leave out its parentheses: `for k, v in m do f k v done` is
//   `for (k, v) in m { f(k, v); }`.
// * `for x in xs` moves `xs`; `for x in &xs` borrows it, and `x` is then
//   a reference.
// * The body is a unit block, as in toy 081.
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;
use std::collections::BTreeMap;

pub const SUMMARY: &str = "`for (k, v) in m do .. done`, `for (i, x) in xs.iter().enumerate()`";

/// A numbered shopping list.
pub fn numbered(items: &[&str]) -> Vec<String> { ml! {
    let mut out = Vec.new () in
    for (i, item) in items.iter().enumerate() do out.push(format!("{}. {item}", i + 1)) done;
    out
}}

/// The stock that is running low, from a map of item to count.
pub fn low_stock(stock: &BTreeMap<&str, u32>) -> Vec<String> { ml! {
    let mut out = Vec.new () in
    // A `BTreeMap` gives its pairs in key order; `n` has type `&u32`.
    for name, n in stock do if !n < 3 then out.push(format!("{name}: {n}")) done;
    out
}}

pub fn run() {
    println!(
        "numbered [\"tea\"; \"jam\"] = {:?}",
        numbered(&["tea", "jam"])
    );
    let stock = BTreeMap::from([("tea", 1), ("jam", 9), ("bread", 2)]);
    println!("low_stock {stock:?} = {:?}", low_stock(&stock));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_and_pairs() {
        assert_eq!(numbered(&["tea", "jam"]), ["1. tea", "2. jam"]);
        let stock = BTreeMap::from([("tea", 1), ("jam", 9), ("bread", 2)]);
        assert_eq!(low_stock(&stock), ["bread: 2", "tea: 1"]);
    }
}

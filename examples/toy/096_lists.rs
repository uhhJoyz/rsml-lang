// Toy 096: lists, `[a; b; c]` and `[]`.
//
// What it is: a list literal, made a Rust `Vec`. `[]` is an empty
// `Vec`, its element type inferred as for `Vec::new()`.
//
// When to use it: for any `Vec` written out item by item. A list is a
// `Vec`, so everything a `Vec` has works: `.len()`, `.push(x)`, `xs[0]`,
// `.iter()`.
//
// How to write it:
// * Items are separated by `;`. A `,` makes a tuple, so `[1, 2]` would
//   be a list of one pair; rsml rejects it to be sure, and a list of one
//   pair is written `[(1, 2)]`.
// * A last `;` is allowed: `[a; b;]`.
// * `x :: xs` and `xs @ ys` (toys 062, 063) build lists from lists;
//   `[|a; b|]` is an array (toy 097).
//
// Reference: [Records and collections](../../docs/notation.md#records-and-collections)

use rsml::ml;

pub const SUMMARY: &str = "Lists are `Vec`s: `[a; b]`, `[]`, `[(1, 2)]`; `[1, 2]` is rejected";

/// A shopping list.
pub fn basket() -> Vec<&'static str> { ml! {
    // `[1; 2; 3]` is `::std::vec![1, 2, 3]`.
    ["bread"; "milk"; "eggs"]
}}

/// An empty list: its type comes from the function's.
pub fn nothing() -> Vec<&'static str> { ml! {
    // `[]` is `::std::vec::Vec::new()`.
    []
}}

/// Prices as pairs, a list of tuples.
pub fn bill() -> u32 { ml! {
    // `[(1, 2)]` is `::std::vec![(1, 2)]`: one item, a pair.
    let lines = [("tea", 4); ("cake", 12)] in
    lines |> .iter() |> .map(fun (_, price) -> !price) |> .sum()
}}

/// A list of lists; and a `Vec`'s methods on it.
pub fn grid_total() -> (usize, i64) { ml! {
    let mut grid = [[1; 2]; [3; 4]] in
    grid.push([5]);
    (grid.len(), grid |> .iter() |> .map(fun row -> row.iter().sum::<i64>()) |> .sum())
}}

// `[1, 2]` is an error: "list items are separated by `;`: write `[1; 2]`,
// or `[(1, 2)]` for a list of one pair"

pub fn run() {
    println!("basket () = {:?}", basket());
    println!("nothing () = {:?}", nothing());
    println!("bill () = {}", bill());
    println!("grid_total () = {:?}", grid_total());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_are_vecs() {
        assert_eq!(basket(), vec!["bread", "milk", "eggs"]);
        assert!(nothing().is_empty());
        assert_eq!(bill(), 16);
        assert_eq!(grid_total(), (3, 15));
    }
}

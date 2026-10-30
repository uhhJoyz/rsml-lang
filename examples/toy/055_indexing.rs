// Toy 055: indexing, `v[i]`, `m[&k]`, `xs[1..]`.
//
// What it is: a `[` written right after a value (a name, a path, or a
// closing `)` or `]`) is Rust's indexing, with an OCaml expression inside.
// With a space, `[..]` is an OCaml list, passed as an argument.
//
// When to use it: to read an element of a vector, array, slice, string
// or map, or to take a slice with a range.
//
// How to write it: no space before the `[`.
// * `v[i]`, `grid[r][c]`, `f(x)[0]` (what a call returns), `m[&k]`.
// * `xs[1..]`, `xs[..2]`: a slice; the range is Rust's, so `1..` starts
//   at index 1 and `..2` stops before index 2 (toy 088).
// * Spaced, `f [1; 2]` passes the list `[1; 2]`; and `v [0]` would call
//   `v` with a list, which rustc rejects for a vector.
// * `[|..|]` is always an array: `f[|1; 2|]` is `f [|1; 2|]`.
//
// Reference: [Rust-style calls and indexing](../../docs/notation.md#rust-style-calls-and-indexing-fa-b-vi)

use rsml::ml;
use std::collections::HashMap;

pub const SUMMARY: &str = "Indexing `v[i]`, `m[&k]`, `f(x)[0]`, `xs[1..]`; `f [1; 2]` is a list";

/// The first, last and middle temperatures of a week.
pub fn week(temps: &[i32]) -> (i32, i32, Vec<i32>) {
    ml! {
        // `temps[temps.len() - 1]` stays `temps[temps.len() - 1]`, and
        // `temps[1..temps.len() - 1]` is a slice without both ends.
        (temps[0], temps[temps.len() - 1], temps[1..temps.len() - 1].to_vec())
    }
}

/// A price looked up in a map: the index is a reference to the key.
pub fn price(prices: &HashMap<String, u32>, item: &str) -> u32 {
    ml! {
        // `prices[&item.to_string()]` stays `prices[&item.to_string()]`.
        prices[&item.to_string()]
    }
}

/// The first word of a line: indexing what a call returns.
pub fn first_word(line: &str) -> String {
    ml! {
        // `words(line)[0]` stays `words(line)[0]`; `words line [0]` would
        // pass two arguments, `line` and the list `[0]`.
        words(line)[0].to_string()
    }
}

fn words(line: &str) -> Vec<&str> {
    line.split_whitespace().collect()
}

fn total(xs: Vec<i32>) -> i32 {
    xs.iter().sum()
}

/// A list and an array as arguments: the space before `[` matters.
pub fn totals() -> (i32, i32) {
    ml! {
        // `total [1; 2; 3]` is `total(::std::vec![1, 2, 3])`;
        // `[|4; 5|].iter().sum()` sums an array.
        (total [1; 2; 3], [|4; 5|].iter().sum())
    }
}

pub fn run() {
    println!("week [3; 5; 4; 6] = {:?}", week(&[3, 5, 4, 6]));
    let prices = HashMap::from([("tea".to_string(), 4)]);
    println!("price \"tea\" = {}", price(&prices, "tea"));
    println!(
        "first_word \"hello world\" = {:?}",
        first_word("hello world")
    );
    println!("total [1; 2; 3], sum [|4; 5|] = {:?}", totals());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elements_and_slices() {
        assert_eq!(week(&[3, 5, 4, 6]), (3, 6, vec![5, 4]));
        let prices = HashMap::from([("tea".to_string(), 4)]);
        assert_eq!(price(&prices, "tea"), 4);
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(totals(), (6, 9));
    }
}

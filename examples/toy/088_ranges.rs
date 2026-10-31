// Toy 088: ranges, `a..b`, `a..=b`, `a..`, `..b`.
//
// What it is: Rust's ranges, with Rust's meaning: `a..b` counts from `a`
// and stops BEFORE `b`, `a..=b` includes `b`, `a..` and `..b` leave an end
// open. They are values (iterators of numbers) and slice indexes.
//
// When to use it: to count (`for i in 0..n`), to make a list of numbers
// (`(1..=n) |> .map(f)`), and to slice (`xs[1..]`).
//
// How to write it:
// * A range binds looser than arithmetic, tighter than comparisons and
//   `|>`: `0..n + 1` is `0..(n + 1)`, and `0..n |> .map(f)` is
//   `(0..n).map(f)`.
// * Ranges do not chain: `a..b..c` is an error: "ranges do not chain: put
//   one in parentheses"
// * In patterns, `..` and `..=` mean the same (toy 075).
// * The end is left out when nothing that starts an expression follows:
//   `xs[1..]`.
//
// Reference: [Ranges](../../docs/notation.md#ranges)

use rsml::ml;

pub const SUMMARY: &str = "`0..n` stops before `n`, `1..=n` includes it; `xs[1..]`, `0..n |> ..`";

/// The squares from 1 to n: an inclusive range in a pipeline.
pub fn squares(n: u64) -> Vec<u64> { ml! {
    // `1..=n |> .map(fun k -> k * k)` is `(1..=n).map(|k| k * k)`.
    1..=n |> .map(fun k -> k * k) |> .collect()
}}

/// The hours of a working day, 9 to 17 excluded.
pub fn office_hours() -> Vec<u32> { ml! {
    9..17 |> .collect()
}}

/// A list without its first and last items.
pub fn middle(items: &[&'static str]) -> Vec<&'static str> { ml! {
    // `items[1..items.len() - 1]` slices from index 1 up to the last.
    if items.len() < 2 then Vec.new () else items[1..items.len() - 1].to_vec()
}}

/// How many numbers below 100 are multiples of `k`.
pub fn multiples(k: u32) -> usize { ml! {
    (1..100) |> .filter(fun n -> !n mod k = 0) |> .count()
}}

pub fn run() {
    println!("squares 4 = {:?}", squares(4));
    println!("office_hours () = {:?}", office_hours());
    println!("middle [a; b; c; d] = {:?}", middle(&["a", "b", "c", "d"]));
    println!("multiples 25 = {}", multiples(25));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusive_and_inclusive() {
        assert_eq!(squares(4), [1, 4, 9, 16]);
        assert_eq!(office_hours().len(), 8);
        assert_eq!(office_hours().last(), Some(&16));
        assert_eq!(middle(&["a", "b", "c", "d"]), ["b", "c"]);
        assert_eq!(middle(&["a"]), Vec::<&str>::new());
        assert_eq!(multiples(25), 3);
    }

    #[test]
    fn open_ends() {
        let xs = [1, 2, 3, 4];
        assert_eq!(ml! { xs[2..].len() + xs[..1].len() }, 3);
        assert_eq!(ml! { (5..).take(2).collect::<Vec<_>>() }, [5, 6]);
    }
}

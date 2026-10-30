// Toy 038: placeholders, `clamp 0 _ 100`.
//
// What it is: `_` in an expression makes the whole expression a function,
// with one parameter for the `_`. `clamp 0 _ 100` is "clamp something
// between 0 and 100": a closure waiting for that something.
//
// When to use it: to fill in some arguments of a function and leave a
// hole for the rest, which a Rust function needs (its arity is unknown,
// toy 018); and for short closures such as `_ * 2`.
//
// How to write it: `_` where the argument goes.
// * In a pipeline the value fills the hole directly:
//   `x |> clamp 0 _ 100` is `clamp(0, x, 100)`, with no closure (toy 025).
// * In patterns `_` is still the wildcard: `Some _` matches any `Some`.
// * Toy 039 numbers the holes, toy 040 says how far a `_` reaches.
//
// Reference: [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-)

use rsml::ml;

pub const SUMMARY: &str = "`_` makes a function: `clamp 0 _ 100`, `_ * 2`";

fn clamp(low: i64, x: i64, high: i64) -> i64 {
    x.max(low).min(high)
}

/// Readings forced into 0..=100.
pub fn percents(readings: &[i64]) -> Vec<i64> {
    ml! {
        // `clamp 0 _ 100` is `move |_0| clamp(0, _0, 100)`.
        let to_percent = clamp 0 _ 100 in
        readings |> .iter() |> .map(fun r -> to_percent !r) |> .collect()
    }
}

/// Every reading doubled.
pub fn doubled(readings: &[i64]) -> Vec<i64> {
    ml! {
        // `_ * 2` is the closure `|_0| _0 * 2`; `map` gives it each `&i64`.
        readings |> .iter() |> .map(_ * 2) |> .collect()
    }
}

/// One reading, clamped: the pipe fills the hole, no closure is made.
pub fn one_percent(reading: i64) -> i64 {
    ml! { reading |> clamp 0 _ 100 }
}

/// How many readings are present; `_` in the pattern is the wildcard.
pub fn present(readings: &[Option<i64>]) -> usize {
    ml! {
        // `iter` gives references, so `o` has type `&Option<i64>`.
        let count (o : &Option<i64>) = match o with Some _ -> 1 | None -> 0 in
        readings |> .iter() |> .map(count) |> .sum()
    }
}

pub fn run() {
    let readings = [-5, 42, 130];
    println!("readings = {readings:?}");
    println!("map (clamp 0 _ 100) = {:?}", percents(&readings));
    println!("map (_ * 2) = {:?}", doubled(&readings));
    println!("130 |> clamp 0 _ 100 = {}", one_percent(130));
    println!(
        "present [Some 1; None; Some 3] = {}",
        present(&[Some(1), None, Some(3)])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holes_make_functions() {
        assert_eq!(percents(&[-5, 42, 130]), [0, 42, 100]);
        assert_eq!(doubled(&[-5, 42, 130]), [-10, 84, 260]);
        assert_eq!(one_percent(130), 100);
    }

    #[test]
    fn the_wildcard_in_patterns() {
        assert_eq!(present(&[Some(1), None, Some(3)]), 2);
    }
}

// Toy 070: list patterns, `[]`, `[x]`, `[a; b]`, `x :: rest`.
//
// What it is: list patterns, matched against a slice: `[]` is
// empty, `[a; b]` has exactly two elements, `x :: rest` has a first
// element `x` and any number after it, as the slice `rest`.
//
// When to use it: to look at the first elements of a `Vec`, a slice or
// an array, and for recursion over a slice.
//
// How to write it:
// * Items are separated by `;`, as in list expressions (toy 096).
// * `x :: y :: rest` takes two elements; `_ :: _ :: _` means "at least two".
// * The match borrows its value as a slice, so `x` has type `&T` and
//   `rest` has type `&[T]`: `!x` copies a number out (toy 092).
// * A recursive function takes a slice, `(xs : &[u32])`, so that `rest`
//   can be passed back to it.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str = "List patterns: `[]`, `[x]`, `[a; b]`, `x :: rest` on slices and Vecs";

/// A list of names, the way people say it.
pub fn names(people: &[&str]) -> String { ml! {
    // `match xs with [] -> 0 | _ :: rest -> rest.len()` is
    // `match ::rsml::__rt::as_slice(&xs) { [] => 0, [_, rest @ ..] => rest.len(), }`.
    match people with
    | [] -> String.from "nobody"
    | [one] -> one.to_string()
    | [a; b] -> format!("{a} and {b}")
    | first :: rest -> format!("{first} and {} others", rest.len())
}}

/// The sum of the prices, by recursion over the slice.
pub fn total(prices: &[u32]) -> u32 { ml! {
    let rec go (xs : &[u32]) : u32 =
      // `x` has type `&u32`, so `!x` reads it; `rest` has type `&[u32]`.
      match xs with [] -> 0 | x :: rest -> !x + go rest
    in
    go prices
}}

/// The first two scores of a `Vec`, if it has two.
pub fn podium(scores: Vec<u32>) -> Option<(u32, u32)> { ml! {
    // Two elements, then anything:
    // `match v with x :: y :: _ -> 2 | _ -> 0` is
    // `match ::rsml::__rt::as_slice(&v) { [x, y, ..] => 2, _ => 0, }`.
    match scores with x :: y :: _ -> Some (!x, !y) | _ -> None
}}

pub fn run() {
    for people in [
        &[][..],
        &["Ada"],
        &["Ada", "Bob"],
        &["Ada", "Bob", "Cy", "Di"],
    ] {
        println!("names {:?} = {:?}", people, names(people));
    }
    println!("total [3; 4; 5] = {}", total(&[3, 4, 5]));
    println!(
        "podium [9; 7; 4] = {:?}, podium [9] = {:?}",
        podium(vec![9, 7, 4]),
        podium(vec![9])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn by_length() {
        assert_eq!(names(&[]), "nobody");
        assert_eq!(names(&["Ada"]), "Ada");
        assert_eq!(names(&["Ada", "Bob"]), "Ada and Bob");
        assert_eq!(names(&["Ada", "Bob", "Cy", "Di"]), "Ada and 3 others");
    }

    #[test]
    fn recursion_and_vecs() {
        assert_eq!(total(&[3, 4, 5]), 12);
        assert_eq!(total(&[]), 0);
        assert_eq!(podium(vec![9, 7, 4]), Some((9, 7)));
        assert_eq!(podium(vec![9]), None);
    }
}

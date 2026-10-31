// Toy 103: generic arguments, the turbofish `::<T>`.
//
// What it is: Rust's way to give the type arguments of a function or
// method in a call, `parse::<i32>()`, `collect::<Vec<_>>()`,
// `Vec::<u8>::new()`. It works as written, in Rust-style calls and in
// ML-style ones.
//
// When to use it: when rustc cannot tell which type a generic function
// should make (`parse`, `collect`, `sum`) and no annotation says it.
//
// How to write it: as in Rust, without spaces around `::`.
// * `"42".parse::<i32>()` stays `"42".parse::<i32>()`.
// * ML style: `Vec::<u8>::new ()` is `Vec::<u8>::new()`, and
//   `std::mem::size_of::<u64> ()` is `std::mem::size_of::<u64>()`.
// * On a method as a function (toy 046): `it |> .sum::<i64>` is
//   `it.sum::<i64>()`.
//
// Reference: [Types](../../docs/notation.md#types)

use rsml::ml;

pub const SUMMARY: &str = "Turbofish: `.parse::<i32>()`, `.sum::<i64>`, `Vec::<u8>::new ()`";

/// The numbers in a line of text, skipping the words.
pub fn numbers(line: &str) -> Vec<i32> { ml! {
    line.split_whitespace() |> .filter_map(fun w -> w.parse::<i32>().ok()) |> .collect::<Vec<_>>()
}}

/// The total of some amounts, as an `i64`.
pub fn total(amounts: &[i64]) -> i64 { ml! {
    amounts |> .iter() |> .sum::<i64>
}}

/// An empty byte buffer and the size of a `u64`.
pub fn sizes() -> (usize, usize) { ml! {
    let buffer = Vec::<u8>::new () in
    (buffer.len(), std::mem::size_of::<u64> ())
}}

pub fn run() {
    println!(
        "numbers \"3 apples and 12 pears\" = {:?}",
        numbers("3 apples and 12 pears")
    );
    println!("total [5; -2; 10] = {}", total(&[5, -2, 10]));
    println!("sizes () = {:?}", sizes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_arguments() {
        assert_eq!(numbers("3 apples and 12 pears"), [3, 12]);
        assert_eq!(total(&[5, -2, 10]), 13);
        assert_eq!(sizes(), (0, 8));
    }
}

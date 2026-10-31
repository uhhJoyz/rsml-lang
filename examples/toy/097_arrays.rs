// Toy 097: arrays, `[|a; b; c|]`.
//
// What it is: the array literal, made a Rust array `[a, b, c]`: a
// fixed number of items, no heap. `[| |]` is an empty array.
//
// When to use it: for fixed-size data (a colour, a week, a small table),
// and to pass a slice: `f &[|1; 2|]` lends `&[1, 2]`.
//
// How to write it:
// * Items are separated by `;`, as in lists; rsml rejects `[|1, 2|]`
//   with a message that says so (quoted below).
// * As a pattern, `[|r; g; b|]` matches the array as a slice, like a list
//   pattern (toy 070): the names are references, `!r` reads one, and a
//   `match` needs a case for the other lengths, even on a `[u8; 3]`.
// * With no space, `f[|1; 2|]` is `f([1, 2])`: `[|` is never indexing
//   (toy 055).
//
// Reference: [Records and collections](../../docs/notation.md#records-and-collections)

use rsml::ml;

pub const SUMMARY: &str = "Arrays: `[|a; b|]` is `[a, b]`; array patterns `[|r; g; b|]`";

/// Orange, as red, green and blue.
pub fn orange() -> [u8; 3] { ml! {
    // `[|1; 2; 3|]` is `[1, 2, 3]`.
    [|255; 165; 0|]
}}

/// The brightness of a colour, from its three parts.
pub fn brightness(rgb: [u8; 3]) -> u32 { ml! {
    match rgb with
    // `r`, `g` and `b` are `&u8`: `!r` reads one.
    | [|r; g; b|] -> (!r as u32 + !g as u32 + !b as u32) / 3
    | _ -> 0
}}

/// The weekend: the last two days of the week.
pub fn weekend() -> Vec<&'static str> { ml! {
    let week = [|"Mon"; "Tue"; "Wed"; "Thu"; "Fri"; "Sat"; "Sun"|] in
    // An array is indexed as in Rust: `week[5..]` is a slice of it.
    week[5..].to_vec()
}}

/// The sum of a slice, called with an array.
pub fn total_of_three() -> i64 { ml! {
    // `sum &[|1; 2; 3|]` is `sum(&[1, 2, 3])`.
    sum &[|1; 2; 3|]
}}

fn sum(xs: &[i64]) -> i64 {
    xs.iter().sum()
}

// `[|1, 2|]` is an error: "array items are separated by `;`: write
// `[|1; 2|]`, or `[|(1, 2)|]` for an array of one pair"

pub fn run() {
    println!("orange () = {:?}", orange());
    println!("brightness (orange ()) = {}", brightness(orange()));
    println!("weekend () = {:?}", weekend());
    println!("sum &[|1; 2; 3|] = {}", total_of_three());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrays() {
        assert_eq!(orange(), [255, 165, 0]);
        assert_eq!(brightness(orange()), 140);
        assert_eq!(weekend(), ["Sat", "Sun"]);
        assert_eq!(total_of_three(), 6);
    }
}

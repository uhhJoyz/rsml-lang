// Toy 072: or-patterns, `p | q -> ..`.
//
// What it is: several patterns for one case, separated by `|`. The case
// is taken when any of them matches. Every alternative must bind the same
// names, with the same types.
//
// When to use it: when several values get the same answer (vowels, the
// days of a weekend), instead of repeating the result.
//
// How to write it:
// * `'a' | 'e' -> ..`: the alternatives come before the case's one `->`.
//   The cases are separated by `|` too: everything between one result
//   and the next `->` is the alternatives of one case.
// * Inside a constructor, put them in parentheses: `Some (1 | 2)`.
// * `(0, k) | (k, 0) -> k`: both alternatives bind `k`.
// * `|` binds looser than `,`, so alternatives inside a tuple need
//   parentheses: `match t with (Some 0 | None, _) -> 1 | _ -> 0` is
//   `match t { (Some(0) | (None, _)) => 1, _ => 0, }`, not what was meant.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str =
    "Or-patterns: `'a' | 'e' -> ..`, `Some 0 | None -> ..`, `(0, k) | (k, 0)`";

/// The number of vowels in a word.
pub fn vowels(word: &str) -> usize { ml! {
    // `match c with 'a' | 'e' -> true | _ -> false` is
    // `match c { 'a' | 'e' => true, _ => false, }`.
    word
    |> .chars()
    |> .filter(fun c -> match c with 'a' | 'e' | 'i' | 'o' | 'u' -> true | _ -> false)
    |> .count()
}}

/// How many seats are free: no booking and a booking of 0 are the same.
pub fn free_seats(booked: Option<u32>, seats: u32) -> u32 { ml! {
    // `Some 0 | None` is one case with two alternatives.
    match booked with Some 0 | None -> seats | Some n -> seats - n
}}

/// The side of a rectangle that is not zero, if one of them is.
pub fn other_side(w: u32, h: u32) -> Option<u32> { ml! {
    // Both alternatives bind `k`.
    match (w, h) with (0, k) | (k, 0) -> Some k | _ -> None
}}

/// Whether a die shows a low number: or-patterns inside `Some`.
pub fn low(roll: Option<u8>) -> bool { ml! {
    match roll with Some (1 | 2) -> true | _ -> false
}}

pub fn run() {
    println!("vowels \"education\" = {}", vowels("education"));
    println!(
        "free_seats None 5 = {}, free_seats (Some 0) 5 = {}, free_seats (Some 2) 5 = {}",
        free_seats(None, 5),
        free_seats(Some(0), 5),
        free_seats(Some(2), 5)
    );
    println!(
        "other_side 0 4 = {:?}, other_side 3 0 = {:?}, other_side 2 2 = {:?}",
        other_side(0, 4),
        other_side(3, 0),
        other_side(2, 2)
    );
    println!(
        "low (Some 2) = {}, low (Some 5) = {}",
        low(Some(2)),
        low(Some(5))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternatives() {
        assert_eq!(vowels("education"), 5);
        assert_eq!(free_seats(None, 5), 5);
        assert_eq!(free_seats(Some(0), 5), 5);
        assert_eq!(free_seats(Some(2), 5), 3);
        assert_eq!(other_side(0, 4), Some(4));
        assert_eq!(other_side(3, 0), Some(3));
        assert_eq!(other_side(2, 2), None);
        assert!(low(Some(2)));
        assert!(!low(Some(5)));
        assert!(!low(None));
    }
}

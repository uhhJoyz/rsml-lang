// Toy 069: tuple patterns, `(a, b)`, in `match`, `let` and `fun`.
//
// What it is: a tuple pattern takes a tuple apart, one pattern per element.
// The elements may be literals (`(0, _)`), names, `_`, or tuples again.
//
// When to use it: to match on two values at once (`match (x, y) with ..`),
// to name the parts of a returned pair (`let (h, m) = .. in`), and for the
// parameter of a closure over pairs (`fun (name, n) -> ..`).
//
// How to write it:
// * In parentheses, separated by `,`, as in Rust.
// * `match (x, y) with` builds the tuple and matches it in one go.
// * A closure over a pair takes the pattern as its one parameter:
//   `fun (a, b) -> ..`, where `fun a b -> ..` would take two arguments.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str = "Tuple patterns: `match (x, y) with (0, _) -> ..`, `let (h, m) = ..`";

/// Where a point lies on a grid.
pub fn place(x: i64, y: i64) -> &'static str {
    ml! {
        // `match (x, y) with (0, 0) -> "origin" | _ -> "plane"` is
        // `match (x, y) { (0, 0) => "origin", _ => "plane", }`.
        match (x, y) with (0, 0) -> "origin" | (0, _) | (_, 0) -> "on an axis" | _ -> "elsewhere"
    }
}

/// Minutes as hours and minutes, `2h05`.
pub fn clock(minutes: u32) -> String {
    ml! {
        // `let (h, m) = (minutes / 60, minutes mod 60) in h` is
        // `{ let (h, m) = (minutes / 60, minutes % 60); h }`.
        let (h, m) = (minutes / 60, minutes mod 60) in
        format!("{h}h{m:02}")
    }
}

/// How many items are in stock: the closure takes each pair apart.
pub fn in_stock(stock: &[(&str, u32)]) -> u32 {
    ml! {
        // `(_, n)` ignores the name; `n` has type `&u32` (the pairs are
        // borrowed), so `!n` reads it.
        stock.iter().map(fun (_, n) -> !n).sum::<u32>()
    }
}

/// Swaps the two halves of a pair.
pub fn swap(pair: (char, u8)) -> (u8, char) {
    ml! {
        // `fun (a, b) -> (b, a)` is `move |(a, b)| (b, a)`; applied on the
        // spot, it is a `let` (toy 014).
        (fun (a, b) -> (b, a)) pair
    }
}

pub fn run() {
    println!(
        "place 0 0 = {}, place 0 3 = {}, place 2 5 = {}",
        place(0, 0),
        place(0, 3),
        place(2, 5)
    );
    println!("clock 125 = {}", clock(125));
    println!(
        "in_stock [(pen, 3); (ink, 4)] = {}",
        in_stock(&[("pen", 3), ("ink", 4)])
    );
    println!("swap ('a', 1) = {:?}", swap(('a', 1)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_two_values() {
        assert_eq!(place(0, 0), "origin");
        assert_eq!(place(0, 3), "on an axis");
        assert_eq!(place(4, 0), "on an axis");
        assert_eq!(place(2, 5), "elsewhere");
    }

    #[test]
    fn taking_pairs_apart() {
        assert_eq!(clock(125), "2h05");
        assert_eq!(in_stock(&[("pen", 3), ("ink", 4)]), 7);
        assert_eq!(swap(('a', 1)), (1, 'a'));
    }
}

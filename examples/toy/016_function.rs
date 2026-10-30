// Toy 016: `function`, a closure that matches its argument.
//
// What it is: `function p1 -> e1 | p2 -> e2` is
// `fun argument -> match argument with p1 -> e1 | p2 -> e2`: a function
// of one argument, given by its cases.
//
// When to use it: for a function that is a `match`, as a value, a `let`
// or an argument.
//
// How to write it:
// * The cases are those of `match` (toys 068-077); a leading `|` is
//   allowed: `function | Some n -> n | None -> 0`.
// * The last case reaches as far right as it can, so as an OCaml argument
//   `function` goes in parentheses (toy 017).
// * With `let rec f : t1 -> t2 = function ..` it is recursive (toy 008).
//
// Reference: [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function)

use rsml::ml;

pub const SUMMARY: &str = "`function Some n -> n | None -> 0`: a closure of cases";

/// The seats left at a show, 0 when it is sold out.
pub fn seats_left() -> impl Fn(Option<u32>) -> u32 {
    ml! {
        // `function Some n -> n | None -> 0` is
        // `move |argument| match argument { Some(n) => n, None => 0, }`.
        function Some n -> n | None -> 0
    }
}

/// Counts in words: `function` as the value of a `let`.
pub fn in_words(a: u32, b: u32) -> (String, String) {
    ml! {
        let describe = function
        | 0 -> String.from "none"
        | 1 -> String.from "one"
        | n -> format!("{n} items")
        in
        (describe a, describe b)
    }
}

/// The sum of the known values: `function` as a Rust argument.
pub fn known_total(readings: &[Option<u32>]) -> u32 {
    ml! {
        // Each item is a `&Option<u32>`, so `n` has type `&u32`, read with `!n`.
        readings.iter().map(function Some n -> !n | None -> 0).sum()
    }
}

pub fn run() {
    let seats = seats_left();
    println!(
        "seats_left (Some 12) = {}, seats_left None = {}",
        seats(Some(12)),
        seats(None)
    );
    println!("in_words 0 7 = {:?}", in_words(0, 7));
    println!(
        "known_total [Some 3; None; Some 4] = {}",
        known_total(&[Some(3), None, Some(4)])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cases() {
        let seats = seats_left();
        assert_eq!(seats(Some(12)), 12);
        assert_eq!(seats(None), 0);
        assert_eq!(in_words(0, 7), ("none".to_string(), "7 items".to_string()));
        assert_eq!(in_words(1, 1).0, "one");
        assert_eq!(known_total(&[Some(3), None, Some(4)]), 7);
    }
}

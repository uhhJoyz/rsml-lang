// Toy 087: `assert c` and `assert false`.
//
// What it is: OCaml's `assert c` stops the program when `c` is false; it
// is Rust's `assert!`: `assert (n > 0)` is `::core::assert!(n > 0)`.
// `assert false` marks code that must never run, as in OCaml, and is
// Rust's `unreachable!`: `assert false` is
// `::core::unreachable!("assert false")`.
//
// When to use it: to state what must hold (an argument's range, a
// result's property), and in a case that cannot happen.
//
// How to write it:
// * `assert` takes one atom: `assert (n > 0)` with its parentheses;
//   `assert n > 0` would be `(assert n) > 0`.
// * Rust's `assert!(c, "message")` works too, with OCaml arguments (toy
//   106).
// * `assert false` has any type, so it fits in a branch that must give a
//   value.
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`assert (n > 0)` and `assert false` for impossible cases";

/// The average of some scores, which must not be empty.
pub fn average(scores: &[u32]) -> u32 {
    ml! {
        assert (not scores.is_empty());
        scores |> .iter() |> .sum::<u32>() |> fun total -> total / scores.len() as u32
    }
}

/// The name of a die face, 1 to 6.
pub fn face(n: u8) -> &'static str {
    ml! {
        match n with
        | 1 -> "one"
        | 2 -> "two"
        | 3 -> "three"
        | 4 -> "four"
        | 5 -> "five"
        | 6 -> "six"
        | _ -> assert false
    }
}

pub fn run() {
    println!("average [3; 4; 8] = {}", average(&[3, 4, 8]));
    println!("face 5 = {}", face(5));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assertions_that_hold() {
        assert_eq!(average(&[3, 4, 8]), 5);
        assert_eq!(face(5), "five");
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn an_assertion_that_fails() {
        average(&[]);
    }

    #[test]
    #[should_panic(expected = "assert false")]
    fn an_impossible_case() {
        face(7);
    }
}

// Toy 106: `assert_eq!`, `assert!` and friends with rsml arguments.
//
// What it is: the arguments of `assert!`, `assert_eq!`, `assert_ne!` and
// their `debug_` forms are rsml expressions, so checks read like the code
// they check, in tests and in functions alike.
//
// When to use it: for preconditions in rsml functions, and in tests.
//
// How to write it:
// * `assert_eq!(mean &[|2; 4|], 3)` compares an application.
// * `=` compares: `assert!(a = b)` is `assert!(a == b)`.
// * Negation is `not`: `!` dereferences and binds tighter than `.`, so
//   Rust's spelling would assert the opposite, and rsml rejects it:
//   `assert!(!xs.is_empty())` is an error: "`!` dereferences, and binds
//   tighter than `.`: `!x.f()` is `(!x).f()`. To negate, write `not
//   (x.f())`; to dereference, `(!x).f()`"
// * Likewise `assert!(!(a = b))` is an error: "a `bool` cannot be
//   dereferenced: `!r` is the value of `r` here, and negation is `not e`"
// * `assert e` without `!` is the `assert` of toy 087.
//
// Reference: [Macros](../../docs/notation.md#macros)

use rsml::rsml;

pub const SUMMARY: &str = "`assert_eq!(add 1 2, 3)`, `assert!(not (xs.is_empty()))`";

rsml! {
/// The mean of some scores, which must not be empty.
pub fn mean(scores: &[u32]) -> u32 {
    // `assert!(not (scores.is_empty()), "no scores")` is
    // `assert!(!scores.is_empty(), "no scores")`.
    assert!(not (scores.is_empty()), "no scores");
    let total = scores |> .iter |> .sum::<u32> in
    // A check in debug builds only. `max` is a reference, `&u32`, and
    // `!max` reads the number it points to.
    let max : &u32 = scores |> .iter |> .max |> .unwrap in
    debug_assert!(total >= !max);
    total / (scores |> .len) as u32
}

/// Checks `mean` on a few inputs, rsml code in every argument.
pub fn check_mean() -> &'static str {
    assert_eq!(mean &[|80; 90; 100|], 90);
    // `assert_ne!(mean &[|1; 2|], 2)` is `assert_ne!(mean(&[1, 2]), 2)`.
    assert_ne!(mean &[|1; 2|], 2);
    // `assert!(mean &[|7|] = 7, "{}", mean &[|7|])` is
    // `assert!(mean(&[7]) == 7, "{}", mean(&[7]))`.
    assert!(mean &[|7|] = 7, "{}", mean &[|7|]);
    "mean is fine"
}
}

pub fn run() {
    println!("mean [|80; 90; 100|] = {}", mean(&[80, 90, 100]));
    println!("mean [|1; 2|] = {} (integer division)", mean(&[1, 2]));
    println!("check_mean () = {:?}", check_mean());
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsml::ml;

    #[test]
    fn checks_pass() {
        assert_eq!(check_mean(), "mean is fine");
        ml! { assert_eq!(mean &[|1; 2|], 1) };
        ml! { assert_eq!(mean &[|2; 4|], 3) };
    }

    #[test]
    #[should_panic(expected = "no scores")]
    fn empty_scores_panic() {
        mean(&[]);
    }
}

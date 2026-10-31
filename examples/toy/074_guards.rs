// Toy 074: guards, `pattern when condition -> ..`.
//
// What it is: a case may add a condition after its pattern, with `when`
// or `if`; the case is taken only when the pattern matches and the
// condition is true. It is Rust's `if` guard.
//
// When to use it: for ranges of values that a pattern cannot say (`s when
// s >= 90`), and for conditions between the bound names (`(a, b) when
// a = b`).
//
// How to write it:
// * The condition is any `bool` expression and may use the names the
//   pattern binds.
// * Cases are tried in order, so the most demanding guard goes first.
// * Guards do not count for exhaustiveness, as in Rust: end with a case
//   without one (`| _ -> ..`).
// * Rust's spelling `p if c -> e` is the same guard as `p when c -> e`:
//   `match score with s if s >= 90 -> "A" | _ -> "C"` is
//   `match score { s if s >= 90 => "A", _ => "C", }`.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str =
    "Guards: `s when s >= 90 -> \"A\"`, `(a, b) when a = b -> ..`, `a if a < 12 -> ..`";

/// A grade for a score.
pub fn grade(score: u32) -> &'static str { ml! {
    // `match score with s when s >= 90 -> "A" | _ -> "C"` is
    // `match score { s if s >= 90 => "A", _ => "C", }`.
    match score with s when s >= 90 -> "A" | s when s >= 75 -> "B" | _ -> "C"
}}

/// The result of a game, from the two scores.
pub fn result(home: u32, away: u32) -> String { ml! {
    // The guard compares two names that the pattern binds: `a = b` is
    // `a == b`.
    match (home, away) with
    | (a, b) when a = b -> String.from "draw"
    | (a, b) when a > b -> format!("home wins by {}", a - b)
    | (a, b) -> format!("away wins by {}", b - a)
}}

/// A parcel's price by its weight, a flat 10 when the weight is not
/// known: a guard on the field of `Some`.
pub fn postage(weight: Option<u32>) -> u32 { ml! {
    match weight with Some kg when kg > 20 -> 30 | Some kg -> 5 + kg | None -> 10
}}

/// A ticket's price by age, with guards written Rust's way.
pub fn ticket(age: u32) -> u32 { ml! {
    // `match age with a if a < 12 -> 5 | _ -> 10` is
    // `match age { a if a < 12 => 5, _ => 10, }`.
    match age with a if a < 12 -> 5 | a if a >= 65 -> 7 | _ -> 10
}}

pub fn run() {
    println!(
        "grade 95 = {}, grade 80 = {}, grade 40 = {}",
        grade(95),
        grade(80),
        grade(40)
    );
    println!(
        "result 2 2 = {:?}, result 3 1 = {:?}, result 0 4 = {:?}",
        result(2, 2),
        result(3, 1),
        result(0, 4)
    );
    println!(
        "postage (Some 25) = {}, postage (Some 3) = {}, postage None = {}",
        postage(Some(25)),
        postage(Some(3)),
        postage(None)
    );
    println!(
        "ticket 8 = {}, ticket 70 = {}, ticket 30 = {}",
        ticket(8),
        ticket(70),
        ticket(30)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_matching_guard_wins() {
        assert_eq!(grade(95), "A");
        assert_eq!(grade(90), "A");
        assert_eq!(grade(80), "B");
        assert_eq!(grade(40), "C");
    }

    #[test]
    fn guards_on_bound_names() {
        assert_eq!(result(2, 2), "draw");
        assert_eq!(result(3, 1), "home wins by 2");
        assert_eq!(result(0, 4), "away wins by 4");
        assert_eq!(postage(Some(25)), 30);
        assert_eq!(postage(Some(3)), 8);
        assert_eq!(postage(None), 10);
    }

    #[test]
    fn rust_style_guards() {
        assert_eq!(ticket(8), 5);
        assert_eq!(ticket(70), 7);
        assert_eq!(ticket(65), 7);
        assert_eq!(ticket(30), 10);
    }
}

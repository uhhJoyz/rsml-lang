// Toy 054: Rust-style calls of a function whose arity the macro knows.
//
// What it is: for a function the macro can see (a local `let`, an
// `external`, a closure-typed `rsml!` parameter, a function of the same
// `rsml!`; toy 018 has the list), the items between the parentheses count
// as that many arguments, exactly as if they were written ML style.
//
// When to use it: to mix the two styles freely. A partial application
// written Rust style, `points(3)`, still makes a closure.
//
// How to write it: as any Rust call. What changes is what it means:
// * fewer items than parameters: a partial application;
// * `f(a)(b)` and `f(a, b)` are the same call;
// * more items than parameters: the extra ones apply the result.
// A Rust function the macro cannot see is called with exactly the items
// written: `f(a)` is complete, and `f(a)(b)` calls its result.
//
// Reference: [Rust-style calls and indexing](../../docs/notation.md#rust-style-calls-and-indexing-fa-b-vi)

use rsml::ml;

pub const SUMMARY: &str =
    "With a known arity, `add(1)` is partial and `add(1)(2)` calls `add(1, 2)`";

/// A score: ten points per star, plus a bonus.
pub fn scores() -> (i64, i64, i64) { ml! {
    let points (stars : i64) (bonus : i64) = stars * 10 + bonus in
    // Given `external points : _ -> _ -> _`, `points(3)` is
    // `fun b -> points 3 b`, and `points(3)(5)` is `points(3, 5)`.
    let three_stars = points(3) in
    (points(3)(5), points(3, 5), three_stars 7)
}}

/// A local function that returns a closure: the third item applies it.
pub fn scaled(x: i64) -> i64 { ml! {
    let scale factor offset = fun y -> y * factor + offset in
    // Given `external scale : _ -> _ -> (_ -> _)`, `scale(2, 1, x)` is
    // `scale(2, 1)(x)`.
    scale(2, 1, x)
}}

fn rust_points(stars: i64, bonus: i64) -> i64 {
    stars * 10 + bonus
}

/// An `external` makes a Rust function's arity known, so that a Rust-style
/// call with one item is a partial application too.
pub fn rust_scores(bonuses: &[i64]) -> Vec<i64> { ml! {
    external rust_points : i64 -> i64 -> i64 ;;
    // Given `external rust_points : i64 -> i64 -> i64`,
    // `rust_points(2) !b` is `rust_points(2, *b)`. Without the
    // `external`, `rust_points(2)` would be a call with one argument,
    // which rustc rejects (it takes two).
    bonuses.iter().map(fun b -> rust_points(2) !b).collect()
}}

pub fn run() {
    println!("points(3)(5), points(3, 5), points(3) 7 = {:?}", scores());
    println!("scale(2, 1, 10) = {}", scaled(10));
    println!("rust_points(2) over [0; 4] = {:?}", rust_scores(&[0, 4]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_count_as_arguments() {
        assert_eq!(scores(), (35, 35, 37));
        assert_eq!(scaled(10), 21);
        assert_eq!(rust_scores(&[0, 4]), [20, 24]);
    }
}

// Toy 039: numbered placeholders, `_0`, `_1`, ...
//
// What it is: `_N` is parameter number N of the function (counting from
// 0), so the same parameter can be used twice, or the parameters can go
// in another order than the arguments. A bare `_` is the lowest number
// that is not written elsewhere in the expression.
//
// When to use it: to swap arguments (`Ord::cmp _1 _0` sorts the other way
// round), or to use a value twice (`( * ) _0 _0` squares).
//
// How to write it: `_0`, `_1`, .. like `_`.
// * `f _1 _0` is `move |_0, _1| f(_1, _0)`.
// * `( * ) _0 _0` is `move |_0| _0 * _0`: one parameter, used twice.
// * `f _1 _ _` is `fun a b c -> f b a c`: the `_`s take 0 and 2, the
//   numbers `_1` leaves free.
// * `_0`, `_1`, .. are not variable names: they are always placeholders.
//
// Reference: [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-)

use rsml::ml;

pub const SUMMARY: &str = "Numbered placeholders: `Ord::cmp _1 _0`, `( * ) _0 _0`, `f _1 _ _`";

fn between(low: i64, x: i64, high: i64) -> bool {
    low <= x && x <= high
}

/// The scores, highest first.
pub fn ranking(mut scores: Vec<i64>) -> Vec<i64> { ml! {
    // `Ord::cmp _1 _0` compares the second with the first: descending.
    .sort_by scores (Ord::cmp _1 _0);
    scores
}}

/// The squares of the sides.
pub fn squares(sides: &[i64]) -> Vec<i64> { ml! {
    // `( * ) _0 _0` multiplies its one parameter by itself.
    sides |> .iter() |> .map(( * ) _0 _0) |> .collect()
}}

/// Whether `x` is in `low..=high`, asked with `x` first.
pub fn in_range(x: i64, low: i64, high: i64) -> bool { ml! {
    // `between _1 _ _` takes `x`, `low`, `high` and calls
    // `between(low, x, high)`.
    let is_in = between _1 _ _ in
    is_in x low high
}}

pub fn run() {
    println!("ranking [3; 9; 4] = {:?}", ranking(vec![3, 9, 4]));
    println!("squares [2; 3] = {:?}", squares(&[2, 3]));
    println!(
        "in_range 5 1 10 = {}, in_range 0 1 10 = {}",
        in_range(5, 1, 10),
        in_range(0, 1, 10)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swapped_and_repeated() {
        assert_eq!(ranking(vec![3, 9, 4]), [9, 4, 3]);
        assert_eq!(squares(&[2, 3]), [4, 9]);
    }

    #[test]
    fn bare_placeholders_take_the_free_numbers() {
        assert!(in_range(5, 1, 10));
        assert!(!in_range(0, 1, 10));
        assert_eq!(ml! { (between _1 _ _) 5 1 10 }, between(1, 5, 10));
    }
}

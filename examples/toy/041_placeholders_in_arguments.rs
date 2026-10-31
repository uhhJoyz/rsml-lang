// Toy 041: placeholders as short closures in arguments.
//
// What it is: a method argument, or an argument in parentheses, is a full
// expression (toy 040), so a `_` in it makes a closure of just that
// argument: `xs.map(_ * 2)` is `xs.map(fun x -> x * 2)`, without naming
// `x`.
//
// When to use it: for the small closures of `map`, `filter`, `any`,
// `all`, `sort_by_key`, when the parameter needs no name.
//
// How to write it:
// * `xs.iter().map(_ * 2)` is `xs.iter().map(|_0| _0 * 2)`.
// * A method of the parameter: `xs.iter().map(_.len())` is
//   `xs.iter().map(|_0| _0.len())`.
// * As an ML-style argument, in parentheses:
//   `count_where (_ > 0) xs` is `count_where(|_0| _0 > 0, xs)`.
// * Each `_` is a new parameter: `(_ > 0 && _ < 10)` is a function of
//   two. To use one parameter twice, number it (toy 039):
//   `(_0 > 0 && _0 < 10)`.
//
// Reference: [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-)

use rsml::ml;

pub const SUMMARY: &str = "Short closures: `.map(_ * 2)`, `.map(_.len())`, `(_0 > 0 && _0 < 10)`";

fn count_where(test: impl Fn(&i64) -> bool, xs: &[i64]) -> usize {
    xs.iter().filter(|x| test(x)).count()
}

/// The prices, doubled.
pub fn doubled(prices: &[i64]) -> Vec<i64> { ml! {
    // `map` passes a `&i64`, and `&i64 * i64` gives an `i64`.
    prices |> .iter() |> .map(_ * 2) |> .collect()
}}

/// The length of each word.
pub fn lengths(words: &[&str]) -> Vec<usize> { ml! {
    // `_.len()` calls the method on the parameter.
    words |> .iter() |> .map(_.len()) |> .collect()
}}

/// How many ratings are from 1 to 9.
pub fn valid_ratings(ratings: &[i64]) -> usize { ml! {
    // `(_0 > &0 && _0 < &10)` is `move |_0| (_0 > (&0)) && (_0 < (&10))`:
    // one parameter, a `&i64`, compared with references.
    count_where (_0 > &0 && _0 < &10) ratings
}}

/// Whether any word is empty.
pub fn any_empty(words: &[&str]) -> bool { ml! {
    words |> .iter() |> .any(_.is_empty())
}}

pub fn run() {
    println!("doubled [3; 5] = {:?}", doubled(&[3, 5]));
    println!(
        "lengths [\"tea\"; \"cocoa\"] = {:?}",
        lengths(&["tea", "cocoa"])
    );
    println!(
        "valid_ratings [0; 4; 9; 12] = {}",
        valid_ratings(&[0, 4, 9, 12])
    );
    println!("any_empty [\"a\"; \"\"] = {}", any_empty(&["a", ""]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closures_without_names() {
        assert_eq!(doubled(&[3, 5]), [6, 10]);
        assert_eq!(lengths(&["tea", "cocoa"]), [3, 5]);
        assert_eq!(valid_ratings(&[0, 4, 9, 12]), 2);
        assert!(any_empty(&["a", ""]));
        assert!(!any_empty(&["a", "b"]));
    }

    #[test]
    fn each_bare_placeholder_is_a_parameter() {
        let between = ml! { (_ > 0 && _ < 10) };
        assert!(between(1, 9));
        assert!(!between(1, 10));
    }
}

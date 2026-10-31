// Toy 040: how far a placeholder reaches.
//
// What it is: a `_` makes a function of the smallest *full* expression
// around it. The full expressions are the inside of parentheses, a list
// or array item, a method argument, a record field, a `let` value or
// body, a branch, a match case, a sequence item, and each side of `|>`,
// `@@`, `>>` and `<<`. Applications and the other operators do not stop
// it.
//
// When to use it: whenever you read or write a `_`: the rule says where
// its `fun` starts and ends.
//
// How to write it:
// * `(1 + _ * 2)` is `fun a -> 1 + a * 2`: one function, through `+`
//   and `*`.
// * `f (g _) x` is `f (fun a -> g a) x`: the parentheses stop it.
// * A placeholder alone, typed or not, belongs to the expression around
//   it: `f (_ : &str) 1` is `fun (a : &str) -> f a 1`.
// * `f _ 1 |> g` is `g (fun a -> f a 1)`: a side of `|>` is full.
// * The gotcha: `apply_to_all (_ + 1) _` is
//   `move |_0| apply_to_all(|_0| _0 + 1, _0)`, two functions: the one in
//   parentheses, and the one of the last `_`.
//
// Reference: [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-)

use rsml::ml;

pub const SUMMARY: &str = "Where a `_` function ends: `(_ * 9 / 5 + 32)`, `f (g _) x`";

fn add_percent(percent: i64, x: i64) -> i64 {
    x + x * percent / 100
}

fn apply_to_all(f: impl Fn(i64) -> i64, xs: &[i64]) -> Vec<i64> {
    xs.iter().map(|x| f(*x)).collect()
}

fn repeat_word(word: &str, times: usize) -> String {
    word.repeat(times)
}

/// Celsius to Fahrenheit: one function, across `*`, `/` and `+`.
pub fn fahrenheit(celsius: &[i64]) -> Vec<i64> { ml! {
    // `_ * 9 / 5 + 32` is `move |_0| ((_0 * 9) / 5) + 32`; as the
    // argument of `.map(..)` it is the closure `|_0| ((_0 * 9) / 5) + 32`.
    celsius |> .iter() |> .map(_ * 9 / 5 + 32) |> .collect()
}}

/// Balances with 10% interest: the parentheses end the function.
pub fn with_interest(balances: &[i64]) -> Vec<i64> { ml! {
    // `apply_to_all (add_percent 10 _) balances` is
    // `apply_to_all(|_0| add_percent(10, _0), balances)`.
    apply_to_all (add_percent 10 _) balances
}}

/// A function that writes any word three times.
pub fn tripler() -> impl Fn(&str) -> String { ml! {
    // `repeat_word (_ : &str) 3` is
    // `move |_0: &str| repeat_word(_0, 3)`: the typed `_` alone is a
    // parameter of the whole call.
    repeat_word (_ : &str) 3
}}

pub fn run() {
    println!(
        "fahrenheit [0; 100; -40] = {:?}",
        fahrenheit(&[0, 100, -40])
    );
    println!(
        "with_interest [100; 250] = {:?}",
        with_interest(&[100, 250])
    );
    println!("tripler () \"ho\" = {:?}", tripler()("ho"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaches() {
        assert_eq!(fahrenheit(&[0, 100, -40]), [32, 212, -40]);
        assert_eq!(with_interest(&[100, 250]), [110, 275]);
        assert_eq!(tripler()("ho"), "hohoho");
    }

    #[test]
    fn two_functions() {
        let bump = ml! { apply_to_all (_ + 1) _ };
        assert_eq!(bump(&[1, 2]), [2, 3]);
        let affine = ml! { (1 + _ * 2) };
        assert_eq!(affine(5), 11);
    }
}

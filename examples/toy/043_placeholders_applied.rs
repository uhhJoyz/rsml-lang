// Toy 043: a placeholder function applied on the spot makes no closure.
//
// What it is: when a placeholder function gets its arguments right away,
// the macro puts them where the holes are and writes a plain call, as
// for a `fun` applied on the spot (toy 014).
//
// When to use it: in pipelines, where the piped value fills the hole
// (`x |> f _ 1`), and to reorder the arguments of one call.
//
// How to write it:
// * `x |> f _ 1` is `f(x, 1)`.
// * `(f _ 1) 2` is `f(2, 1)`.
// * `(g _1 _0) a b` is `g(b, a)`: the arguments go to `_0`, then `_1`.
// * With fewer arguments than holes it is a partial application:
//   `(g _1 _0) a` is `move |_1| g(_1, a)`, waiting for `_1`.
//
// Reference: [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-)

use rsml::ml;

pub const SUMMARY: &str = "`x |> f _ 1` and `(g _1 _0) a b` are plain calls";

/// `amount` as a share of `total`, in percent.
fn percent_of(amount: u32, total: u32) -> u32 {
    amount * 100 / total
}

/// The share of a budget that one expense takes: the pipe fills the hole.
pub fn share(expense: u32, budget: u32) -> u32 {
    ml! {
        // `expense |> percent_of _ budget` is `percent_of(expense, budget)`.
        expense |> percent_of _ budget
    }
}

/// The same with the arguments given the other way round.
pub fn share_of(budget: u32, expense: u32) -> u32 {
    ml! {
        // `(percent_of _1 _0) budget expense` is
        // `percent_of(expense, budget)`.
        (percent_of _1 _0) budget expense
    }
}

/// A function of the expense, for a fixed budget.
pub fn shares_of(budget: u32, expenses: &[u32]) -> Vec<u32> {
    ml! {
        // `(percent_of _1 _0) budget` is `move |_1| percent_of(_1, budget)`.
        let of_budget = (percent_of _1 _0) budget in
        expenses |> .iter() |> .map(fun e -> of_budget !e) |> .collect()
    }
}

pub fn run() {
    println!("share 30 120 = {}", share(30, 120));
    println!("share_of 120 30 = {}", share_of(120, 30));
    println!("shares_of 200 [50; 20] = {:?}", shares_of(200, &[50, 20]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holes_filled_at_once() {
        assert_eq!(share(30, 120), 25);
        assert_eq!(share_of(120, 30), 25);
        assert_eq!(shares_of(200, &[50, 20]), [25, 10]);
    }

    #[test]
    fn applied_in_parentheses() {
        assert_eq!(ml! { (percent_of _ 200) 50 }, 25);
    }
}

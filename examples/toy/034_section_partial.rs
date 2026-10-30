// Toy 034: a section applied to one operand, `( + ) 1`, `( - ) 10`.
//
// What it is: a section applied to one argument is a partial application:
// the argument is the LEFT operand, and the result waits for the right
// one. `( + ) 1` adds one; `( - ) 10` subtracts from ten.
//
// When to use it: for "add n", "multiply by n", "n divided by" as a
// function, without naming the parameter.
//
// How to write it: `( op ) left`.
// * The gotcha: `( - ) 10` is `fun x -> 10 - x`, not `x - 10`. For the
//   other side, use a placeholder (toy 038): `(_ - 10)`.
// * The same goes for `( / )`, `( < )` and every operator that is not
//   symmetric.
//
// Reference: [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators)

use rsml::ml;

pub const SUMMARY: &str = "`( + ) 1` and `( - ) 10`: the argument is the left operand";

/// Every score raised by a bonus.
pub fn with_bonus(scores: &[i64], bonus: i64) -> Vec<i64> {
    ml! {
        // `( + ) bonus` is `move |rhs| bonus + rhs`.
        scores |> .iter() |> .copied() |> .map(( + ) bonus) |> .collect()
    }
}

/// How many points each score is short of 10.
pub fn short_of_ten(scores: &[i64]) -> Vec<i64> {
    ml! {
        // `( - ) 10` is `move |rhs| 10 - rhs`: ten minus the score.
        scores |> .iter() |> .copied() |> .map(( - ) 10) |> .collect()
    }
}

/// Every score minus 10: the placeholder puts the score on the left.
pub fn minus_ten(scores: &[i64]) -> Vec<i64> {
    ml! {
        // `(_ - 10)` is `move |_0| _0 - 10`.
        scores |> .iter() |> .copied() |> .map(_ - 10) |> .collect()
    }
}

/// The scores above a threshold.
pub fn above(scores: &[i64], threshold: i64) -> Vec<i64> {
    ml! {
        // `( < ) threshold` is `move |rhs| threshold < rhs`: "is the score
        // above the threshold", read with the threshold on the left.
        let is_above = ( < ) threshold in
        // `filter` passes a reference: `!s` is the score itself.
        scores |> .iter() |> .copied() |> .filter(fun s -> is_above !s) |> .collect()
    }
}

pub fn run() {
    let scores = [4, 9, 12];
    println!("scores = {scores:?}");
    println!("map (( + ) 5) = {:?}", with_bonus(&scores, 5));
    println!("map (( - ) 10) = {:?}", short_of_ten(&scores));
    println!("map (_ - 10) = {:?}", minus_ten(&scores));
    println!("filter (( < ) 8) = {:?}", above(&scores, 8));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_argument_is_the_left_operand() {
        assert_eq!(with_bonus(&[4, 9, 12], 5), [9, 14, 17]);
        assert_eq!(short_of_ten(&[4, 9, 12]), [6, 1, -2]);
        assert_eq!(minus_ten(&[4, 9, 12]), [-6, -1, 2]);
        assert_eq!(above(&[4, 9, 12], 8), [9, 12]);
    }
}

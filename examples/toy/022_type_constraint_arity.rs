// Toy 022: `(e : t)`, a type constraint, which can give an arity.
//
// What it is: `(e : t)` says that `e` has type `t`. With a plain type it
// is a type check that rustc makes. With a function type (`a -> b`), it
// tells the macro how many arguments the function takes, as `external`
// does (toy 020): `(std::cmp::max 0 : i64 -> i64)` is a function of one
// more argument, so a partial application.
//
// When to use it: to partially apply a Rust function once, in place,
// without declaring it for the whole block.
//
// How to write it: in parentheses, the expression, `:`, the type.
// * With a function type only the arrows count, as in `external`: the
//   types themselves are not checked.
// * With any other type the value is checked against it.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application)

use rsml::ml;

pub const SUMMARY: &str = "`(std::cmp::max 0 : i64 -> i64)` is a partial application";

/// Balances, with debts shown as zero.
pub fn floored(balances: &[i64]) -> Vec<i64> {
    ml! {
        // `(std::cmp::max 0 : i64 -> i64)` is
        // `move |arg2| std::cmp::max(0, arg2)`.
        let floor_at_zero = (std::cmp::max 0 : i64 -> i64) in
        balances |> .iter() |> .copied() |> .map(floor_at_zero) |> .collect()
    }
}

/// Only the arrows of a function type count, so its types can be `_`.
pub fn capped(balances: &[i64]) -> Vec<i64> {
    ml! {
        // `(std::cmp::min 100 : _ -> _)` is
        // `move |arg2| std::cmp::min(100, arg2)`. The types are not
        // checked at all: `(std::cmp::min 100 : String -> bool)` is
        // `move |arg2| std::cmp::min(100, arg2)` too.
        balances |> .iter() |> .copied() |> .map((std::cmp::min 100 : _ -> _)) |> .collect()
    }
}

/// With a plain type, a check: `(t : i64)` must be an `i64`.
pub fn total(balances: &[i64]) -> i64 {
    ml! {
        // `(t : i64)` is `{ let typed: i64 = t; typed }`.
        let t = balances |> .iter() |> .sum in
        (t : i64)
    }
}

pub fn run() {
    let balances = [120, -30, 250];
    println!("floored {balances:?} = {:?}", floored(&balances));
    println!("capped {balances:?} = {:?}", capped(&balances));
    println!("total {balances:?} = {}", total(&balances));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_types_give_arities() {
        assert_eq!(floored(&[120, -30, 250]), [120, 0, 250]);
        assert_eq!(capped(&[120, -30, 250]), [100, -30, 100]);
    }

    #[test]
    fn plain_types_are_checked() {
        assert_eq!(total(&[120, -30, 250]), 340);
    }
}

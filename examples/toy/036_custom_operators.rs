// Toy 036: your own operators, `let ( +^ ) a b = ..`.
//
// What it is: `let ( op ) a b = e in ..` defines an infix operator, a
// function of two arguments written between them. Its precedence and
// associativity come from its first character: `*^` binds like `*`, `+^`
// like `+`.
//
// When to use it: for a small algebra that reads better infix, like the
// 2D vectors here.
//
// How to write it: `let ( +^ ) (ax, ay) (bx, by) = (ax + bx, ay + by) in`.
// The parameters are patterns, as for any `let` function (toy 007). Then
// use it infix, `p +^ q`, or in parentheses as a function, `( +^ )`.
// * An operator nobody defined is an error: `2 +^ 3` is an error:
//   "unknown operator `+^`; define it with `let ( +^ ) a b = ..`"
// * It is a closure, like a `let` function, so its parameter types are
//   inferred and it is used at one type.
//
// Reference: [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators)

use rsml::ml;

pub const SUMMARY: &str = "Defining infix operators: `let ( +^ ) a b = ..`, precedence";

/// Where a boat ends up after drifting with the wind for some hours.
pub fn drift(start: (i64, i64), wind: (i64, i64), hours: i64) -> (i64, i64) { ml! {
    let ( +^ ) (ax, ay) (bx, by) = (ax + bx, ay + by) in
    let ( *^ ) k (x, y) = (k * x, k * y) in
    // `*^` starts with `*`, so it binds tighter than `+^`, as `*` does
    // than `+`. Given `let ( +^ ) a b = a + b let ( *^ ) k v = k * v`,
    // `p +^ 2 *^ q` is `p +^ (2 *^ q)`.
    start +^ hours *^ wind
}}

/// Where a walk of steps ends, from the origin.
pub fn walk(steps: &[(i64, i64)]) -> (i64, i64) { ml! {
    let ( +^ ) (ax, ay) (bx, by) = (ax + bx, ay + by) in
    // In parentheses the operator is a function, as `( + )` is (toy
    // 033): here `fold`'s function of the position and a step.
    steps |> .iter() |> .copied() |> .fold((0, 0), ( +^ ))
}}

pub fn run() {
    println!("drift (1, 1) (2, -1) 3 = {:?}", drift((1, 1), (2, -1), 3));
    println!(
        "walk [(1, 0); (0, 2); (-3, 1)] = {:?}",
        walk(&[(1, 0), (0, 2), (-3, 1)])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence_from_the_first_character() {
        // (1, 1) + 3 * (2, -1), not ((1, 1) + 3) * (2, -1).
        assert_eq!(drift((1, 1), (2, -1), 3), (7, -2));
    }

    #[test]
    fn an_operator_as_a_function() {
        assert_eq!(walk(&[(1, 0), (0, 2), (-3, 1)]), (-2, 3));
        assert_eq!(walk(&[]), (0, 0));
    }
}

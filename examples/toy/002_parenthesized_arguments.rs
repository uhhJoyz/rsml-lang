// Toy 002: which arguments need parentheses.
//
// What it is: in `f a b`, each argument is one "atom": a name, a literal,
// a path, a field or method call written after a name (`p.x`,
// `s.len()`), or anything in parentheses or brackets. Everything else, an
// operator expression, another application, a negative number, goes in
// parentheses.
//
// When to use it: whenever an argument is computed.
//
// How to write it: `f (x + 1) y`, `f (g x)`, `f (-1)`. The gotchas:
// * `f -1` is a subtraction, `f - 1`, as in OCaml.
// * `f s.len () 2` passes the field `s.len`, then `()`: a method call has
//   no space before its parenthesis, `s.len()` (toy 052).
//
// Reference: [Application](../../docs/notation.md#application)

use rsml::ml;

pub const SUMMARY: &str = "Arguments are atoms: `f (x + 1) (g y)`, `f (-1)`, `f s.len()`";

fn clamp(low: i64, x: i64, high: i64) -> i64 {
    x.max(low).min(high)
}

fn double(x: i64) -> i64 {
    x * 2
}

/// `x + 1`, kept between -1 and 10.
pub fn next_level(x: i64) -> i64 {
    ml! {
        // `clamp (-1) (x + 1) 10` is `clamp(-1, x + 1, 10)`.
        clamp (-1) (x + 1) 10
    }
}

/// An application as an argument is in parentheses too.
pub fn doubled_level(x: i64) -> i64 {
    ml! {
        // `clamp 0 (double x) 10` is `clamp(0, double(x), 10)`.
        clamp 0 (double x) 10
    }
}

/// Fields and method calls after a name are atoms: no parentheses needed.
pub fn width_or_len(size: (i64, i64), word: &str) -> i64 {
    ml! {
        // `std::cmp::max size.0 (word.len() as i64)` is
        // `std::cmp::max(size.0, word.len() as i64)`; the cast needs the
        // parentheses, `word.len()` alone would not.
        std::cmp::max size.0 (word.len() as i64)
    }
}

/// Without parentheses, `-` is subtraction.
pub fn minus_one(x: i64) -> i64 {
    ml! {
        // rsmlfmt: skip
        // (The formatter would print `x - 1`; the spelling is the lesson.)
        //
        // `double -1` would be `double - 1`, a function minus a number,
        // which rustc rejects; `x -1` is `x - 1`.
        x -1
    }
}

pub fn run() {
    println!(
        "next_level 3 = {}, next_level 20 = {}",
        next_level(3),
        next_level(20)
    );
    println!("doubled_level 7 = {}", doubled_level(7));
    println!(
        "width_or_len (3, 9) \"hello\" = {}",
        width_or_len((3, 9), "hello")
    );
    println!("minus_one 5 = {}", minus_one(5));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computed_arguments() {
        assert_eq!(next_level(3), 4);
        assert_eq!(next_level(20), 10);
        assert_eq!(next_level(-7), -1);
        assert_eq!(doubled_level(7), 10);
        assert_eq!(width_or_len((3, 9), "hello"), 5);
        assert_eq!(width_or_len((8, 9), "hello"), 8);
    }

    #[test]
    fn a_space_before_minus_one_subtracts() {
        assert_eq!(minus_one(5), 4);
    }
}

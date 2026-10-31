// Toy 129: Rust habits in expressions, and what to write instead.
//
// What it is: some Rust spellings mean something else in rsml code, or
// nothing. rsml rejects them with an error that says what to write. The
// wrong spellings cannot compile, so they are quoted here with their
// errors (scripts/check_docs.py checks each message), and the code below
// is the ML spelling.
//
// When to use it: when an error message sends you here, or before you
// write Rust from memory.
//
// How to write it:
// * Closures are not among them: `|x| x + 1` is accepted, the same
//   closure as `fun x -> x + 1`. As an argument it goes in parentheses,
//   as `fun` does: `f (|x| x)` is `f(|x| x)`, and `xs.map(|x| x + 1)` is
//   `xs.map(|x| x + 1)`.
// * `if let Some(x) = o { x } else { 0 }` is an error: "this is Rust's
//   `if let`: write `match e with p -> .. | _ -> ..`"
// * `match o { Some(x) => x, None => 0 }` is an error: "this is Rust's
//   `match x { p => e }`: write `match x with p -> e | ..`"
// * `x += 1` is an error: "rsml has no `+=`: write `x <- x + e` (for a
//   `ref`: `r := !r + e`, or `incr r`), or define the operator with
//   `let ( += ) a b = ..`"
// * `assert!(!xs.is_empty())` is an error: "`!` dereferences, and binds
//   tighter than `.`: `!x.f()` is `(!x).f()`. To negate, write
//   `not (x.f())`; to dereference, `(!x).f()`"
// * `!(a = b)` is an error: "a `bool` cannot be dereferenced: `!r` is the
//   value of `r` here, and negation is `not e`"
//
// Reference: [Rust habits](../../docs/notation.md#rust-habits)

use rsml::ml;

pub const SUMMARY: &str =
    "Rust habits rejected: `if let`, `match {}`, `+=`, `!` for not; `|x| e` is accepted";

/// Each price with tax: a Rust closure, as an argument in parentheses.
pub fn with_tax(prices: &[u32]) -> Vec<u32> { ml! {
    // `fun p -> p + p / 5` is `|p| p + p / 5`.
    prices |> .iter() |> .map(|p| p + p / 5) |> .collect()
}}

/// A discount code's value: `match .. with`, not `if let` or `match {}`.
pub fn discount(code: Option<u32>) -> u32 { ml! {
    match code with Some percent -> percent | None -> 0
}}

/// A running count: `n <- n + 1` (or `incr`), not `+=`.
pub fn count_over(limit: u32, xs: &[u32]) -> u32 { ml! {
    let mut n = 0 in
    for &x in xs do if x > limit then n <- n + 1 done;
    n
}}

/// A check that a list has items: `not`, not `!`.
pub fn check_not_empty(xs: &[u32]) -> usize { ml! {
    assert!(not (xs.is_empty()), "no items");
    assert!(not (xs.len() = 0));
    xs.len()
}}

pub fn run() {
    println!("with_tax [100; 50] = {:?}", with_tax(&[100, 50]));
    println!(
        "discount (Some 15) = {}, discount None = {}",
        discount(Some(15)),
        discount(None)
    );
    println!(
        "count_over 10 [5; 20; 30] = {}",
        count_over(10, &[5, 20, 30])
    );
    println!("check_not_empty [1; 2] = {}", check_not_empty(&[1, 2]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ocaml_spellings() {
        assert_eq!(with_tax(&[100, 50]), [120, 60]);
        assert_eq!(discount(Some(15)), 15);
        assert_eq!(discount(None), 0);
        assert_eq!(count_over(10, &[5, 20, 30]), 2);
        assert_eq!(check_not_empty(&[1, 2]), 2);
    }

    #[test]
    #[should_panic(expected = "no items")]
    fn the_assertion_holds_its_meaning() {
        check_not_empty(&[]);
    }
}

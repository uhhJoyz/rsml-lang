// Toy 008: recursive functions, `let rec`.
//
// What it is: `let rec f params : result = body in ..` defines a function
// that can call itself. It becomes a nested Rust `fn`, not a closure (a
// closure cannot call itself).
//
// When to use it: for recursion.
//
// How to write it:
// * Annotate every parameter and the result, as a Rust `fn` needs:
//   `let rec fact (n : u64) : u64 = ..`.
// * A nested `fn` cannot use the locals around it: pass them as
//   parameters. rustc says "can't capture dynamic environment in a fn
//   item" otherwise.
// * With `function` (toy 016), the type goes after the name and the
//   parameter is the matched value:
//   `let rec fib : u64 -> u64 = function 0 -> 0 | ..`.
// * A function that calls itself only as its result (a tail call: the
//   branch of an `if` or `match`, the body of a `let`, the end of a
//   sequence) becomes a loop, which runs in the same stack space however
//   deep it goes, in debug builds too (Rust does not promise that for a
//   recursive `fn`):
//   `let rec go (k : u64) : u64 = if k = 0 then 0 else go (k - 1) in go 3`
//   is `{ fn go(mut arg1: u64) -> u64 { loop { let k = arg1; break if k == 0 { 0 } else { arg1 = k - 1; continue }; } } go(3) }`.
//   One call that is not a tail call (`1 + go (k - 1)`), or `go` used as
//   a value, keeps the plain recursive `fn`; so do `let rec .. and ..`.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)

use rsml::ml;

pub const SUMMARY: &str =
    "`let rec f (n : t) : t = ..`: a recursive, nested `fn` (a loop for tail calls)";

/// n! = 1 * 2 * .. * n.
pub fn factorial(n: u64) -> u64 { ml! {
    // `let rec fact (k : u64) : u64 = if k = 0 then 1 else k * fact (k - 1) in fact n`
    // is `{ fn fact(k: u64) -> u64 { if k == 0 { 1 } else { k * fact(k - 1) } } fact(n) }`.
    let rec fact (k : u64) : u64 = if k = 0 then 1 else k * fact (k - 1) in
    fact n
}}

/// How many digits `n` has in base `base`.
pub fn digit_count(n: u64, base: u64) -> u32 { ml! {
    // `base` is a parameter of `count`: a nested `fn` cannot see the
    // `base` of `digit_count`.
    let rec count (n : u64) (base : u64) : u32 = if n < base then 1 else 1 + count (n / base) base in
    count n base
}}

/// The Fibonacci numbers 0, 1, 1, 2, 3, 5, ..
pub fn fibonacci(n: u64) -> u64 { ml! {
    // `let rec fib : u64 -> u64 = function 0 -> 0 | k -> k + fib (k - 1) in fib n`
    // is `{ fn fib(argument: u64) -> u64 { match argument { 0 => 0, k => k + fib(k - 1), } } fib(n) }`.
    let rec fib : u64 -> u64 = function 0 -> 0 | 1 -> 1 | k -> fib (k - 1) + fib (k - 2) in
    fib n
}}

/// 1 + 2 + .. + n, by a tail call: a loop, however large `n` is.
pub fn sum_to(n: u64) -> u64 { ml! {
    let rec go (acc : u64) (k : u64) : u64 = if k = 0 then acc else go (acc + k) (k - 1) in
    go 0 n
}}

pub fn run() {
    println!(
        "factorial 5 = {}, factorial 0 = {}",
        factorial(5),
        factorial(0)
    );
    println!(
        "digit_count 255 16 = {}, digit_count 1000 10 = {}",
        digit_count(255, 16),
        digit_count(1000, 10)
    );
    println!("fibonacci 10 = {}", fibonacci(10));
    println!("sum_to 1000000 = {}", sum_to(1_000_000));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursion() {
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(0), 1);
        assert_eq!(digit_count(255, 16), 2);
        assert_eq!(digit_count(1000, 10), 4);
        assert_eq!(digit_count(0, 2), 1);
        assert_eq!(fibonacci(10), 55);
    }

    #[test]
    fn a_tail_call_is_a_loop() {
        // A million calls deep: a recursive `fn` would overflow a test
        // thread's stack in a debug build.
        assert_eq!(sum_to(1_000_000), 500_000_500_000);
    }
}

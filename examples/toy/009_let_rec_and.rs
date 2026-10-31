// Toy 009: functions that call each other, `let rec .. and ..`.
//
// What it is: `let rec f .. = .. and g .. = .. in ..` defines several
// recursive functions at once, each able to call the others: nested Rust
// `fn`s, annotated as in toy 008. Without `rec`, `let a = e1 and b = e2 in`
// binds several values at once: neither value sees the other new name.
//
// When to use it: `let rec .. and ..` for mutual recursion; `let .. and ..`
// to compute new values from the old ones, as in a swap.
//
// How to write it:
// * Each function is annotated: `and odd (k : u32) : bool = ..`.
// * In `let a = b and b = a + b in ..`, both values are computed first,
//   from the `a` and `b` that were there before.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)

use rsml::ml;

pub const SUMMARY: &str = "Mutual recursion `let rec .. and ..`; `let a = .. and b = ..`";

/// "even" or "odd", by two functions that call each other.
pub fn parity(n: u32) -> &'static str { ml! {
    let rec even (k : u32) : bool = if k = 0 then true else odd (k - 1)
    and odd (k : u32) : bool = if k = 0 then false else even (k - 1) in
    if even n then "even" else "odd"
}}

/// One step of the Fibonacci sequence: (a, b) becomes (b, a + b).
pub fn fib_step(a: u64, b: u64) -> (u64, u64) { ml! {
    // `let a = b and b = a + b in (a, b)` is
    // `{ let value1 = b; let value2 = a + b; let a = value1; let b = value2; (a, b) }`:
    // `a + b` uses the old `a`.
    let a = b
    and b = a + b in
    (a, b)
}}

/// The same step with `let .. in`, one after the other: the second `let`
/// would see the new `a`, so the old one is saved first.
pub fn fib_step_in_turn(a: u64, b: u64) -> (u64, u64) { ml! {
    let old_a = a in
    let a = b in
    let b = old_a + b in
    (a, b)
}}

pub fn run() {
    println!("parity 10 = {}, parity 7 = {}", parity(10), parity(7));
    println!("fib_step 3 5 = {:?}", fib_step(3, 5));
    println!("fib_step_in_turn 3 5 = {:?}", fib_step_in_turn(3, 5));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutual_recursion() {
        assert_eq!(parity(10), "even");
        assert_eq!(parity(7), "odd");
        assert_eq!(parity(0), "even");
    }

    #[test]
    fn simultaneous_bindings() {
        assert_eq!(fib_step(3, 5), (5, 8));
        assert_eq!(fib_step_in_turn(3, 5), (5, 8));
    }
}

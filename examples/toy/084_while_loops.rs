// Toy 084: `while c do .. done`.
//
// What it is: the loop that runs its body as long as the condition holds.
// It is Rust's `while`: `while c do .. done` is `while c { ..; }`.
//
// When to use it: when the number of steps is not known in advance, such
// as halving until a value is small enough.
//
// How to write it: `while condition do body done`. The body is a unit
// block, usually a sequence of updates (`n <- ..`, toy 090, or `r := ..`,
// toy 089). `while true` is the endless loop (toy 085).
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`while n > 1 do .. done`: loop while a condition holds";

/// The steps of the Collatz sequence from `n` down to 1.
pub fn collatz_steps(n: u64) -> u32 { ml! {
    let mut n = n in
    let mut steps = 0 in
    while n > 1 do
      n <- if n mod 2 = 0 then n / 2 else 3 * n + 1;
      steps <- steps + 1
    done;
    steps
}}

/// How many years until a savings account doubles, at `rate` percent.
pub fn years_to_double(rate: f64) -> u32 { ml! {
    let mut amount = 1.0 in
    let mut years = 0 in
    while amount < 2.0 do
      amount <- amount *. (1.0 +. rate /. 100.0);
      years <- years + 1
    done;
    years
}}

pub fn run() {
    println!("collatz_steps 6 = {}", collatz_steps(6));
    println!("years_to_double 7.0 = {}", years_to_double(7.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn until_the_condition_fails() {
        // 6 3 10 5 16 8 4 2 1: eight steps.
        assert_eq!(collatz_steps(6), 8);
        assert_eq!(collatz_steps(1), 0);
        assert_eq!(years_to_double(7.0), 11);
    }
}

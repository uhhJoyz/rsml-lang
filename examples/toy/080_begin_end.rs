// Toy 080: grouping, `begin a; b end` and `( a; b )`.
//
// What it is: `begin .. end` groups a sequence into one expression, as
// parentheses do; both are a Rust block: `begin a; b end` is `{ a; b }`,
// and so is `( a; b )`.
//
// When to use it: for a branch of `if` or a case of `match` with several
// steps: without the grouping, only the first step belongs to the branch
// (toy 078).
//
// How to write it:
// * `if c then begin a; b end else c`, or `if c then (a; b) else c`.
// * Rust's braces are not a block in rsml code: `{ a; b }` is an error:
//   "`{ .. }` is not a block in rsml: write `begin .. end` or `( .. )`"
//   (toy 130).
// * The braces of `unsafe { .. }` and `async { .. }` hold rsml code as
//   `begin .. end` does (toys 120, 121).
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`begin a; b end` and `( a; b )` group steps into one";

/// A withdrawal: when it is allowed, log it and lower the balance.
pub fn withdraw(balance: u32, amount: u32) -> (u32, Vec<String>) { ml! {
    let mut log = Vec.new () in
    let left =
      if amount <= balance then begin
        log.push(format!("withdrew {amount}"));
        balance - amount
      end
      else begin
        log.push(String.from "refused");
        balance
      end
    in
    (left, log)
}}

/// The same choice, grouped with parentheses.
pub fn deposit(balance: u32, amount: u32) -> (u32, usize) { ml! {
    let mut count = 0 in
    // `( count <- count + 1; balance + amount )` is one branch.
    let total =
      if amount > 0 then
        (count <- count + 1;
         balance + amount)
      else
        balance
    in
    (total, count)
}}

pub fn run() {
    println!("withdraw 100 30 = {:?}", withdraw(100, 30));
    println!("withdraw 100 300 = {:?}", withdraw(100, 300));
    println!("deposit 100 5 = {:?}", deposit(100, 5));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_branches() {
        assert_eq!(withdraw(100, 30), (70, vec!["withdrew 30".to_string()]));
        assert_eq!(withdraw(100, 300), (100, vec!["refused".to_string()]));
        assert_eq!(deposit(100, 5), (105, 1));
        assert_eq!(deposit(100, 0), (100, 0));
    }
}

// Toy 078: `if c then a else b`.
//
// What it is: the conditional, an expression with a value like Rust's
// `if c { a } else { b }`. Without `else` it is a statement: its `then`
// branch must be `()`, the unit value.
//
// When to use it: for every two-way choice; `else if` chains for more.
//
// How to write it:
// * No braces: `if total > 100 then 0 else 5`. Each branch reaches as far
//   as a `let`'s value would: `else 2 + 3` is the whole sum.
// * `if` binds tighter than `;`: in `if c then a; b`, `b` runs whatever
//   `c` is. Put a branch of several steps in `begin .. end` or `( .. )`
//   (toy 080).
// * The condition is a `bool`: `n = 0` compares (toy 067).
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`if c then a else b`, `else if` chains, `if` without `else`";

/// Shipping costs nothing above 100.
pub fn shipping(total: u32) -> u32 { ml! {
    // `if c then a else b` is `if c { a } else { b }`.
    if total > 100 then 0 else 5
}}

/// The sign of a number, in words.
pub fn sign(n: i64) -> &'static str { ml! {
    // `if n < 0 then "-" else if n = 0 then "0" else "+"` is
    // `if n < 0 { "-" } else if n == 0 { "0" } else { "+" }`.
    if n < 0 then "negative" else if n = 0 then "zero" else "positive"
}}

/// The warnings for an order: `if` without `else` for the optional ones.
pub fn warnings(total: u32, items: usize) -> Vec<&'static str> { ml! {
    let mut out = Vec.new () in
    // `if c then log x; log y` is `{ if c { log(x) }; log(y) }`:
    // only the first step is conditional.
    if total = 0 then out.push("empty order");
    if items > 10 then begin
      out.push("many items");
      out.push("check the stock")
    end;
    out
}}

pub fn run() {
    println!(
        "shipping 120 = {}, shipping 30 = {}",
        shipping(120),
        shipping(30)
    );
    println!(
        "sign -4 = {}, sign 0 = {}, sign 9 = {}",
        sign(-4),
        sign(0),
        sign(9)
    );
    println!("warnings 0 0 = {:?}", warnings(0, 0));
    println!("warnings 50 12 = {:?}", warnings(50, 12));
    println!("warnings 50 1 = {:?}", warnings(50, 1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        assert_eq!(shipping(120), 0);
        assert_eq!(shipping(30), 5);
        assert_eq!(sign(-4), "negative");
        assert_eq!(sign(0), "zero");
        assert_eq!(sign(9), "positive");
        assert_eq!(ml! { if true then 1 else 2 + 3 }, 1);
        assert_eq!(ml! { if false then 1 else 2 + 3 }, 5);
    }

    #[test]
    fn statements() {
        assert_eq!(warnings(0, 0), ["empty order"]);
        assert_eq!(warnings(50, 12), ["many items", "check the stock"]);
        assert!(warnings(50, 1).is_empty());
    }
}

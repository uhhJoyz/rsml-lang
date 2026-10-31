// Toy 114: functions of the same `rsml!` know each other's arity.
//
// What it is: the macro reads every signature in an `rsml!` block, so in
// each body the block's functions are known like local `let` functions:
// `add 1` is a partial application, `add 1 2` a call, with no `external`
// (toy 020). Their shape comes from the signature:
// * one group of parameters, `()` for none, so `now ()` is `now()`;
// * one more call for a returned closure (`-> impl Fn(..)`,
//   `-> Box<dyn Fn(..)>`): with `fn adder(n: i64) -> impl Fn(i64) -> i64`,
//   `adder 1 2` calls `adder(1)`, then the closure it returns with 2;
// * a pattern parameter such as `(a, b): (u32, u32)` counts as one.
//
// When to use it: always, in `rsml!`: write the block's functions ML
// style and partially apply them freely.
//
// How to write it: nothing to declare. In full:
// `rsml! { fn add(a: i64, b: i64) -> i64 { a + b } fn inc() -> impl Fn(i64) -> i64 { add 1 } }`
// is `fn add(a: i64, b: i64) -> i64 { a + b } fn inc() -> impl Fn(i64) -> i64 { move |arg2| add(1, arg2) }`,
// and `rsml! { fn adder(n: i64) -> impl Fn(i64) -> i64 { fun m -> n + m } fn three() -> i64 { adder 1 2 } }`
// is `fn adder(n: i64) -> impl Fn(i64) -> i64 { move |m| n + m } fn three() -> i64 { adder(1)(2) }`.
// * Methods taking `self` are left out (call them as methods), and so
//   are functions of other `rsml!` blocks and plain Rust ones.
// * Parameters and local bindings of the same name hide them.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "Functions of one `rsml!` are known: `with_tax 20` is partial";

rsml! {
/// A price with tax, in cents.
pub fn with_tax(rate: u32, cents: u32) -> u32 {
    cents + cents * rate / 100
}

/// The tax rate today: a function without parameters.
pub fn rate_today() -> u32 {
    20
}

/// A discount function, returned: `discount 10` is a closure.
pub fn discount(percent: u32) -> impl Fn(u32) -> u32 {
    fun cents -> cents - cents * percent / 100
}

/// A basket's prices with today's tax and 10% off.
pub fn basket(prices: &[u32]) -> Vec<u32> {
    // `with_tax (rate_today ())` waits for the price, and
    // `discount 10 >> ..` composes the returned closure.
    prices |> .iter() |> .copied() |> .map(with_tax (rate_today ()) >> discount 10) |> .collect()
}

/// One price: `discount 10` makes the closure, which gets `cents`.
pub fn one(cents: u32) -> u32 {
    discount 10 cents
}
}

pub fn run() {
    println!("basket [100; 250] = {:?}", basket(&[100, 250]));
    println!("one 1000 = {}", one(1000));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_shapes() {
        // 100 + 20% = 120, then 10% off = 108; 250 -> 300 -> 270.
        assert_eq!(basket(&[100, 250]), [108, 270]);
        assert_eq!(one(1000), 900);
        assert_eq!(rate_today(), 20);
    }
}

// Toy 023: closure-typed parameters of `rsml!` functions.
//
// What it is: in `rsml! { fn .. }` the macro reads the signature. A
// parameter whose type is a closure, `impl Fn(u32, u32) -> u32`, has a
// known arity, so the body can partially apply it like a `let` function.
//
// When to use it: in functions that take a policy, a callback or an
// operation as an argument, and apply it to many values.
//
// How to write it: give the parameter a closure type (`impl Fn(..)`, also
// `impl FnMut(..)`, `&dyn Fn(..)` or a generic bounded by `Fn(..)`: toy
// 113), then use it OCaml style in the body.
// * Its arguments in `Fn(a, b)` count, one each: `discount 10` misses one.
// * The caller passes any closure, written in Rust or in OCaml.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application)

use rsml::rsml;

pub const SUMMARY: &str = "A parameter `discount: impl Fn(u32, u32) -> u32` allows `discount 10`";

rsml! {
    /// Each price with a 10% discount, as `discount` computes it.
    pub fn sale(discount: impl Fn(u32, u32) -> u32, prices: &[u32]) -> Vec<u32> {
        // `discount 10` is a closure waiting for the price.
        prices |> .iter() |> .copied() |> .map(discount 10) |> .collect()
    }

    /// The parameter called with every argument: a plain call.
    pub fn one_sale(discount: impl Fn(u32, u32) -> u32, price: u32) -> u32 {
        discount 25 price
    }
}

// In full: `rsml! { fn inc(add: impl Fn(i64, i64) -> i64)
// -> impl Fn(i64) -> i64 { add 1 } }` is `fn inc(add: impl Fn(i64, i64)
// -> i64) -> impl Fn(i64) -> i64 { move |arg2| add(1, arg2) }`.

/// The usual discount: `percent` off the price.
fn percent_off(percent: u32, price: u32) -> u32 {
    price - price * percent / 100
}

pub fn run() {
    println!(
        "sale percent_off [200; 50] = {:?}",
        sale(percent_off, &[200, 50])
    );
    // A caller's closure, here one that never goes under 100.
    let floor = |percent: u32, price: u32| percent_off(percent, price).max(100);
    println!("sale floor [200; 50] = {:?}", sale(floor, &[200, 50]));
    println!("one_sale percent_off 80 = {}", one_sale(percent_off, 80));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partially_applied_parameter() {
        assert_eq!(sale(percent_off, &[200, 50]), [180, 45]);
        assert_eq!(
            sale(|p, x| percent_off(p, x).max(100), &[200, 50]),
            [180, 100]
        );
    }

    #[test]
    fn fully_applied_parameter() {
        assert_eq!(one_sale(percent_off, 80), 60);
    }
}

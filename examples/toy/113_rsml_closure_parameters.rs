// Toy 113: every closure-typed parameter has a known arity.
//
// What it is: toy 023 partially applied an `impl Fn(..)` parameter. The
// macro reads the other ways a Rust signature says "a closure" too: a
// `&dyn Fn(..)` parameter, a generic `F` bounded by `Fn(..)` in the
// angle brackets or in a `where` clause, and the same with `FnMut` and
// `FnOnce`. A function that returns a closure (`impl Fn(..)`,
// `Box<dyn Fn(..)>`) takes one more call (toy 114).
//
// When to use it: for higher-order functions written in `rsml!`: the
// body applies its function parameters ML style, fully or partially.
//
// How to write it:
// * `rsml! { fn f(g: &dyn Fn(i64, i64) -> i64) -> i64 { g 1 2 } }` is
//   `fn f(g: &dyn Fn(i64, i64) -> i64) -> i64 { g(1, 2) }`.
// * With a generic bound, `g 1` is a partial application:
//   `rsml! { fn f<F: Fn(i64, i64) -> i64>(g: F) -> impl Fn(i64) -> i64 { g 1 } }`
//   is `fn f<F: Fn(i64, i64) -> i64>(g: F) -> impl Fn(i64) -> i64 { move |arg2| g(1, arg2) }`.
// * A Rust-style call counts its items against the arity (toy 054):
//   `rsml! { fn f<F>(g: F) -> i64 where F: Fn(i64, i64) -> i64 { g(1)(2) } }`
//   is `fn f<F>(g: F) -> i64 where F: Fn(i64, i64) -> i64 { g(1, 2) }`.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "`&dyn Fn`, `F: Fn(..)`, `where F: Fn(..)` parameters, applied ML style";

rsml! {
/// A price list run through a pricing rule given as `&dyn Fn`.
pub fn priced(rule: &dyn Fn(u32, u32) -> u32, quantities: &[u32], unit: u32) -> Vec<u32> {
    // `rule !q unit` calls the rule with both arguments.
    quantities |> .iter() |> .map(fun q -> rule !q unit) |> .collect()
}

/// A rule fixed to a quantity: the generic `F` is partially applied.
pub fn for_quantity<F: Fn(u32, u32) -> u32>(rule: F, quantity: u32) -> impl Fn(u32) -> u32 {
    rule quantity
}

/// The same with a `where` clause, called Rust style.
pub fn dozen<F>(rule: F, unit: u32) -> u32
where
    F: Fn(u32, u32) -> u32,
{
    // `rule(12)(unit)` and `rule(12, unit)` are the same call.
    rule(12)(unit)
}
}

/// The pricing rule: every tenth item is free.
fn tenth_free(quantity: u32, unit: u32) -> u32 {
    (quantity - quantity / 10) * unit
}

pub fn run() {
    println!(
        "priced tenth_free [5; 10; 20] 3 = {:?}",
        priced(&tenth_free, &[5, 10, 20], 3)
    );
    let ten = for_quantity(tenth_free, 10);
    println!(
        "for_quantity tenth_free 10, at 4 and at 7 = {:?}",
        (ten(4), ten(7))
    );
    println!("dozen tenth_free 2 = {}", dozen(tenth_free, 2));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_parameters() {
        assert_eq!(priced(&tenth_free, &[5, 10, 20], 3), [15, 27, 54]);
        let ten = for_quantity(tenth_free, 10);
        assert_eq!((ten(4), ten(7)), (36, 63));
        assert_eq!(dozen(tenth_free, 2), 22);
        assert_eq!(dozen(|q, u| q * u, 2), 24);
    }
}

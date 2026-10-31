// Toy 024: the pipeline `x |> f`.
//
// What it is: `x |> f` applies `f` to `x`, and `x |> f a` applies `f a`
// to `x`, so `x` goes LAST: `f(a, x)`. A chain of `|>` reads in the order
// things happen, where nested calls read inside out.
//
// When to use it: whenever a value goes through several steps.
//
// How to write it: the value, then `|>` and a function before each step.
// * No arity is needed: the result is always one plain call, so this
//   works with any Rust function, as long as it takes the piped value as
//   its last parameter (toy 025 has the others).
// * `|>` binds looser than application: `x |> f a` is `x |> (f a)`.
//
// Reference: [Pipelines: |> and @@](../../docs/notation.md#pipelines--and-)

use rsml::ml;

pub const SUMMARY: &str = "`items |> subtotal |> discount 10 |> add_shipping 5`";

/// The total of `(quantity, unit price)` lines, in cents.
fn subtotal(items: &[(u32, u32)]) -> u32 {
    items.iter().map(|(q, p)| q * p).sum()
}

/// `percent` off a total.
fn discount(percent: u32, total: u32) -> u32 {
    total - total * percent / 100
}

/// A shipping fee added to a total.
fn add_shipping(fee: u32, total: u32) -> u32 {
    total + fee
}

/// The bill: subtotal, 10% off, then 5 cents of shipping.
pub fn bill(items: &[(u32, u32)]) -> u32 { ml! {
    // `items |> subtotal |> discount 10 |> add_shipping 5` is
    // `add_shipping(5, discount(10, subtotal(items)))`.
    items |> subtotal |> discount 10 |> add_shipping 5
}}

/// One step at a time.
pub fn steps(items: &[(u32, u32)]) -> (u32, u32, u32) { ml! {
    // `items |> subtotal` is `subtotal(items)`.
    let s = items |> subtotal in
    // `s |> discount 10` is `discount(10, s)`: the value goes last.
    let d = s |> discount 10 in
    (s, d, d |> add_shipping 5)
}}

pub fn run() {
    let items = [(2, 300), (1, 400)];
    println!("items = {items:?}");
    println!("steps items = {:?}", steps(&items));
    println!("bill items = {}", bill(&items));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pipeline_is_nested_calls() {
        let items = [(2, 300), (1, 400)];
        assert_eq!(steps(&items), (1000, 900, 905));
        assert_eq!(bill(&items), 905);
        assert_eq!(
            bill(&items),
            add_shipping(5, discount(10, subtotal(&items)))
        );
    }
}

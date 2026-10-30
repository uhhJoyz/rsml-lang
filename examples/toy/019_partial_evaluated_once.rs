// Toy 019: the arguments of a partial application are evaluated once.
//
// What it is: in `with_tax (lookup_rate &lookups)`, the argument is
// computed when the partial application is made, as in OCaml, and the
// closure keeps its value. A `fun` that makes the call itself computes it
// on every call.
//
// When to use it: when an argument is costly, or must be read once (a
// rate, a configuration), and the function is then used many times.
//
// How to write it: as any partial application (toy 018). The macro binds
// each argument that is computed (anything but a name, a path or a
// literal) to a local first:
// `let add x y = x + y in add (expensive ())` is
// `{ let add = |x, y| x + y; { let value1 = expensive(); move |arg2| add(value1, arg2) } }`.
//
// Given `external add : _ -> _ -> _`, `add x` is
// `move |arg2| add(x, arg2)`, while `add (x + 1)` is
// `{ let value1 = x + 1; move |arg2| add(value1, arg2) }`.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application)

use rsml::ml;
use std::cell::Cell;

pub const SUMMARY: &str = "`with_tax (lookup ())` computes `lookup ()` once, not per call";

/// The tax rate, counting how many times it was looked up.
fn lookup_rate(lookups: &Cell<u32>) -> u32 {
    lookups.set(lookups.get() + 1);
    20
}

/// Prices with tax, and how many lookups that took.
pub fn partial(prices: &[u32]) -> (Vec<u32>, u32) {
    let lookups = Cell::new(0);
    let taxed = ml! {
        let with_tax rate price = price + price * rate / 100 in
        // `lookup_rate &lookups` runs here, once, whatever the number of
        // prices.
        let vat = with_tax (lookup_rate &lookups) in
        prices |> .iter() |> .copied() |> .map(vat) |> .collect()
    };
    (taxed, lookups.get())
}

/// The same with a `fun`: its body runs for every price.
pub fn lambda(prices: &[u32]) -> (Vec<u32>, u32) {
    let lookups = Cell::new(0);
    let taxed = ml! {
        let with_tax rate price = price + price * rate / 100 in
        let vat = fun price -> with_tax (lookup_rate &lookups) price in
        prices |> .iter() |> .copied() |> .map(vat) |> .collect()
    };
    (taxed, lookups.get())
}

pub fn run() {
    let prices = [1000, 250, 50];
    println!("partial: {:?}", partial(&prices));
    println!("lambda:  {:?}", lambda(&prices));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_lookup_for_a_partial_application() {
        assert_eq!(partial(&[1000, 250, 50]), (vec![1200, 300, 60], 1));
    }

    #[test]
    fn one_lookup_per_call_for_a_fun() {
        assert_eq!(lambda(&[1000, 250, 50]), (vec![1200, 300, 60], 3));
    }
}

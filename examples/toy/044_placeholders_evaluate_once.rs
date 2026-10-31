// Toy 044: the rest of a placeholder function is evaluated once.
//
// What it is: in `f (lookup ()) _`, the argument `lookup ()` is computed
// when the function is made, as for a partial application (toy 019), and
// every call of the function reuses it. A `fun` that makes the same call
// computes it on every call.
//
// When to use it: when an argument is costly, or must be read once, and
// the function is used many times.
//
// How to write it: as any placeholder function. The macro binds each
// computed argument (not a name, a path or a literal) to a local first:
// `f (expensive ()) _` is
// `{ let value1 = expensive(); move |_0| f(value1, _0) }`, and
// `f x.y _` is `{ let value1 = x.y; move |_0| f(value1, _0) }`.
//
// Reference: [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-)

use rsml::ml;
use std::cell::Cell;

pub const SUMMARY: &str = "`convert (rate ()) _` computes `rate ()` once, not per call";

/// The exchange rate in cents per unit, counting how often it was read.
fn rate(reads: &Cell<u32>) -> u32 {
    reads.set(reads.get() + 1);
    92
}

fn convert(rate: u32, amount: u32) -> u32 {
    amount * rate / 100
}

/// Amounts converted with one reading of the rate.
pub fn with_placeholder(amounts: &[u32]) -> (Vec<u32>, u32) {
    let reads = Cell::new(0);
    let converted = ml! {
    // `rate &reads` runs here, once.
    let to_euros = convert (rate &reads) _ in
    amounts |> .iter() |> .map(fun a -> to_euros !a) |> .collect()
    };
    (converted, reads.get())
}

/// The same with a `fun`: the rate is read for every amount.
pub fn with_fun(amounts: &[u32]) -> (Vec<u32>, u32) {
    let reads = Cell::new(0);
    let converted = ml! {
    let to_euros = fun a -> convert (rate &reads) a in
    amounts |> .iter() |> .map(fun a -> to_euros !a) |> .collect()
    };
    (converted, reads.get())
}

pub fn run() {
    let amounts = [100, 250, 1000];
    println!(
        "with_placeholder {amounts:?} = {:?}",
        with_placeholder(&amounts)
    );
    println!("with_fun {amounts:?} = {:?}", with_fun(&amounts));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_reading_for_the_placeholder_function() {
        assert_eq!(with_placeholder(&[100, 250, 1000]), (vec![92, 230, 920], 1));
    }

    #[test]
    fn one_reading_per_call_for_the_fun() {
        assert_eq!(with_fun(&[100, 250, 1000]), (vec![92, 230, 920], 3));
    }
}

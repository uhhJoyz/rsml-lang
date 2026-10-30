// Toy 018: partial application, `with_tax 20`.
//
// What it is: OCaml functions are curried, so a function of two arguments
// given only the first is a function that waits for the second. Rust
// functions take all their arguments at once: the macro calls a function
// directly when all its arguments are there, and makes a closure for the
// rest when some are missing.
//
// When to use it: to make a specialised function out of a general one,
// above all to hand it to `map` or `filter`.
//
// How to write it: give the function fewer arguments than it takes. The
// macro must see how many it takes (its arity). It sees that of a `let`
// function in the same block, as here; toy 020 declares a Rust function's.
// * `let add x y = x + y in add 1` is
//   `{ let add = |x, y| x + y; move |arg2| add(1, arg2) }`.
// * `let add x y = x + y in add 1 2` is
//   `{ let add = |x, y| x + y; add(1, 2) }`: all there, a plain call.
// * The missing arguments are always the last ones.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application)

use rsml::ml;

pub const SUMMARY: &str = "Partial application: `with_tax 20` waits for the price";

/// Each price (in cents) with 20% tax added.
pub fn with_vat(prices: &[u32]) -> Vec<u32> {
    ml! {
        let with_tax rate price = price + price * rate / 100 in
        // Given `let with_tax rate price = price + price * rate / 100`,
        // `with_tax 20` is `fun p -> with_tax 20 p`: a closure that waits
        // for the price.
        prices |> .iter() |> .copied() |> .map(with_tax 20) |> .collect()
    }
}

/// One price with tax: both arguments given, so a direct call.
pub fn one_price(rate: u32, price: u32) -> u32 {
    ml! {
        let with_tax rate price = price + price * rate / 100 in
        // Given `let with_tax rate price = price + price * rate / 100`,
        // `with_tax rate price` is `with_tax(rate, price)`.
        with_tax rate price
    }
}

/// A partial application is a value: it can be named and used later.
pub fn reduced_rate(food: u32, books: u32) -> (u32, u32) {
    ml! {
        let with_tax rate price = price + price * rate / 100 in
        let reduced = with_tax 5 in
        (reduced food, reduced books)
    }
}

pub fn run() {
    println!("with_vat [1000; 250] = {:?}", with_vat(&[1000, 250]));
    println!("with_tax 20 1000 = {}", one_price(20, 1000));
    println!(
        "(with_tax 5) 100, (with_tax 5) 2000 = {:?}",
        reduced_rate(100, 2000)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_applications() {
        assert_eq!(with_vat(&[1000, 250]), [1200, 300]);
        assert_eq!(reduced_rate(100, 2000), (105, 2100));
    }

    #[test]
    fn complete_calls() {
        assert_eq!(one_price(20, 1000), 1200);
        assert_eq!(
            ml! {
                let add x y = x + y in
                add 1 2
            },
            3
        );
        let inc = ml! {
            let add x y = x + y in
            add 1
        };
        assert_eq!(inc(2), 3);
    }
}

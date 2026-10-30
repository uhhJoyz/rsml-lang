// Toy 020: `external`, the arity of a Rust function.
//
// What it is: a macro cannot see functions defined outside its block, so
// it calls them with every argument it is given. `external name : type`
// tells it how many arguments `name` takes, and then `name` can be
// partially applied like a `let` function.
//
// When to use it: to partially apply a Rust function, or a function of
// another crate.
//
// How to write it: `external name : t1 -> t2 -> result ;;` at the top of
// the block, a phrase of its own ended by `;;` (toy 011).
// * Only the arrows count: `a -> b -> c` is two arguments, and the types
//   may be `_`. They are not checked: rustc checks the real function.
// * `external` makes no code: it only changes how `name` is called.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application)

use rsml::ml;

pub const SUMMARY: &str = "`external with_tax : u32 -> u32 -> u32` makes `with_tax 20` partial";

/// An ordinary Rust function of two arguments.
fn with_tax(rate: u32, price: u32) -> u32 {
    price + price * rate / 100
}

/// Each price with 20% tax.
pub fn with_vat(prices: &[u32]) -> Vec<u32> {
    ml! {
        external with_tax : u32 -> u32 -> u32 ;;
        // Given `external with_tax : u32 -> u32 -> u32`, `with_tax 20` is
        // `move |arg2| with_tax(20, arg2)`.
        prices |> .iter() |> .copied() |> .map(with_tax 20) |> .collect()
    }
}

/// The same, with the types left out.
pub fn with_reduced_vat(prices: &[u32]) -> Vec<u32> {
    ml! {
        external with_tax : _ -> _ -> _ ;;
        prices |> .iter() |> .copied() |> .map(with_tax 5) |> .collect()
    }
}

/// With both arguments, a known function is called as any other.
pub fn one_price() -> u32 {
    ml! {
        external with_tax : _ -> _ -> _ ;;
        // Given `external with_tax : _ -> _ -> _`,
        // `with_tax 20 1000` is `with_tax(20, 1000)`.
        with_tax 20 1000
    }
}

// Without the `external`, `with_tax 20` is `with_tax(20)`: a call with one
// argument, which rustc rejects (`with_tax` takes 2).

pub fn run() {
    println!("with_vat [1000; 250] = {:?}", with_vat(&[1000, 250]));
    println!(
        "with_reduced_vat [1000; 250] = {:?}",
        with_reduced_vat(&[1000, 250])
    );
    println!("with_tax 20 1000 = {}", one_price());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partially_applied_rust_functions() {
        assert_eq!(with_vat(&[1000, 250]), [1200, 300]);
        assert_eq!(with_reduced_vat(&[1000, 250]), [1050, 262]);
    }

    #[test]
    fn complete_call() {
        assert_eq!(one_price(), 1200);
    }
}

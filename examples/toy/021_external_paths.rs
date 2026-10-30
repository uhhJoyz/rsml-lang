// Toy 021: `external .. = "path"`, and functions that return closures.
//
// What it is: two more things an `external` declaration can say.
// * `= "path"` gives the function a local name: `external at_least : _ ->
//   _ -> _ = "std::cmp::max"` makes `at_least` mean `std::cmp::max`, with
//   two arguments.
// * Parentheses in the type mark a function that returns a closure:
//   `f64 -> (f64 -> f64)` takes one argument, and its result takes the
//   next. Each `->` at the top is one argument of the Rust function.
//
// When to use it: to give a library function a name that reads well, and
// to call a Rust function that returns a closure OCaml style.
//
// How to write it: `external name : type = "path" ;;`, with the path as a
// string. As in toy 020, only the arrows of the type count.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application)

use rsml::ml;

pub const SUMMARY: &str = "`external at_least : _ -> _ -> _ = \"std::cmp::max\"`, `f -> (g -> h)`";

/// Balances, with debts shown as zero.
pub fn floored(balances: &[i64]) -> Vec<i64> {
    ml! {
        external at_least : _ -> _ -> _ = "std::cmp::max" ;;
        // Given `external at_least : _ -> _ -> _ = "std::cmp::max"`,
        // `at_least 0` is `move |arg2| std::cmp::max(0, arg2)`.
        balances |> .iter() |> .copied() |> .map(at_least 0) |> .collect()
    }
}

/// A Rust function that returns a closure.
fn scale(factor: f64) -> impl Fn(f64) -> f64 {
    move |x| x * factor
}

/// A length in centimetres, from inches.
pub fn to_cm(inches: f64) -> f64 {
    ml! {
        external scale : f64 -> (f64 -> f64) ;;
        // Given `external scale : f64 -> (f64 -> f64)`,
        // `scale 2.54 inches` is `scale(2.54)(inches)`.
        scale 2.54 inches
    }
}

// Without the declaration, `scale 2.54 inches` is
// `scale(2.54, inches)`: one call with two arguments, which rustc
// rejects.

/// The closure itself: `scale 2.54` alone is the Rust call.
pub fn all_to_cm(lengths: &[f64]) -> Vec<f64> {
    ml! {
        external scale : f64 -> (f64 -> f64) ;;
        // Given `external scale : f64 -> (f64 -> f64)`, `scale 2.54` is
        // `scale(2.54)`.
        lengths |> .iter() |> .copied() |> .map(scale 2.54) |> .collect()
    }
}

pub fn run() {
    println!("floored [120; -30; 0] = {:?}", floored(&[120, -30, 0]));
    println!("to_cm 10.0 = {}", to_cm(10.0));
    println!("all_to_cm [1.0; 2.0] = {:?}", all_to_cm(&[1.0, 2.0]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_under_another_name() {
        assert_eq!(floored(&[120, -30, 0]), [120, 0, 0]);
    }

    #[test]
    fn a_function_returning_a_closure() {
        assert_eq!(to_cm(10.0), 25.4);
        assert_eq!(all_to_cm(&[1.0, 2.0]), [2.54, 5.08]);
    }
}

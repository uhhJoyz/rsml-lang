// Toy 028: piping into a Rust-style call, `x |> f(a)`.
//
// What it is: `f(a)`, with the parenthesis right after the name, is a
// Rust call (toy 053), and a Rust call is complete: `x |> f(a)` passes
// `x` to the RESULT of `f(a)`. To pass `x` to `f` itself, mark its place
// with `_`, or use a function whose arity the macro knows.
//
// When to use it: `x |> make(a)` with a Rust function that returns a
// closure; `x |> f(_, a)` to pipe into a Rust-style call.
//
// How to write it:
// * `x |> f(a)` is `f(a)(x)`.
// * `x |> f(_, a)` is `f(x, a)`.
// * With a known arity (a `let` function, an `external`), `f(a)` is a
//   partial application: given `external f : _ -> _ -> _`, `x |> f(a)`
//   is `f(a, x)`, and `let add a b = a + b in x |> add(1)` is
//   `{ let add = |a, b| a + b; add(1, x) }`.
//
// Reference: [Pipelines: |> and @@](../../docs/notation.md#pipelines--and-)

use rsml::ml;

pub const SUMMARY: &str = "`x |> f(a)` is `f(a)(x)`; `x |> f(_, a)` is `f(x, a)`";

/// A Rust function that returns a closure.
fn scale(factor: f64) -> impl Fn(f64) -> f64 {
    move |x| x * factor
}

/// Inches to centimetres: the inches go to the closure `scale(2.54)`.
pub fn to_cm(inches: f64) -> f64 {
    ml! {
        // `inches |> scale(2.54)` is `scale(2.54)(inches)`.
        inches |> scale(2.54)
    }
}

/// A percentage kept between 0 and 100, with the value in first place.
pub fn percent(p: i64) -> i64 {
    ml! {
        // `p |> i64::clamp(_, 0, 100)` is `i64::clamp(p, 0, 100)`.
        p |> i64::clamp(_, 0, 100)
    }
}

/// With a local function, `with_tax(20)` waits for the price.
pub fn price_with_tax(price: u32) -> u32 {
    ml! {
        let with_tax rate price = price + price * rate / 100 in
        // Given `let with_tax rate price = price + price * rate / 100`,
        // `price |> with_tax(20)` is `with_tax(20, price)`.
        price |> with_tax(20)
    }
}

pub fn run() {
    println!("10.0 |> scale(2.54) = {}", to_cm(10.0));
    println!("130 |> i64::clamp(_, 0, 100) = {}", percent(130));
    println!("1000 |> with_tax(20) = {}", price_with_tax(1000));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_calls_get_the_value_afterwards() {
        assert_eq!(to_cm(10.0), 25.4);
    }

    #[test]
    fn placeholders_and_known_arities() {
        assert_eq!(percent(130), 100);
        assert_eq!(percent(-5), 0);
        assert_eq!(percent(42), 42);
        assert_eq!(price_with_tax(1000), 1200);
    }
}

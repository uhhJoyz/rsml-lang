// Toy 053: Rust-style calls, `add(1, 2)`.
//
// What it is: a `(` written right after a function (a name, a path, or a
// closing `)` or `]`) is a Rust call: each item between the parentheses is
// one argument. With a space it is an ML-style application to one value.
//
// When to use it: when Rust's spelling reads better, for example calls
// with several computed arguments, or to call what a call returns.
//
// How to write it: no space before the `(`.
// * `f (a, b)`, spaced, passes one argument, the tuple `(a, b)`.
// * `f()` is `f ()`: a call without arguments; `f(())` passes `()`.
// * `f(a)(b)` calls the result of `f(a)` with `b`; `f(a) b` is `f(a)(b)`.
// * Spaced and then glued, `f (a)(b)` is `f(a(b))`: `(a)(b)` calls `a`.
// * A placeholder makes the call a function: `f(_, 1)` waits for its first
//   argument (toy 038).
//
// Reference: [Rust-style calls and indexing](../../docs/notation.md#rust-style-calls-and-indexing-fa-b-vi)

use rsml::ml;

pub const SUMMARY: &str = "Rust calls `add(1, 2)`, `f()`, `f(a)(b)`; spaced `f (a, b)` is a tuple";

fn price(base: u32, tax_percent: u32) -> u32 {
    base + base * tax_percent / 100
}

fn total((a, b): (u32, u32)) -> u32 {
    a + b
}

fn discount(percent: u32) -> impl Fn(u32) -> u32 {
    move |p| p - p * percent / 100
}

fn standard_tax() -> u32 {
    20
}

/// The price of 50 with tax, with the tax rate from a function call.
pub fn taxed() -> u32 { ml! {
    // Written as in Rust, and `price(50, standard_tax())` is
    // `price 50 (standard_tax ())`.
    price(50, standard_tax())
}}

/// A spaced tuple is one argument.
pub fn pair_total(a: u32, b: u32) -> u32 { ml! {
    // `total (a, b)` is `total((a, b))`.
    total (a, b)
}}

/// Calls what `discount(10)` returns: a closure.
pub fn on_sale(p: u32) -> (u32, u32) { ml! {
    // `discount(10)(p)` calls the closure that `discount(10)` returns;
    // `discount(10) p` is `discount(10)(p)`.
    (discount(10)(p), discount(10) p)
}}

/// A Rust call with a placeholder is a function of the missing argument.
pub fn prices(bases: &[u32]) -> Vec<u32> { ml! {
    // `price(_, 10)` is `fun p -> price(p, 10)`.
    bases.iter().copied().map(price(_, 10)).collect()
}}

pub fn run() {
    println!("price(50, standard_tax()) = {}", taxed());
    println!("total (2, 3) = {}", pair_total(2, 3));
    println!("discount(10)(200), discount(10) 200 = {:?}", on_sale(200));
    println!("map price(_, 10) [100; 30] = {:?}", prices(&[100, 30]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_calls() {
        assert_eq!(taxed(), 60);
        assert_eq!(pair_total(2, 3), 5);
        assert_eq!(on_sale(200), (180, 180));
        assert_eq!(prices(&[100, 30]), [110, 33]);
    }

    #[test]
    fn spaced_then_glued() {
        let double = |n: u32| n * 2;
        let inc = |n: u32| n + 1;
        // `double (inc)(3)` is `double(inc(3))`.
        assert_eq!(ml! { double (inc)(3) }, 8);
    }
}

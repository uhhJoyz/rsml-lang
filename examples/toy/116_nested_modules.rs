// Toy 116: modules inside `rsml!`, and `use super::*`.
//
// What it is: an `rsml!` block may hold `mod` items, whose function
// bodies are rsml code too. A module knows the arities of its own
// functions, and those of its parent when it imports them, as Rust code
// would see them: `use super::*` brings all, `use super::name` one.
//
// When to use it: to group the functions of a file, or for a
// `#[cfg(test)] mod tests` whose tests partially apply the functions
// they test (toy 125).
//
// How to write it: as in Rust.
// * With `use super::*`:
//   `rsml! { fn add(a: i64, b: i64) -> i64 { a + b } mod m { use super::*; pub fn inc() -> impl Fn(i64) -> i64 { add 1 } } }`
//   is `fn add(a: i64, b: i64) -> i64 { a + b } mod m { use super::*; pub fn inc() -> impl Fn(i64) -> i64 { move |arg2| add(1, arg2) } }`.
// * Without an import, the parent's functions are neither in scope (for
//   rustc) nor known (for the macro).
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "`mod` in `rsml!`: `use super::*` brings the parent's arities along";

rsml! {
/// Degrees from one scale to another: `x * factor + offset`.
pub fn convert(offset: f64, factor: f64, x: f64) -> f64 {
    x *. factor +. offset
}

pub mod weather {
    use super::*;

    /// Celsius to Fahrenheit: `convert 32.0 1.8` is partial here too.
    pub fn fahrenheit(celsius: &[f64]) -> Vec<f64> {
        celsius |> .iter() |> .copied() |> .map(convert 32.0 1.8) |> .collect()
    }
}

pub mod kitchen {
    use super::convert;

    /// Gas mark from Celsius, roughly: one name imported.
    pub fn gas_mark(celsius: f64) -> f64 {
        let mark = convert (-7.0) (1.0 /. 14.0) in
        (mark celsius).round()
    }
}
}

pub fn run() {
    println!(
        "weather::fahrenheit [0; 100] = {:?}",
        weather::fahrenheit(&[0.0, 100.0])
    );
    println!("kitchen::gas_mark 180 = {}", kitchen::gas_mark(180.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn functions_of_the_parent() {
        assert_eq!(weather::fahrenheit(&[0.0, 100.0]), [32.0, 212.0]);
        // 180 / 14 - 7 is 5.86, so gas mark 6.
        assert_eq!(kitchen::gas_mark(180.0), 6.0);
    }
}

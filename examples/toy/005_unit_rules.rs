// Toy 005: `()`, "no arguments" or the unit value.
//
// What it is: Rust calls a function without arguments as `f()`; rsml
// writes `f ()`. `()` alone after a function means "no arguments";
// everywhere else it is the unit value.
//
// When to use it: `f ()` for every call without arguments. `f (())` in
// the rare case where a Rust function takes a `()` parameter.
//
// How to write it:
// * `f ()` is `f()`: no arguments.
// * `f (())` is `f(())`: the double parentheses pass the unit value.
// * `f () x` is `f((), x)`: with other arguments, `()` is a value.
// * A constructor takes the value: `Ok ()` is `Ok(())`, the usual "it
//   worked, nothing to return".
//
// Reference: [Application](../../docs/notation.md#application)

use rsml::ml;
use std::collections::HashMap;

pub const SUMMARY: &str = "`f ()` calls with no arguments, `f (())` passes `()`, `Ok ()`";

fn default_port() -> u16 {
    8080
}

/// A port, or the default one.
pub fn port(given: Option<u16>) -> u16 { ml! {
    // `default_port ()` is `default_port()`.
    match given with Some p -> p | None -> default_port ()
}}

/// A set as a map to `()`: `()` among other arguments is the value.
pub fn seen(words: &[&'static str]) -> usize { ml! {
    let mut set = HashMap.new () in
    for w in words do
      // `HashMap::insert &mut set !w ()` is
      // `HashMap::insert(&mut set, *w, ())`.
      HashMap::insert &mut set !w ()
    done;
    set.len()
}}

/// Checks a port, the Rust way: `Ok(())` when it is fine.
pub fn check(port: u16) -> Result<(), String> { ml! {
    // `Ok ()` is `Ok(())`.
    if port >= 1024 then Ok () else Err (format!("{port} is reserved"))
}}

/// A function that takes the unit value itself.
fn unit_count(u: ()) -> usize {
    let () = u;
    1
}

pub fn units() -> usize { ml! {
    // `unit_count (())` is `unit_count(())`.
    unit_count (())
}}

pub fn run() {
    println!(
        "port None = {}, port (Some 22) = {}",
        port(None),
        port(Some(22))
    );
    println!("seen [a; b; a] = {}", seen(&["a", "b", "a"]));
    println!("check 80 = {:?}, check 8080 = {:?}", check(80), check(8080));
    println!("unit_count (()) = {}", units());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_arguments() {
        assert_eq!(port(None), 8080);
        assert_eq!(port(Some(22)), 22);
    }

    #[test]
    fn unit_values() {
        assert_eq!(seen(&["a", "b", "a"]), 2);
        assert_eq!(check(8080), Ok(()));
        assert_eq!(check(80), Err("80 is reserved".to_string()));
        assert_eq!(units(), 1);
    }
}

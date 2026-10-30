// Toy 100: function types `a -> b` and tuple types `a * b`.
//
// What it is: OCaml writes the type of a function with arrows, `i64 ->
// i64`, and the type of a pair with a star, `i64 * String`. Inside types,
// rsml reads both: a star between types makes Rust's tuple type, and
// arrows say how many arguments a function takes.
//
// When to use it: tuple types wherever a type goes; arrows in `external`
// (toy 020), in `(e : t)` (toy 022) and in `let rec f : a -> b =
// function ..` (toy 008).
//
// How to write it:
// * `(e : i64 * String)` is `{ let typed: (i64, String) = e; typed }`, and
//   a star works inside other types: `(xs : Vec<u32 * &str>)` is
//   `{ let typed: Vec<(u32, &str)> = xs; typed }`.
// * `let rec f : u32 -> u32 = function ..` gives the parameter and result
//   types of a recursive function.
// * Elsewhere arrows are not checked: only their number counts.
//
// Reference: [Types](../../docs/notation.md#types)

use rsml::ml;

pub const SUMMARY: &str = "Types with OCaml's `*` and `->`: `u32 * &str`, `u32 -> u32`";

/// A price list as pairs, typed with `*`.
pub fn price_list() -> Vec<(u32, &'static str)> {
    ml! {
        let list : Vec<u32 * &str> = [(4, "tea"); (12, "cake")] in
        list
    }
}

/// The most expensive item: the pattern takes the pair apart.
pub fn dearest(list: &[(u32, &'static str)]) -> Option<&'static str> {
    ml! {
        list |> .iter() |> .max_by_key(fun (&(price, _)) -> price) |> .map(fun (&(_, name)) -> name)
    }
}

/// Digits of a number, summed by a recursive function typed with `->`.
pub fn digit_sum(n: u32) -> u32 {
    ml! {
        // `let rec f : u32 -> u32 = function 0 -> 0 | k -> k in f` is
        // `{ fn f(argument: u32) -> u32 { match argument { 0 => 0, k => k, } } f }`.
        let rec sum : u32 -> u32 = function 0 -> 0 | k -> k mod 10 + sum (k / 10) in
        sum n
    }
}

pub fn run() {
    println!("price_list () = {:?}", price_list());
    println!("dearest (price_list ()) = {:?}", dearest(&price_list()));
    println!("digit_sum 1234 = {}", digit_sum(1234));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuple_and_function_types() {
        assert_eq!(price_list(), [(4, "tea"), (12, "cake")]);
        assert_eq!(dearest(&price_list()), Some("cake"));
        assert_eq!(dearest(&[]), None);
        assert_eq!(digit_sum(1234), 10);
    }

    #[test]
    fn a_pair_type() {
        let p = ml! { ((1, String::from("one")) : i64 * String) };
        assert_eq!(p, (1, "one".to_string()));
    }
}

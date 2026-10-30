// Toy 099: type annotations, `(e : t)`, `let x : t = ..`, `fun (x : t)`.
//
// What it is: OCaml writes a type after a colon. The types are Rust's
// (`i64`, `&str`, `Vec<String>`), and an annotation does what it does in
// Rust: it picks a type rustc could not guess, or checks one.
//
// When to use it: when rustc says "type annotations needed" (a `parse`,
// a `collect`, a closure parameter whose methods are called), and to make
// a type visible to the reader.
//
// How to write it:
// * On an expression, in parentheses: `(n : i64)` is
//   `{ let typed: i64 = n; typed }`.
// * On a `let`: `let x : u8 = 200 in x` is `{ let x: u8 = 200; x }`.
// * On a parameter: `fun (s : &str) -> s.len()` is
//   `move |s: &str| s.len()`.
// * The gotcha: with a function type, `(e : a -> b)` only gives an arity
//   (toy 022): `(fun x -> x : i64 -> i64)` is `move |x| x`, unchecked.
//
// Reference: [Types](../../docs/notation.md#types)

use rsml::ml;

pub const SUMMARY: &str = "`(e : t)`, `let x : t = ..`, `fun (x : t) -> ..`: Rust types";

/// A number read from text: the annotation tells `parse` what to make.
pub fn parsed(text: &str) -> Option<u16> {
    ml! {
        let n : Result<u16, _> = text.trim().parse() in
        n.ok()
    }
}

/// The square of a number whose type is fixed in place.
pub fn square(n: i64) -> i64 {
    ml! {
        // `(n : i64).pow(2)` is `({ let typed: i64 = n; typed }).pow(2)`.
        (n : i64).pow(2)
    }
}

/// The longest word's length: a typed parameter lets `len` be called.
pub fn longest(words: &[&str]) -> usize {
    ml! {
        let length = fun (w : &str) -> w.len() in
        words |> .iter() |> .map(fun w -> length !w) |> .max() |> .unwrap_or(0)
    }
}

pub fn run() {
    println!(
        "parsed \" 8080 \" = {:?}, parsed \"x\" = {:?}",
        parsed(" 8080 "),
        parsed("x")
    );
    println!("square 12 = {}", square(12));
    println!(
        "longest [\"tea\"; \"coffee\"] = {}",
        longest(&["tea", "coffee"])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annotations() {
        assert_eq!(parsed(" 8080 "), Some(8080));
        assert_eq!(parsed("x"), None);
        assert_eq!(parsed("70000"), None);
        assert_eq!(square(12), 144);
        assert_eq!(longest(&["tea", "coffee"]), 6);
        assert_eq!(longest(&[]), 0);
    }
}

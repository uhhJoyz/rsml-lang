// Toy 130: Rust habits in blocks and bindings, and what to write instead.
//
// What it is: more Rust spellings that rsml rejects, with the error each
// one gives (checked by scripts/check_docs.py, as in toy 129), and the
// OCaml spelling in the code below.
//
// When to use it: when an error message sends you here.
//
// How to write it:
// * Braces: `{ a; b }` is an error: "`{ .. }` is not a block in OCaml:
//   write `begin .. end` or `( .. )` (a record needs its struct name:
//   `Point { x = 1 }`)"
// * Commas in lists: `[1, 2]` is an error: "list items are separated by
//   `;`: write `[1; 2]`, or `[(1, 2)]` for a list of one pair"
// * `let` with `;`: `let x = 1; x` is an error: "this reads as Rust's
//   `let x = e;`, but after `=` a `;` makes the rest part of the value:
//   write `let x = e in ..`"
// * Keywords as names: `let loop = 1 in loop` is an error: "`loop` is a
//   Rust keyword, not a name (as a name: `r#loop`); an endless loop is
//   `while true do .. done`"
// * `fn` in a body: `fn f x = x` is an error: "`fn` is a Rust keyword,
//   not a name (as a name: `r#fn`); a named local function is
//   `let f x = .. in ..`, and a closure is `fun x -> e`"
// * Exceptions: `try f x with _ -> 0` is an error: "`try .. with` is not
//   supported: Rust has no exceptions, use `Result` and `match`"
//
// Reference: [Rust habits](../../docs/notation.md#rust-habits)

use rsml::ml;

pub const SUMMARY: &str = "Rust habits rejected: `{ a; b }`, `[1, 2]`, `let x = e;`, `fn f x`";

/// Steps grouped with `begin .. end`, not braces.
pub fn log_twice(msg: &str) -> Vec<String> {
    ml! {
        let mut log = Vec.new () in
        if not (msg.is_empty()) then begin
          log.push(msg.to_string());
          log.push(msg.to_uppercase())
        end;
        log
    }
}

/// A list with `;` between its items.
pub fn primes() -> Vec<u32> {
    ml! { [2; 3; 5; 7] }
}

/// Bindings with `in`, a local function with `let`, a keyword with `r#`.
pub fn area_of_square(side: u32) -> u32 {
    ml! {
        let r#loop = side in
        let square x = x * x in
        square r#loop
    }
}

/// A failure as a `Result`, matched, instead of `try .. with`.
pub fn parsed_or_zero(text: &str) -> i64 {
    ml! { match text.parse::<i64>() with Ok n -> n | Err _ -> 0 }
}

pub fn run() {
    println!("log_twice \"hi\" = {:?}", log_twice("hi"));
    println!("primes () = {:?}", primes());
    println!("area_of_square 7 = {}", area_of_square(7));
    println!(
        "parsed_or_zero \"42\" = {}, parsed_or_zero \"x\" = {}",
        parsed_or_zero("42"),
        parsed_or_zero("x")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ocaml_spellings() {
        assert_eq!(log_twice("hi"), ["hi", "HI"]);
        assert!(log_twice("").is_empty());
        assert_eq!(primes(), [2, 3, 5, 7]);
        assert_eq!(area_of_square(7), 49);
        assert_eq!(parsed_or_zero("42"), 42);
        assert_eq!(parsed_or_zero("x"), 0);
    }
}

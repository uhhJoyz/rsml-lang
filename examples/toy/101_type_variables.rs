// Toy 101: type variables, `'a`, in `let rec` signatures.
//
// What it is: "any type" is written `'a`, `'b`, .. In a `let rec`
// signature each one becomes a generic parameter of the nested `fn`:
// `let rec id (x : 'a) : 'a = x in id` is `{ fn id<A>(x: A) -> A { x } id }`.
// Inside other types too: `let rec count (xs : Vec<'a>) : usize =
// xs.len() in count` is `{ fn count<A>(xs: Vec<A>) -> usize { xs.len() } count }`.
//
// When to use it: for a local recursive function that works for every
// element type, which a closure (`let f x = ..`) cannot do: a closure is
// used at one type (toy 007).
//
// How to write it:
// * `'a` where the type goes: `(xs : &['a])`, `: Vec<'a>`.
// * A type variable has no bounds, so the function can only move its
//   values around: it cannot print, compare, copy or clone them. For
//   bounds, write an `rsml!` function with a Rust signature
//   (`fn f<T: Clone>(..)`, toy 112).
// * A `'name` right after `&` is a lifetime (toy 102), so a reference to
//   a type variable cannot be written: `&['a]` works, a bare `&'a` does
//   not.
//
// Reference: [Types](../../docs/notation.md#types)

use rsml::ml;

pub const SUMMARY: &str = "`let rec len (xs : &['a]) : usize`: `'a` becomes a generic parameter";

/// The lengths of a list of numbers and a list of words, by one function.
pub fn lengths(numbers: &[i64], words: &[&str]) -> (usize, usize) { ml! {
    // `len` is generic, so it takes both kinds of slice.
    let rec len (xs : &['a]) : usize = match xs with [] -> 0 | _ :: rest -> 1 + len rest in
    (len numbers, len words)
}}

/// Two lists reversed by one generic function that moves the items.
pub fn reversed(numbers: Vec<i64>, words: Vec<&'static str>) -> (Vec<i64>, Vec<&'static str>) {
    ml! {
    // Each item moves from the end of `from` onto the end of `into`.
    let rec move_all (from : Vec<'a>) (into : Vec<'a>) : Vec<'a> =
      let mut from = from in
      let mut into = into in
      match from.pop() with
      | None -> into
      | Some x ->
          into.push(x);
          move_all from into
    in
    (move_all numbers [], move_all words [])
    }
}

pub fn run() {
    println!(
        "lengths [1; 2; 3] [\"a\"] = {:?}",
        lengths(&[1, 2, 3], &["a"])
    );
    println!(
        "reversed [1; 2; 3] [\"x\"; \"y\"] = {:?}",
        reversed(vec![1, 2, 3], vec!["x", "y"])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_function_many_types() {
        assert_eq!(lengths(&[1, 2, 3], &["a"]), (3, 1));
        assert_eq!(
            reversed(vec![1, 2, 3], vec!["x", "y"]),
            (vec![3, 2, 1], vec!["y", "x"])
        );
        assert_eq!(reversed(vec![], vec![]), (vec![], vec![]));
    }

    #[test]
    fn the_header_example() {
        // Inside the block `id` is generic; the value the block returns
        // would be `id` at one type.
        let pair = ml! {
        let rec id (x : 'a) : 'a = x in
        (id 5, id "five")
        };
        assert_eq!(pair, (5, "five"));
    }
}

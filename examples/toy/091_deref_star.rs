// Toy 091: Rust's dereference, `*e`.
//
// What it is: where an expression starts, `*e` is Rust's dereference: it reads
// or writes what a reference, a `Box` or a map entry points to, and it
// binds as in Rust, so `*w.cents` dereferences the field `w.cents`.
//
// When to use it: to write through a `&mut` with `*p <- e`, and wherever
// Rust code would write `*`. `!x` (toy 092) reads the same way.
//
// How to write it:
// * `(*` opens an ML comment: a dereference in parentheses is `( *x)`
//   (the formatter prints it so).
// * After an argument, `*` multiplies: `double *x` would be `double * x`.
//   Write `double ( *x)`, or `double !x`.
// * Rust's glued `**q` and `&*s` are one operator each where an
//   expression starts; as arguments they need parentheses too: `f (&*s)`,
//   since `f **q` is `f ** q`, a power (toy 065).
//
// Reference: [Rust-style methods, fields and calls](../../docs/notation.md#rust-style-methods-fields-and-calls)

use rsml::ml;
use std::collections::BTreeMap;

pub const SUMMARY: &str = "Rust's `*`: `*c <- *c + 1`, `*w.cents`, `( *x)`, `**q`, `&*s`";

/// How often each word occurs, in alphabetical order.
pub fn word_counts(text: &str) -> Vec<(String, usize)> { ml! {
    let mut counts = BTreeMap.new () in
    for w in text.split_whitespace() do
      // The entry is a `&mut usize`: `*c <- *c + 1` writes through it.
      let c = counts.entry(w.to_string()).or_insert(0) in
      *c <- *c + 1
    done;
    counts.into_iter().collect()
}}

/// A wallet that keeps its cents in a box.
pub struct Wallet {
    pub cents: Box<i64>,
}

/// `*w.cents` dereferences the box in the field, as in Rust.
pub fn cents(w: &Wallet) -> i64 { ml! {
    *w.cents
}}

fn double(x: i64) -> i64 {
    x * 2
}

/// Doubles a value behind a reference.
pub fn doubled(x: &i64) -> i64 { ml! {
    // `double ( *x)` is `double(*x)`.
    double ( *x)
}}

/// Resets a counter through a `&mut`.
pub fn reset(p: &mut i64) { ml! {
    // `*p <- 0` is `*p = 0`.
    *p <- 0
}}

/// The first letter of a name: `&*name` borrows the `String` as a `&str`.
pub fn initial(name: String, nested: &&char) -> (Option<char>, char) { ml! {
    // `&*name` is `&(*name)`, and `**q` is `*(*q)`.
    (str::chars (&*name) |> .next, **nested)
}}

pub fn run() {
    println!("word_counts \"b a b\" = {:?}", word_counts("b a b"));
    let w = Wallet {
        cents: Box::new(250),
    };
    println!("cents of a wallet with 250 = {}", cents(&w));
    println!("doubled &21 = {}", doubled(&21));
    let mut counter = 9;
    reset(&mut counter);
    println!("after reset, counter = {counter}");
    println!(
        "initial \"Ada\" &&'z' = {:?}",
        initial("Ada".to_string(), &&'z')
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn through_references() {
        assert_eq!(
            word_counts("b a b"),
            [("a".to_string(), 1), ("b".to_string(), 2)]
        );
        assert_eq!(
            cents(&Wallet {
                cents: Box::new(250)
            }),
            250
        );
        assert_eq!(doubled(&21), 42);
        let mut counter = 9;
        reset(&mut counter);
        assert_eq!(counter, 0);
        assert_eq!(initial("Ada".to_string(), &&'z'), (Some('A'), 'z'));
    }
}

// Toy 051: the receiver of `.name`, and `()` after it.
//
// What it is: in `.name r a ..`, the first argument is the receiver,
// evaluated once, and the rest are the method's arguments. Because the
// receiver already says "call it", `()` after the receiver is the unit
// value, not "no arguments" (compare toy 005).
//
// When to use it: for any method call written ML style; the rules
// matter for `()` and for computed receivers.
//
// How to write it:
// * `.send tx ()` is `tx.send(())`: `()` is the message.
// * A computed receiver goes in parentheses: `.get (f x) 0` is
//   `f(x).get(0)`.
// * Generic arguments: `.collect::<Vec<_>> it` is
//   `it.collect::<Vec<_>>()`.
// * No space after the dot: `. len xs` is an error: "no space is
//   allowed between `.` and the method name"
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name)

use rsml::ml;
use std::sync::mpsc;

pub const SUMMARY: &str = "`.send tx ()` sends `()`; `.get (f x) 0`; `.collect::<Vec<_>> it`";

/// Signals sent down a channel and counted at the other end.
pub fn signals(n: usize) -> usize { ml! {
    let (tx, rx) = mpsc::channel::<()> () in
    for _ in 0..n do
      // `.send tx ()` is `tx.send(())`: the unit value is sent.
      .send tx () |> .unwrap
    done;
    drop tx;
    rx.iter().count()
}}

fn words_of(line: &str) -> Vec<&str> {
    line.split_whitespace().collect()
}

/// The first word of a line: the receiver is a call, in parentheses.
pub fn first_word(line: &str) -> Option<&str> { ml! {
    // `.get (words_of line) 0` is `words_of(line).get(0)`.
    .get (words_of line) 0 |> .copied
}}

/// The letters of a word, in a vector.
pub fn letters(word: &str) -> Vec<char> { ml! {
    // `.collect::<Vec<_>> (word.chars())` is `word.chars().collect::<Vec<_>>()`.
    .collect::<Vec<_>> (word.chars())
}}

pub fn run() {
    println!("signals 3 = {}", signals(3));
    println!(
        "first_word \"tea for two\" = {:?}",
        first_word("tea for two")
    );
    println!("letters \"tea\" = {:?}", letters("tea"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receivers() {
        assert_eq!(signals(3), 3);
        assert_eq!(first_word("tea for two"), Some("tea"));
        assert_eq!(first_word(""), None);
        assert_eq!(letters("tea"), ['t', 'e', 'a']);
    }
}

// Toy 059: OCaml's dotted paths, `Vec.new ()`, `Shape.Circle r`.
//
// What it is: OCaml writes a module's item as `Module.item`. Here a
// capitalized name followed by `.name` is the same path as with `::`:
// `Vec.new` is `Vec::new`, `Shape.Circle` is `Shape::Circle`.
//
// When to use it: when you like OCaml's look; `::` always works too.
//
// How to write it: no spaces around the `.`.
// * After a capitalized name, `.name` always continues the path, even a
//   method's: `Level::High.label()` is `Level::High::label()`. Put the
//   value in parentheses: `(Level::High).label()`.
// * The path stops at a lowercase name: `p.name` is a field.
// * `::` with spaces is a cons (toy 062), and so is `h::t` written glued
//   when `h` is bound in the block: a bound name is a value, never a
//   module.
//
// Reference: [Paths](../../docs/notation.md#paths)

use rsml::{ml, rsml};
use std::collections::HashMap;

pub const SUMMARY: &str = "Dotted paths `Vec.new ()`, `Level.High`; `h::t` with a bound `h`";

#[derive(Debug, PartialEq)]
pub enum Level {
    Low,
    High,
}

impl Level {
    fn label(&self) -> &'static str {
        match self {
            Level::Low => "low",
            Level::High => "high",
        }
    }
}

/// A word count, built with dotted paths.
pub fn counts(words: &[&'static str]) -> (usize, String) {
    ml! {
        // `HashMap.new ()` is `HashMap::new()`.
        let mut seen = HashMap.new () in
        for w in words do
          let c = seen.entry(!w).or_insert(0) in
          *c <- *c + 1
        done;
        // `String.from "words"` is `String::from("words")`.
        (seen.len(), String.from "words")
    }
}

/// A level from a number: the variants are paths too.
pub fn level(n: u32) -> Level {
    ml! {
        // `Level.High` is `Level::High`.
        if n > 5 then Level.High else Level.Low
    }
}

/// A method of a variant: the value goes in parentheses.
pub fn label(n: u32) -> &'static str {
    ml! {
        // `(Level.High).label()` is `(Level::High).label()`.
        if n > 5 then (Level.High).label() else (Level.Low).label()
    }
}

rsml! {
    /// A bound name before `::` is a value: in `rsml!` the parameters are
    /// bound, so `h::t` is a cons.
    pub fn with_head(h: u32, t: Vec<u32>) -> Vec<u32> {
        // rsmlfmt: skip
        // (The formatter would print `h :: t`; the glued spelling is the lesson.)
        h::t
    }
}

pub fn run() {
    println!("counts [a; b; a] = {:?}", counts(&["a", "b", "a"]));
    println!("level 9 = {:?}, label 9 = {:?}", level(9), label(9));
    println!("with_head 1 [2; 3] = {:?}", with_head(1, vec![2, 3]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotted_paths() {
        assert_eq!(counts(&["a", "b", "a"]), (2, "words".to_string()));
        assert_eq!(level(9), Level::High);
        assert_eq!(level(1), Level::Low);
        assert_eq!((label(9), label(1)), ("high", "low"));
    }

    #[test]
    fn a_bound_head_is_consed() {
        assert_eq!(with_head(1, vec![2, 3]), [1, 2, 3]);
    }
}

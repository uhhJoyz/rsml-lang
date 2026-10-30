// Toy 050: the glued `|>.`, `xs |>.iter() |>.count()`.
//
// What it is: Rust lexes `xs |>.iter()` with one operator, `|>.`, as
// OCaml does. Unless you define a `( |>. )` (toy 036), rsml reads it as
// `|>` and the section `.iter()`: the habit of writing a method chain's
// dot against the pipe works. The same holds for `>>.`, `<<.` and `@@.`.
//
// When to use it: when you write it by habit; rsml-fmt prints the space,
// `|> .iter()`, which is the usual spelling.
//
// How to write it:
// * `xs |>.iter() |>.count()` is `xs |> .iter() |> .count()`.
// * `t |>.0` is `t |> .0`, and `.iter >>.count` is `.iter >> .count`.
// * Only these four operators, and only when a name or a number follows
//   the dot: `x +.y` stays the float operator `+.`, and `x +.y` is
//   `x + y` (toy 065).
// * With a space after the dot it is an unknown operator:
//   `xs |>. iter` is an error: "unknown operator `|>.`; for a method or
//   field section after `|>`, write `|> .name` or `|> .0`"
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name)

use rsml::ml;

pub const SUMMARY: &str = "`xs |>.iter() |>.count()` reads as `xs |> .iter() |> .count()`";

/// How many words are longer than three letters.
pub fn long_words(words: &[&str]) -> usize {
    ml! {
        // rsmlfmt: skip
        // (The formatter would put a space after each `|>`; the glued
        // spelling is the lesson.)
        words |>.iter() |>.filter(fun w -> w.len() > 3) |>.count()
    }
}

/// The second field of a pair, glued.
pub fn second(pair: (&str, u32)) -> u32 {
    ml! {
        // rsmlfmt: skip
        // (Glued on purpose, as above.)
        pair |>.1
    }
}

/// The same pipeline, as rsml-fmt prints it.
pub fn long_words_spaced(words: &[&str]) -> usize {
    ml! { words |> .iter() |> .filter(fun w -> w.len() > 3) |> .count() }
}

pub fn run() {
    let words = ["tea", "cocoa", "milk"];
    println!("long_words {words:?} = {}", long_words(&words));
    println!(
        "long_words_spaced {words:?} = {}",
        long_words_spaced(&words)
    );
    println!("second (\"tea\", 3) = {}", second(("tea", 3)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glued_and_spaced_are_the_same() {
        let words = ["tea", "cocoa", "milk"];
        assert_eq!(long_words(&words), 2);
        assert_eq!(long_words_spaced(&words), 2);
        assert_eq!(second(("tea", 3)), 3);
    }

    #[test]
    fn float_operators_stay() {
        let (x, y) = (1.5, 2.0);
        let sum = ml! {
            // rsmlfmt: skip
            // (Glued on purpose: `+.` is one operator.)
            x +.y
        };
        assert_eq!(sum, 3.5);
    }
}

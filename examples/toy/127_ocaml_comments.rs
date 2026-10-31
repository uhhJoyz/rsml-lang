// Toy 127: ML comments, `(* .. *)`.
//
// What it is: an ML comment, `(* like this *)`, works anywhere in rsml
// code, and nests: `1 + (* a (* b *) c *) 2` is `1 + 2`.
//
// When to use it: when the code should look like ML throughout. Prefer
// `//` and `/* */` (toy 126) otherwise: editors color `(* *)` as code,
// except Neovim and Vim with this repository's files, and Rust's lexer
// reads a `(* *)` comment before the macro does (below).
//
// How to write it:
// * The text inside must be Rust tokens: an unbalanced `"` or bracket in
//   it stops rustc before rsml sees the comment, and so does an
//   apostrophe inside a word (from edition 2021 on, "don" right before
//   it is a reserved prefix): write "do not".
// * `(*` always opens a comment, so the multiplication section is
//   `( * )` with its spaces, and a dereference in parentheses is `( *x)`:
//   `( * ) 6 7` is `6 * 7`, and `f ( *x)` is `f(*x)`.
// * `(**)` is no comment either: it is the section of `**`, the float
//   power: `(**)` is `|lhs, rhs| ::core::primitive::f64::powf(lhs, rhs)`.
// * rsml-fmt keeps these comments; with `comments = "rust"` it rewrites
//   each as `//` on a line of its own and as `/* */` elsewhere.
// * This toy's own explanations are `//` comments: the checks of
//   scripts/check_docs.py read code in comments only there.
//
// Reference: [Comments](../../docs/notation.md#comments)

use rsml::ml;

pub const SUMMARY: &str = "`(* .. *)` comments, nested; `( * )`, `( *x)` and `(**)` are not";

/// The area of a rectangle, with ML comments around the code.
pub fn area(w: u32, h: u32) -> u32 { ml! {
    (* width times height, in square metres *)
    w * h (* no rounding needed (* integers *) *)
}}

/// The product of some numbers: `( * )` keeps its spaces.
pub fn product(xs: &[u64]) -> u64 { ml! {
    xs |> .iter() |> .copied() |> .fold(1, ( * )) (* one for none *)
}}

/// Twice what a reference points to: `( *x)`, never `(*x)`.
pub fn twice(x: &u32) -> u32 { ml! {
    let double n = n * 2 in
    double ( *x)
}}

/// A power, by the `**` section, which looks like an empty comment.
pub fn power(base: f64, exp: f64) -> f64 {
    ml! {
        // rsmlfmt: skip
        // (The formatter would print `( ** )`; the look-alike is the
        // lesson.)
        let pow = (**) in
        pow base exp
    }
}

pub fn run() {
    println!("area 3 4 = {}", area(3, 4));
    println!("product [2; 3; 4] = {}", product(&[2, 3, 4]));
    println!("twice &21 = {}", twice(&21));
    println!("power 2.0 10.0 = {}", power(2.0, 10.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_are_skipped() {
        assert_eq!(area(3, 4), 12);
        assert_eq!(ml! { 1 + (* a (* b *) c *) 2 }, 3);
        assert_eq!(ml! { (* do not *) 7 }, 7);
    }

    #[test]
    fn look_alikes() {
        assert_eq!(product(&[2, 3, 4]), 24);
        assert_eq!(twice(&21), 42);
        assert_eq!(power(2.0, 10.0), 1024.0);
    }
}

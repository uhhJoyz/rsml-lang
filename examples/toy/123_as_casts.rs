// Toy 123: `e as t`, Rust's casts.
//
// What it is: Rust's `as` converts between number types (and `u8` to
// `char`), written the same in rsml code. There is no `float_of_int`:
// `n as f64` is how an integer becomes a float.
//
// When to use it: to mix integers and floats, to index with a `u32`, to
// turn a letter into a number.
//
// How to write it: `e as t` binds tighter than every infix operator, so
// arithmetic around a cast needs no parentheses. An application and a
// `!` bind tighter still.
// * `n as f64 *. 2.0` is `(n as f64) * 2.0`.
// * `f x as i64` is `f(x) as i64`: the cast takes the whole application.
// * `-n as i64` is `-(n as i64)`: the cast binds tighter than a minus in
//   front, too. Rust reads it as `(-n) as i64`, which a `u8` cannot do.
//
// Reference: [Operators that become calls](../../docs/notation.md#operators-that-become-calls),
// [Types](../../docs/notation.md#types)

use rsml::ml;

pub const SUMMARY: &str = "Casts: `total as f64 /. count as f64`, `c as u8 - b'a'`";

/// The mean of some scores, as a float.
pub fn mean(scores: &[u32]) -> f64 { ml! {
    let total = scores |> .iter() |> .sum::<u32>() in
    // `total as f64 /. scores.len() as f64` is
    // `(total as f64) / (scores.len() as f64)`.
    total as f64 /. scores.len() as f64
}}

/// A letter's place in the alphabet, from 0.
pub fn letter_index(c: char) -> usize { ml! {
    // `(c as u8 - b'a') as usize` is `((c as u8) - b'a') as usize`.
    (c as u8 - b'a') as usize
}}

/// The letter at a place in the alphabet: `u8` to `char` is a cast too.
pub fn letter(i: u8) -> char { ml! {
    (b'a' + i) as char
}}

/// A loss: the minus applies to the cast value, so a `u8` works.
pub fn loss(n: u8) -> i64 { ml! {
    -n as i64
}}

pub fn run() {
    println!("mean [|3; 4; 8|] = {}", mean(&[3, 4, 8]));
    println!("letter_index 'd' = {}", letter_index('d'));
    println!("letter 25 = {:?}", letter(25));
    println!("loss 7 = {}", loss(7));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn casts() {
        assert_eq!(mean(&[3, 4, 8]), 5.0);
        assert_eq!(mean(&[1, 2]), 1.5);
        assert_eq!(letter_index('d'), 3);
        assert_eq!(letter(25), 'z');
        assert_eq!(loss(7), -7);
    }
}

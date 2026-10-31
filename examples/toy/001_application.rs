// Toy 001: application, `f x y`.
//
// What it is: rsml applies a function by writing its arguments after it,
// separated by spaces. There are no parentheses and no commas: `area 3 4`
// calls `area` with 3 and 4.
//
// When to use it: for every call. Rust's own `area(3, 4)` works too (toy
// 053); pick whichever reads better.
//
// How to write it: the function, then each argument. A function defined
// outside the `ml!` block, as `area` is here, is called with every
// argument it is given, so `area 3` alone would be `area(3)`, which rustc
// rejects because `area` takes two.
//
// Reference: [Application](../../docs/notation.md#application)

use rsml::ml;

pub const SUMMARY: &str = "Juxtaposition: `area 3 4` calls `area` with two arguments";

/// An ordinary Rust function: rsml code calls it like any other.
fn area(width: i64, height: i64) -> i64 {
    width * height
}

/// The area of a 3 by 4 room.
pub fn room_area() -> i64 { ml! {
    // `area 3 4` is `area(3, 4)`.
    area 3 4
}}

/// The larger of two numbers, through a function of the standard library.
pub fn larger(a: i64, b: i64) -> i64 { ml! {
    // A path is a function like any other:
    // `std::cmp::max a b` is `std::cmp::max(a, b)`.
    std::cmp::max a b
}}

/// Arguments can be any literal or name: strings, characters, booleans.
pub fn padded(word: &str) -> String { ml! {
    // `pad word '.' 6 true` is `pad(word, '.', 6, true)`.
    pad word '.' 6 true
}}

fn pad(word: &str, fill: char, width: usize, left: bool) -> String {
    let fill: String = std::iter::repeat_n(fill, width.saturating_sub(word.len())).collect();
    if left {
        fill + word
    } else {
        word.to_string() + &fill
    }
}

pub fn run() {
    println!("area 3 4 = {}", room_area());
    println!("std::cmp::max 7 2 = {}", larger(7, 2));
    println!("pad \"ab\" '.' 6 true = {:?}", padded("ab"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_with_every_argument() {
        assert_eq!(room_area(), 12);
        assert_eq!(larger(7, 2), 7);
        assert_eq!(padded("ab"), "....ab");
    }

    #[test]
    fn the_same_call_in_rust() {
        assert_eq!(ml! { area 3 4 }, area(3, 4));
    }
}

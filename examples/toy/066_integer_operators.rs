// Toy 066: OCaml's word operators on integers, `mod`, `land`, `lsl`, ...
//
// What it is: OCaml spells some integer operators as words, and so does
// rsml:
// * `a mod b` is `a % b`, the remainder;
// * `a land b`, `a lor b`, `a lxor b` are `a & b`, `a | b`, `a ^ b`;
// * `a lsl n` is `a << n`; `a lsr n` and `a asr n` are both `a >> n`,
//   which Rust makes arithmetic (sign-filling) on signed types and logical
//   on unsigned ones.
//
// When to use it: for remainders, bit flags and shifts.
//
// How to write it: between the operands, like `+`. They bind like
// OCaml's: `mod`, `land`, `lor`, `lxor` like `*`, the shifts tighter
// still. Some of Rust's spellings mean something else here: `>>` and
// `<<` are composition (toy 029), and `^` is string concatenation (toy
// 064). Rust's `%` is accepted too: `a % b` stays `a % b`.
//
// Reference: [Operators that become calls](../../docs/notation.md#operators-that-become-calls)

use rsml::ml;

pub const SUMMARY: &str = "`n mod 7`, `flags land MASK`, `1 lsl k`: integer word operators";

/// The weekday of day `n` (0 is Monday).
pub fn weekday(n: u32) -> &'static str {
    ml! {
        // `n mod 7` is `n % 7`.
        match n mod 7 with
        | 0 -> "Mon"
        | 1 -> "Tue"
        | 2 -> "Wed"
        | 3 -> "Thu"
        | 4 -> "Fri"
        | 5 -> "Sat"
        | _ -> "Sun"
    }
}

/// Flags packed in one byte, and read back.
pub fn flags(bold: bool, italic: bool) -> (u8, bool, bool) {
    ml! {
        // `1 lsl 0` is `1 << 0`, `1 lsl 1` is `1 << 1`.
        let bit_bold = 1 lsl 0 in
        let bit_italic = 1 lsl 1 in
        let packed = (if bold then bit_bold else 0) lor (if italic then bit_italic else 0) in
        // `packed land bit_bold <> 0` is `(packed & bit_bold) != 0`.
        (packed, packed land bit_bold <> 0, packed land bit_italic <> 0)
    }
}

/// A value with its low bit flipped, and halved by a shift.
pub fn flip_and_halve(n: i32) -> (i32, i32) {
    ml! {
        // `n lxor 1` is `n ^ 1`; `n asr 1` is `n >> 1`.
        (n lxor 1, n asr 1)
    }
}

pub fn run() {
    println!("weekday 9 = {}", weekday(9));
    println!("flags true false = {:?}", flags(true, false));
    println!(
        "flip_and_halve 6 = {:?}, flip_and_halve (-7) = {:?}",
        flip_and_halve(6),
        flip_and_halve(-7)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remainders() {
        assert_eq!(weekday(0), "Mon");
        assert_eq!(weekday(9), "Wed");
        assert_eq!(weekday(13), "Sun");
    }

    #[test]
    fn bits() {
        assert_eq!(flags(true, false), (1, true, false));
        assert_eq!(flags(true, true), (3, true, true));
        assert_eq!(flip_and_halve(6), (7, 3));
        // Arithmetic shift: -7 >> 1 is -4, rounding down.
        assert_eq!(flip_and_halve(-7), (-8, -4));
    }
}

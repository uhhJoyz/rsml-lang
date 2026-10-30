// Toy 058: Rust paths, `std::mem::swap a b`, `i32::MAX`.
//
// What it is: a Rust path names a function, constant or type through its
// modules, as in Rust: `std::mem::swap`, `::std::cmp::max`, `u8::MAX`,
// `Vec::<String>::new`. As a function it is applied OCaml style.
//
// When to use it: for everything that lives in a module or a type, without
// a `use`.
//
// How to write it: exactly as in Rust, with no space around `::`.
// * Spaced, `::` is OCaml's cons: `x :: xs` puts `x` in front of the
//   list `xs` (toy 062): `std :: mem` is `::rsml::__rt::cons(std, mem)`.
// * A leading `::` starts from the crate root, as in Rust.
// * Generic arguments go where Rust puts them, turbofish included.
//
// Reference: [Paths](../../docs/notation.md#paths)

use rsml::ml;

pub const SUMMARY: &str = "Paths `std::mem::swap a b`, `::std::cmp::max`, `u8::MAX`, turbofish";

/// Swaps the two players' scores.
pub fn swapped(mut first: u32, mut second: u32) -> (u32, u32) {
    ml! {
        // `std::mem::swap &mut first &mut second` is
        // `std::mem::swap(&mut first, &mut second)`.
        std::mem::swap &mut first &mut second;
        (first, second)
    }
}

/// The best of two scores, through a path from the crate root.
pub fn best(a: u32, b: u32) -> u32 {
    ml! {
        // `::std::cmp::max a b` is `::std::cmp::max(a, b)`.
        ::std::cmp::max a b
    }
}

/// How far a volume level is from the top: a constant of a type.
pub fn headroom(level: u8) -> u8 {
    ml! {
        // `u8::MAX - level` stays `u8::MAX - level`; `u8::MAX` is 255.
        u8::MAX - level
    }
}

/// An empty log, with its element type given by a turbofish.
pub fn empty_log() -> usize {
    ml! {
        // `Vec::<String>::new ()` is `Vec::<String>::new()`.
        let log = Vec::<String>::new () in
        log.len()
    }
}

pub fn run() {
    println!("swapped 3 9 = {:?}", swapped(3, 9));
    println!("best 3 9 = {}", best(3, 9));
    println!("headroom 200 = {}", headroom(200));
    println!("empty_log () = {}", empty_log());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths() {
        assert_eq!(swapped(3, 9), (9, 3));
        assert_eq!(best(3, 9), 9);
        assert_eq!(headroom(200), 55);
        assert_eq!(empty_log(), 0);
    }

    #[test]
    fn spaced_colons_are_cons() {
        let (x, xs) = (1, vec![2, 3]);
        // `x :: xs` is `::rsml::__rt::cons(x, xs)`.
        assert_eq!(ml! { x :: xs }, [1, 2, 3]);
    }
}

// Toy 121: `unsafe { .. }`.
//
// What it is: Rust's `unsafe` block, whose inside is OCaml, like the
// inside of `begin .. end`. It marks the code where you, not the
// compiler, promise that the rules hold: calling an `unsafe fn`,
// reading through a raw pointer, skipping a check.
//
// When to use it: as in Rust, rarely, around the smallest expression that
// needs it, with a `SAFETY:` comment saying why it is fine.
//
// How to write it: `unsafe { e }`, an expression like any other. In an
// `unsafe fn`, the unsafe calls still go in `unsafe { }`: edition 2024
// asks for it, and every edition accepts it.
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::{ml, rsml};

pub const SUMMARY: &str = "`unsafe { e }` with OCaml inside: `unsafe { text_of bytes }`";

rsml! {
    /// The text of bytes that are known to be UTF-8, without checking.
    ///
    /// # Safety
    ///
    /// `bytes` must be valid UTF-8.
    unsafe fn text_of(bytes: &[u8]) -> &str {
        // `unsafe { std::str::from_utf8_unchecked bytes }` is
        // `unsafe { std::str::from_utf8_unchecked(bytes) }`.
        unsafe { std::str::from_utf8_unchecked bytes }
    }
}

/// The middle byte, read without a second bounds check.
pub fn middle(xs: &[u8]) -> Option<u8> {
    ml! {
        if xs.is_empty() then
          None
        else
          let i = xs.len() / 2 in
          // SAFETY: `i < xs.len()`, as `xs` is not empty. `*` binds like
          // Rust's, so `*xs.get_unchecked(i)` reads the byte it points to.
          Some (unsafe { *xs.get_unchecked(i) })
    }
}

/// A word in capitals, through `text_of`.
pub fn shout(word: &str) -> String {
    ml! {
        // SAFETY: the bytes of a `&str` are UTF-8.
        // `unsafe { text_of (word.as_bytes()) } |> .to_uppercase` is
        // `unsafe { text_of(word.as_bytes()) }.to_uppercase()`.
        unsafe { text_of (word.as_bytes()) } |> .to_uppercase
    }
}

pub fn run() {
    println!("middle b\"abcde\" = {:?}", middle(b"abcde").map(char::from));
    println!("middle b\"\" = {:?}", middle(b""));
    println!("shout \"héllo\" = {:?}", shout("héllo"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchecked_but_right() {
        assert_eq!(middle(b"abcde"), Some(b'c'));
        assert_eq!(middle(b"ab"), Some(b'b'));
        assert_eq!(middle(b""), None);
        assert_eq!(shout("héllo"), "HÉLLO");
    }
}

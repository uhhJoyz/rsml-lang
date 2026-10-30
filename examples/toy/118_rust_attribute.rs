// Toy 118: `#[rust]`, a body left as Rust.
//
// What it is: in an `rsml!` block, `#[rust]` on a `fn`, `impl`, `trait`
// or `mod` keeps every function body under it as Rust. The attribute is
// removed; rsml-fmt leaves those bodies alone too.
//
// When to use it: for code that reads better in Rust (loops with `+=`,
// `let .. else`, chains of `?`) next to OCaml functions, in one file.
//
// How to write it: `#[rust]` before the item.
// * A `#[rust]` function still counts for the arities of the block
//   (toy 114): `rsml! { #[rust] fn f(x: i64) -> i64 { x + 1 } fn g() ->
//   i64 { f 1 } }` is `fn f(x: i64) -> i64 { x + 1 } fn g() -> i64 { f(1) }`.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "`#[rust]` keeps a function's body Rust inside `rsml!`";

rsml! {
    /// A checksum, written in Rust: a loop with `+=`, which OCaml lacks.
    #[rust]
    pub fn checksum(bytes: &[u8]) -> u32 {
        let mut sum: u32 = 0;
        for b in bytes {
            sum = sum.wrapping_mul(31).wrapping_add(u32::from(*b));
        }
        sum
    }

    /// Two numbers from a line such as "3, 4", with Rust's `?`s.
    #[rust]
    pub fn parse_pair(line: &str) -> Option<(i64, i64)> {
        let (a, b) = line.split_once(',')?;
        Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
    }

    /// OCaml again, calling the Rust ones: `checksum` is known (one
    /// argument).
    pub fn signature(text: &str) -> String {
        format!("{:08x}", checksum text.as_bytes())
    }
}

pub fn run() {
    println!("checksum \"ab\" = {}", checksum(b"ab"));
    println!("parse_pair \"3, 4\" = {:?}", parse_pair("3, 4"));
    println!("signature \"rsml\" = {}", signature("rsml"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_bodies() {
        // 'a' is 97, 'b' 98: 97 * 31 + 98.
        assert_eq!(checksum(b"ab"), 3105);
        assert_ne!(checksum(b"ab"), checksum(b"ba"));
        assert_eq!(parse_pair("3, 4"), Some((3, 4)));
        assert_eq!(parse_pair("3"), None);
        assert_eq!(signature("ab"), "00000c21");
    }
}

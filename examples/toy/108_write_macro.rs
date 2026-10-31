// Toy 108: `write!`/`writeln!`, and macros that take Rust.
//
// What it is: `write!(out, ..)` and `writeln!(out, ..)` format into a
// `String` or a file, with rsml arguments; `?` after them passes an error
// on. Only std's formatting, assertion and panicking macros and `vec!`
// read rsml code: every other macro (`matches!`, yours) gets its tokens as
// Rust.
//
// When to use it: `write!` to build text piece by piece; `matches!` with
// a Rust pattern for a quick test.
//
// How to write it:
// * `write!(out, "{} rows", rows |> .len)?` is
//   `write!(out, "{} rows", rows.len())?`.
// * `std::fmt::Write` must be in scope for a `String`, as in Rust.
// * `matches!(c, 'a' | 'e')` stays `matches!(c, 'a' | 'e')`: a Rust
//   pattern. rsml code inside it is not translated:
//   `matches!(word |> .len, 0)` stays `matches!(word |> .len, 0)`, which
//   rustc rejects; `ml! { .. }` works inside such a macro.
//
// Reference: [Macros](../../docs/notation.md#macros)

use rsml::{ml, rsml};

pub const SUMMARY: &str = "`write!(out, \"{}\", f x)?`; other macros (`matches!`) take Rust";

rsml! {
use std::fmt::Write;

/// A stock table: one line per item, then a count.
pub fn render(rows: &[(&str, u32)]) -> Result<String, std::fmt::Error> {
    let mut out = String.new () in
    for (name, qty) in rows do
      // `!qty` reads the `&u32` the pattern binds; `?` passes on an error.
      writeln!(out, "{:<6}{:>3}", name, !qty)?
    done;
    write!(out, "{} rows", rows |> .len)?;
    Ok out
}

/// Whether a letter is a vowel: `matches!` takes a Rust pattern.
pub fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// Whether a word is empty, with rsml code inside `matches!` through `ml!`.
pub fn is_empty_word(word: &str) -> bool {
    matches!(ml! { word |> .len }, 0)
}
}

pub fn run() {
    println!(
        "render [(tea, 4); (jam, 12)] =\n{}",
        render(&[("tea", 4), ("jam", 12)]).unwrap()
    );
    println!(
        "is_vowel 'e' = {}, is_vowel 'x' = {}",
        is_vowel('e'),
        is_vowel('x')
    );
    println!("is_empty_word \"\" = {}", is_empty_word(""));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn written() {
        assert_eq!(
            render(&[("tea", 4), ("jam", 12)]).unwrap(),
            "tea     4\njam    12\n2 rows"
        );
        assert_eq!(render(&[]).unwrap(), "0 rows");
    }

    #[test]
    fn rust_macros() {
        assert!(is_vowel('e'));
        assert!(!is_vowel('x'));
        assert!(is_empty_word(""));
        assert!(!is_empty_word("jam"));
    }
}

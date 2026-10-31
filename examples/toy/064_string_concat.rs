// Toy 064: string concatenation, `a ^ b`.
//
// What it is: `a ^ b` joins two strings. It is a call of a
// small helper that makes a `String`: `a ^ b` is
// `::rsml::__rt::concat(a, b)`. It takes a `String` or a `&str` on either
// side.
//
// When to use it: to build a string from a few pieces. For anything with
// formatting, `format!` (toy 105) is clearer.
//
// How to write it: `a ^ b`. It groups to the right:
// `a ^ b ^ c` is `a ^ (b ^ c)`.
// * `^` is never Rust's bitwise xor; that is `lxor` (toy 066).
//
// Reference: [Operators that become calls](../../docs/notation.md#operators-that-become-calls)

use rsml::ml;

pub const SUMMARY: &str = "`\"Dear \" ^ name ^ \",\"` builds a `String`";

/// The first line of a letter.
pub fn salutation(name: &str) -> String { ml! {
    // `"Dear " ^ name ^ ","` is
    // `::rsml::__rt::concat("Dear ", ::rsml::__rt::concat(name, ","))`.
    "Dear " ^ name ^ ","
}}

/// A file name from a stem: a `String` on the left works too.
pub fn file_name(stem: &str, n: u32) -> String { ml! {
    stem.to_lowercase() ^ "-" ^ n.to_string() ^ ".txt"
}}

pub fn run() {
    println!("salutation \"Ada\" = {:?}", salutation("Ada"));
    println!("file_name \"Report\" 3 = {:?}", file_name("Report", 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joined_strings() {
        assert_eq!(salutation("Ada"), "Dear Ada,");
        assert_eq!(file_name("Report", 3), "report-3.txt");
    }
}

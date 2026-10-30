// Toy 062: cons, `x :: xs`.
//
// What it is: OCaml's `x :: xs` is the list `xs` with `x` in front. Lists
// are Rust `Vec`s here, so `::` with spaces is a call of a small helper
// that inserts at the front: `x :: xs` is `::rsml::__rt::cons(x, xs)`.
// When the tail is a list literal the macro builds the vector at once:
// `1 :: [2; 3]` is `::std::vec![1, 2, 3]`, which is `[1; 2; 3]`.
//
// When to use it: to put an item before a list. Inserting at the front of
// a `Vec` moves every element (O(n)): in a loop, `push` at the end and
// reverse once, as Rust code would.
//
// How to write it: `x :: xs`, with spaces; it groups to the right:
// `a :: b :: rest` is `a :: (b :: rest)`.
// * Without spaces, `std::mem` is a Rust path (toy 058); `h::t` is a cons
//   only when `h` is bound in the block (toy 059).
// * In patterns, `x :: rest` matches a slice (toy 070).
//
// Reference: [Operators that become calls](../../docs/notation.md#operators-that-become-calls)

use rsml::ml;

pub const SUMMARY: &str = "`x :: xs` puts `x` in front of a `Vec`; `1 :: [2; 3]` is a literal";

/// A queue of tasks with an urgent one put first.
pub fn urgent_first(urgent: &'static str, tasks: Vec<&'static str>) -> Vec<&'static str> {
    ml! { urgent :: tasks }
}

/// Two headers in front of the lines.
pub fn with_headers(lines: Vec<String>) -> Vec<String> {
    ml! {
        // `a :: b :: rest` is `::rsml::__rt::cons(a, ::rsml::__rt::cons(b, rest))`.
        String.from "Name" :: String.from "----" :: lines
    }
}

/// A list literal, built item by item.
pub fn countdown() -> Vec<u32> {
    ml! {
        // `3 :: 2 :: 1 :: []` is `::std::vec![3, 2, 1]`.
        3 :: 2 :: 1 :: []
    }
}

pub fn run() {
    println!(
        "urgent_first \"fix\" [\"read\"; \"write\"] = {:?}",
        urgent_first("fix", vec!["read", "write"])
    );
    println!(
        "with_headers [\"Ada\"] = {:?}",
        with_headers(vec!["Ada".to_string()])
    );
    println!("countdown () = {:?}", countdown());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_front() {
        assert_eq!(
            urgent_first("fix", vec!["read", "write"]),
            ["fix", "read", "write"]
        );
        assert_eq!(
            with_headers(vec!["Ada".to_string()]),
            ["Name", "----", "Ada"]
        );
        assert_eq!(countdown(), [3, 2, 1]);
    }

    #[test]
    fn a_literal_tail() {
        assert_eq!(ml! { 1 :: [2; 3] }, ml! { [1; 2; 3] });
    }
}

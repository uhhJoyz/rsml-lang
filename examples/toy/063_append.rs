// Toy 063: append, `xs @ ys`.
//
// What it is: OCaml's `xs @ ys` is the list of the items of `xs`, then
// those of `ys`. On `Vec`s it is a call of a small helper that moves
// `ys`'s items onto the end of `xs`: `xs @ ys` is
// `::rsml::__rt::append(xs, ys)`.
//
// When to use it: to join two lists into one.
//
// How to write it: `xs @ ys`; both are moved (pass clones to keep them).
// It groups to the right, as in OCaml: `a @ b @ c` is `a @ (b @ c)`.
//
// Reference: [Operators that become calls](../../docs/notation.md#operators-that-become-calls)

use rsml::ml;

pub const SUMMARY: &str = "`morning @ evening` joins two lists";

/// A day's appointments, the morning's first.
pub fn day(morning: Vec<&'static str>, evening: Vec<&'static str>) -> Vec<&'static str> {
    ml! { morning @ evening }
}

/// A week of shifts: weekdays, then the weekend, then a holiday.
pub fn shifts(weekdays: Vec<u32>, weekend: Vec<u32>) -> Vec<u32> {
    ml! {
        // `a @ b @ c` is `a @ (b @ c)`.
        weekdays @ weekend @ [0]
    }
}

pub fn run() {
    println!(
        "day [\"gym\"] [\"film\"; \"dinner\"] = {:?}",
        day(vec!["gym"], vec!["film", "dinner"])
    );
    println!("shifts [8; 8] [4] = {:?}", shifts(vec![8, 8], vec![4]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joined() {
        assert_eq!(
            day(vec!["gym"], vec!["film", "dinner"]),
            ["gym", "film", "dinner"]
        );
        assert_eq!(day(vec![], vec!["film"]), ["film"]);
        assert_eq!(shifts(vec![8, 8], vec![4]), [8, 8, 4, 0]);
    }
}

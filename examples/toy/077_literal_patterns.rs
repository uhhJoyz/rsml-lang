// Toy 077: literal and constant patterns: numbers, characters, strings,
// booleans, `i32::MAX`, `std::cmp::Ordering::Less`.
//
// What it is: a literal in a pattern matches that one value. So does a
// path to a constant or to a variant without fields.
//
// When to use it: for commands, answers, special values.
//
// How to write it:
// * Numbers in any Rust spelling (`-1`, `0x10`, `1_000`), characters
//   (`'\n'`), `true` and `false`.
// * A string pattern works on a `String` too: rsml matches `s` through
//   `::rsml::__rt::as_str(&s)`, a `&str`. That is done for the value
//   matched, and for each part of a tuple written after `match`, not
//   deeper (not inside `Some`): match an `Option<String>` through
//   `o.as_deref()`.
// * A path may start lowercase when it reaches a capitalized name:
//   `i32::MAX`, `std::cmp::Ordering::Less`.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str =
    "Literal patterns: `\"yes\"` (on a String too), `-1`, `'\\n'`, `i32::MAX`";

/// Whether an answer, as typed, means yes.
pub fn yes(answer: String) -> bool {
    ml! {
        // `match answer with "y" -> true | _ -> false` is
        // `match ::rsml::__rt::as_str(&answer) { "y" => true, _ => false, }`.
        match answer with "yes" | "y" -> true | _ -> false
    }
}

/// Whether a character is blank, and whether that counts.
pub fn blank(c: char, strict: bool) -> bool {
    ml! { match (c, strict) with (' ', _) | ('\t', false) | ('\n', false) -> true | _ -> false }
}

/// A command and its argument; the `String` inside the tuple is matched too.
pub fn apply(cmd: String, n: i32, total: i32) -> i32 {
    ml! {
        // `match (s, n) with ("add", n) -> 1 | _ -> 0` is
        // `match (::rsml::__rt::as_str(&s), n) { ("add", n) => 1, _ => 0, }`.
        match (cmd, n) with ("add", n) -> total + n | ("reset", _) -> 0 | _ -> total
    }
}

/// Names for special numbers.
pub fn name(n: i32) -> &'static str {
    ml! {
        match n with
        | i32::MAX -> "the largest i32"
        | 0 -> "zero"
        | -1 -> "minus one"
        | 0x10 -> "sixteen"
        | _ -> "a number"
    }
}

/// A comparison, in words: the variants are written with their path.
pub fn compare(a: u32, b: u32) -> &'static str {
    ml! {
        match a.cmp(&b) with
        | std::cmp::Ordering::Less -> "less"
        | std::cmp::Ordering::Equal -> "equal"
        | std::cmp::Ordering::Greater -> "greater"
    }
}

/// An optional answer: `as_deref` turns `Option<String>` into `Option<&str>`.
pub fn maybe_yes(answer: Option<String>) -> bool {
    ml! { match answer.as_deref() with Some "yes" -> true | _ -> false }
}

pub fn run() {
    println!(
        "yes \"y\" = {}, yes \"no\" = {}",
        yes("y".into()),
        yes("no".into())
    );
    println!(
        "apply add 5 10 = {}, apply reset 0 10 = {}",
        apply("add".into(), 5, 10),
        apply("reset".into(), 0, 10)
    );
    println!(
        "name i32::MAX = {}, name -1 = {}, name 16 = {}",
        name(i32::MAX),
        name(-1),
        name(16)
    );
    println!(
        "blank '\\t' false = {}, blank '\\t' true = {}",
        blank('\t', false),
        blank('\t', true)
    );
    println!(
        "compare 2 3 = {}, compare 3 3 = {}",
        compare(2, 3),
        compare(3, 3)
    );
    println!(
        "maybe_yes (Some \"yes\") = {}",
        maybe_yes(Some("yes".into()))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings() {
        assert!(yes("y".into()));
        assert!(yes("yes".to_string()));
        assert!(!yes("no".into()));
        assert_eq!(apply("add".into(), 5, 10), 15);
        assert_eq!(apply("reset".into(), 0, 10), 0);
        assert_eq!(apply("undo".into(), 0, 10), 10);
        assert!(maybe_yes(Some("yes".into())));
        assert!(!maybe_yes(None));
    }

    #[test]
    fn characters_and_booleans() {
        assert!(blank(' ', true));
        assert!(blank('\t', false));
        assert!(!blank('\t', true));
        assert!(!blank('x', false));
    }

    #[test]
    fn numbers_and_constants() {
        assert_eq!(name(i32::MAX), "the largest i32");
        assert_eq!(name(0), "zero");
        assert_eq!(name(-1), "minus one");
        assert_eq!(name(16), "sixteen");
        assert_eq!(name(7), "a number");
        assert_eq!(compare(2, 3), "less");
        assert_eq!(compare(3, 3), "equal");
        assert_eq!(compare(4, 3), "greater");
    }
}

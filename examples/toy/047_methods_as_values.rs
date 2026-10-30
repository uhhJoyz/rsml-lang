// Toy 047: a method alone is a closure, `.len`, `(.trim)`.
//
// What it is: `.name` with no receiver is the function that calls the
// method on its argument: `.len` is `|receiver| receiver.len()`. It can
// be passed to `map`, stored, or composed (toy 032).
//
// When to use it: for `map`, `filter` and the like, when the closure
// would only call one method.
//
// How to write it:
// * In a method's argument list it starts an expression, so no
//   parentheses: `xs.iter().map(.len)` is
//   `xs.iter().map(|receiver| receiver.len())`.
// * As an OCaml argument it needs parentheses, `(.len)`: after a value,
//   `.len` is that value's field, so `f .len` is `f.len`, and
//   `apply_all (.len) xs` is `apply_all(|receiver| receiver.len(), xs)`.
// * A closure that calls a method needs its receiver's type. Inside
//   `map` the iterator gives it; elsewhere write it: `.len (_ : &str)`
//   is `move |_0: &str| _0.len()` (toy 042), or annotate the variable.
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name)

use rsml::ml;

pub const SUMMARY: &str = "`.map(.len)`, `apply_all (.trim) xs`, `.len (_ : &str)`";

fn apply_all<'a>(f: impl Fn(&'a str) -> &'a str, xs: &[&'a str]) -> Vec<&'a str> {
    xs.iter().map(|&x| f(x)).collect()
}

/// The length of each word.
pub fn lengths(words: &[&str]) -> Vec<usize> {
    ml! { words |> .iter() |> .map(.len) |> .collect() }
}

/// The words without their spaces, through a function that takes one.
pub fn trimmed<'a>(words: &[&'a str]) -> Vec<&'a str> {
    ml! {
        // `apply_all (.trim) words` is
        // `apply_all(|receiver| receiver.trim(), words)`.
        apply_all (.trim) words
    }
}

/// A stored method, with the type of its receiver given.
pub fn shouted(words: &[&str]) -> Vec<String> {
    ml! {
        // `.to_uppercase (_ : &str)` is `move |_0: &str| _0.to_uppercase()`.
        let shout = .to_uppercase (_ : &str) in
        words |> .iter() |> .map(fun w -> shout !w) |> .collect()
    }
}

pub fn run() {
    println!(
        "lengths [\"tea\"; \"cocoa\"] = {:?}",
        lengths(&["tea", "cocoa"])
    );
    println!(
        "trimmed [\" tea\"; \"cocoa \"] = {:?}",
        trimmed(&[" tea", "cocoa "])
    );
    println!(
        "shouted [\"tea\"; \"and\"] = {:?}",
        shouted(&["tea", "and"])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods_as_closures() {
        assert_eq!(lengths(&["tea", "cocoa"]), [3, 5]);
        assert_eq!(trimmed(&[" tea", "cocoa "]), ["tea", "cocoa"]);
        assert_eq!(shouted(&["tea", "and"]), ["TEA", "AND"]);
    }

    #[test]
    fn a_method_annotated_through_its_variable() {
        let count: fn(&[i64]) -> usize = ml! { .len };
        assert_eq!(count(&[1, 2, 3]), 3);
    }
}

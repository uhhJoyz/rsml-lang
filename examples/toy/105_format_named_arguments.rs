// Toy 105: `format!` with named arguments, `name = e`.
//
// What it is: in `format!`, `write!`, `println!` and the other formatting
// macros, an argument written `name = e` after the format string is a
// named argument, as in Rust, and its value `e` is rsml code.
//
// When to use it: to give a long argument a name the format string can
// refer to (`{total}`), or to use a value twice, or as a width.
//
// How to write it:
// * `format!("{} {x}", f a, x = g b)` is `format!("{} {x}", f(a), x = g(b))`.
// * `=` compares (toy 067), so a positional comparison needs parentheses:
//   `format!("{}", a = b)` stays `format!("{}", a = b)`, a named argument
//   `a`, while `format!("{}", (a = b))` is `format!("{}", a == b)`.
// * After `name =`, a comparison needs none:
//   `format!("{same}", same = a = b)` is `format!("{same}", same = a == b)`.
//
// Reference: [Macros](../../docs/notation.md#macros)

use rsml::rsml;

pub const SUMMARY: &str = "`format!(\"{} {x}\", f a, x = g b)`; `(a = b)` to compare";

rsml! {
/// A receipt line: the name padded to 8, the price with two decimals.
pub fn receipt_line(name: &str, price: f64, qty: u32) -> String {
    // `total = ..` is a named argument. `as` binds tighter than `*.`:
    // `price *. qty as f64` is `price * (qty as f64)`.
    format!("{:<8}{total:>7.2}", name, total = price *. qty as f64)
}

/// A count against what was expected, and whether they agree.
pub fn tally(counted: u32, expected: u32) -> String {
    // The comparison is in parentheses: without them, `counted =
    // expected` would be a named argument `counted`.
    format!("{} of {}: {}", counted, expected, (counted = expected))
}

/// A right-aligned label, with a width computed in rsml code.
pub fn aligned(label: &str, indent: usize) -> String {
    // `width = indent + label.len()` names the width of `{:>width$}`.
    format!("[{:>width$}]", label, width = indent + label.len())
}
}

pub fn run() {
    println!(
        "receipt_line \"tea\" 1.25 3 = {:?}",
        receipt_line("tea", 1.25, 3)
    );
    println!("tally 4 5 = {:?}", tally(4, 5));
    println!("aligned \"ok\" 3 = {:?}", aligned("ok", 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_arguments() {
        assert_eq!(receipt_line("tea", 1.25, 3), "tea        3.75");
        assert_eq!(aligned("ok", 3), "[   ok]");
    }

    #[test]
    fn a_comparison_in_parentheses() {
        assert_eq!(tally(4, 5), "4 of 5: false");
        assert_eq!(tally(5, 5), "5 of 5: true");
    }
}

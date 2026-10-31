// Toy 031: a composition applied on the spot makes no closure.
//
// What it is: when a composition gets its argument right away, the macro
// writes the nested calls directly, as for a pipeline: no closure is
// made.
//
// When to use it: to apply a chain of steps once, where a named
// composition (toy 029) would be one name too many.
//
// How to write it:
// * `(f >> g) x` is `g(f(x))`, and so is `(f >> g)(x)`.
// * `x |> (f >> g)` is `g(f(x))`.
// * Without the parentheses, `>>` binds looser than `|>`:
//   `x |> f >> g` is `move |argument| g(f(x, argument))`, a closure that
//   composes `x |> f` with `g`. Keep the parentheses.
// * More arguments go to the last step: `(f >> g) a b` is
//   `g(f(a), b)`.
//
// Reference: [Composition: >> and <<](../../docs/notation.md#composition--and-)

use rsml::ml;

pub const SUMMARY: &str = "`(f >> g) x` and `x |> (f >> g)` are plain nested calls";

fn add_tax(rate: u32, cents: u32) -> u32 {
    cents + cents * rate / 100
}

fn round_to(step: u32, cents: u32) -> u32 {
    cents / step * step
}

/// One price, with tax and rounded, by a composition given its argument.
pub fn price(cents: u32) -> u32 { ml! {
    // `(add_tax 20 >> round_to 10) cents` is
    // `round_to(10, add_tax(20, cents))`.
    (add_tax 20 >> round_to 10) cents
}}

/// The same, with the price piped in.
pub fn piped_price(cents: u32) -> u32 { ml! {
    // `cents |> (add_tax 20 >> round_to 10)` is
    // `round_to(10, add_tax(20, cents))`.
    cents |> (add_tax 20 >> round_to 10)
}}

/// The length of a word without its spaces, Rust-call style.
pub fn trimmed_length(word: &str) -> usize { ml! {
    // `(str::trim >> str::len)(word)` is `str::len(str::trim(word))`.
    (str::trim >> str::len)(word)
}}

pub fn run() {
    println!("(add_tax 20 >> round_to 10) 1249 = {}", price(1249));
    println!(
        "1249 |> (add_tax 20 >> round_to 10) = {}",
        piped_price(1249)
    );
    println!(
        "(str::trim >> str::len)(\"  tea \") = {}",
        trimmed_length("  tea ")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applied_compositions() {
        assert_eq!(price(1249), 1490);
        assert_eq!(piped_price(1249), 1490);
        assert_eq!(trimmed_length("  tea "), 3);
    }

    #[test]
    fn extra_arguments_go_to_the_last_step() {
        let double = |n: u32| n * 2;
        // `(double >> add_tax) 50 100` is `add_tax(double(50), 100)`.
        assert_eq!(ml! { (double >> add_tax) 50 100 }, 200);
    }
}

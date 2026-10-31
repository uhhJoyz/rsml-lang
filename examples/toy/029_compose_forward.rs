// Toy 029: forward composition, `f >> g`.
//
// What it is: `f >> g` is the function that applies `f`, then `g` to the
// result, `fun x -> g (f x)`: a closure built from the two functions. It
// reads in the order things happen, as a pipeline does.
//
// When to use it: to name a chain of steps once and use it many times, or
// to hand the chain to `map`.
//
// How to write it: `f >> g >> h`, any number of steps.
// * Like `|>`, it needs no arity: each step gets the result of the one
//   before as its last argument, so `f a >> g` is
//   `move |argument| g(f(a, argument))`.
// * `>>` and `<<` are always composition, never shifts. Shifts are
//   spelled `lsl`, `lsr` and `asr` (toy 066): `n lsl 2` is `n << 2`.
//
// Reference: [Composition: >> and <<](../../docs/notation.md#composition--and-)

use rsml::ml;

pub const SUMMARY: &str = "`add_tax 20 >> round_to 10 >> dollars`: one step, then the next";

fn add_tax(rate: u32, cents: u32) -> u32 {
    cents + cents * rate / 100
}

fn round_to(step: u32, cents: u32) -> u32 {
    cents / step * step
}

fn dollars(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

/// A shelf label: the price with tax, rounded down to 10 cents, in dollars.
pub fn label(cents: u32) -> String { ml! {
    // `add_tax 20 >> round_to 10 >> dollars` is
    // `move |argument| dollars(round_to(10, add_tax(20, argument)))`.
    let price_tag = add_tax 20 >> round_to 10 >> dollars in
    price_tag cents
}}

/// Tags as people type them, cleaned: trimmed, then in lower case.
pub fn clean_tags(tags: &[&str]) -> Vec<String> { ml! {
    // `str::trim >> str::to_lowercase` is
    // `move |argument| str::to_lowercase(str::trim(argument))`.
    tags |> .iter() |> .copied() |> .map(str::trim >> str::to_lowercase) |> .collect()
}}

pub fn run() {
    println!("label 1249 = {}", label(1249));
    println!(
        "clean_tags [\" Rust\"; \"OCaml \"] = {:?}",
        clean_tags(&[" Rust", "OCaml "])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_in_order() {
        // 1249 + 249 = 1498, rounded down to 1490.
        assert_eq!(label(1249), "$14.90");
        assert_eq!(label(1249), dollars(round_to(10, add_tax(20, 1249))));
        assert_eq!(clean_tags(&[" Rust", "OCaml "]), ["rust", "ocaml"]);
    }

    #[test]
    fn shifts_are_words() {
        let n = 3u32;
        assert_eq!(ml! { n lsl 2 }, 12);
    }
}

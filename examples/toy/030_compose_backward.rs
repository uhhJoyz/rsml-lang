// Toy 030: backward composition, `f << g`.
//
// What it is: `f << g` applies `g` first, then `f`: `fun x -> f (g x)`,
// the mathematician's "f after g". It is `g >> f` written the other way
// round.
//
// When to use it: when the outer step is the one to read first, as in
// "the brackets around the upper case of the title".
//
// How to write it: `f << g << h` applies `h`, then `g`, then `f`.
// * The result is a closure, as with `>>` (toy 029), and no arity is
//   needed.
// * `<<` is never a left shift; that is `lsl` (toy 066).
//
// Reference: [Composition: >> and <<](../../docs/notation.md#composition--and-)

use rsml::ml;

pub const SUMMARY: &str = "`bracket << str::to_uppercase`: the right function runs first";

fn bracket(s: String) -> String {
    format!("[{s}]")
}

fn add_tax(rate: u32, cents: u32) -> u32 {
    cents + cents * rate / 100
}

fn round_to(step: u32, cents: u32) -> u32 {
    cents / step * step
}

/// A heading: the title in upper case, in brackets.
pub fn heading(title: &str) -> String {
    ml! {
        // `bracket << str::to_uppercase` is
        // `move |argument| bracket(str::to_uppercase(argument))`.
        let shout = bracket << str::to_uppercase in
        shout title
    }
}

/// The price of toy 029, rounded after the tax, composed backwards.
pub fn rounded_price(cents: u32) -> u32 {
    ml! {
        // `round_to 10 << add_tax 20` is `add_tax 20 >> round_to 10`.
        let price = round_to 10 << add_tax 20 in
        price cents
    }
}

pub fn run() {
    println!("heading \"menu\" = {:?}", heading("menu"));
    println!("rounded_price 1249 = {}", rounded_price(1249));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_to_left() {
        assert_eq!(heading("menu"), "[MENU]");
        assert_eq!(rounded_price(1249), 1490);
        assert_eq!(rounded_price(1249), round_to(10, add_tax(20, 1249)));
    }

    #[test]
    fn three_steps() {
        let f = |n: i32| n + 1;
        let g = |n: i32| n * 10;
        let h = |n: i32| n - 3;
        // `f << g << h` applies `h`, then `g`, then `f`.
        let all = ml! { f << g << h };
        assert_eq!(all(5), 21);
    }
}

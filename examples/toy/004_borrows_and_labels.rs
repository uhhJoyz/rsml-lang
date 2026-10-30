// Toy 004: borrowed arguments `f &xs`, `f &mut xs`, and labels `~x:e`.
//
// What it is: values move into a call, as in Rust; `&xs` lends them and
// `&mut xs` lends them for changing. OCaml's labels, `~name:value` and
// `~name` (short for `~name:name`), are accepted and dropped.
//
// When to use it: `&`/`&mut` whenever the Rust function takes a
// reference. Labels only as reminders of what an argument means.
//
// How to write it: `&xs` and `&mut xs` are one argument each, no
// parentheses needed. Labels do not reorder: the arguments are passed in
// the order written, so they must be written in the function's order.
//
// Reference: [Application](../../docs/notation.md#application)

use rsml::ml;

pub const SUMMARY: &str = "Borrowing `f &xs`, `f &mut xs`; labels `f ~x:1 ~y` are dropped";

fn total(prices: &[u32]) -> u32 {
    prices.iter().sum()
}

fn add_tax(prices: &mut [u32], percent: u32) {
    for p in prices.iter_mut() {
        *p += *p * percent / 100;
    }
}

fn describe(name: &str, price: u32) -> String {
    format!("{name}: {price}")
}

/// The total before and after a 10% tax.
pub fn totals(mut prices: Vec<u32>) -> (u32, u32) {
    ml! {
        // `total &prices` is `total(&prices)`: `prices` is only lent.
        let before = total &prices in
        // `add_tax &mut prices 10` is `add_tax(&mut prices, 10)`.
        add_tax &mut prices 10;
        (before, total &prices)
    }
}

/// Labels say what the arguments are, and disappear.
pub fn label(name: &str, price: u32) -> String {
    ml! {
        // `describe ~name ~price:(price * 2)` is `describe(name, price * 2)`.
        describe ~name ~price:(price * 2)
    }
}

pub fn run() {
    println!("totals [100; 250] = {:?}", totals(vec![100, 250]));
    println!("label \"tea\" 4 = {:?}", label("tea", 4));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lent_and_changed() {
        assert_eq!(totals(vec![100, 250]), (350, 385));
    }

    #[test]
    fn labels_are_positional() {
        assert_eq!(label("tea", 4), "tea: 8");
    }
}

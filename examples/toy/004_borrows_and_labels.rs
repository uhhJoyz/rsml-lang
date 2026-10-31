// Toy 004: borrowed arguments `f &xs`, `f &mut xs`, and labels `~x:e`.
//
// What it is: values move into a call, as in Rust; `&xs` lends them and
// `&mut xs` lends them for changing. Labels, `~name:value` and
// `~name` (short for `~name:name`), give an argument to the parameter of
// that name, in any order.
//
// When to use it: `&`/`&mut` whenever the Rust function takes a
// reference. Labels to say what an argument means, or to give the
// arguments in another order.
//
// How to write it:
// * `&xs` and `&mut xs` are one argument each, no parentheses needed.
// * Labels need a function whose parameters the macro sees by name: one
//   defined by `let f ~x ~y = ..` (or `let rec`, or `let f = fun ~x ..`),
//   or a `fn` of the same `rsml!`. A Rust function defined elsewhere is
//   not: `Option::unwrap_or None ~default:3` is an error: "a labeled
//   argument needs a function whose parameters rsml sees by name".
// * The arguments are evaluated in the order written, whatever order the
//   function takes them in. Unlabeled ones fill the parameters left, and a
//   parameter left out before a given one makes a function of it: with
//   `let f ~x ~y = ..`, `f ~y:2` waits for `x`.
//
// Reference: [Application](../../docs/notation.md#application)

use rsml::ml;

pub const SUMMARY: &str = "Borrowing `f &xs`, `f &mut xs`; labels `f ~y ~x:1` in any order";

fn total(prices: &[u32]) -> u32 {
    prices.iter().sum()
}

fn add_tax(prices: &mut [u32], percent: u32) {
    for p in prices.iter_mut() {
        *p += *p * percent / 100;
    }
}

/// The total before and after a 10% tax.
pub fn totals(mut prices: Vec<u32>) -> (u32, u32) { ml! {
    // `total &prices` is `total(&prices)`: `prices` is only lent.
    let before = total &prices in
    // `add_tax &mut prices 10` is `add_tax(&mut prices, 10)`.
    add_tax &mut prices 10;
    (before, total &prices)
}}

/// Labels say what the arguments are, in any order.
pub fn label(name: &str, price: u32) -> String { ml! {
    let describe ~(name : &str) ~(price : u32) = format!("{name}: {price}") in
    // Given `let f ~x ~y = x - y`, `f ~y:1 ~x:3` is `f(3, 1)`.
    describe ~price:(price * 2) ~name
}}

/// A label left out: `describe ~price` waits for the name.
pub fn price_tags(names: &[&str], price: u32) -> Vec<String> { ml! {
    let describe ~(name : &str) ~(price : u32) = format!("{name}: {price}") in
    let with_price = describe ~price in
    names.iter().map(fun n -> with_price n).collect()
}}

pub fn run() {
    println!("totals [100; 250] = {:?}", totals(vec![100, 250]));
    println!("label \"tea\" 4 = {:?}", label("tea", 4));
    println!(
        "price_tags [tea; jam] 3 = {:?}",
        price_tags(&["tea", "jam"], 3)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lent_and_changed() {
        assert_eq!(totals(vec![100, 250]), (350, 385));
    }

    #[test]
    fn labels_in_any_order() {
        assert_eq!(label("tea", 4), "tea: 8");
        assert_eq!(price_tags(&["tea", "jam"], 3), ["tea: 3", "jam: 3"]);
    }
}

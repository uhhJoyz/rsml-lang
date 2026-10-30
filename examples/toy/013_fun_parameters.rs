// Toy 013: the parameters of `fun`: types, tuples, `&x`, labels.
//
// What it is: each parameter of `fun` is a name or a pattern in
// parentheses, as in a Rust closure.
//
// When to use it: a type when rustc needs it (to call a method on the
// parameter), a tuple pattern to take a pair apart, `&x` to copy a value
// out of a reference.
//
// How to write it:
// * A type: `fun (w : &str) -> w.len()`.
// * A tuple: `fun (name, qty) -> ..`, one parameter that is a pair.
// * A reference pattern needs parentheses: `fun (&p) -> p * 2`;
//   `fun &p -> p * 2` is an error: "expected a parameter after `fun`".
// * Labels are dropped, as in calls: `fun ~x ~y:z -> x + z` is
//   `move |x, z| x + z`.
// * No parameters: `fun () -> ..`.
//
// Reference: [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function)

use rsml::ml;

pub const SUMMARY: &str = "`fun (x : t)`, `fun (a, b)`, `fun (&x)`, `fun ~x` parameters";

/// The length of each word: the parameter's type allows `w.len()`.
pub fn lengths(words: &[&str]) -> Vec<usize> {
    ml! {
        // `it.map(fun (w : &str) -> w.len())` is `it.map(|w: &str| w.len())`.
        words.iter().copied().map(fun (w : &str) -> w.len()).collect()
    }
}

/// Order lines from (name, quantity) pairs.
pub fn order_lines(pairs: &[(&str, u32)]) -> Vec<String> {
    ml! {
        // One parameter, a pair: `fun (name, qty) -> qty` is
        // `move |(name, qty)| qty`.
        pairs.iter().map(fun (name, qty) -> format!("{qty} x {name}")).collect()
    }
}

/// The total of doubled prices: `&p` copies each price out.
pub fn doubled_total(prices: &[u32]) -> u32 {
    ml! {
        // `prices.iter().map(fun (&p) -> p * 2)` is
        // `prices.iter().map(|&p| p * 2)`.
        prices.iter().map(fun (&p) -> p * 2).sum()
    }
}

/// A rectangle's area, with labelled parameters.
pub fn labelled_area(w: u32, h: u32) -> u32 {
    ml! {
        // The labels are for the reader: `area` takes two parameters.
        let area = fun ~width ~height -> width * height in
        area w h
    }
}

pub fn run() {
    println!("lengths [tea; coffee] = {:?}", lengths(&["tea", "coffee"]));
    println!("order_lines [(tea, 2)] = {:?}", order_lines(&[("tea", 2)]));
    println!("doubled_total [3; 4] = {}", doubled_total(&[3, 4]));
    println!("labelled_area 3 4 = {}", labelled_area(3, 4));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters() {
        assert_eq!(lengths(&["tea", "coffee"]), [3, 6]);
        assert_eq!(order_lines(&[("tea", 2)]), ["2 x tea"]);
        assert_eq!(doubled_total(&[3, 4]), 14);
        assert_eq!(labelled_area(3, 4), 12);
    }

    #[test]
    fn labels_are_dropped() {
        let plus = ml! { fun ~x ~y:z -> x + z };
        assert_eq!(plus(1, 2), 3);
    }
}

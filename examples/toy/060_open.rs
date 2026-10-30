// Toy 060: `open`, a module's names for the rest of the block.
//
// What it is: OCaml's `open M` makes the names of `M` usable without the
// `M.` in front. Here it is Rust's glob import:
// `open std::cmp ;; e` is `{ use std::cmp::*; e }`.
//
// When to use it: when a block calls several functions of one module.
//
// How to write it: `open path ;;` as a phrase of the block (toy 011), with
// a Rust path. Rust's rules apply: a local name hides an opened one, and
// two opened modules may not both give a name you use.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)
// (and [Paths](../../docs/notation.md#paths))

use rsml::ml;

pub const SUMMARY: &str = "`open std::cmp ;;`: Rust's `use std::cmp::*;` for the rest of the block";

/// A value kept between two bounds, with `std::cmp`'s functions.
pub fn clamped(low: i32, x: i32, high: i32) -> i32 {
    ml! {
        // `open std::cmp ;; max low x` is `{ use std::cmp::*; max(low, x) }`.
        open std::cmp ;;
        min (max low x) high
    }
}

/// The circumference and area of a circle, with `PI` and `TAU` opened.
pub fn circle(r: f64) -> (f64, f64) {
    ml! {
        open std::f64::consts ;;
        // `TAU` names `std::f64::consts::TAU`, through the `open`.
        (TAU *. r, PI *. r *. r)
    }
}

/// A count of the words of each length, with a map from `std::collections`.
pub fn lengths(text: &str) -> Vec<(usize, u32)> {
    ml! {
        open std::collections ;;
        let mut by_len = BTreeMap.new () in
        for w in text.split_whitespace() do
          let n = by_len.entry(w.len()).or_insert(0) in
          *n <- *n + 1
        done;
        by_len.into_iter().collect()
    }
}

pub fn run() {
    println!(
        "clamped 0 15 10 = {}, clamped 0 (-3) 10 = {}",
        clamped(0, 15, 10),
        clamped(0, -3, 10)
    );
    let (c, a) = circle(1.0);
    println!("circle 1.0 = ({c:.4}, {a:.4})");
    println!("lengths \"a bb cc d\" = {:?}", lengths("a bb cc d"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opened_names() {
        assert_eq!(clamped(0, 15, 10), 10);
        assert_eq!(clamped(0, -3, 10), 0);
        assert_eq!(clamped(0, 4, 10), 4);
        assert_eq!(circle(1.0), (std::f64::consts::TAU, std::f64::consts::PI));
        assert_eq!(lengths("a bb cc d"), [(1, 2), (2, 2)]);
    }
}

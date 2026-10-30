// Toy 048: method sections, `.map(f)`, `.push(item) cart`.
//
// What it is: a method with its arguments written Rust style, right after
// the name, takes exactly those arguments and waits for its receiver:
// `.map(f)` is `fun r -> r.map(f)`. Applied, as in `xs |> .map(f)`, it
// is `xs.map(f)`, with no closure. A pipeline of them reads like a Rust
// method chain.
//
// When to use it: for iterator pipelines, when Rust's own call syntax
// reads better than `.map _ f` (toy 046).
//
// How to write it:
// * `xs |> .iter() |> .map(fun x -> x * 2) |> .sum::<i64>()` is
//   `xs.iter().map(|x| x * 2).sum::<i64>()`.
// * `.push(1) xs` is `xs.push(1)`: the receiver comes after the section.
// * The space matters: `.max(a) b` is `b.max(a)`, but `.max (a) b` is
//   `a.max(b)`, the method of `a` (toy 045).
// * A section alone is a closure: `.map(f)` is
//   `move |receiver| receiver.map(f)`.
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name)

use rsml::ml;

pub const SUMMARY: &str = "`xs |> .iter() |> .map(f) |> .sum::<i64>()`, `.push(1) xs`";

/// The total weight of the parcels over 1 kg, in grams.
pub fn heavy_total(grams: &[i64]) -> i64 {
    ml! { grams |> .iter() |> .copied() |> .filter(fun g -> !g > 1000) |> .sum::<i64>() }
}

/// Two more parcels on the list.
pub fn with_two_more(mut grams: Vec<i64>) -> Vec<i64> {
    ml! {
        // `.push(250) grams` is `grams.push(250)`.
        .push(250) grams;
        grams |> .push(4000);
        grams
    }
}

/// Every weight at least a minimum: `.max(min)` waits for its receiver.
pub fn at_least(min: i64, grams: &[i64]) -> Vec<i64> {
    ml! {
        // `.max(min) !g` is `(*g).max(min)`.
        grams |> .iter() |> .map(fun g -> .max(min) !g) |> .collect()
    }
}

pub fn run() {
    println!(
        "heavy_total [500; 1500; 3000] = {}",
        heavy_total(&[500, 1500, 3000])
    );
    println!("with_two_more [100] = {:?}", with_two_more(vec![100]));
    println!("at_least 200 [100; 500] = {:?}", at_least(200, &[100, 500]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections() {
        assert_eq!(heavy_total(&[500, 1500, 3000]), 4500);
        assert_eq!(with_two_more(vec![100]), [100, 250, 4000]);
        assert_eq!(at_least(200, &[100, 500]), [200, 500]);
    }

    #[test]
    fn the_space_decides_the_receiver() {
        // Unsigned, so that `3 - 8` saturates at 0.
        let (a, b) = (3u64, 8u64);
        assert_eq!(ml! { .max(a) b }, 8);
        assert_eq!(ml! { .min(a) b }, 3);
        assert_eq!(ml! { .saturating_sub(a) b }, 5);
        assert_eq!(ml! { .saturating_sub (a) b }, 0);
    }
}

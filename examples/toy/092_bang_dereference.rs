// Toy 092: `!x`, reading through a reference.
//
// What it is: `!r` is the value of a `ref` (toy 089), and more generally
// a dereference of any reference, `*x`. Iterators over borrowed data hand
// out references, so `!x` is how a closure reads the element.
//
// When to use it: in closures over `.iter()` (`fun x -> !x * 2`), on list
// pattern elements (toy 070), and on `ref` cells (toy 089).
//
// How to write it:
// * `!` binds tighter than `.`: `!m.age` is the field of
//   `!m`, where Rust's `*m.age` would be the dereferenced field (toy 091).
// * `!` never negates; `not` does. rsml gives an error for `!` before a
//   comparison or before `true` (quoted below), so the mix-up cannot
//   compile there.
// * One `!` per reference: `filter` hands out `&&i64`, read as `!!x`.
//
// Reference: [Rust-style methods, fields and calls](../../docs/notation.md#rust-style-methods-fields-and-calls)

use rsml::ml;

pub const SUMMARY: &str = "`!x` reads through a reference; `!m.age` is `(!m).age`; `not` negates";

/// Every price doubled: `.iter()` hands out `&i64`.
pub fn doubled(prices: &[i64]) -> Vec<i64> { ml! {
    // `fun x -> !x * 2` is `fun x -> ( *x) * 2`.
    prices |> .iter() |> .map(fun x -> !x * 2) |> .collect()
}}

/// How many prices are above 10: `filter` hands out `&&i64`.
pub fn above_ten(prices: &[i64]) -> usize { ml! {
    // `!!x` is `**x`: one `!` per reference.
    prices |> .iter() |> .filter(fun x -> !!x > 10) |> .count()
}}

pub struct Member {
    pub age: u32,
    pub paid: bool,
}

/// The ages of the members.
pub fn ages(members: &[Member]) -> Vec<u32> { ml! {
    // `!m.age` is `(!m).age`: the member, then its field.
    members |> .iter() |> .map(fun m -> !m.age) |> .collect()
}}

/// How many members have not paid yet.
pub fn unpaid(members: &[Member]) -> usize { ml! {
    // `not m.paid` is `!m.paid` (Rust's negation).
    members |> .iter() |> .filter(fun m -> not m.paid) |> .count()
}}

// `!(x > 0)` is an error: "a `bool` cannot be dereferenced: `!r` is the
// value of `r` here, and negation is `not e`"

pub fn run() {
    println!("doubled [3; 12] = {:?}", doubled(&[3, 12]));
    println!("above_ten [3; 12; 40] = {}", above_ten(&[3, 12, 40]));
    let club = [
        Member {
            age: 34,
            paid: true,
        },
        Member {
            age: 17,
            paid: false,
        },
    ];
    println!("ages = {:?}, unpaid = {}", ages(&club), unpaid(&club));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_through_references() {
        assert_eq!(doubled(&[3, 12]), [6, 24]);
        assert_eq!(above_ten(&[3, 12, 40]), 2);
        let club = [
            Member {
                age: 34,
                paid: true,
            },
            Member {
                age: 17,
                paid: false,
            },
        ];
        assert_eq!(ages(&club), [34, 17]);
        assert_eq!(unpaid(&club), 1);
    }
}

// Toy 098: tuples, `(a, b)`, `p.0` and `let (a, b) = .. in`.
//
// What it is: tuples are Rust's, written the same way. They are read by
// position (`p.0`, `p.1`) or taken apart with a pattern.
//
// When to use it: to return two values at once, or to keep a few values
// together without naming a struct.
//
// How to write it:
// * `f (a, b)` passes ONE argument, the tuple; `f a b` passes two (toy
//   053 has the Rust-style `f(a, b)`).
// * `let (q, r) = e in ..`, `fun (x, y) -> ..` and `match` cases (toy
//   069) take tuples apart; `_` skips a part.
// * `()` is the empty tuple, the unit value (toy 005).
//
// Reference: [Records and collections](../../docs/notation.md#records-and-collections)

use rsml::ml;

pub const SUMMARY: &str = "Tuples: `(q, r)`, `p.0`, `let (q, r) = .. in`, `f (a, b)`";

/// The quotient and the remainder, as one pair.
pub fn divide(n: i64, d: i64) -> (i64, i64) {
    ml! {
        // `(n / d, n mod d)` is `(n / d, n % d)`.
        (n / d, n mod d)
    }
}

/// Minutes and seconds back to seconds, through the pair.
pub fn seconds(total: i64) -> i64 {
    ml! {
        // `let (q, r) = divide 17 5 in q * 10 + r` is
        // `{ let (q, r) = divide(17, 5); (q * 10) + r }`.
        let (minutes, secs) = divide total 60 in
        minutes * 60 + secs
    }
}

/// How far a point is from the origin, walking along the grid.
fn manhattan(p: (i64, i64)) -> i64 {
    ml! { p.0.abs() + p.1.abs() }
}

pub fn walk() -> i64 {
    ml! {
        // `manhattan (3, -4)` is `manhattan((3, -4))`: one tuple.
        manhattan (3, -4)
    }
}

/// Nested tuples come apart in one pattern.
pub fn total(order: (&str, (u32, u32))) -> u32 {
    ml! {
        let (_, (count, price)) = order in
        count * price
    }
}

pub fn run() {
    println!("divide 17 5 = {:?}", divide(17, 5));
    println!("seconds 125 = {}", seconds(125));
    println!("manhattan (3, -4) = {}", walk());
    println!("total (\"tea\", (3, 4)) = {}", total(("tea", (3, 4))));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs() {
        assert_eq!(divide(17, 5), (3, 2));
        assert_eq!(seconds(125), 125);
        assert_eq!(walk(), 7);
        assert_eq!(total(("tea", (3, 4))), 12);
    }
}

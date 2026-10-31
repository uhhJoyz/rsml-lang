// Toy 115: associated functions by name, `Self::new` and `Money::new`.
//
// What it is: an associated function without `self` (a constructor, a
// helper) is known by the macro as `Self::name` inside its `impl`, and as
// `Type::name` everywhere in the module of the `rsml!`, so it can be
// partially applied like the block's other functions (toy 114).
//
// When to use it: for constructors applied ML style, and to hand one
// to `map` with some arguments fixed.
//
// How to write it:
// * In the `impl`: `Self::new 0 0`; outside it: `Money::new 5 0`.
// * `Money::new 5` alone waits for the cents; in full,
//   `rsml! { struct P; impl P { fn new(a: i64, b: i64) -> i64 { a + b } } fn g() -> impl Fn(i64) -> i64 { P::new 2 } }`
//   is `struct P; impl P { fn new(a: i64, b: i64) -> i64 { a + b } } fn g() -> impl Fn(i64) -> i64 { move |arg2| P::new(2, arg2) }`.
// * Only inherent `impl`s count; functions of trait `impl`s are not
//   known by `Type::name`.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "`Self::new 0 0` in the `impl`, `Money::new 5` partial outside";

rsml! {
/// An amount of money, in euros and cents.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Money {
    pub euros: u32,
    pub cents: u32,
}

impl Money {
    /// A normalised amount: cents above 99 carry into euros.
    pub fn new(euros: u32, cents: u32) -> Money {
        Money { euros = euros + cents / 100; cents = cents mod 100 }
    }

    /// No money: `Self::new` called ML style.
    pub fn zero() -> Money {
        Self::new 0 0
    }

    /// The sum of two amounts.
    pub fn plus(&self, other: Money) -> Money {
        Self::new (self.euros + other.euros) (self.cents + other.cents)
    }
}

/// Prices of five euros and some cents: `Money::new 5` waits for them.
pub fn five_and(cents: &[u32]) -> Vec<Money> {
    cents |> .iter() |> .copied() |> .map(Money::new 5) |> .collect()
}
}

pub fn run() {
    println!("Money::zero () = {:?}", Money::zero());
    println!("five_and [50; 120] = {:?}", five_and(&[50, 120]));
    println!(
        "(Money::new 1 60).plus (Money::new 2 70) = {:?}",
        Money::new(1, 60).plus(Money::new(2, 70))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_by_name() {
        assert_eq!(Money::zero(), Money { euros: 0, cents: 0 });
        assert_eq!(
            five_and(&[50, 120]),
            [
                Money {
                    euros: 5,
                    cents: 50
                },
                Money {
                    euros: 6,
                    cents: 20
                }
            ]
        );
        assert_eq!(
            Money::new(1, 60).plus(Money::new(2, 70)),
            Money {
                euros: 4,
                cents: 30
            }
        );
    }
}

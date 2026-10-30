// Toy 026: application with `@@`.
//
// What it is: `f @@ x` applies `f` to `x`, like `f x`, but `@@` binds
// looser than application and arithmetic, and groups to the right. So
// `f @@ g @@ x` is `f (g x)`: everything after `@@` is one argument.
//
// When to use it: to drop the parentheses around a last argument that is
// itself an application or an operation.
//
// How to write it: `f a @@ rest`, where `rest` is the last argument.
// * `@@` binds tighter than `|>`: `x |> f @@ y` is `x |> (f y)`.
// * Like `|>`, it needs no arity: the result is a plain call.
//
// Reference: [Pipelines: |> and @@](../../docs/notation.md#pipelines--and-)

use rsml::ml;

pub const SUMMARY: &str = "`round_to 10 @@ with_tax 20 @@ subtotal items`, no parentheses";

fn subtotal(items: &[u32]) -> u32 {
    items.iter().sum()
}

fn with_tax(rate: u32, total: u32) -> u32 {
    total + total * rate / 100
}

/// `total` rounded down to a multiple of `step`.
fn round_to(step: u32, total: u32) -> u32 {
    total / step * step
}

/// Subtotal, with tax, rounded: the same as a pipeline, read right to left.
pub fn bill(items: &[u32]) -> u32 {
    ml! {
        // `round_to 10 @@ with_tax 20 @@ subtotal items` is
        // `round_to(10, with_tax(20, subtotal(items)))`.
        round_to 10 @@ with_tax 20 @@ subtotal items
    }
}

/// An operation after `@@` needs no parentheses either.
pub fn reserve(stock: u32, held: u32) -> Option<u32> {
    ml! {
        // `Some @@ stock - held` is `Some (stock - held)`.
        if held <= stock then Some @@ stock - held else None
    }
}

pub fn run() {
    println!("bill [199; 250] = {}", bill(&[199, 250]));
    println!("reserve 10 3 = {:?}", reserve(10, 3));
    println!("reserve 2 3 = {:?}", reserve(2, 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applied_right_to_left() {
        // 449 with 20% tax is 538 (rounded down), then 530.
        assert_eq!(bill(&[199, 250]), 530);
        assert_eq!(
            bill(&[199, 250]),
            round_to(10, with_tax(20, subtotal(&[199, 250])))
        );
    }

    #[test]
    fn operations_after_at_at() {
        assert_eq!(reserve(10, 3), Some(7));
        assert_eq!(reserve(2, 3), None);
    }
}

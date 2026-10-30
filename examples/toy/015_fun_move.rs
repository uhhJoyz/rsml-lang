// Toy 015: `fun move`, and when a `fun` moves by itself.
//
// What it is: a Rust closure borrows what it uses, unless it is `move`,
// which moves (or copies) what it uses into the closure. In rsml:
// * a `fun` that is the value of the block (or of an enclosing `fun`) is
//   `move`, so that it can be returned;
// * except when it assigns to something: then it borrows, so the
//   assignment changes the variable outside;
// * a `fun` passed as an argument borrows, like a Rust closure;
//   `fun move` makes one `move`.
//
// When to use it: `fun move` when a closure passed somewhere must outlive
// the function, as in a returned iterator.
//
// How to write it:
// * `fun move x -> x + by`; the rest is as for `fun` (toy 012).
// * Without the `move` below, rustc rejects `shifted`: the closure would
//   borrow `by`, a parameter that ends with the function.
//
// Reference: [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function)

use rsml::ml;

pub const SUMMARY: &str = "`fun move`; a returned `fun` moves, an argument borrows";

/// A function that adds a tax rate: the returned `fun` owns `rate`.
pub fn tax_adder(rate: u32) -> impl Fn(u32) -> u32 {
    ml! {
        // `fun price -> price + price * rate / 100` is
        // `fun move price -> price + price * rate / 100`.
        fun price -> price + price * rate / 100
    }
}

/// Each number shifted by `by`, lazily: the iterator outlives `by`.
pub fn shifted(xs: &[i64], by: i64) -> impl Iterator<Item = i64> + '_ {
    ml! {
        // `xs.iter().map(fun move x -> x + by)` is
        // `xs.iter().map(move |x| x + by)`.
        xs.iter().map(fun move x -> x + by)
    }
}

/// Each number scaled, collected at once: a borrowing closure is enough.
pub fn scaled(xs: &[i64], k: i64) -> Vec<i64> {
    ml! {
        // `xs.iter().map(fun x -> x * k)` is `xs.iter().map(|x| x * k)`.
        xs.iter().map(fun x -> x * k).collect()
    }
}

/// How often a callback was called: it assigns, so it is not `move`.
pub fn count_calls(n: u32) -> u32 {
    let mut calls = 0;
    {
        // Were it `move`, it would change a copy of `calls`.
        let mut call = ml! { fun () -> calls <- calls + 1 };
        for _ in 0..n {
            call();
        }
    }
    calls
}

pub fn run() {
    println!("tax_adder 20 100 = {}", tax_adder(20)(100));
    println!(
        "shifted [1; 2] 10 = {:?}",
        shifted(&[1, 2], 10).collect::<Vec<_>>()
    );
    println!("scaled [1; 2] 3 = {:?}", scaled(&[1, 2], 3));
    println!("count_calls 4 = {}", count_calls(4));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moved_or_borrowed() {
        assert_eq!(tax_adder(20)(100), 120);
        assert_eq!(shifted(&[1, 2], 10).collect::<Vec<_>>(), [11, 12]);
        assert_eq!(scaled(&[1, 2], 3), [3, 6]);
        assert_eq!(count_calls(4), 4);
    }
}

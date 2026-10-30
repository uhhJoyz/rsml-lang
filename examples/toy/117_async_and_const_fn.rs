// Toy 117: `async fn` and `const fn` with OCaml bodies.
//
// What it is: an `rsml!` function may be `async` or `const`, as in Rust;
// only the body is OCaml.
// * An `async fn` returns a future, so the macro gives it no closure to
//   call after its arguments: `total 1 2` is a plain call that makes the
//   future, and `.await` (toy 120) runs it.
// * A `const fn` can run at compile time; its OCaml body must be what
//   Rust allows in a `const fn` (arithmetic, `if`, `match`, ..).
//
// When to use it: `async fn` for code that waits (on I/O, a timer, a
// channel); `const fn` for values computed into constants.
//
// How to write it: `async fn` / `const fn` in the signature.
// * `rsml! { const fn sq(x: i64) -> i64 { x * x } const N: i64 = sq(4); }`
//   is `const fn sq(x: i64) -> i64 { x * x } const N: i64 = sq(4);`.
// * The constant's initializer is Rust, outside any body.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

pub const SUMMARY: &str = "`async fn` and `const fn` in `rsml!`, OCaml bodies";

rsml! {
    /// Seconds in a number of days, at compile time when it can be.
    pub const fn seconds_in(days: u64) -> u64 {
        days * 24 * 60 * 60
    }

    /// A week in seconds, computed by the compiler.
    pub const WEEK: u64 = seconds_in(7);

    /// The total of two orders, as a future.
    pub async fn total(a: u32, b: u32) -> u32 {
        a + b
    }

    /// Awaits two futures one after the other.
    pub async fn both() -> u32 {
        let first = total 1 2 |> .await in
        // `|>` binds looser than `+`: the parentheses keep `.await` on
        // the future.
        first + (total 10 20 |> .await)
    }
}

/// Runs a future that never has to wait, with the no-op waker of `std`.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("this future waits; a real executor is needed"),
    }
}

pub fn run() {
    println!("WEEK = {WEEK} seconds");
    println!("block_on (total 4 5) = {}", block_on(total(4, 5)));
    println!("block_on (both ()) = {}", block_on(both()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_and_async() {
        assert_eq!(WEEK, 604_800);
        const DAY: u64 = seconds_in(1);
        assert_eq!(DAY, 86_400);
        assert_eq!(block_on(total(4, 5)), 9);
        assert_eq!(block_on(both()), 33);
    }
}

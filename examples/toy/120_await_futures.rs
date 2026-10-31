// Toy 120: `.await`, `async fn` and `async { .. }`.
//
// What it is: Rust's futures, from rsml code. `e.await` waits for the
// future `e`, inside an `async fn` or an `async` block; `.await` is also a
// method reference (toy 045), so `.await fut` and `fut |> .await` work
// too. `async { e }` and `async move { e }` are futures whose body is
// rsml code, as the braces of `begin .. end` would hold.
//
// When to use it: with any async Rust code. The futures here never wait,
// so one poll runs them to their end; real programs use an executor.
//
// How to write it:
// * `.await` right after an expression: `price(item).await`.
// * `async move { e }` when the future must take what it uses (here, to
//   outlive the closure that makes it).
// * An `async fn` returns a future, so `price item` is a plain call that
//   makes one; nothing runs until it is awaited or polled.
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name),
// [Control flow](../../docs/notation.md#control-flow)

use rsml::{ml, rsml};
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

pub const SUMMARY: &str = "`price(item).await`, `fut |> .await`, `async move { e }`";

rsml! {
/// A price, as an asynchronous lookup (a database, a web shop) gives it.
async fn price(item: &str) -> u32 {
    match item with "tea" -> 4 | "cake" -> 6 | _ -> 0
}

/// The total of an order: each price is awaited in turn.
async fn order_total(items: &[&str]) -> u32 {
    let mut total = 0 in
    for item in items do total <- total + price(item).await done;
    total
}

/// The cheaper of two items, with `.await` as a method reference.
async fn cheaper(a: &str, b: &str) -> u32 {
    // `price a |> .await` is `price(a).await`.
    let pa = price a |> .await in
    // `.await (price b)` is `price(b).await`.
    let pb = .await (price b) in
    std::cmp::min pa pb
}
}

/// Futures made by `async` blocks, one per total: none runs until it is
/// polled.
pub fn with_tips(totals: &[u32]) -> Vec<impl Future<Output = u32>> { ml! {
    // `async move { t + t / 10 }` is `async move { t + (t / 10) }`;
    // `move` takes `t` into the future, which outlives the closure.
    totals |> .iter() |> .map(fun (&t) -> async move { t + t / 10 }) |> .collect()
}}

/// Runs a future to its end with one poll: these never wait for anything.
/// `Waker::noop()` is a waker that does nothing when woken.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("a toy future had to wait"),
    }
}

pub fn run() {
    let order = ["tea", "cake", "tea"];
    println!(
        "order_total [tea; cake; tea] = {}",
        block_on(order_total(&order))
    );
    println!("cheaper tea cake = {}", block_on(cheaper("tea", "cake")));
    let tipped: Vec<u32> = with_tips(&[50, 120]).into_iter().map(block_on).collect();
    println!("with_tips [50; 120], each polled = {tipped:?}");
    let later = ml! { async { price "cake" |> .await } };
    println!("an async block awaiting price cake = {}", block_on(later));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awaited() {
        assert_eq!(block_on(order_total(&["tea", "cake", "tea"])), 14);
        assert_eq!(block_on(cheaper("tea", "cake")), 4);
        let tipped: Vec<u32> = with_tips(&[50, 120]).into_iter().map(block_on).collect();
        assert_eq!(tipped, [55, 132]);
        assert_eq!(block_on(ml! { async { price "cake" |> .await } }), 6);
    }
}

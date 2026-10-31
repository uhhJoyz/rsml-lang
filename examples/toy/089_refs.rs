// Toy 089: mutable cells, `ref`, `!`, `:=`, `incr` and `decr`.
//
// What it is: a way to change a local, ML style. `let r = ref 0 in ..`
// makes a mutable local (Rust's `let mut r = 0`), `!r` reads it, `r := e`
// writes it, and `incr r` and `decr r` add or take away one.
//
// When to use it: for counters and accumulators written ML style.
// `let mut` with `<-` (toy 090) makes the same local, spelled Rust's way.
//
// How to write it:
// * `!r` is the value and `r` alone the cell: passing `r` to a function
//   passes `&mut r`, so the function can change it.
// * On a `&mut` parameter, which is not a `ref` of the block, `incr c`,
//   `!c` and `c := e` go through the reference.
// * `!` reads; it never negates (`not` does, toy 067).
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)

use rsml::ml;

pub const SUMMARY: &str = "Mutable cells: `ref 0`, `!r`, `r := e`, `incr r`, `decr r`";

/// The sum of the prices, accumulated in a cell.
pub fn total(prices: &[u32]) -> u32 { ml! {
    // `let total = ref 0 in total := !total + !p` is
    // `let mut total = 0 in total <- total + !p` (toy 090).
    let total = ref 0 in
    // `!p` reads the price through the reference the loop gives.
    for p in prices do total := !total + !p done;
    !total
}}

/// Adds one to a counter that someone else owns.
fn bump(counter: &mut u32) { ml! {
    // `counter` has type `&mut u32` here: `incr counter` is `*counter += 1`.
    incr counter
}}

/// How many words are longer than four letters.
pub fn count_long(words: &[&str]) -> u32 { ml! {
    // `let n = ref 0 in bump n` is `{ let mut n = 0; bump(&mut n) }`.
    let n = ref 0 in
    for w in words do if w.len() > 4 then bump n done;
    !n
}}

/// The numbers of a countdown, from `start` down to 1.
pub fn countdown(start: u32) -> Vec<u32> { ml! {
    let left = ref start in
    let steps = ref [] in
    while !left > 0 do
      steps.push(!left);
      // `let r = ref 3 in decr r; !r` is `{ let mut r = 3; r -= 1; r }`.
      decr left
    done;
    !steps
}}

pub fn run() {
    println!("total [3; 4; 5] = {}", total(&[3, 4, 5]));
    let words = ["apple", "fig", "banana"];
    println!("count_long {words:?} = {}", count_long(&words));
    println!("countdown 3 = {:?}", countdown(3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells() {
        assert_eq!(total(&[3, 4, 5]), 12);
        assert_eq!(count_long(&["apple", "fig", "banana"]), 2);
        assert_eq!(countdown(3), [3, 2, 1]);
        assert!(countdown(0).is_empty());
    }

    #[test]
    fn through_a_reference() {
        let mut calls = 7;
        bump(&mut calls);
        assert_eq!(calls, 8);
    }
}

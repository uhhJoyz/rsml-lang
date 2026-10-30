// Toy 085: the endless loop, `while true do .. done`, and `break e`.
//
// What it is: OCaml's endless loop is `while true do .. done`; it is
// Rust's `loop`, which rustc would ask for: `while true do break done` is
// `loop { break; }`. Like Rust's `loop`, it can give a value: `break e`
// leaves it with `e`.
//
// When to use it: for a search or a retry whose end is found inside the
// body.
//
// How to write it:
// * `break e` gives the loop the value `e`; `break` alone leaves it with
//   `()`.
// * The value of `break` extends as an `if` branch does, up to a `;`,
//   `done`, `else` and the like.
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`while true do .. break e .. done`: Rust's `loop`, with a value";

/// The first square number at least as large as `n`.
pub fn square_at_least(n: u64) -> u64 {
    ml! {
        let mut k = 0 in
        // `while true do f () done` is `loop { f(); }`.
        while true do
          if k * k >= n then break k * k;
          k <- k + 1
        done
    }
}

/// The first free seat number, from a list of taken ones.
pub fn first_free(taken: &[u32]) -> u32 {
    ml! {
        let mut seat = 1 in
        while true do if not (taken.contains(&seat)) then break seat else seat <- seat + 1 done
    }
}

pub fn run() {
    println!("square_at_least 20 = {}", square_at_least(20));
    println!("first_free [1; 2; 4] = {}", first_free(&[1, 2, 4]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_from_break() {
        assert_eq!(square_at_least(20), 25);
        assert_eq!(square_at_least(16), 16);
        assert_eq!(first_free(&[1, 2, 4]), 3);
        assert_eq!(first_free(&[]), 1);
    }
}

// Toy 086: `return`, `break` and `continue`.
//
// What it is: Rust's jumps, written as in Rust: `return e` leaves the
// function, `continue` goes on with the next round of a loop, `break`
// leaves the loop. OCaml has none of them; rsml takes them as Rust has
// them.
//
// When to use it: for an early exit (a guard at the top of a function, a
// search that has found its answer) and to skip items in a loop.
//
// How to write it:
// * `if n < 0 then return 0; n` is `{ if n < 0 { return 0 } ; n }`.
// * The value of `return` reaches as an `if` branch does: up to a `;`,
//   `in`, `then`, `else`, `with`, a match `|`, `done` or a closing
//   bracket.
// * As an argument it needs parentheses: `f (return x)`.
// * In an `ml!` block, `return` leaves the Rust function the block is in.
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "Early `return e`, `continue` and `break` in loops";

/// The position of the first negative reading, if any.
pub fn first_negative(readings: &[i32]) -> Option<usize> {
    ml! {
        for (i, r) in readings.iter().enumerate() do if !r < 0 then return Some i done;
        None
    }
}

/// The sum of the readings before the first zero, skipping odd ones.
pub fn even_sum_until_zero(readings: &[i32]) -> i32 {
    ml! {
        let mut sum = 0 in
        for &r in readings do
          if r = 0 then break;
          if r mod 2 <> 0 then continue;
          sum <- sum + r
        done;
        sum
    }
}

/// A safe division: an early return guards the divisor.
pub fn per_person(total: u32, people: u32) -> u32 {
    ml! {
        if people = 0 then return 0;
        total / people
    }
}

pub fn run() {
    println!(
        "first_negative [3; -1; -2] = {:?}",
        first_negative(&[3, -1, -2])
    );
    println!(
        "even_sum_until_zero [2; 3; 4; 0; 6] = {}",
        even_sum_until_zero(&[2, 3, 4, 0, 6])
    );
    println!(
        "per_person 10 0 = {}, per_person 10 4 = {}",
        per_person(10, 0),
        per_person(10, 4)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jumps() {
        assert_eq!(first_negative(&[3, -1, -2]), Some(1));
        assert_eq!(first_negative(&[3]), None);
        assert_eq!(even_sum_until_zero(&[2, 3, 4, 0, 6]), 6);
        assert_eq!(per_person(10, 0), 0);
        assert_eq!(per_person(10, 4), 2);
    }
}

// Toy 128: formatter directives, `rsmlfmt: ..` comments.
//
// What it is: rsml-fmt formats the rsml code in `ml!` blocks and `rsml!`
// bodies. A comment that starts with `rsmlfmt:` before the first token of
// a block changes how that block is formatted:
// * `rsmlfmt: skip` leaves the block exactly as written;
// * `rsmlfmt: key = value; key = value` sets options of rsmlfmt.toml for
//   the block (`rsml-fmt --print-config` lists them all);
// * before the first item of an `rsml!`, it applies to every function in
//   it.
//
// When to use it: `skip` only where the spelling is the point (a table
// laid out by hand, a lesson about spacing, as in toys 050 and 052);
// options where one block reads better laid out differently.
//
// How to write it: `// rsmlfmt: ..`, `/* rsmlfmt: .. */` or
// `(* rsmlfmt: .. *)`, as the first thing inside the braces. Values of one
// word need no quotes. The blocks below are what rsml-fmt prints under
// their directives: `scripts/check_all.sh` runs `rsml-fmt --check` on
// this file, so they cannot drift.
//
// Reference: [Comments](../../docs/notation.md#comments),
// [rsml-fmt's options](../../rsml-fmt/README.md#configuring)

use rsml::{ml, rsml};

pub const SUMMARY: &str = "`// rsmlfmt: skip`, `(* rsmlfmt: max_width = 40 *)` per block";

/// A table laid out by hand: `skip` keeps the columns.
pub fn shipping(zone: u32) -> u32 {
    ml! {
        // rsmlfmt: skip
        match zone with
        | 1 ->  5    | 2 ->  8
        | 3 -> 12    | _ -> 20
    }
}

/// A narrow block: with `max_width = 40` the pipeline breaks per stage.
pub fn positive_total(xs: &[i64]) -> i64 {
    ml! {
    (* rsmlfmt: max_width = 40 *)
    xs
    |> .iter()
    |> .copied()
    |> .filter(fun x -> !x > 0)
    |> .sum::<i64>()
    }
}

rsml! {
// rsmlfmt: let.body = fit
// (For every function of this `rsml!`: a `let` body stays on the
// line of its `in` when it fits.)

/// The cost of a box, its sides in centimetres.
pub fn box_cost(w: u32, h: u32, d: u32) -> u32 {
    let area = 2 * (w * h + h * d + w * d) in let cents_per_cm2 = 3 in area * cents_per_cm2 / 100
}
}

pub fn run() {
    println!(
        "shipping 1 = {}, shipping 3 = {}, shipping 9 = {}",
        shipping(1),
        shipping(3),
        shipping(9)
    );
    println!(
        "positive_total [3; -2; 5] = {}",
        positive_total(&[3, -2, 5])
    );
    println!("box_cost 10 20 30 = {}", box_cost(10, 20, 30));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directives_change_only_the_layout() {
        assert_eq!(
            (shipping(1), shipping(2), shipping(3), shipping(9)),
            (5, 8, 12, 20)
        );
        assert_eq!(positive_total(&[3, -2, 5]), 8);
        // 2 * (200 + 600 + 300) = 2200 cm2, at 3 cents each: 66.
        assert_eq!(box_cost(10, 20, 30), 66);
    }
}

// Toy 035: sections of OCaml's word operators, `( lor )`, `( land )`, `( mod )`.
//
// What it is: OCaml spells some operators as words: `mod` (Rust's `%`),
// `land`, `lor`, `lxor` (`&`, `|`, `^` on integers), `lsl`, `lsr`, `asr`
// (shifts) and `or` (`||`). In parentheses they are sections like `( + )`.
//
// When to use it: bit flags, remainders and shifts passed as functions.
//
// How to write it: `( land )`, `( lor ) mask`, `( mod ) a b`.
// * `lsr` and `asr` are both Rust's `>>`, whose kind comes from the type:
//   it shifts in copies of the sign bit on signed integers and zeros on
//   unsigned ones. For OCaml's logical `lsr`, shift an unsigned integer.
// * `>>` and `<<` are composition in rsml (toy 029), never shifts.
//
// Reference: [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators)

use rsml::ml;

pub const SUMMARY: &str = "Word operators as sections: `( lor )`, `( land )`, `( mod )`, `( lsl )`";

pub const READ: u32 = 4;
pub const WRITE: u32 = 2;
pub const EXEC: u32 = 1;

/// The permission bits of several flags together.
pub fn mode(flags: &[u32]) -> u32 {
    ml! {
        // `( lor )` is `|lhs, rhs| lhs | rhs`.
        flags |> .iter() |> .copied() |> .fold(0, ( lor ))
    }
}

/// Whether `mode` has the flag.
pub fn allows(mode: u32, flag: u32) -> bool {
    ml! {
        // `( land ) mode flag <> 0` is `(mode & flag) != 0`.
        ( land ) mode flag <> 0
    }
}

/// The value of each bit position: 1, 2, 4, ..
pub fn bit_values(n: u32) -> Vec<u32> {
    ml! {
        // `( lsl ) 1u32` is `move |rhs| 1u32 << rhs`.
        (0..n) |> .map(( lsl ) 1u32) |> .collect()
    }
}

/// The remainders of dividing each total among some people.
pub fn leftovers(shares: &[(u32, u32)]) -> Vec<u32> {
    ml! {
        // `( mod ) total people` is `total % people`. (A `&` pattern as a
        // parameter goes in parentheses, toy 076.)
        shares |> .iter() |> .map(fun (&(total, people)) -> ( mod ) total people) |> .collect()
    }
}

/// Halving by a right shift, signed and unsigned.
pub fn halves(n: i64) -> (i64, u64) {
    ml! {
        // `( lsr ) n 1` is `n >> 1`: an arithmetic shift, as `n` is signed.
        (( lsr ) n 1, ( lsr ) (n as u64) 1)
    }
}

pub fn run() {
    let rw = mode(&[READ, WRITE]);
    println!("mode [READ; WRITE] = {rw:03b}");
    println!(
        "allows rw WRITE = {}, allows rw EXEC = {}",
        allows(rw, WRITE),
        allows(rw, EXEC)
    );
    println!("bit_values 4 = {:?}", bit_values(4));
    println!(
        "leftovers [(10, 3); (12, 4)] = {:?}",
        leftovers(&[(10, 3), (12, 4)])
    );
    println!("halves (-8) = {:?}", halves(-8));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags() {
        assert_eq!(mode(&[READ, WRITE]), 6);
        assert_eq!(mode(&[]), 0);
        assert!(allows(6, WRITE));
        assert!(!allows(6, EXEC));
    }

    #[test]
    fn shifts_and_remainders() {
        assert_eq!(bit_values(4), [1, 2, 4, 8]);
        assert_eq!(leftovers(&[(10, 3), (12, 4)]), [1, 0]);
        assert_eq!(halves(8), (4, 4));
        // Signed: -8 >> 1 is -4. Unsigned: the bits of -8, shifted with a
        // zero coming in on the left.
        assert_eq!(halves(-8), (-4, (-8i64 as u64) / 2));
    }
}

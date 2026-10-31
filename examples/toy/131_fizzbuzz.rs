// Toy 131: FizzBuzz.
//
// What it is: the classic counting game: from 1 on, say "Fizz" for the
// multiples of 3, "Buzz" for the multiples of 5, "FizzBuzz" for the
// multiples of both, and the number otherwise.
//
// When to use it: as a first program that combines notations: a `match`
// on a tuple (toy 069), `mod` (toy 066), a range piped into method
// sections (toys 088 and 048), and a `for .. to` loop over `ref` counters
// (toys 081 and 089).
//
// How to write it: the rules are one `match` on the two remainders, and
// the numbers come from a range. Both `1..=last` and `for n = 1 to last`
// include `last`.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::rsml;

pub const SUMMARY: &str = "FizzBuzz: a `match` on a tuple, `mod`, a range and pipes";

rsml! {
/// The word for one number.
pub fn word(n: u32) -> String {
    // `n mod 3` is `n % 3`; the pair is matched position by position,
    // and the first case that fits wins.
    match (n mod 3, n mod 5) with
    | (0, 0) -> String.from "FizzBuzz"
    | (0, _) -> String.from "Fizz"
    | (_, 0) -> String.from "Buzz"
    | _ -> n.to_string()
}

/// The words for 1 to `last`, separated by spaces.
pub fn line(last: u32) -> String {
    // A range binds tighter than `|>`:
    // `1..=last |> .map(word)` is `(1..=last).map(word)`.
    // The gotcha: `xs |> .join " "` is `" ".join(xs)`, since the
    // receiver of `.join` is its first argument; the method section
    // `.join(" ")` waits for its receiver (toy 048).
    1..=last |> .map(word) |> .collect::<Vec<_>>() |> .join(" ")
}

/// How many of 1 to `last` are Fizz, Buzz and FizzBuzz.
pub fn tally(last: u32) -> (u32, u32, u32) {
    let fizz = ref 0 in
    let buzz = ref 0 in
    let both = ref 0 in
    for n = 1 to last do
      // A `String` is matched against string literals (toy 077), and
      // `incr both` adds one to the `ref` (toy 089).
      match word n with
      | "FizzBuzz" -> incr both
      | "Fizz" -> incr fizz
      | "Buzz" -> incr buzz
      | _ -> ()
    done;
    (!fizz, !buzz, !both)
}
}

pub fn run() {
    println!("1 to 15: {}", line(15));
    let (fizz, buzz, both) = tally(100);
    println!("1 to 100: {fizz} Fizz, {buzz} Buzz, {both} FizzBuzz");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        assert_eq!(word(9), "Fizz");
        assert_eq!(word(10), "Buzz");
        assert_eq!(word(30), "FizzBuzz");
        assert_eq!(word(7), "7");
    }

    #[test]
    fn counting() {
        assert_eq!(
            line(15),
            "1 2 Fizz 4 Buzz Fizz 7 8 Fizz Buzz 11 Fizz 13 14 FizzBuzz"
        );
        assert_eq!(tally(100), (27, 14, 6));
    }
}

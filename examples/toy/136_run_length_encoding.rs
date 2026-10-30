// Toy 136: run-length encoding.
//
// What it is: a text as runs of one character: aaab is the runs (a, 3)
// and (b, 1), written 3a1b. Encoding, decoding, and reading the written
// form back.
//
// When to use it: an example of a loop that updates the last element of
// a list through a reference (`match runs.last_mut() with ..`, toys 068
// and 074), of `fun (&(c, n)) -> ..` taking pairs apart (toy 076), and of
// a small parser over characters.
//
// How to write it: `encode` and `parse` are loops over characters that
// either extend the current run or start a new one; `to_text` and
// `decode` are pipelines.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::rsml;

pub const SUMMARY: &str = "Run-length encoding: loops over chars, `last_mut`, `fun (&(c, n))`";

rsml! {
    /// The runs of `text`: each character and how many times it repeats.
    pub fn encode(text: &str) -> Vec<(char, usize)> {
        let mut runs : Vec<char * usize> = Vec.new () in
        for c in text.chars() do
          // `last_mut` lends the last run; `Some (last, n)` takes it apart
          // into two `&mut`s, and the guard reads `!last`.
          match runs.last_mut() with
          | Some (last, n) when !last = c -> *n <- *n + 1
          | _ -> runs.push((c, 1))
        done;
        runs
    }

    /// The runs written compactly, the count before its character.
    pub fn to_text(runs: &[(char, usize)]) -> String {
        runs |> .iter() |> .map(fun (c, n) -> format!("{}{}", n, c)) |> .collect()
    }

    /// The text again.
    pub fn decode(runs: &[(char, usize)]) -> String {
        // `fun (&(c, n)) -> ..` matches the reference to each pair, so
        // `c` and `n` are copies. The pattern needs its parentheses:
        // `fun &(c, n) -> c` is an error: "expected a parameter after `fun`"
        runs |> .iter() |> .flat_map(fun (&(c, n)) -> std::iter::repeat_n c n) |> .collect()
    }

    /// The runs of a compact text: digits make the count of the next
    /// character.
    pub fn parse(compact: &str) -> Vec<(char, usize)> {
        let mut runs = Vec.new () in
        let mut count = 0 in
        for c in compact.chars() do
          // `d as usize` binds tighter than `+` and `*` (toy 123).
          match c.to_digit(10) with
          | Some d -> count <- count * 10 + d as usize
          | None ->
              runs.push((c, count));
              count <- 0
        done;
        runs
    }
}

pub fn run() {
    let runs = encode("aaabccdddd");
    println!("encode \"aaabccdddd\" = {runs:?}");
    println!(
        "to_text = {:?}, decode = {:?}",
        to_text(&runs),
        decode(&runs)
    );
    println!("parse \"12x2y\" = {:?}", parse("12x2y"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let runs = encode("aaabccdddd");
        assert_eq!(runs, [('a', 3), ('b', 1), ('c', 2), ('d', 4)]);
        assert_eq!(to_text(&runs), "3a1b2c4d");
        assert_eq!(decode(&runs), "aaabccdddd");
        assert_eq!(parse(&to_text(&runs)), runs);
    }

    #[test]
    fn counts_of_more_digits() {
        assert_eq!(parse("12x2y"), [('x', 12), ('y', 2)]);
        assert!(encode("").is_empty());
    }
}

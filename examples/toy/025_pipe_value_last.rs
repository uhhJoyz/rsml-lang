// Toy 025: piping into Rust functions that take their data first.
//
// What it is: `|>` passes the value last, as OCaml's functions expect.
// Rust's functions and methods mostly take their data FIRST (`self`, then
// the rest). A placeholder `_` marks where the piped value goes:
// `x |> f _ 1` is `f(x, 1)`. (`f _ 1` is a function of `_`, applied to
// `x` on the spot, so it is a plain call, with no closure.)
//
// When to use it: to pipe into a method through its type
// (`str::starts_with`, `i64::clamp`, `Iterator::map`) or into any Rust
// function whose data parameter is not the last.
//
// How to write it: the step as a call with `_` in the data's place.
// * Without the `_`, `x |> i64::clamp 0 10` would be `i64::clamp(0, 10,
//   x)`: the bounds first, which is wrong.
// * Toy 046 pipes into methods with `.name` instead.
//
// Reference: [Pipelines: |> and @@](../../docs/notation.md#pipelines--and-)

use rsml::ml;

pub const SUMMARY: &str = "Data first: `x |> i64::clamp _ 0 10`, `s |> str::starts_with _ \"he\"`";

/// A volume kept between 0 and 10.
pub fn volume(level: i64) -> i64 {
    ml! {
        // `level |> i64::clamp _ 0 10` is `i64::clamp(level, 0, 10)`.
        level |> i64::clamp _ 0 10
    }
}

/// Whether a title starts with "The", spaces aside.
pub fn starts_with_the(title: &str) -> bool {
    ml! {
        // `title |> str::trim |> str::starts_with _ "The"` is
        // `str::starts_with(str::trim(title), "The")`.
        title |> str::trim |> str::starts_with _ "The"
    }
}

/// The length of the longest word, through `Iterator`'s functions.
pub fn longest(words: &[&str]) -> Option<usize> {
    ml! {
        // `it |> Iterator::map _ f` is `Iterator::map(it, f)`.
        words.iter() |> Iterator::map _ (fun w -> w.len()) |> Iterator::max
    }
}

pub fn run() {
    println!("volume 14 = {}, volume (-3) = {}", volume(14), volume(-3));
    println!(
        "starts_with_the \"  The End\" = {}",
        starts_with_the("  The End")
    );
    println!(
        "longest [\"to\"; \"be\"; \"or\"; \"not\"] = {:?}",
        longest(&["to", "be", "or", "not"])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_value_goes_where_the_placeholder_is() {
        assert_eq!(volume(14), 10);
        assert_eq!(volume(-3), 0);
        assert_eq!(volume(7), 7);
        assert!(starts_with_the("  The End"));
        assert!(!starts_with_the("An End"));
        assert_eq!(longest(&["to", "be", "or", "not"]), Some(3));
    }
}

// Toy 126: Rust's comments in rsml code, `//` and `/* .. */`.
//
// What it is: Rust's own comments work everywhere in rsml code: `// ..`
// runs to the end of the line, `/* .. */` goes anywhere and nests. Rust's
// lexer removes them before the macro runs, so they may hold anything,
// quotes and brackets included.
//
// When to use it: for most comments. Every editor colors them as
// comments, while ML comments, `(* .. *)` (toy 127), are colored as code
// except in Neovim and Vim with this repository's files.
//
// How to write it:
// * `std::cmp::max /* the larger */ a b` is `std::cmp::max(a, b)`.
// * `/* outer /* inner */ outer */` is one comment: they nest.
// * Doc comments, `///` and `/** */`, are attributes to Rust: they
//   document the items of an `rsml!`, but inside rsml code they are an
//   error: `/** note */ 1` is an error: "unexpected operator `#`".
//
// Reference: [Comments](../../docs/notation.md#comments)

use rsml::rsml;

pub const SUMMARY: &str = "`//` and `/* */` in rsml code; `///` documents `rsml!` items";

rsml! {
/// The words longer than `min` letters, in capitals. This `///`
/// documents the function, which is Rust: only its body is rsml code.
fn long_words(sentence: &str, min: usize) -> Vec<String> {
    sentence
    // A comment between the stages of a pipeline,
    |> .split_whitespace
    |> .filter _ (fun w -> w.len() > min) /* or after one, */
    |> .map _ str::to_uppercase
    |> .collect
}

/// The larger of two prices, in cents.
fn dearer(a: u32, b: u32) -> u32 {
    std::cmp::max /* the larger, "max" in Rust */ a b
}

/// A price with the tax added, the rate in percent.
fn with_tax(cents: u32, rate: u32) -> u32 {
    let tax = cents * rate / 100 in // rounded down
    cents + tax /* nested /* comments */ are fine */
}
}

pub fn run() {
    println!(
        "long_words \"a cup of green tea\" 3 = {:?}",
        long_words("a cup of green tea", 3)
    );
    println!("dearer 250 199 = {}", dearer(250, 199));
    println!("with_tax 1000 8 = {}", with_tax(1000, 8));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_change_nothing() {
        assert_eq!(long_words("a cup of green tea", 3), ["GREEN"]);
        assert_eq!(dearer(250, 199), 250);
        assert_eq!(with_tax(1000, 8), 1080);
        assert_eq!(with_tax(99, 8), 106);
    }
}

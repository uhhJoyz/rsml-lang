// Toy 017: `function` in pipelines, with or-patterns and guards.
//
// What it is: `function` is a closure of cases (toy 016), so it fits
// wherever a pipeline takes a function: in `.map(..)`, or after `|>`.
//
// When to use it: to classify the items of a pipeline.
//
// How to write it:
// * A case may list several patterns, `200 | 204 -> ..`, and have a
//   guard, `c when !c >= 500 -> ..`.
// * The items of `.iter()` are references, so a guard reads them with
//   `!c`, while literal patterns match through the reference.
// * `n |> function ..` applies the closure to `n`:
//   `n |> function 0 -> "zero" | _ -> "many"` is
//   `(|argument| match argument { 0 => "zero", _ => "many", })(n)`.
// * The last case reaches as far right as it can:
//   `n |> function 0 -> 1 | _ -> 2 |> g` is
//   `(|argument| match argument { 0 => 1, _ => g(2), })(n)`. In
//   parentheses, `(n |> function 0 -> 1 | _ -> 2) |> g`, `g` gets the
//   result.
//
// Reference: [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function)

use rsml::ml;

pub const SUMMARY: &str = "`.map(function 200 | 204 -> .. | c when .. -> ..)`, `n |> function`";

/// What each HTTP status code means.
pub fn meanings(codes: &[u16]) -> Vec<&'static str> { ml! {
    codes
    |> .iter()
    |> .map(function
       | 200 | 204 -> "ok"
       | 301 | 302 -> "moved"
       | c when !c >= 500 -> "server error"
       | _ -> "client error")
    |> .collect()
}}

/// How many: a value piped into `function`.
pub fn how_many(n: usize) -> &'static str { ml! {
    n |> function 0 -> "none" | 1 -> "one" | _ -> "many"
}}

/// The length of the word for `n`: parentheses end the `function`.
pub fn word_length(n: usize) -> usize { ml! {
    (n |> function 0 -> "none" | 1 -> "one" | _ -> "many") |> str::len
}}

pub fn run() {
    println!(
        "meanings [200; 302; 404; 503] = {:?}",
        meanings(&[200, 302, 404, 503])
    );
    println!("how_many 0 = {}, how_many 5 = {}", how_many(0), how_many(5));
    println!("word_length 5 = {}", word_length(5));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classified() {
        assert_eq!(
            meanings(&[200, 204, 302, 404, 503]),
            ["ok", "ok", "moved", "client error", "server error"]
        );
        assert_eq!(how_many(0), "none");
        assert_eq!(how_many(1), "one");
        assert_eq!(how_many(5), "many");
        assert_eq!(word_length(5), 4);
    }

    #[test]
    fn the_last_case_reaches_right() {
        let g = |n: i32| n * 10;
        assert_eq!(ml! { 0 |> function 0 -> 1 | _ -> 2 |> g }, 1);
        assert_eq!(ml! { 7 |> function 0 -> 1 | _ -> 2 |> g }, 20);
        assert_eq!(ml! { (0 |> function 0 -> 1 | _ -> 2) |> g }, 10);
    }
}

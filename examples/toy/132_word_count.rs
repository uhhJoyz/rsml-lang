// Toy 132: word count.
//
// What it is: how often each word occurs in a text, and the most frequent
// words.
//
// When to use it: as an example of an iterator pipeline (toy 046) feeding
// a `for .. in` loop (toy 083) that counts through a reference into a map
// (`*c <- *c + 1`, toy 091), and of sorting with a two-parameter `fun`
// (toy 012).
//
// How to write it: clean the words in one pipeline, count them in one
// loop, sort the counts with `.sort_by`, called ML style with the list
// first (toy 045).
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name)

use rsml::rsml;
use std::collections::HashMap;

pub const SUMMARY: &str = "Word count: a cleaning pipeline, a counting loop, a sort";

rsml! {
/// The words of `text`, in lowercase, without punctuation around them.
fn words(text: &str) -> Vec<String> {
    // `trim_matches` takes a function of a `char`: its parameter is
    // annotated, as Rust cannot infer it (toy 013).
    text
    |> .split_whitespace()
    |> .map(fun w -> w.trim_matches(fun (c : char) -> not c.is_alphanumeric()))
    |> .filter(fun w -> not (w.is_empty()))
    |> .map(str::to_lowercase)
    |> .collect()
}

/// How many times each word occurs.
pub fn counts(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap.new () in
    for w in words text do
      // `or_insert` lends the count; `*c <- *c + 1` writes through it.
      let c = counts.entry(w).or_insert(0) in
      *c <- *c + 1
    done;
    counts
}

/// The `n` most frequent words, most frequent first, ties in
/// alphabetical order.
pub fn top(text: &str, n: usize) -> Vec<(String, usize)> {
    let mut pairs = counts text |> .into_iter() |> .collect::<Vec<_>>() in
    // `.sort_by pairs f` is `pairs.sort_by(f)`: the receiver comes
    // first. Comparing `b.1` with `a.1` puts the larger counts first.
    .sort_by pairs (fun a b -> b.1.cmp(&a.1).then_with(fun () -> a.0.cmp(&b.0)));
    pairs |> .into_iter() |> .take(n) |> .collect()
}
}

const TEXT: &str = "The cat and the hat. The cat sat!";

pub fn run() {
    println!("text: {TEXT:?}");
    println!("{} different words", counts(TEXT).len());
    println!("top 3: {:?}", top(TEXT, 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleaned_words() {
        assert_eq!(
            words(TEXT),
            ["the", "cat", "and", "the", "hat", "the", "cat", "sat"]
        );
        assert!(words(" -- ").is_empty());
    }

    #[test]
    fn counted_and_sorted() {
        assert_eq!(counts(TEXT).len(), 5);
        assert_eq!(counts(TEXT)["the"], 3);
        let top3 = top(TEXT, 3);
        let top3: Vec<(&str, usize)> = top3.iter().map(|(w, n)| (w.as_str(), *n)).collect();
        assert_eq!(top3, [("the", 3), ("cat", 2), ("and", 1)]);
    }
}

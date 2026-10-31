// Toy 125: a whole file with its tests, and a `#[rust]` function.
//
// What it is: a module in one flush-left `rsml!` (toy 124) can hold its
// own `#[cfg(test)] mod tests`. The tests are functions of the same
// `rsml!`, so their bodies are rsml code, and so are the arguments of
// `assert_eq!` and the other std macros (toy 106). `#[rust]` on an item
// keeps its bodies Rust (toy 118).
//
// When to use it: when a module's tests should read like its code.
//
// How to write it:
// * `mod tests { use super::*; .. }` inside the `rsml!`: `use super::*`
//   also brings the arities of the parent's functions (toy 116).
// * `#[rust]` before a function whose body reads better as Rust; it may
//   stand with `#[test]` or any other attribute.
//
// Reference: [Macros](../../docs/notation.md#macros),
// [Whole files](../../README.md#whole-files)

rsml::rsml! {

pub const SUMMARY: &str = "A whole file whose tests are rsml code, with a `#[rust]` function";

/// The letter grade of a score out of 100.
pub fn grade(score: u32) -> char {
    if score >= 90 then 'A' else if score >= 80 then 'B' else if score >= 70 then 'C' else 'F'
}

/// The average of the scores, rounded down; 0 for none.
pub fn average(scores: &[u32]) -> u32 {
    if scores.is_empty() then 0 else (scores |> .iter() |> .sum::<u32>()) / scores.len() as u32
}

/// A hash of the names, kept as Rust: a fold with two closures reads
/// well there.
#[rust]
pub fn roster_hash(names: &[&str]) -> u32 {
    names
        .iter()
        .flat_map(|n| n.bytes())
        .fold(7, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)))
}

pub fn run() {
    let scores = [|95; 85; 71|] in
    println!("grades = {:?}", scores |> .iter() |> .map(fun s -> grade !s) |> .collect::<String>());
    println!("average = {}", average &scores);
    println!(
      "roster_hash [ab] <> roster_hash [ba]: {}",
      roster_hash &[|"ab"|] <> roster_hash &[|"ba"|],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades() {
        assert_eq!(grade 95, 'A');
        assert_eq!([|95; 85; 71; 3|] |> .map(grade), [|'A'; 'B'; 'C'; 'F'|])
    }

    #[test]
    fn averages() {
        assert_eq!(average &[|95; 85; 71|], 83);
        assert_eq!(average &[||], 0)
    }

    /// A test in Rust, among the rsml ones.
    #[rust]
    #[test]
    fn the_hash_sees_the_order() {
        assert_ne!(roster_hash(&["ab"]), roster_hash(&["ba"]));
        assert_eq!(roster_hash(&[]), 7);
    }
}

}

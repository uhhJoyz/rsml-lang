// Toy 032: composing methods and partial applications.
//
// What it is: any function can be a step of a composition: a method as a
// function, `.trim` (toy 045), a method section, `.split(',')` (toy 048),
// or a partial application, `add 1` (toy 018).
//
// When to use it: to build the function a `map` or a `filter` needs out
// of methods, without a `fun`.
//
// How to write it:
// * `.trim >> .len` is `move |argument| argument.trim().len()`: one
//   receiver, one method chain.
// * A method needs its receiver's type, as in Rust. Passed to `map`, the
//   composition gets it from the items; named with `let` and used
//   later, it may need an annotation.
// * Given `let add x y = x + y`, `add 1 >> add 2` is
//   `fun n -> add 2 (add 1 n)`.
//
// Reference: [Composition: >> and <<](../../docs/notation.md#composition--and-)

use rsml::ml;

pub const SUMMARY: &str = "`.trim >> .len`, `.split(',') >> .count()`, `add 1 >> add 2`";

/// The length of each name, without the spaces around it.
pub fn name_lengths(names: &[&str]) -> Vec<usize> {
    ml! {
        // `.trim >> .len` is `move |argument| argument.trim().len()`.
        names |> .iter() |> .map(.trim >> .len) |> .collect()
    }
}

/// How many fields each CSV line has.
pub fn field_counts(lines: &[&str]) -> Vec<usize> {
    ml! {
        // `.split(',') >> .count()` is
        // `move |argument| argument.split(',').count()`.
        lines |> .iter() |> .map(.split(',') >> .count()) |> .collect()
    }
}

/// Two partial applications of a local function, composed.
pub fn plus_three(xs: &[i64]) -> Vec<i64> {
    ml! {
        let add x y = x + y in
        xs |> .iter() |> .copied() |> .map(add 1 >> add 2) |> .collect()
    }
}

pub fn run() {
    println!(
        "name_lengths [\" Ada \"; \"Grace\"] = {:?}",
        name_lengths(&[" Ada ", "Grace"])
    );
    println!(
        "field_counts [\"a,b,c\"; \"d\"] = {:?}",
        field_counts(&["a,b,c", "d"])
    );
    println!("plus_three [1; 10] = {:?}", plus_three(&[1, 10]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods_composed() {
        assert_eq!(name_lengths(&[" Ada ", "Grace"]), [3, 5]);
        assert_eq!(field_counts(&["a,b,c", "d"]), [3, 1]);
    }

    #[test]
    fn partial_applications_composed() {
        assert_eq!(plus_three(&[1, 10]), [4, 13]);
    }

    #[test]
    fn a_named_composition_with_its_type() {
        // Named and used later, the composition is given its type.
        let count: fn(&Vec<i64>) -> usize = ml! { .iter >> .count };
        assert_eq!(count(&vec![4, 5, 6]), 3);
    }
}

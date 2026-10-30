// Toy 107: `vec![e; n]` and `vec![a, b]` with OCaml inside.
//
// What it is: `vec!` takes OCaml expressions, written as in Rust:
// `vec![e; n]` makes `n` copies of `e`, and `vec![a, b]` lists items with
// commas.
//
// When to use it: `vec![0; n]` for a zeroed buffer of a computed size,
// which a list literal cannot say. For a list of items, OCaml's `[a; b]`
// (toy 096) builds the same `Vec`.
//
// How to write it:
// * `vec![0; sides + 1]` stays `vec![0; sides + 1]`: inside `vec!` the
//   `;` repeats, as in Rust.
// * `vec![face 1, face 6]` is `vec![face(1), face(6)]`: inside `vec!` items
//   are separated by `,`. In a list literal a `,` would make a tuple, one
//   item, so `[face 1, face 6]` is an error: "list items are separated by
//   `;`: write `[1; 2]`, or `[(1, 2)]` for a list of one pair" (toy 130).
// * Each item, and the count, ends at a `,` or `;` outside brackets.
// * Not to be confused with OCaml's list, where `;` separates items:
//   `[0; n]` is `::std::vec![0, n]`, the two items `0` and `n`.
//
// Reference: [Macros](../../docs/notation.md#macros)

use rsml::rsml;

pub const SUMMARY: &str = "`vec![0; n + 1]` repeats, `vec![f 1, g 2]` lists";

rsml! {
    /// How many rolls gave each face of a die with `sides` faces (index
    /// 0 is face 1).
    pub fn tally(rolls: &[usize], sides: usize) -> Vec<u32> {
        // `sides` zeros: the count is OCaml too, `vec![0; sides]`.
        let mut counts = vec![0; sides] in
        for r in rolls do
          let i = !r - 1 in
          counts[i] <- counts[i] + 1
        done;
        counts
    }

    /// The name of a face.
    fn face(n: usize) -> String {
        format!("face {n}")
    }

    /// The names of the lowest and highest faces.
    pub fn extremes(sides: usize) -> Vec<String> {
        // `vec![face 1, face sides]` is `vec![face(1), face(sides)]`.
        vec![face 1, face sides]
    }

    /// A row of `width` cells, each `fill` repeated twice.
    pub fn row(fill: char, width: usize) -> Vec<String> {
        // The item is any expression, here a pipeline:
        // `String.from fill |> .repeat _ 2` is `String::from(fill).repeat(2)`.
        vec![String.from fill |> .repeat _ 2; width]
    }
}

pub fn run() {
    println!("tally [1; 3; 3; 6] 6 = {:?}", tally(&[1, 3, 3, 6], 6));
    println!("extremes 6 = {:?}", extremes(6));
    println!("row '#' 3 = {:?}", row('#', 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated() {
        assert_eq!(tally(&[1, 3, 3, 6], 6), [1, 0, 2, 0, 0, 1]);
        assert_eq!(row('#', 3), ["##", "##", "##"]);
    }

    #[test]
    fn listed() {
        assert_eq!(extremes(6), ["face 1", "face 6"]);
    }

    #[test]
    fn a_list_literal_is_not_a_repeat() {
        let n = 3;
        assert_eq!(rsml::ml! { [0; n] }, [0, 3]);
    }
}

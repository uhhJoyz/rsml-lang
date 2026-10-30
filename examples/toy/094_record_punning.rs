// Toy 094: field punning, `Size { width; height }`.
//
// What it is: a field written alone stands for `field = field`, in an
// expression (the value of the variable of that name) and in a pattern
// (a variable bound to the field), as in OCaml and in Rust.
//
// When to use it: to build a struct from variables of the same names, and
// to take a struct apart.
//
// How to write it:
// * Punned and `field = e` fields mix: `Size { width = height; height }`.
// * A pattern may leave fields out; `; _` at the end says so, as OCaml
//   asks, and Rust's `..` is added either way.
// * The struct's name is needed, in patterns too.
//
// Reference: [Records and collections](../../docs/notation.md#records-and-collections)

use rsml::ml;

pub const SUMMARY: &str = "Punning: `Size { width; height }` in expressions and patterns";

#[derive(Debug, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// A size from two numbers named like its fields.
pub fn size(width: u32, height: u32) -> Size {
    ml! {
        // `Size { width; height }` is `Size { width, height }`.
        Size { width; height }
    }
}

/// The size turned a quarter: punned and explicit fields together.
pub fn turned(s: Size) -> Size {
    ml! {
        // `let Size { width; height } = s in width` is
        // `{ let Size { width, height, .. } = s; width }`.
        let Size { width; height } = s in
        Size { width = height; height = width }
    }
}

/// What a size looks like, with record patterns.
pub fn shape(s: &Size) -> &'static str {
    ml! {
        match s with
        // `Size { width = 0; _ }` matches when the field `width` is 0.
        | Size { width = 0; _ } | Size { height = 0; _ } -> "flat"
        | Size { width; height } when width = height -> "square"
        | _ -> "oblong"
    }
}

pub fn run() {
    println!("size 3 2 = {:?}", size(3, 2));
    println!("turned (size 3 2) = {:?}", turned(size(3, 2)));
    for s in [size(0, 5), size(4, 4), size(3, 2)] {
        println!("shape {s:?} = {}", shape(&s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punned_fields() {
        assert_eq!(
            size(3, 2),
            Size {
                width: 3,
                height: 2
            }
        );
        assert_eq!(
            turned(size(3, 2)),
            Size {
                width: 2,
                height: 3
            }
        );
    }

    #[test]
    fn punned_patterns() {
        assert_eq!(shape(&size(0, 5)), "flat");
        assert_eq!(shape(&size(5, 0)), "flat");
        assert_eq!(shape(&size(4, 4)), "square");
        assert_eq!(shape(&size(3, 2)), "oblong");
    }
}

// Toy 112: generic functions and `where` clauses in `rsml!`.
//
// What it is: an `rsml!` function's signature is Rust, so it can have
// generic parameters with bounds (`<T: Display>`) and `where` clauses; the
// body is OCaml and uses them as Rust code would.
//
// When to use it: whenever a function works for many types. OCaml's own
// type variables (`'a` in a `let rec`, toy 101) become generic parameters
// too, but they cannot have bounds: a bound needs a Rust signature.
//
// How to write it:
// * Bounds in the signature: `fn joined<T: Display>(items: &[T], ..)`.
// * Or a `where` clause after the return type, as in Rust.
// * The body calls the bound's methods as usual: `x.to_string()`.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "Generic `rsml!` functions: `<T: Display>` and `where` clauses";

rsml! {
    use std::fmt::Display;

    /// Items of any `Display` type, joined into one line.
    pub fn joined<T: Display>(items: &[T], sep: &str) -> String {
        items |> .iter() |> .map(fun x -> x.to_string()) |> .collect::<Vec<_>>() |> .join(sep)
    }

    /// The largest item, for any type that can be compared and copied.
    pub fn largest<T>(items: &[T]) -> Option<T>
    where
        T: PartialOrd + Copy,
    {
        // `fun best x -> ..` gets its types from `fold`: `Option<T>`, `T`.
        items
        |> .iter()
        |> .copied()
        |> .fold(None, fun best x -> match best with Some b when b >= x -> Some b | _ -> Some x)
    }

    /// The sum of anything that iterates over `i64`s.
    pub fn total<I>(numbers: I) -> i64
    where
        I: IntoIterator<Item = i64>,
    {
        numbers |> .into_iter |> .sum
    }
}

pub fn run() {
    println!("joined [|3; 1; 2|] \", \" = {:?}", joined(&[3, 1, 2], ", "));
    println!("joined [|'a'; 'b'|] \"-\" = {:?}", joined(&['a', 'b'], "-"));
    println!(
        "largest [|2.5; 9.0; 4.0|] = {:?}",
        largest(&[2.5, 9.0, 4.0])
    );
    println!("largest [| |] = {:?}", largest::<u8>(&[]));
    println!("total (1..=4) = {}", total(1..=4));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn many_types() {
        assert_eq!(joined(&[3, 1, 2], ", "), "3, 1, 2");
        assert_eq!(joined(&['a', 'b'], "-"), "a-b");
        assert_eq!(largest(&[2.5, 9.0, 4.0]), Some(9.0));
        assert_eq!(largest(&["pear", "apple"]), Some("pear"));
        assert_eq!(largest::<u8>(&[]), None);
        assert_eq!(total(1..=4), 10);
        assert_eq!(total(vec![5, -5]), 0);
    }
}

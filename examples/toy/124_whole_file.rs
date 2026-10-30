// Toy 124: a whole module in one flush-left `rsml::rsml! { .. }`.
//
// What it is: one `rsml!` can hold every item of a file: `use`s, types,
// `impl`s, constants and functions. The items are Rust, the function
// bodies OCaml. Written flush left, with the items not indented, the file
// reads as an ordinary module, and rustfmt and rsml-fmt keep it so.
//
// When to use it: for a module that is mostly OCaml-bodied functions.
// rust-analyzer expands the whole `rsml!` on every edit, so split a file
// of more than about 1500 lines into several (README, "Whole files").
//
// How to write it:
// * The first line is `rsml::rsml! {`, the last `}`: the macro's full path
//   needs no `use`.
// * Comments like this header, a file's `//!` docs and its `#![..]`
//   attributes stay outside: a macro call cannot hold them.
// * The functions of the block know each other's arity, as `let`s do
//   (toy 114), and `Book::new` is known in the whole module (toy 115).
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole files](../../README.md#whole-files)

rsml::rsml! {

use std::fmt;

pub const SUMMARY: &str = "A whole module in one flush-left `rsml::rsml! { .. }`";

/// A book on a shelf.
#[derive(Debug, Clone, PartialEq)]
pub struct Book {
    pub title: String,
    pub pages: u32,
}

impl Book {
    /// A record with OCaml fields: `Book { title = ..; pages }`.
    pub fn new(title: &str, pages: u32) -> Book {
        Book { title = String.from title; pages }
    }

    pub fn is_long(&self) -> bool {
        self.pages > 300
    }
}

impl fmt::Display for Book {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} pages)", self.title, self.pages)
    }
}

/// Three books. `Book::new "Dune"` alone would be a partial application.
pub fn shelf() -> Vec<Book> {
    [Book::new "Dune" 412; Book::new "Momo" 304; Book::new "Matilda" 240]
}

pub fn total_pages(books: &[Book]) -> u32 {
    books |> .iter() |> .map(fun b -> b.pages) |> .sum()
}

pub fn long_titles(books: &[Book]) -> Vec<&str> {
    books
    |> .iter()
    |> .filter(fun b -> b.is_long())
    |> .map(fun b -> b.title.as_str())
    |> .collect()
}

pub fn run() {
    let books = shelf () in
    for b in &books do println!("{}", b) done;
    println!("total_pages = {}", total_pages &books);
    println!("long_titles = {:?}", long_titles &books)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shelf() {
        let books = shelf () in
        assert_eq!(total_pages &books, 956);
        assert_eq!(long_titles &books, ["Dune"; "Momo"]);
        assert_eq!(books[2].to_string(), "Matilda (240 pages)")
    }

    #[test]
    fn a_partial_application_of_book_new() {
        let dune = Book::new "Dune" in
        assert_eq!(dune 412, Book { title = String.from "Dune"; pages = 412 })
    }
}

}

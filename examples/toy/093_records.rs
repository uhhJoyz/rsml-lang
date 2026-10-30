// Toy 093: building records, `Item { name = e; price = e }`.
//
// What it is: a Rust struct value, written with OCaml's `=` and `;`
// between the fields. The struct is defined in Rust as usual; only the
// expression that builds it is OCaml.
//
// When to use it: to make a struct value inside OCaml code.
//
// How to write it:
// * The struct's name comes first, as in Rust: OCaml's bare
//   `{ x = 1 }` has no type to build here.
// * `field = e` and `;` between fields (a last `;` is allowed). Each
//   value is an OCaml expression, so `price = cents 450` needs no
//   parentheses.
// * Fields are read with `.`: `item.price`.
//
// Reference: [Records and collections](../../docs/notation.md#records-and-collections)

use rsml::ml;

pub const SUMMARY: &str = "Records: `Item { name = \"tea\"; price = 4 }`, `item.price`";

#[derive(Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub price: u32,
}

fn cents(c: u32) -> u32 {
    c
}

/// A menu item.
pub fn tea() -> Item {
    ml! {
        // `Item { name = String.from "tea"; price = 4 }` is
        // `Item { name: String::from("tea"), price: 4 }`.
        Item { name = String.from "tea"; price = 4 }
    }
}

/// The fields may come in any order, each value is OCaml.
pub fn cake() -> Item {
    ml! {
        // `Item { price = cents 450; name = s }` is
        // `Item { price: cents(450), name: s }`.
        let s = "cake".to_string() in
        Item { price = cents 450; name = s }
    }
}

/// The price of two: fields are read with `.`.
pub fn two_of(item: &Item) -> u32 {
    ml! { item.price * 2 }
}

// `{ x = 1 }` is an error: "record expressions need the struct name in
// Rust: write `Point { x = 1; y = 2 }`"

pub fn run() {
    println!("tea () = {:?}", tea());
    println!("cake () = {:?}", cake());
    println!("two_of (tea ()) = {}", two_of(&tea()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_and_read() {
        assert_eq!(
            tea(),
            Item {
                name: "tea".to_string(),
                price: 4
            }
        );
        assert_eq!(
            cake(),
            Item {
                name: "cake".to_string(),
                price: 450
            }
        );
        assert_eq!(two_of(&tea()), 8);
    }
}

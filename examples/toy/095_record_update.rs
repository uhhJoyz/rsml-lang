// Toy 095: functional update, `{ p with field = e }`.
//
// What it is: a copy of a record with some fields changed, Rust's
// `Item { price: 5, ..p }`. The old record is moved into the new one, so
// for a struct that is not `Copy` it cannot be used afterwards.
//
// When to use it: to derive a value from another one (settings from the
// defaults, the next state from the current one).
//
// How to write it:
// * `{ base with f = e; g = e }`: the base is any expression, the fields
//   are separated by `;`, and the struct name is not written.
// * A field alone takes the variable of its name: `{ standard () with
//   price }` sets `price` to `price`.
// * The new values are computed first, before the base is moved: they may
//   read the old record, so `{ s with width = s.height; height = s.width }`
//   swaps the two fields.
//
// Reference: [Records and collections](../../docs/notation.md#records-and-collections)

use rsml::ml;

pub const SUMMARY: &str = "Update: `{ p with price = 5 }`; new values read the old record";

#[derive(Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub price: u32,
}

fn standard() -> Item {
    Item {
        name: "coffee".to_string(),
        price: 3,
    }
}

/// The standard item at another price.
pub fn priced(price: u32) -> Item {
    ml! {
        // `{ p with price = 5 }` is `{ let mut record = p; record.price = 5; record }`.
        { standard () with price }
    }
}

/// The large size: both new values read the old item before it moves.
pub fn large(item: Item) -> Item {
    ml! {
        // `format!` only borrows `item.name`; moving it out with
        // `item.name + ".."` would leave `item` partly moved, and rustc
        // would refuse to move the rest into the new record.
        { item with name = format!("{} (large)", item.name); price = 2 * item.price }
    }
}

#[derive(Debug, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// Width and height swapped, in one update.
pub fn turned(s: Size) -> Size {
    ml! {
        // `{ s with width = s.height; height = s.width }` is
        // `{ let value1 = s.height; let value2 = s.width; let mut record = s;
        // record.width = value1; record.height = value2; record }`.
        { s with width = s.height; height = s.width }
    }
}

pub fn run() {
    println!("priced 4 = {:?}", priced(4));
    println!("large (priced 4) = {:?}", large(priced(4)));
    println!(
        "turned 3x2 = {:?}",
        turned(Size {
            width: 3,
            height: 2
        })
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updated_copies() {
        assert_eq!(
            priced(4),
            Item {
                name: "coffee".to_string(),
                price: 4
            }
        );
        assert_eq!(
            large(priced(4)),
            Item {
                name: "coffee (large)".to_string(),
                price: 8
            }
        );
        assert_eq!(
            turned(Size {
                width: 3,
                height: 2
            }),
            Size {
                width: 2,
                height: 3
            }
        );
    }
}

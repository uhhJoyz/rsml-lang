// Toy 045: methods as functions, `.len s`, `.push cart item`.
//
// What it is: where an expression starts, `.name` is the method `name` as
// a function of its receiver and arguments, written OCaml style:
// `.push cart item` is `cart.push(item)`. The first argument is the
// receiver, every other one goes to the method.
//
// When to use it: in pipelines (toy 046), and wherever the method should
// read like the other functions of an OCaml expression.
//
// How to write it: `.name receiver arg1 arg2 ..`.
// * The receiver alone is a call without arguments: `.len s` is
//   `s.len()`.
// * The name follows the dot directly: `. len s` is an error: "no space
//   is allowed between `.` and the method name" (toy 051).
// * Only where an expression starts: after a value, `.name` is that
//   value's field or method (`f .len` is `f.len`), so as an argument a
//   method goes in parentheses, `(.len)` (toy 047).
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name)

use rsml::ml;

pub const SUMMARY: &str = "`.len s`, `.push cart item`: the receiver comes first";

/// A cart with one more item, and its size.
pub fn add_item(mut cart: Vec<String>, item: &str) -> (Vec<String>, usize) {
    ml! {
        // `.push cart (String.from item)` is `cart.push(String::from(item))`.
        .push cart (String.from item);
        // `.len cart` is `cart.len()`.
        let size = .len cart in
        (cart, size)
    }
}

/// Whether the cart holds an item.
pub fn has(cart: &[String], item: &str) -> bool {
    ml! {
        // `.contains cart &item.to_string()` is
        // `cart.contains(&item.to_string())`.
        .contains cart &item.to_string()
    }
}

/// The item at a position, if any.
pub fn item_at(cart: &[String], i: usize) -> Option<&String> {
    ml! {
        // `.get cart i` is `cart.get(i)`.
        .get cart i
    }
}

pub fn run() {
    let (cart, size) = add_item(vec!["tea".to_string()], "milk");
    println!("add_item [\"tea\"] \"milk\" = ({cart:?}, {size})");
    println!(
        "has cart \"milk\" = {}, has cart \"jam\" = {}",
        has(&cart, "milk"),
        has(&cart, "jam")
    );
    println!(
        "item_at cart 0 = {:?}, item_at cart 5 = {:?}",
        item_at(&cart, 0),
        item_at(&cart, 5)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods_with_the_receiver_first() {
        let (cart, size) = add_item(vec!["tea".to_string()], "milk");
        assert_eq!(cart, ["tea", "milk"]);
        assert_eq!(size, 2);
        assert!(has(&cart, "milk"));
        assert!(!has(&cart, "jam"));
        assert_eq!(item_at(&cart, 0).map(String::as_str), Some("tea"));
        assert_eq!(item_at(&cart, 5), None);
    }
}

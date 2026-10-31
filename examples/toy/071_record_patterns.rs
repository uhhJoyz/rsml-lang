// Toy 071: record patterns, `Order { item; qty = 0; _ }`.
//
// What it is: a pattern for a struct (or a struct-like variant) that
// names its fields, ML style: `field` binds the field to a variable
// of the same name, `field = p` matches it against the pattern `p`.
//
// When to use it: to take a struct apart in `match`, `let` or `fun`, and
// to match on the value of some of its fields.
//
// How to write it:
// * The struct name comes first, as Rust needs it:
//   `match o with { qty; _ } -> qty` is an error: "record patterns need
//   the struct name in Rust: write `Point { x; y }`".
// * Fields are separated by `;`, and may be left out: the pattern always
//   ends with Rust's `..`. A `; _` at the end (for "the other fields") is
//   accepted and means the same.
// * `qty = 0` matches, `qty = n` renames: `=`, where Rust writes `:`.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str = "Record patterns: `Order { item; qty = 0; _ }`, `let Order { .. } = o`";

pub struct Order {
    pub item: &'static str,
    pub qty: u32,
    pub price: u32,
}

/// A line of a receipt.
pub fn line(o: &Order) -> String { ml! {
    // `match o with Order { item; qty = 0; _ } -> item` is
    // `match o { Order { item, qty: 0, .. } => item, }`.
    match o with
    | Order { item; qty = 0; _ } -> format!("{item}: sold out")
    | Order { item; qty = n; price } -> format!("{item}: {n} x {price}")
}}

/// The value of an order: `let` takes the record apart.
pub fn value(o: &Order) -> u32 { ml! {
    // `let Order { qty; price } = o in qty * price` is
    // `{ let Order { qty, price, .. } = o; qty * price }`.
    let Order { qty; price } = o in
    qty * price
}}

/// The items of the orders: a `fun` whose parameter is a record pattern.
pub fn items(orders: &[Order]) -> Vec<&'static str> { ml! {
    // The pattern is in parentheses, as every `fun` parameter that is
    // more than a name.
    orders |> .iter() |> .map(fun (Order { item; _ }) -> !item) |> .collect()
}}

pub fn run() {
    let orders = [
        Order {
            item: "tea",
            qty: 2,
            price: 3,
        },
        Order {
            item: "jam",
            qty: 0,
            price: 5,
        },
    ];
    for o in &orders {
        println!("line = {:?}, value = {}", line(o), value(o));
    }
    println!("items = {:?}", items(&orders));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_by_name() {
        let tea = Order {
            item: "tea",
            qty: 2,
            price: 3,
        };
        let jam = Order {
            item: "jam",
            qty: 0,
            price: 5,
        };
        assert_eq!(line(&tea), "tea: 2 x 3");
        assert_eq!(line(&jam), "jam: sold out");
        assert_eq!(value(&tea), 6);
        assert_eq!(items(&[tea, jam]), ["tea", "jam"]);
    }
}

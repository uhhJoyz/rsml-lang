// Toy 104: `println!` with rsml arguments.
//
// What it is: the arguments of std's printing macros, `print!`,
// `println!`, `eprint!` and `eprintln!`, are rsml expressions. The format
// string stays as written; each argument after it, up to a `,`, is rsml
// code.
//
// When to use it: to print from rsml code, with pipelines, applications
// and comparisons as arguments.
//
// How to write it:
// * Rust's macro call: `println!("{} items", items |> .len)`.
// * An argument ends at a `,` outside brackets, so `f a b` and `i + 1`
//   need no parentheses there.
// * `println!("{total}")` names a variable in the format string, as in
//   Rust, and `println!()` prints an empty line.
// * A placeholder reaches out of the macro: `println!("- {}", _)` is a
//   function that prints its argument.
//
// Reference: [Macros](../../docs/notation.md#macros)

use rsml::rsml;

pub const SUMMARY: &str = "`println!(\"{} items\", items |> .len)`: macro arguments are rsml";

rsml! {
/// Prints a shopping list: a count, then the numbered items on a line.
pub fn print_list(items: &[&str]) {
    // `println!("{} items:", items |> .len)` is
    // `println!("{} items:", items.len())`.
    println!("{} items:", items |> .len);
    for (i, item) in items.iter().enumerate() do
      // `print!("{}. {} ", i + 1, item)` stays `print!("{}. {} ", i + 1, item)`.
      print!("{}. {} ", i + 1, item)
    done;
    println!()
}

/// Prints each item on its own line, through a placeholder function.
pub fn print_each(items: &[&str]) {
    // `.for_each(println!("- {}", _))` is
    // `.for_each(fun item -> println!("- {}", item))`.
    items |> .iter() |> .for_each(println!("- {}", _))
}

/// The total in cents; a warning goes to stderr when it is over a limit.
pub fn checked_total(prices: &[u32], limit: u32) -> u32 {
    let total = prices |> .iter |> .sum::<u32> in
    // `eprintln!` takes rsml code too: `total - limit` is an argument.
    if total > limit then eprintln!("total {total} is over {limit} by {}", total - limit);
    total
}
}

pub fn run() {
    print_list(&["eggs", "milk", "bread"]);
    print_each(&["eggs", "milk"]);
    println!(
        "checked_total [250; 400] 500 = {}",
        checked_total(&[250, 400], 500)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsml::ml;

    #[test]
    fn printing_returns_unit() {
        print_list(&["eggs"]);
        print_each(&[]);
    }

    #[test]
    fn arguments_are_ocaml() {
        let items = ["eggs", "milk", "bread"];
        assert_eq!(ml! { items |> .len }, 3);
        assert_eq!(checked_total(&[250, 400], 500), 650);
        assert_eq!(checked_total(&[100], 500), 100);
    }
}

// Toy 037: the rules of user-defined operators.
//
// What it is: a few things to know once you define operators (toy 036):
// how they group, which ones cannot exist, and that they may keep state.
//
// When to use it: when a chain of your operators must read the way it
// computes, as `cli <|> env |? 8080` below.
//
// How to write it:
// * The operators you define that start with `<`, `>`, `|`, `&`, `=` or
//   `$` bind like comparisons and group to the left.
// * An operator starting with `!` cannot be defined: it would be a prefix
//   operator, and `!` before an expression is a dereference.
//   `let ( !+ ) a b = a + b in 1` is an error: "`!+` cannot be an
//   operator: one starting with `!` is a prefix operator, and here `!`
//   before an expression is a dereference (`!r` is `*r`), so `!+` could
//   never be used; define a function instead"
// * `!=` is infix, so it can be defined; then it is yours:
//   Given `let ( != ) a b = a - b`, `5 != 3` is `( != ) 5 3`.
// * An operator may change what it captures (Rust's `FnMut`): rsml binds
//   every operator with `let mut`, so nothing more is needed.
//
// Reference: [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators)

use rsml::ml;

pub const SUMMARY: &str = "Operator rules: grouping, no `!` operators, operators with state";

/// The port to use: from the command line, else the environment, else 8080.
pub fn port(cli: Option<u16>, env: Option<u16>) -> u16 { ml! {
    // `<|>` keeps the first `Some`, `|?` unwraps with a default.
    let ( <|> ) a b = Option::or a b in
    let ( |? ) o default = Option::unwrap_or o default in
    // Both group to the left. Given `let ( <|> ) a b = Option::or a b
    // let ( |? ) o d = Option::unwrap_or o d`, `cli <|> env |? 8080`
    // is `(cli <|> env) |? 8080`.
    cli <|> env |? 8080
}}

/// Where `name` is in `names`, and how many comparisons it took.
pub fn find(names: &[&str], name: &str) -> (Option<usize>, u32) { ml! {
    let mut comparisons = 0 in
    // This operator counts its uses in a local it captures.
    let ( =? ) a b =
      comparisons <- comparisons + 1;
      a = b
    in
    // `position` stops at the first match.
    let at = names |> .iter() |> .position(fun n -> !n =? name) in
    (at, comparisons)
}}

/// The difference of two numbers, through a `!=` defined as subtraction.
pub fn redefined(a: i64, b: i64) -> i64 { ml! {
    let ( != ) a b = a - b in
    a != b
}}

pub fn run() {
    println!("port None (Some 3000) = {}", port(None, Some(3000)));
    println!("port None None = {}", port(None, None));
    let names = ["ada", "alan", "grace", "linus", "margaret"];
    println!("find {names:?} \"linus\" = {:?}", find(&names, "linus"));
    println!("find {names:?} \"bob\" = {:?}", find(&names, "bob"));
    println!("with ( != ) defined as ( - ): 5 != 3 = {}", redefined(5, 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping_to_the_left() {
        assert_eq!(port(Some(1), Some(2)), 1);
        assert_eq!(port(None, Some(3000)), 3000);
        assert_eq!(port(None, None), 8080);
    }

    #[test]
    fn an_operator_with_state() {
        let names = ["ada", "alan", "grace", "linus", "margaret"];
        assert_eq!(find(&names, "linus"), (Some(3), 4));
        assert_eq!(find(&names, "bob"), (None, 5));
    }

    #[test]
    fn a_defined_not_equal() {
        assert_eq!(redefined(5, 3), 2);
    }
}

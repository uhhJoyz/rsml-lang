// Toy 079: sequences, `a; b`.
//
// What it is: `a; b` runs `a`, then `b`, and its value is `b`'s: it is a
// Rust block, `a; b` is `{ a; b }`. The steps before the last are run for
// what they do, their values dropped.
//
// When to use it: for side effects in order: pushing, printing, updating
// a variable, then giving the result.
//
// How to write it:
// * `;` separates steps; the last one is the value: `f x; g y; z` is
//   `{ f(x); g(y); z }`.
// * A `;` may end a sequence before `end`, `done` or a closing bracket:
//   `begin a; b; end` is `begin a; b end`.
// * `a; ; b` is an error: "`; ;` is two `;`s: write `;;` (with no space)
//   to end a toplevel phrase, or one `;`"
// * After a `;` must come an expression: `a; + b` is an error: "expected
//   an expression after `;`"
// * `;` binds loosest of all: `x |> f; y` is `{ f(x); y }`.
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`a; b`: steps in order, the last one is the value";

/// A to-do list built step by step; the last step is its value.
pub fn todo() -> Vec<&'static str> { ml! {
    let mut list = Vec.new () in
    list.push("buy milk");
    list.push("call mum");
    list.sort();
    list
}}

/// A counter that counts its steps, and the total at the end.
pub fn steps(n: u32) -> (u32, u32) { ml! {
    let mut count = 0 in
    let mut total = 0 in
    for i = 1 to n do
      count <- count + 1;
      total <- total + i
    done;
    (count, total)
}}

pub fn run() {
    println!("todo () = {:?}", todo());
    println!("steps 4 = {:?}", steps(4));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_order() {
        assert_eq!(todo(), ["buy milk", "call mum"]);
        assert_eq!(steps(4), (4, 10));
    }

    #[test]
    fn the_value_is_the_last_step() {
        let mut log = Vec::new();
        let v = ml! {
        log.push(1);
        log.push(2);
        log.len()
        };
        assert_eq!((v, log), (2, vec![1, 2]));
    }
}

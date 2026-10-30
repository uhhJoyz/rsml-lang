// Toy 122: raw identifiers (`r#match`) and reserved words as names.
//
// What it is: a Rust keyword is never a name in rsml, so a Rust item
// named after one is written as a raw identifier, `r#match`, as in Rust.
// A raw identifier is a name wherever one goes (values, functions,
// fields, methods, patterns) and stays raw in the Rust. The words Rust
// only reserves, `gen`, `try`, `yield`, `box`, `macro` and others, are
// ordinary names, written plainly and emitted raw, so they work in every
// edition.
//
// When to use it: for Rust items called `match`, `type`, `ref`, `loop`,
// .. (generated code is full of them), and to use a reserved word as a
// name.
//
// How to write it:
// * `r#match "#" line` is `r#match("#", line)`: the function `match`.
// * `let gen = 3 in gen` is `{ let r#gen = 3; r#gen }`.
// * `async`, `await` and `dyn` stay keywords: write `r#async` for a name.
// * `let loop = 1 in loop` is an error: "`loop` is a Rust keyword, not a
//   name (as a name: `r#loop`); an endless loop is `while true do .. done`".
//
// Reference: [Rust habits](../../docs/notation.md#rust-habits)

use rsml::ml;

pub const SUMMARY: &str = "Raw names `r#match`, `t.r#type`; reserved words: `let gen = ..`";

/// A function named after a keyword, as generated code often has one.
fn r#match(prefix: &str, line: &str) -> bool {
    line.starts_with(prefix)
}

/// A token, with a field named after a keyword.
pub struct Token {
    r#type: &'static str,
    text: String,
}

/// A dice with a method `gen`, a keyword from edition 2024 on.
pub struct Dice {
    state: u32,
}

impl Dice {
    fn r#gen(&mut self) -> u32 {
        self.state = (self.state * 5 + 3) % 16;
        self.state % 6 + 1
    }
}

/// How many lines are comments.
pub fn comment_lines(lines: &[&str]) -> usize {
    ml! { lines |> .iter() |> .filter(fun l -> r#match "#" l) |> .count() }
}

/// A token, built and read through its field `type`.
pub fn describe(word: &str) -> String {
    ml! {
        let t = Token { r#type = "word"; text = String.from word } in
        format!("{} {:?}", t.r#type, t.text)
    }
}

/// Three rolls; `gen` and `try` are plain names here.
pub fn rolls(dice: &mut Dice) -> Vec<u32> {
    ml! {
        let mut out = Vec.new () in
        let mut try = 0 in
        while try < 3 do
          // `let gen = dice.gen() in out.push(gen)` is
          // `{ let r#gen = dice.r#gen(); out.push(r#gen) }`.
          let gen = dice.gen() in
          out.push(gen);
          try <- try + 1
        done;
        out
    }
}

pub fn run() {
    println!(
        "comment_lines = {}",
        comment_lines(&["# a", "b = 1", "# c"])
    );
    println!("describe \"hi\" = {}", describe("hi"));
    println!("rolls = {:?}", rolls(&mut Dice { state: 1 }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_names() {
        assert_eq!(comment_lines(&["# a", "b = 1", "# c"]), 2);
        assert_eq!(describe("hi"), "word \"hi\"");
        assert_eq!(rolls(&mut Dice { state: 1 }), [3, 6, 5]);
    }
}

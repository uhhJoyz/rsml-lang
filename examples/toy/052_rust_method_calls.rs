// Toy 052: Rust method calls, `xs.push(1)`, `line.trim().len()`.
//
// What it is: Rust's own method syntax works inside OCaml code, with OCaml
// expressions as the arguments. A chain may be broken across lines, one
// `.method()` per line, as rustfmt would print it.
//
// When to use it: whenever a method chain reads well, which is most of the
// time in Rust code. The OCaml-style `.push xs 1` (toy 045) is for
// pipelines.
//
// How to write it: no space between the method name and its `(`.
// * `x.f(a)` calls the method `f`; `x.f (a)`, with a space, applies the
//   *field* `f` to `a`, as OCaml reads it. A field holding a function is
//   called that way.
// * So `s.len ()` is `(s.len)()`, a field `len` applied, which rustc
//   rejects for a string: write `s.len()`.
// * After a function, a field is one argument and a spaced `(..)` the
//   next one: `area t.width (t.height + 1)` is `area(t.width, t.height + 1)`.
//
// Reference: [Rust-style methods, fields and calls](../../docs/notation.md#rust-style-methods-fields-and-calls)

use rsml::ml;

pub const SUMMARY: &str = "Method calls `xs.push(1)`, chains, and `x.f (a)` calling a field";

/// A button: its label, and what a click does (a function in a field).
pub struct Button {
    pub label: String,
    pub on_click: fn(u32) -> String,
}

/// The number of words longer than three letters, by a chain of methods.
pub fn long_words(line: &str) -> usize {
    ml! {
        // `line.split_whitespace().filter(fun w -> w.len() > 3).count()` is
        // `line.split_whitespace().filter(|w| w.len() > 3).count()`.
        line.split_whitespace().filter(fun w -> w.len() > 3).count()
    }
}

/// A shopping list, built by `push` calls; the chain below is broken
/// across lines.
pub fn shopping(extra: &str) -> Vec<String> {
    ml! {
        // rsmlfmt: skip
        // (The formatter would join the chain at the end onto one line;
        // the line breaks are the lesson.)
        let mut items = Vec.new () in
        // `items.push(String::from("tea"))` is a method call with a Rust
        // call as its argument.
        items.push(String::from("tea"));
        items.push(extra.trim().to_uppercase());
        items
        .into_iter()
        .filter(fun s -> not (s.is_empty()))
        .collect()
    }
}

/// Clicks a button: its `on_click` field is a function.
pub fn click(b: &Button, times: u32) -> String {
    ml! {
        // `b.on_click (times)` is `(b.on_click)(times)`; written
        // `b.on_click(times)`, it would look for a method `on_click`.
        b.label.clone() ^ ": " ^ b.on_click (times)
    }
}

fn clicked(times: u32) -> String {
    format!("clicked {times} times")
}

/// A room's size, in metres.
pub struct Size {
    pub width: u32,
    pub depth: u32,
}

fn area(width: u32, depth: u32) -> u32 {
    width * depth
}

/// The floor area with a one metre deeper room: fields as arguments.
pub fn extended_area(room: &Size) -> u32 {
    ml! {
        // `area room.width (room.depth + 1)` is
        // `area(room.width, room.depth + 1)`.
        area room.width (room.depth + 1)
    }
}

pub fn run() {
    let ok = Button {
        label: "OK".to_string(),
        on_click: clicked,
    };
    println!(
        "long_words \"a quick brown fox\" = {}",
        long_words("a quick brown fox")
    );
    println!("shopping \" milk \" = {:?}", shopping(" milk "));
    println!("click ok 2 = {:?}", click(&ok, 2));
    let room = Size { width: 3, depth: 4 };
    println!("extended_area 3x4 = {}", extended_area(&room));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chains() {
        assert_eq!(long_words("a quick brown fox"), 2);
        assert_eq!(shopping(" milk "), ["tea", "MILK"]);
        assert_eq!(shopping("  "), ["tea"]);
    }

    #[test]
    fn a_field_called_with_a_space() {
        let ok = Button {
            label: "OK".to_string(),
            on_click: clicked,
        };
        assert_eq!(click(&ok, 2), "OK: clicked 2 times");
    }

    #[test]
    fn fields_as_arguments() {
        assert_eq!(extended_area(&Size { width: 3, depth: 4 }), 15);
    }
}

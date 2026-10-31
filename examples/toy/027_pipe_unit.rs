// Toy 027: piping `()`: `() |> f` and `(()) |> f`.
//
// What it is: the unit rules of toy 005 hold in pipelines too. A piped
// `()` means "no arguments", and a piped `(())` is the unit value.
//
// When to use it: to start a pipeline with a function that takes no
// arguments (a default, a reading), and in the rare case where a Rust
// function takes `()`.
//
// How to write it:
// * `() |> f` is `f()`.
// * `(()) |> f` is `f(())`.
// * A constructor takes the value either way: `() |> Ok` is `Ok(())`.
//
// Reference: [Pipelines: |> and @@](../../docs/notation.md#pipelines--and-)

use rsml::ml;

pub const SUMMARY: &str = "`() |> f` calls `f` with no arguments, `(()) |> f` passes `()`";

/// The default list of greetings.
fn default_greetings() -> Vec<&'static str> {
    vec!["hello", "hi"]
}

/// Words joined and in capitals.
fn shout(words: Vec<&str>) -> String {
    words.join(" ").to_uppercase()
}

/// The default greetings, shouted.
pub fn shouted() -> String { ml! {
    // `() |> default_greetings |> shout` is
    // `shout(default_greetings())`.
    () |> default_greetings |> shout
}}

/// A function that takes the unit value itself, as callbacks sometimes do.
fn on_done(signal: ()) -> &'static str {
    let () = signal;
    "done"
}

pub fn finish() -> &'static str { ml! {
    // `(()) |> on_done` is `on_done(())`.
    (()) |> on_done
}}

/// "It worked", piped into a constructor.
pub fn saved() -> Result<(), String> { ml! {
    // `() |> Ok` is `Ok(())`.
    () |> Ok
}}

pub fn run() {
    println!("() |> default_greetings |> shout = {:?}", shouted());
    println!("(()) |> on_done = {:?}", finish());
    println!("() |> Ok = {:?}", saved());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_in_pipelines() {
        assert_eq!(shouted(), "HELLO HI");
        assert_eq!(finish(), "done");
        assert_eq!(saved(), Ok(()));
    }
}

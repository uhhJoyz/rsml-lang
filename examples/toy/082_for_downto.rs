// Toy 082: counting down, `for i = a downto b do .. done`.
//
// What it is: the counting loop going down, from `a` to `b`, both
// included. It is Rust's reversed inclusive range:
// `for i = n downto 1 do f i done` is `for i in (1..=n).rev() { f(i); }`.
//
// When to use it: for countdowns, and to walk a slice from its end.
//
// How to write it: as `for .. to` (toy 081), with `downto`. With unsigned
// numbers, stop at 0 or above: the range never goes below its lower end,
// so `for i = n downto 0` is fine, even for `u32`.
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::ml;

pub const SUMMARY: &str = "`for i = 3 downto 1 do .. done`: a countdown, ends included";

/// A rocket countdown.
pub fn countdown(from: u32) -> String {
    ml! {
        let mut out = String.new () in
        for i = from downto 1 do out.push_str(&format!("{i}.. ")) done;
        out ^ "lift off"
    }
}

/// The letters of a word, last first.
pub fn backwards(word: &str) -> String {
    ml! {
        let letters = word.chars().collect::<Vec<_>>() in
        let mut out = String.new () in
        // `letters.len()` is at least 1 here, so `- 1` does not underflow.
        if not letters.is_empty() then
          for i = letters.len() - 1 downto 0 do out.push(letters[i]) done;
        out
    }
}

pub fn run() {
    println!("countdown 3 = {:?}", countdown(3));
    println!("backwards \"stressed\" = {:?}", backwards("stressed"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downwards() {
        assert_eq!(countdown(3), "3.. 2.. 1.. lift off");
        assert_eq!(countdown(0), "lift off");
        assert_eq!(backwards("stressed"), "desserts");
        assert_eq!(backwards(""), "");
    }
}

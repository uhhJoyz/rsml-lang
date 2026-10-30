// Toy 010: local functions that change what they capture, `let mut f x = ..`.
//
// What it is: a closure that assigns to a variable it captures is Rust's
// `FnMut`, and rustc calls one only through a `mut` binding.
// `let mut f x = ..` is that binding.
//
// When to use it: for a counter, an accumulator, a function that pushes
// onto a list.
//
// How to write it:
// * `let mut next () = incr c; !c in ..`: the body is a sequence, and its
//   value is the last expression.
// * Without `mut`, rustc says "cannot borrow `next` as mutable, as it is
//   not declared as mutable" and suggests adding it.
// * The function borrows what it changes until its last call, so read
//   the variable after that.
// * In full, `let c = ref 0 in let mut next () = incr c; !c in next ()`
//   is `{ let mut c = 0; let mut next = || { c += 1; c }; next() }`.
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)

use rsml::ml;

pub const SUMMARY: &str = "`let mut next () = ..`: a local function that changes its captures";

/// Three ticket numbers from a counter that starts at 100.
pub fn tickets() -> (u32, u32, u32) {
    ml! {
        let counter = ref 100 in
        // `next` increments `counter`, so it is `let mut`.
        let mut next () =
          incr counter;
          !counter
        in
        let first = next () in
        let second = next () in
        (first, second, next ())
    }
}

/// A log that a local function appends to.
pub fn log_lines(events: &[&str]) -> Vec<String> {
    ml! {
        let mut log = Vec.new () in
        // `note` pushes onto `log`: `let mut note e = log.push(..)`.
        let mut note (e : &str) = log.push(format!("event: {e}")) in
        for e in events do note !e done;
        // The last call of `note` is above, so `log` is free again.
        log
    }
}

pub fn run() {
    println!("tickets () = {:?}", tickets());
    println!(
        "log_lines [start; stop] = {:?}",
        log_lines(&["start", "stop"])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_and_logs() {
        assert_eq!(tickets(), (101, 102, 103));
        assert_eq!(
            log_lines(&["start", "stop"]),
            ["event: start", "event: stop"]
        );
    }

    #[test]
    fn the_header_example() {
        assert_eq!(
            ml! {
                let c = ref 0 in
                let mut next () =
                  incr c;
                  !c
                in
                next ()
            },
            1
        );
    }
}

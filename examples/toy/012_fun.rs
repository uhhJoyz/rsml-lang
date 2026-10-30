// Toy 012: anonymous functions, `fun x -> e`.
//
// What it is: `fun params -> body` is a closure, Rust's `|params| body`,
// with the parameters separated by spaces.
//
// When to use it: wherever Rust takes a closure (`map`, `filter`,
// `sort_by_key`, callbacks), and to return a function.
//
// How to write it:
// * `fun x y -> x + y` takes two parameters, `fun () -> e` none.
// * The body reaches as far right as it can, so as an argument of an
//   OCaml application a `fun` goes in parentheses:
//   `apply_twice (fun n -> n * 2) x`. Inside a Rust call's `( .. )` it
//   ends at the `,` or `)`.
// * A `fun` that is the block's value is a `move` closure (unless it
//   assigns to what it captures); one passed as an argument is not (toy
//   015).
// * Rust's closure syntax is not OCaml: `|x| x + 1` is an error: "Rust
//   closures are written `fun x -> e` in OCaml" (toy 129).
//
// Reference: [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function)

use rsml::ml;

pub const SUMMARY: &str = "`fun x y -> x + y`: closures, as values and as arguments";

/// A function of two numbers, returned.
pub fn adder() -> impl Fn(i64, i64) -> i64 {
    ml! {
        // `fun x y -> x + y` is `move |x, y| x + y`.
        fun x y -> x + y
    }
}

/// A closure as the argument of a Rust method.
pub fn doubled(xs: &[i64]) -> Vec<i64> {
    ml! {
        // `xs.iter().map(fun x -> x * 2)` is `xs.iter().map(|x| x * 2)`.
        xs.iter().map(fun x -> x * 2).collect()
    }
}

fn apply_twice(f: impl Fn(i64) -> i64, x: i64) -> i64 {
    f(f(x))
}

/// A closure as an OCaml argument, in parentheses.
pub fn quadrupled(x: i64) -> i64 {
    ml! {
        // `apply_twice (fun n -> n * 2) x` is `apply_twice(|n| n * 2, x)`.
        apply_twice (fun n -> n * 2) x
    }
}

/// A closure without parameters.
pub fn greeter(name: String) -> impl Fn() -> String {
    ml! {
        // `fun () -> 1` is `move || 1`; this one formats a greeting.
        fun () -> format!("welcome, {name}")
    }
}

pub fn run() {
    println!("adder () 2 3 = {}", adder()(2, 3));
    println!("doubled [1; 2; 3] = {:?}", doubled(&[1, 2, 3]));
    println!("quadrupled 5 = {}", quadrupled(5));
    println!("greeter \"Ada\" () = {:?}", greeter("Ada".to_string())());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closures() {
        assert_eq!(adder()(2, 3), 5);
        assert_eq!(doubled(&[1, 2, 3]), [2, 4, 6]);
        assert_eq!(quadrupled(5), 20);
        let hello = greeter("Ada".to_string());
        assert_eq!(hello(), "welcome, Ada");
        assert_eq!(hello(), "welcome, Ada");
    }
}

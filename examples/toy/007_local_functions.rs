// Toy 007: local functions, `let f x = .. in`.
//
// What it is: `let name params = body in ..` defines a function for the
// expression after `in`. It is a Rust closure:
// `let double x = x * 2 in double 3` is `{ let double = |x| x * 2; double(3) }`.
//
// When to use it: for a helper used a few times in one function.
//
// How to write it:
// * The parameters follow the name, separated by spaces, as arguments do.
// * Their types are inferred from the uses, as a closure's are, so a local
//   function is used at one type: calling it with an integer and then a
//   float is a type error.
// * To call a method on a parameter, give the parameter its type,
//   `let shout (s : &str) = s.to_uppercase() in ..`: rustc must know a
//   closure parameter's type before a method call on it.
// * It may use the locals around it, which it borrows, as a closure does.
// * The macro knows how many parameters it has, so it can be partially
//   applied (toy 018).
//
// Reference: [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings)

use rsml::ml;

pub const SUMMARY: &str = "`let f x = .. in`: a local function is a Rust closure";

/// The cost of 1, 3 and 10 items at `unit` each.
pub fn price_list(unit: u32) -> (u32, u32, u32) {
    ml! {
        // `cost` uses the local `unit`: the closure borrows it.
        // `let cost n = n * unit in cost 3` is
        // `{ let cost = |n| n * unit; cost(3) }`.
        let cost n = n * unit in
        (cost 1, cost 3, cost 10)
    }
}

/// A local function of two parameters is called with both.
pub fn squares_sum(a: i64, b: i64) -> i64 {
    ml! {
        // `let sum_sq x y = x * x + y * y in sum_sq a b` is
        // `{ let sum_sq = |x, y| (x * x) + (y * y); sum_sq(a, b) }`.
        let sum_sq x y = x * x + y * y in
        sum_sq a b
    }
}

/// A parameter with a type, so that its methods can be called.
pub fn shouts() -> (String, String) {
    ml! {
        // `(s : &str)` is the closure parameter `s: &str`.
        let shout (s : &str) = s.to_uppercase() + "!" in
        (shout "hi", shout "bye")
    }
}

pub fn run() {
    println!("price_list 4 = {:?}", price_list(4));
    println!("squares_sum 3 4 = {}", squares_sum(3, 4));
    println!("shouts () = {:?}", shouts());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_functions() {
        assert_eq!(price_list(4), (4, 12, 40));
        assert_eq!(squares_sum(3, 4), 25);
        assert_eq!(shouts(), ("HI!".to_string(), "BYE!".to_string()));
    }

    #[test]
    fn the_header_example() {
        assert_eq!(
            ml! {
                let double x = x * 2 in
                double 3
            },
            6
        );
    }
}

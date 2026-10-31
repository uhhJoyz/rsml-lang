// Toy 111: traits with rsml method bodies.
//
// What it is: in a `trait` inside `rsml!`, a default method's body is
// rsml code, and so are the bodies in each `impl Trait for Type`. Method
// declarations without a body (`fn area(&self) -> f64;`) stay as they are.
//
// When to use it: to give a family of types shared behaviour, written in
// rsml like the rest of the code.
//
// How to write it:
// * Declarations and signatures are Rust; each body is one rsml
//   expression.
// * A default method calls the others Rust style, `self.area()`. With a
//   space, `self.area ()` is `(self.area)()`, a call of a field `area`.
// * Methods that take `self` are not functions the macro knows the arity
//   of (toy 114): call them as methods.
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "A `trait` with an rsml default method, and its `impl`s";

rsml! {
/// A plane figure.
pub trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> String;

    /// A default method, in rsml.
    fn describe(&self) -> String {
        format!("{} of area {:.1}", self.name(), self.area())
    }
}

pub struct Circle {
    pub r: f64,
}

pub struct Square {
    pub side: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI *. self.r *. self.r
    }

    fn name(&self) -> String {
        String.from "circle"
    }
}

impl Shape for Square {
    // `self.side ** 2.0` is `::core::primitive::f64::powf(self.side, 2.0)`.
    fn area(&self) -> f64 {
        self.side ** 2.0
    }

    fn name(&self) -> String {
        String.from "square"
    }

    /// Overrides the default.
    fn describe(&self) -> String {
        format!("a square of side {}", self.side)
    }
}

/// The total area of some shapes of any kind.
pub fn total_area(shapes: &[&dyn Shape]) -> f64 {
    shapes |> .iter() |> .map(fun s -> s.area()) |> .sum()
}
}

pub fn run() {
    let (c, s) = (Circle { r: 1.0 }, Square { side: 2.0 });
    println!("{}", c.describe());
    println!("{}", s.describe());
    println!(
        "total_area [circle 1; square 2] = {:.3}",
        total_area(&[&c, &s])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_overridden() {
        assert_eq!(Circle { r: 1.0 }.describe(), "circle of area 3.1");
        assert_eq!(Square { side: 2.0 }.describe(), "a square of side 2");
        assert_eq!(Square { side: 2.0 }.name(), "square");
    }

    #[test]
    fn through_the_trait() {
        let (c, s) = (Circle { r: 1.0 }, Square { side: 2.0 });
        assert_eq!(total_area(&[&c, &s]), std::f64::consts::PI + 4.0);
    }
}

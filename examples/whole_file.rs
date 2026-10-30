//! A whole file written in one `rsml!`: the items are Rust, the function
//! bodies are OCaml. `cargo run --example whole_file` runs it, and
//! `cargo test` runs its tests.
//!
//! Comments like this one and `#![..]` attributes stay outside the macro:
//! they belong to the file, and a macro call cannot hold them.

#![deny(warnings)]

rsml::rsml! {

use std::collections::HashMap;
use std::fmt;

/// A shape in the plane.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle(f64),
    Rect(f64, f64),
}

impl Shape {
    pub fn square(side: f64) -> Shape {
        Shape::Rect(side, side)
    }

    /// `Self::square` is a function of this `rsml!`, so its arity is known.
    pub fn unit() -> Shape {
        Self::square 1.0
    }

    pub fn area(&self) -> f64 {
        match self with
        | Shape::Circle(r) -> std::f64::consts::PI *. !r *. !r
        | Shape::Rect(w, h) -> !w *. !h
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self with
        | Shape::Circle(r) -> write!(f, "circle of radius {}", r)
        | Shape::Rect(w, h) -> write!(f, "{} by {} rectangle", w, h)
    }
}

pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// `add 1` is a partial application: `add` is defined in this `rsml!`.
pub fn incremented(xs: &[i64]) -> Vec<i64> {
    xs |> .iter() |> .copied() |> .map(add 1) |> .collect()
}

pub fn word_counts(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new () in
    for w in .split_whitespace text do
      let c = counts.entry(w).or_insert(0) in
      *c <- *c + 1
    done;
    counts
}

/// A body that reads better as Rust stays Rust.
#[rust]
fn checksum(bytes: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for b in bytes {
        sum = sum.wrapping_mul(31).wrapping_add(u32::from(*b));
    }
    sum
}

fn main() {
    let shapes = [Shape::unit (); Shape::square 2.0; Shape::Circle(1.0)] in
    for s in &shapes do println!("{}: area {:.2}", s, s.area()) done;
    println!("{:?}", incremented &[|1; 2; 3|]);
    println!("{:?}", word_counts "the cat and the hat" |> .get("the"));
    println!("checksum {}", checksum "rsml".as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn areas() {
        assert_eq!((Shape::square 3.0).area() as i64, 9);
        assert!((Shape::Circle 1.0).area() > 3.0);
        assert_eq!(Shape::unit (), Shape::Rect(1.0, 1.0))
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Shape::square 2.0), "2 by 2 rectangle");
        assert_eq!((Shape::Circle 0.5).to_string(), "circle of radius 0.5")
    }

    /// `use super::*` brings the arities along: `add 10` is a closure.
    #[test]
    fn sibling_partial_application() {
        let add10 = add 10 in
        assert_eq!(add10 5, 15);
        assert_eq!(incremented &[|1; 2|], vec![2, 3])
    }

    #[test]
    fn counts() {
        let c = word_counts "a b a" in
        assert_eq!(c["a"], 2);
        assert_eq!(c.get("c"), None)
    }

    #[test]
    fn rust_body() {
        assert_ne!(checksum b"ab", checksum b"ba")
    }
}

}

// Toy 056: constructors, `Some x`, `Shape::Rect (w, h)`, `Some (1, 2)`.
//
// What it is: an enum variant or a tuple struct is applied like a
// function, ML style: `Some x`, `Shape::Circle r`. Applied to a tuple,
// a constructor gets one field per element, except `Some`, which has one
// field and wraps the tuple.
//
// When to use it: to build any enum or tuple struct value.
//
// How to write it: the constructor, then its argument.
// * `Shape::Rect (w, h)` fills the two fields of `Rect`.
// * `Some (1, 2)` is `Some((1, 2))`: an option of a pair.
// * Double parentheses keep a tuple as one field: `Span ((a, b))`.
// * `Ok ()` is `Ok(())` (toy 005); `None` alone is a value.
// * The argument is one atom: write `Some (f x)`, since `Some f x` is
//   `Some(f, x)`, two fields, which rustc rejects.
// Patterns spell constructors the same way (toy 068).
//
// Reference: [Application](../../docs/notation.md#application)

use rsml::ml;

pub const SUMMARY: &str = "Constructors: `Shape::Rect (w, h)`, `Some (1, 2)`, `Span ((a, b))`";

#[derive(Debug, PartialEq)]
pub enum Shape {
    Circle(f64),
    Rect(f64, f64),
}

/// A tuple struct whose one field is a pair.
#[derive(Debug, PartialEq)]
pub struct Span((u32, u32));

/// A circle and a rectangle of the same width.
pub fn shapes(w: f64) -> Vec<Shape> { ml! {
    // `Shape::Circle w` is `Shape::Circle(w)`, and
    // `Shape::Rect (w, 2.0)` is `Shape::Rect(w, 2.0)`.
    [Shape::Circle w; Shape::Rect (w, 2.0)]
}}

/// The smallest and largest value, if there is one.
pub fn min_max(xs: &[u32]) -> Option<(u32, u32)> { ml! {
    match (xs.iter().min(), xs.iter().max()) with
    // `Some (!lo, !hi)` is `Some((*lo, *hi))`: one field, the pair.
    | (Some lo, Some hi) -> Some (!lo, !hi)
    | _ -> None
}}

/// A span from `a` to `b`: the pair is the struct's one field.
pub fn span(a: u32, b: u32) -> Span { ml! {
    // `Span ((a, b))` is `Span((a, b))`; `Span (a, b)` would be
    // `Span(a, b)`, two fields, which rustc rejects for `Span`.
    Span ((a, b))
}}

/// A checked width: `Ok` or `Err` around one value.
pub fn width(w: f64) -> Result<f64, String> { ml! {
    // A macro call is one atom, so these parentheses are only for the
    // reader: `Err format!("{w}")` is `Err(format!("{w}"))` too.
    if w > 0.0 then Ok w else Err (format!("{w} is not positive"))
}}

pub fn run() {
    println!("shapes 3.0 = {:?}", shapes(3.0));
    println!(
        "min_max [4; 1; 7] = {:?}, min_max [] = {:?}",
        min_max(&[4, 1, 7]),
        min_max(&[])
    );
    println!("span 2 5 = {:?}", span(2, 5));
    println!(
        "width 2.5 = {:?}, width (-1.0) = {:?}",
        width(2.5),
        width(-1.0)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_from_a_tuple() {
        assert_eq!(shapes(3.0), [Shape::Circle(3.0), Shape::Rect(3.0, 2.0)]);
        assert_eq!(span(2, 5), Span((2, 5)));
    }

    #[test]
    fn some_wraps_the_tuple() {
        assert_eq!(min_max(&[4, 1, 7]), Some((1, 7)));
        assert_eq!(min_max(&[]), None);
        assert_eq!(width(2.5), Ok(2.5));
        assert_eq!(width(-1.0), Err("-1 is not positive".to_string()));
    }
}

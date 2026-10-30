//! Formatter stress input for the Rust inside `rsml!`, formatted by
//! rustfmt with the OCaml bodies put back: only tokenized and expanded by
//! tests/corpus.rs, never compiled.

rsml::rsml! {

use std::fmt;   use std::collections::{HashMap,BTreeMap};

/// A point,
/// with docs.
#[derive(Debug,Clone, PartialEq)]
pub struct   Point{ pub x:i64,pub y :i64 }   // trailing comment

pub enum Shape{Circle(f64),Rect(f64,f64)}

// Before `add`.
pub fn   add(a:i64,b:i64)->i64 { a+b |> fun x -> x (* in the body *) }


/* A block comment between items. */
impl   Point {
    pub fn norm(&self)->i64 { self.x |> abs }

    fn   long_signature_name<T:Clone+fmt::Debug>(first_argument: T, second_argument: T, third: i64) -> i64 where T: Copy { third }
    pub const ORIGIN:Point=Point{x:0,y:0};
}

impl fmt::Display for Point { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f, "({}, {})", self.x, self.y) } }

pub trait Area { fn area(&self)->f64; fn twice(&self)->f64 { self.area () *. 2.0 } }

#[rust]
fn   checksum(bytes:&[u8])->u32{ let mut s=0u32; for b in bytes { s=s.wrapping_mul(31).wrapping_add(u32::from(*b)); } s }

#[rust]
impl Area for Shape { fn area(&self)->f64 { match self { Shape::Circle(r)=>3.14*r*r, Shape::Rect(w,h)=>w*h } } }

pub fn noop() {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn adds() { assert_eq!(add 1 2, 3) }
}

}

mod nested {
    rsml::rsml! {
      pub fn   sub(a:i64,b:i64)->i64 { a - b }
        pub(crate) async fn later() -> i64 { 1 }
    }
}

fn inline() {
    rsml::rsml! { fn   one()->i64 { 1 } }
}

#![doc = include_str!("../README.md")]

// Lets `ml!` expansions (which name `::rsml`) work inside this crate's own tests.
extern crate self as rsml;

pub use rsml_macros::{ml, rsml};

/// Support code that `ml!` expansions call. Not a stable API.
#[doc(hidden)]
#[path = "rt.rs"]
pub mod __rt;

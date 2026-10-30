//! rsml's integration tests, compiled as edition 2021 (the main crate is
//! 2024). `cargo test --workspace` runs them.

#[path = "../../items.rs"]
mod items;
#[path = "../../ml.rs"]
mod ml;
#[path = "../../more.rs"]
mod more;
#[path = "../../notation.rs"]
mod notation;
#[path = "../../reserved.rs"]
mod reserved;
#[path = "../../rust_style.rs"]
mod rust_style;

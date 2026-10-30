#!/bin/sh
# Runs clippy, warnings denied, over the workspace: the crates, the examples
# and the tests, among them tests/notation.rs, which uses every notation of
# docs/notation.md. Clippy lints the Rust the macros generate like any other
# code, so a lint that OCaml code cannot act on (OCaml has no `+=`, so
# `x <- x + 1` must not draw `assign_op_pattern`) shows up here. Run it after
# changing the code generator, and add new notations to tests/notation.rs.
set -eu
cd "$(dirname "$0")/.."
cargo clippy --workspace --all-targets -- -D warnings

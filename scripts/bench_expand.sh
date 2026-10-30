#!/bin/sh
# Checks that `rsml!` expansion time grows linearly with the number of
# functions: times 300 and 2400 functions in release mode and requires
# roughly ×8 (rsml-compiler/tests/scaling.rs).
#
#     scripts/bench_expand.sh
set -e
cd "$(dirname "$0")/.."
cargo test -q --release -p rsml-compiler --test scaling -- --ignored --nocapture

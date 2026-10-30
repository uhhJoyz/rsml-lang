#!/bin/sh
# Runs the fuzzer (rsml-fmt/tests/fuzz.rs) long: ITERS mutations per block
# (default 200), in release mode, printing up to 100 problems.
#
#     scripts/fuzz.sh [ITERS]
#
# `cargo test` runs the same fuzzer on a small sample.
set -e
cd "$(dirname "$0")/.."
FUZZ_ITERS="${1:-200}" FUZZ_SHOW="${FUZZ_SHOW:-100}" \
    cargo test --release -p rsml-fmt --test fuzz -- --nocapture

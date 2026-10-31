#!/bin/bash
# Runs every check in this repository and prints a summary:
#
#     scripts/check_all.sh           # everything (about 8 minutes)
#     scripts/check_all.sh --quick   # without the editor checks, the long
#                                    # corpus and fuzz runs and the latency
#                                    # measurement
#
# A check whose tool is missing (rust-analyzer, Neovim, Vim, rustfmt) is skipped
# and says so. The output of each check is in target/check-all/NAME.log.
# Exits 1 if any check failed.
set -u
cd "$(dirname "$0")/.."
QUICK=0
[ "${1:-}" = "--quick" ] && QUICK=1
LOGS=target/check-all
mkdir -p "$LOGS"
cargo build -q --workspace || exit 1

RESULTS=()
FAILED=0

has() { command -v "$1" > /dev/null 2>&1; }

# check NAME COMMAND..: runs the command, keeps its output, and records the
# result with the last line of the output as its summary.
check() {
    local name=$1
    shift
    local start=$SECONDS
    printf '%-18s ' "$name"
    if "$@" > "$LOGS/$name.log" 2>&1; then
        local status=ok
    else
        local status=FAIL
        FAILED=1
    fi
    local last
    last=$(grep -v '^\s*$' "$LOGS/$name.log" | tail -1 | cut -c1-90)
    printf '%-4s %4ss  %s\n' "$status" $((SECONDS - start)) "$last"
    RESULTS+=("$(printf '%-18s %-4s %s' "$name" "$status" "$last")")
}

skip() {
    printf '%-18s skip        %s\n' "$1" "$2"
    RESULTS+=("$(printf '%-18s %-4s %s' "$1" "skip" "$2")")
}

# The workspace's tests, with the counts added up.
tests() {
    local out status
    out=$(cargo test -q --workspace 2>&1)
    status=$?
    echo "$out"
    echo "$out" | awk '/test result/ { p += $4; f += $6; i += $8 }
        END { printf "%d passed, %d failed, %d ignored\n", p, f, i }'
    return $status
}

# examples/toy: every toy's demo runs to its end (`cargo test` runs their
# tests).
toys() {
    local out status last
    out=$(cargo run -q --example toy 2>&1)
    status=$?
    echo "$out"
    last=$(echo "$out" | tail -1)
    [ $status = 0 ] && [[ $last =~ ^ran\ ([0-9]+)\ of\ ([0-9]+)\ toys$ ]] &&
        [ "${BASH_REMATCH[1]}" = "${BASH_REMATCH[2]}" ]
}

# examples/zero_cost.rs: each `ml_*` function's machine code against its
# hand-written `rs_*` twin.
asm() {
    local out status asm
    cargo rustc -q --release --example zero_cost -- --emit=asm || return 1
    # The newest: cargo leaves the file of an up-to-date build in place, and
    # does not write it again if it was deleted, unless the source changed.
    if ! ls target/release/examples/zero_cost-*.s > /dev/null 2>&1; then
        touch examples/zero_cost.rs
        cargo rustc -q --release --example zero_cost -- --emit=asm || return 1
    fi
    asm=$(ls -t target/release/examples/zero_cost-*.s | head -1)
    out=$(python3 scripts/compare_asm.py "$asm")
    status=$?
    echo "$out"
    echo "$(echo "$out" | grep -c identical) of $(echo "$out" | grep -c .) pairs identical"
    return $status
}

# rustfmt alone on the crates without `ml!` code: in src, tests and examples
# (tests/edition2021 includes the tests), `rsml-fmt --rustfmt` runs rustfmt,
# and puts back what rustfmt undoes (`fn f() { ml! {` .. `}}`).
rustfmt_check() {
    cargo fmt -p rsml-compiler -p rsml-macros -p rsml-fmt -- --check &&
        echo "nothing to reformat outside src, tests, examples"
}
rsmlfmt_check() {
    # The formatter's fixtures (rsml-fmt/tests/fixtures) stay messy: not here.
    target/debug/rsml-fmt --rustfmt --check src tests examples &&
        echo "nothing to reformat in src, tests, examples"
}
clippy() { ./scripts/check_clippy.sh && echo "no warnings"; }
rustdoc() {
    RUSTDOCFLAGS="-D warnings -D rustdoc::missing_crate_level_docs" \
        cargo doc -q --no-deps --workspace && echo "no warnings"
}

# What `cargo package` would put in each published crate: no scripts,
# scratch or build output, and the README where the crate has one.
package() {
    local crate files bad=0
    for crate in rsml-lang rsml-macros rsml-compiler rsml-fmt; do
        files=$(cargo package --list --allow-dirty -p "$crate" 2> /dev/null) || {
            echo "$crate: cargo package --list failed"
            return 1
        }
        if echo "$files" | grep -E '^(scripts|target)/|__pycache__|\.pyc$|\.orig\.|^tests/edition2021'; then
            echo "$crate: files that should not be published (above)"
            bad=1
        fi
        case $crate in rsml-lang | rsml-fmt)
            echo "$files" | grep -qx README.md || { echo "$crate: no README.md"; bad=1; }
        esac
        echo "$crate: $(echo "$files" | wc -l | tr -d ' ') files"
    done
    [ $bad = 0 ] && echo "4 crates, nothing that should not be published"
}
bench() {
    local out status
    out=$(./scripts/bench_expand.sh 2>&1)
    status=$?
    echo "$out"
    echo "$out" | grep "ratio"
    return $status
}

# rsml-fmt/tests/corpus.rs with every config for every file (`cargo test`
# samples the configs for the toy examples).
corpus_all() {
    local out status
    out=$(CORPUS_ALL=1 cargo test -q --release -p rsml-fmt --test corpus 2>&1)
    status=$?
    echo "$out"
    echo "$out" | grep "test result"
    return $status
}

fuzz_long() {
    local out status
    out=$(./scripts/fuzz.sh 2>&1)
    status=$?
    echo "$out"
    echo "$out" | grep "^fuzz:"
    return $status
}

check tests tests
check toys toys
check docs python3 scripts/check_docs.py
check rustdoc rustdoc
check package package
check rustfmt rustfmt_check
check rsml-fmt rsmlfmt_check
check clippy clippy
check bench-expand bench
check asm asm
if [ $QUICK = 1 ]; then
    skip corpus-all "--quick"
    skip fuzz-long "--quick"
else
    check corpus-all corpus_all
    check fuzz-long fuzz_long
fi
if [ $QUICK = 1 ]; then
    for c in completion highlight spans format-lsp nvim-query nvim-rsml vim latency; do skip $c "--quick"; done
elif ! has "${RUST_ANALYZER:-rust-analyzer}"; then
    for c in completion highlight format-lsp latency; do skip $c "no rust-analyzer"; done
    check spans python3 scripts/check_spans.py
else
    check completion python3 scripts/check_completion.py
    check highlight python3 scripts/check_highlight.py
    check spans python3 scripts/check_spans.py
    if has nvim; then
        check format-lsp python3 scripts/check_format_lsp.py --nvim
    else
        check format-lsp python3 scripts/check_format_lsp.py
    fi
    check latency python3 scripts/check_latency.py
fi
if [ $QUICK = 0 ]; then
    if has nvim; then
        check nvim-query nvim --headless -u NONE -l scripts/check_nvim_query.lua
        check nvim-rsml nvim --headless -u NONE -l scripts/check_nvim_rsml.lua
    else
        skip nvim-query "no nvim"
        skip nvim-rsml "no nvim"
    fi
    if has vim; then
        check vim ./scripts/check_vim.sh
    else
        skip vim "no vim"
    fi
fi

echo
echo "Summary"
printf '%s\n' "${RESULTS[@]}"
if [ $FAILED = 1 ]; then
    echo "Some checks failed: see $LOGS/"
    exit 1
fi
echo "All checks passed."

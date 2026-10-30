#!/bin/sh
# Checks the Vim support of docs/editors.md ("Vim") in a headless Vim
# without plugins, and in Neovim too if it is installed:
#
#     scripts/check_vim.sh
#
# * `:%!rsml-fmt ..` and `'formatprg'` with `gq` format a messy file as
#   `rsml-fmt --rustfmt` does on the command line (and what they do with a
#   file that is not Rust, and with part of a file);
# * the format-on-save code of docs/editors.md (taken from there): the
#   buffer and the file are formatted, the cursor stays on its text,
#   saving adds no undo step and a formatted file is left alone, a file
#   that is not Rust is saved as it is, messages go to `:messages`;
# * editors/vim/after/syntax/rust.vim: the syntax item at positions in
#   `ml!` and `rsml!` blocks, with the file and without it, and that every
#   position outside the macros is highlighted as without it.
#
# The checks are scripts/check_vim.vim; the files are in target/vim-check.
set -u
cd "$(dirname "$0")/.."
cargo build -q -p rsml-fmt || exit 1
DIR=target/vim-check
rm -rf "$DIR"
mkdir -p "$DIR/src"
cat > "$DIR/Cargo.toml" << 'TOML'
[package]
name = "vim-check"
version = "0.0.0"
edition = "2021"

[workspace]
TOML
cat > "$DIR/messy.rs" << 'RS'
use rsml::{ml, rsml};
use x::{a9, a10};
fn   main( ) { let xs=vec![1,2,3];
  let n = ml!{ xs |>   .iter() |> .map(fun x ->x*2)|> .sum::<i64>() };
    let   total = n+1;
    let bad = ml! { 1 + };
  println!("{}", total) }

fn   shout(s: &str) -> String {
    ml! { s |> .to_uppercase   }
}

fn   rest( ) -> i64 {
    // cursor goes here
    ml! { let   y = 2 in y*  3 }
}
// the end
RS
printf 'fn main() {\n    let x = ml! { 1 };\n' > "$DIR/broken.rs"
cp "$DIR/messy.rs" "$DIR/src/cli.rs"
target/debug/rsml-fmt --rustfmt --stdin-filepath "$DIR/src/cli.rs" \
    < "$DIR/messy.rs" > "$DIR/expected.rs" 2> /dev/null || {
    echo "FAIL rsml-fmt --rustfmt on the messy sample"
    exit 1
}
export PATH="$PWD/target/debug:$PATH"

FAILED=0
SUMMARY=""
# run NAME COMMAND..: runs the checks in that editor and prints the results.
run() {
    name=$1
    shift
    rm -f "$DIR/result-$name.txt"
    "$@" -c "let g:rsml_check_name = '$name'" -S scripts/check_vim.vim < /dev/null > "$DIR/$name.log" 2>&1
    if [ -f "$DIR/result-$name.txt" ]; then
        echo "$name:"
        sed 's/^/  /' "$DIR/result-$name.txt"
        grep -q '^FAIL' "$DIR/result-$name.txt" && FAILED=1
        SUMMARY="$SUMMARY${SUMMARY:+, }$name $(tail -1 "$DIR/result-$name.txt" | cut -d' ' -f1-3)"
    else
        echo "FAIL $name did not finish the checks:"
        cat "$DIR/$name.log"
        FAILED=1
    fi
}

if command -v vim > /dev/null 2>&1; then
    run vim vim -Nu NONE -i NONE -es
else
    echo "no vim"
    FAILED=1
fi
if command -v nvim > /dev/null 2>&1; then
    run nvim nvim --clean -i NONE -es
fi
echo
echo "checks passed: $SUMMARY"
exit $FAILED

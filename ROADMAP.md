# rsml roadmap

What is left after the 2026-10-28/29 work (eight review rounds). Ordered
roughly by value; each item says what is known about it.

## Before a first release

- **Push** to github.com/uhhJoyz/rsml-lang, then
  `cargo publish --workspace` (license, the `rsml-lang` package name and
  `repository` are set; the Quick start already gives the crates.io forms).
- **README links on docs.rs**: the crate docs *are* the README, and its
  relative links (`docs/notation.md`, `examples/toy/README.md`) resolve
  against the docs.rs page, so they 404 there. crates.io rewrites them to
  the repository; docs.rs does not. Absolute GitHub links would fix both.
- **CI** running `scripts/check_all.sh --quick` (rust-analyzer/nvim checks
  need those tools on the runner).
- **MSRV CI** with an actual Rust 1.88 toolchain (today only clippy's
  `incompatible_msrv` and let-chains guard it).

## Editors

- **VS Code**: color `(* *)` comments (a TextMate injection grammar in a tiny
  extension). Unverified so far because there is no headless way to check
  rendering; could be checked with `vscode-textmate` in Node.
- **Helix / Zed**: comment queries (Helix needs a full copy of the Rust
  highlights query, no `; extends`); formatter setup is documented but not
  run in either editor.
- **conform.nvim**: documented, not executed (plugin not installed here).
- **Vim LSP plugins** (vim-lsp, coc.nvim, ALE): settings documented from
  their docs, none run. Vim formatting, format-on-save and the syntax file
  are verified (`scripts/check_vim.sh`).
- **Vim filter/`gq` on a file that does not parse** empties the buffer
  (`u` restores it; Vim replaces the range even when the command fails).
  The documented format-on-save function avoids this; a `--keep-on-error`
  mode that echoes the input back on failure would make the one-liners safe
  too (must not change the stdin contract rust-analyzer/Helix rely on).
- **Vim syntax file approximations**: `(* *)` with more than two nested
  paren levels or a `)` inside a string, spaced `( * c * )`, and Rust items
  inside `rsml!` are also scanned for OCaml keywords.
- **Generated names in completion**: 4 positions still list `__rsml_*`
  (`function` case bodies; `f n ‸` after a variable for 3+ params). Proven
  unavoidable with today's rust-analyzer (it lists locals without a hygiene
  check, `hir-def/src/resolver.rs`); an upstream rust-analyzer change
  (filter locals whose hygiene differs from the cursor's) would fix it.
- **Ranking regression** at the last argument after a *variable* argument of
  a function with mixed parameter types (Phase 7 trade-off).
- **rust-analyzer's own diagnostics** point at the whole `ml!`; rustc's
  (flycheck) point at the token. Upstream limitation.
- **Inlay hints** are never shown inside `!` macro calls (rust-analyzer).
- **Big single `rsml!` blocks**: ~190 ms completion at 3,300 lines vs 16 ms
  plain Rust (identity-macro floor ~58 ms). Guidance: split above ~1,500
  lines. Ideas: an opt-level profile tip is documented; caching the
  adjacency fallback per token (not per pair) under rust-analyzer.
- **Cross-block arity**: functions in another `rsml!` block are of unknown
  arity (no partial application across blocks). A way to declare them
  (`external`-like import of a sibling block's signatures) would help split
  files.

## Language

- **Loop labels** (`'outer:` / `break 'outer`).
- **Range patterns with non-literal bounds** (`i32::MIN..=0`).
- **Design question: `a..b` in patterns.** In a pattern `..` is OCaml's
  inclusive range however it is spaced, so a Rust habit `0..10 -> ..`
  silently matches 10 too (Rust's own exclusive range pattern would not).
  Options: keep it (documented in notation.md and toy 075), make the
  unspaced form an error asking for `a .. b` (OCaml) or `a..=b`, or follow
  the adjacency rule and read unspaced `a..b` as Rust's exclusive range.
- `x?` as a callee (`f(x)?(y)` style); `[a, b]` in *patterns* not flagged
  like it is in expressions.
- `Type::name` arity for functions from trait impls (only inherent ones are
  known).
- Bounds on `let rec` type variables (`'a` can't say `Display`); today use an
  `rsml!` item with a Rust signature.
- Lifetimes in `let rec` signatures: `(a : &'a str)` makes the nested `fn`
  use an undeclared `'a` (rustc E0261), and `&'a` always reads as a
  lifetime, so a reference to a type variable cannot be written (toys 101,
  102 show the workarounds).
- Trailing comma kept in method-call args under rust-analyzer
  (`xs.push(1, ‸)`).
- Edition 2015 (expansions use `::core` paths); low priority.
- `try .. with` / `raise` stay unsupported by design (use `Result`/`?`).

## Formatter

- `ml!` blocks inside `#[rust]` bodies are not formatted.
- A trailing `//` comment breaks the group it ends (`x <- y + 1 // c`
  prints over two lines); an option to let a group ignore a final `//`.
- `[// c` puts the first item at nested indentation (fixed only for `(`).
- The fuzzer keeps `rust_items = preserve` (rustfmt inside `rsml!` is only
  covered by the corpus test); add a sampled rustfmt mode.

## Errors

- rustc's own *suggestions* inside macro input can be nonsense
  (`..clone()0` for `.map(.0)`); investigate span choices that avoid them.

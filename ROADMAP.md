# rsml roadmap

What is left, as of 2026-10-30, grouped by the release it belongs to.
Each item says what is known, how it could be done, and a rough size:
**S** (an hour or two, one area), **M** (a day: parser or codegen plus
formatter, docs, a toy and the editor checks), **L** (several days, or
work outside this repository). **Decision** marks what needs the owner's
call before work starts.

Every change ends with `scripts/check_all.sh` (all 20 checks, not only
`--quick`), as CI runs only part of it.

**Convention** (the owner's rule, 2026-10-30): where OCaml and Rust give
the same notation different meanings, rsml takes Rust's. So `..` is
Rust's exclusive range in patterns too, and OCaml's `s.[i]` is an error.
OCaml-only notations keep their OCaml meaning (`!r` dereferences, `^`
concatenates, `for i = a to b` includes `b`; `til` is the exclusive
form); `lsr` and `asr` are both Rust's `>>` (a documented limitation).
Ask the owner when a new case comes up.

## Releases

0.1.0 and 0.2.0 are on crates.io. 0.2.0 added Rust's closures (`|x| e`)
and guards (`p if c`), nested `match`es without parentheses, Rust's `use`
in place of `open` (no longer a keyword), and the formatter's new layout
(`block_indent = 0`, `hug_fn_body`, `indent_rsml_items = false`). A
release: `cargo publish --workspace` (it orders rsml-compiler,
rsml-macros, rsml-fmt, rsml-lang), tag `vX.Y.Z`, and check the crates.io
pages (README, links, keywords) and that docs.rs built `rsml-lang`.

## 0.2.x: no change in what accepted code means

### Editors

- **Hidden macros outside Neovim**: `editors/nvim/lua/rsml` hides `ml! {`
  and `}`, marks their lines and shows `- RSML -` in the statusline. Vim
  could do the same with `conceal` syntax items and a statusline function;
  VS Code with decorations in the extension planned below. M each.

- **VS Code: color `(* *)` comments.** A tiny extension with a TextMate
  injection grammar (`injectTo: ["source.rust"]`) that starts after
  `ml!`/`rsml!` and an opening bracket, nests brackets with
  `include: "$self"`, and marks `(* .. *)` as `comment.block`. TextMate
  matches line by line, so the same limits as the Vim syntax file apply.
  Checkable headless with `vscode-textmate` and `vscode-oniguruma` in Node
  (tokenize a sample, compare scopes as `scripts/check_vim.vim` does). M,
  plus publishing to the Marketplace / Open VSX.
- **Helix and Zed: comment queries.** Helix has no `; extends`, so it needs
  a full copy of the Rust highlights query plus the rsml patterns (and
  updating with Helix); Zed takes a language extension. Neither editor has
  a headless highlight dump; Helix's tree-sitter queries can be run with
  the `tree-sitter` CLI (`tree-sitter query`) against the Rust grammar. M
  each.
- **Formatter setups not run**: conform.nvim, vim-lsp, coc.nvim, ALE,
  Helix and Zed are documented from their docs only. Each can be run
  headless the way `scripts/check_format_lsp.py --nvim` runs Neovim's
  client (plugin cloned into a temporary `packpath`). S each.
- **Neovim checks in CI**: `nvim-query` needs Neovim 0.11+ and the Rust
  tree-sitter parser, and `format-lsp --nvim` Neovim 0.12; Ubuntu's
  Neovim is older. Download a release tarball and build the parser with
  `tree-sitter build`. S.
- **Vim syntax file approximations**: `(* *)` with more than two nested
  paren levels or a `)` inside a string, spaced `( * c * )`, and Rust items
  inside `rsml!` are also scanned for OCaml keywords. Fixing the first two
  needs a region per nesting level (or giving up on patterns); probably
  not worth it.
- **Generated names in completion**: 4 positions still list `__rsml_*`
  (`function` case bodies; `f n ‸` after a variable for 3+ params).
  Unavoidable with today's rust-analyzer (it lists locals without a
  hygiene check, `hir-def/src/resolver.rs`). Next step: an upstream issue
  with a minimal macro that shows it, proposing to filter locals whose
  hygiene differs from the cursor's. S here, L upstream.
- **Ranking regression** at the last argument after a *variable* argument
  of a known function with mixed parameter types: there, completion ranks
  by the *first* parameter's type (a Phase 7 trade-off: under
  rust-analyzer, arguments are bound with `let __rsml_aN` before the
  closure so that the fake and real expansions line up). Something to
  try: lay the bindings out so that the cursor's argument is the one the
  expected type is taken from. M, with `scripts/check_completion.py`'s
  ranking cases (`a<b`) as the judge.
- **Upstream rust-analyzer limits**: its own diagnostics point at the
  whole `ml!` (rustc's, through flycheck, point at the token), and inlay
  hints are never shown inside `!` macro calls. Issues to file, nothing
  to do here.
- **Big single `rsml!` blocks**: ~190 ms completion at 3,300 lines vs 16 ms
  plain Rust (identity-macro floor ~58 ms). Guidance: split above ~1,500
  lines. Next idea: cache the adjacency fallback per token instead of per
  pair under rust-analyzer (`lexer::adjacent` parses RA's span `Debug`
  text). M; `scripts/check_latency.py` measures it.
- **Cross-block arity**: functions in another `rsml!` block are of unknown
  arity (no partial application across blocks) unless declared with
  `external`. A shorter way: `external use other_module::{f, g}` reading
  shapes from a `#[doc(hidden)] const` table that every `rsml!` block
  emits (`__RSML_SHAPES_f: usize`)? Needs a design that works without
  type information. **Decision**, then M.

### Language (additions only)

- **OCaml-only keywords as names**: `to`, `end`, `done`, `of`, `val`,
  `type`, `module`, `lazy`, .. cannot name a variable (`r#to` works;
  `open` is a name since `use` replaced it). Most could be names outside
  the positions where they are keywords (`to` only follows a `for` bound).
  Contextual keywords change what the parser must look ahead at;
  **Decision** on which, then M.
- **Nested `match` evidence** (`rsml-compiler/src/arms.rs`): a case moves
  to the enclosing `match` by what its pattern shows of its type, or when
  the inner `match` already matches it. Unqualified variants (`use E::*`)
  and struct patterns show nothing, and a type alias would count as another
  type. A `type_key` for a single-segment constructor seen qualified
  elsewhere in the same `match` would help the first. S.

- **Mutual tail recursion**: `let rec f .. and g ..` stays recursive
  (only a function calling itself in tail position is a loop). Several
  functions could share one loop over an `enum` of their argument lists.
  M.
- **Labels for `external`**: `external f : x:int -> y:int -> int` could
  declare labels for a Rust function the macro cannot see. Labels work
  for `let`, `let rec`, `let f = fun ~x ..` and `fn`s of the same
  `rsml!` today. S.
- **OCaml's optional arguments** (`?x`, `?(x = 1)`) are not supported;
  `Option` parameters are the Rust way. **Decision** if wanted.

- **Loop labels** (`'outer: for i = 0 to n do .. done`, `break 'outer`,
  `continue 'outer`). Rust's lexer hands `'outer` over as a lifetime, so
  the parser sees it before `:` and a loop keyword, which no other
  construct starts with. Codegen is direct; the formatter, the Vim syntax
  file (`'outer` stays a lifetime color), a toy and completion cases go
  with it. M.
- **Range patterns with non-literal bounds** (`i32::MIN..=0`,
  `'a'..='z'` works already): accept a path pattern as a bound in
  `finish_lit_pat` and its counterpart for a leading path. S–M.
- **`Type::name` arity for trait impl functions**: items.rs records
  inherent `impl` functions only; record `impl Trait for Type` ones in
  the same block too (ambiguous names across traits stay unknown). S–M.
- **Bounds on `let rec` type variables** (`'a` can't say `Display`;
  today: an `rsml!` item with a Rust signature). Syntax is the hard part:
  OCaml has nothing for it. **Decision**, then M.
- **Lifetimes in `let rec` signatures**: `(a : &'a str)` makes the nested
  `fn` use an undeclared `'a` (rustc E0261), and `&'a` always reads as a
  lifetime, so a reference to a type variable cannot be written (toys 101,
  102 show the workarounds). Declare every lifetime the signature names on
  the nested `fn`; for `&'a` as a type variable there is no unambiguous
  spelling. M.
- **Edition 2015**: the expansions name `::core::..`. Would need
  `$crate`-style paths through a `macro_rules!` shim in `rsml`. Low value.
- `try .. with` / `raise` stay unsupported by design (use `Result`/`?`).

### Formatter

- **`fn f() { ml! {` and rustfmt**: plain rustfmt (`cargo fmt`) undoes the
  `hug_fn_body` layout, so a project formatting with both churns. The fix
  is upstream-shaped (rustfmt leaves macro calls alone but not the block
  around them); until then the docs say to run `rsml-fmt --rustfmt`.
  `check_all.sh` runs rustfmt only on the crates without rsml code for this
  reason.

- `ml!` blocks inside `#[rust]` bodies are not formatted: the formatter
  treats those bodies as Rust (rustfmt formats them with
  `rust_items = "rustfmt"`) and does not look for blocks in them; it could
  run its file-level block search over them. S–M.
- A trailing `//` comment breaks the group it ends (`x <- y + 1 // c`
  prints over two lines). An option (`trailing_comment = "keep_line"`?)
  letting a group ignore a final `//`, the fuzzer checking idempotence. M.
- `[// c` puts the first item at nested indentation (fixed for `(` only;
  the same `path_comments`-style fix in the list printer). S.
- The fuzzer keeps `rust_items = preserve`, so rustfmt inside `rsml!` is
  covered by the corpus test only. Add rustfmt to the rotating configs,
  with the rustfmt binary required in CI. S.

### Errors

- rustc's own *suggestions* inside macro input can be nonsense
  (`..clone()0` for `.map(.0)`): rustc suggests edits at spans of the
  input that the expansion reuses. Try giving the generated tokens around
  the reused ones `Span::mixed_site()` or call-site spans so that the
  suggestion has nothing to anchor to. M (investigation first).

### Project

- **Release automation**: a workflow on `v*` tags that runs
  `check_all.sh --quick` and `cargo publish --workspace` with crates.io
  trusted publishing (no stored token). S.
- **Doc links per release**: README links go to `blob/main`, so the
  crates.io page of an old version shows today's docs. Pointing them at
  the tag (`blob/v0.2.0`) at release time would fix it; `check_docs.py`
  would accept both. S.

## 0.3: changes that can break code

Nothing is planned yet. What would go here: any change to what an
expression expands to that is visible to the caller (types, evaluation
order, moves), or to the formatter's default layout.

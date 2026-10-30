# Editor support

rsml's macros are ordinary procedural macros, so rust-analyzer understands
the code inside `ml! { .. }` and `rsml! { .. }` through their expansion:
semantic highlighting, hover, go to definition, diagnostics and completion
all work inside OCaml code, in any editor that runs rust-analyzer (Neovim
with blink.cmp or nvim-cmp, VS Code, Helix, Zed, ..). Inlay hints do not:
rust-analyzer shows none inside a macro call written with `!`, any macro
(not even for `let q = 5u8;` in a `macro_rules!` call), so the types of
`let`s in OCaml code are not shown inline. Hover shows them. For every
notation the macros accept, and what completes where, see
[notation.md](notation.md). For files of thousands of lines, see
[Big files](#big-files).

## Setup

Nothing rsml-specific is needed beyond rust-analyzer with procedural
macros enabled (the default). For Neovim 0.11+ with blink.cmp:

```lua
vim.lsp.config("rust-analyzer", {
    cmd = { "rust-analyzer" },
    filetypes = { "rust" },
    root_markers = { "Cargo.toml", "rust-project.json" },
    settings = {
        ["rust-analyzer"] = {
            procMacro = { enable = true },
            check = { command = "clippy" },
        },
    },
})
vim.lsp.enable("rust-analyzer")

require("blink.cmp").setup({
    sources = { default = { "lsp", "path", "snippets", "buffer" } },
})
```

After updating rsml, restart rust-analyzer so that it loads the new
macros: `:lsp restart` in Neovim 0.12, `:LspRestart` with nvim-lspconfig,
"rust-analyzer: Restart server" in VS Code.

If completion inside a macro offers only words from the buffer, the `lsp`
source returned nothing and blink.cmp fell back to `buffer`: rust-analyzer
is still loading, procedural macros are disabled, or the macros are an old
build.

## Highlighting

rust-analyzer colors a token inside a macro call by what it becomes in the
expansion: `+` in `a + b` stays a `+` and is colored as an operator, `x`
becomes a variable, `f` a function. So that `|>`, `@@`, `>>`, `::`, `**`,
user-defined operators and the like, which become calls, are colored the
same way, the expansion of a block that uses them starts with
`const _: usize = 1 + 1 + ..;`, one `+` per operator character, each
carrying that character's span.

OCaml keywords that Rust has too (`let`, `in`, `if`, `else`, `match`,
`for`, `while`, `do`, `as`) are colored as keywords anyway, and `match`,
`if`, `for` and `while` become the same Rust keywords. The others have
no Rust token in the expansion, so the expansion also starts with a
`const _: () = { .. };` holding one for each: `if true {}` with its `if`
at a `then` or `when`, `match () { .. }` for `with` and `function`,
`loop { break }` for `done`, `to` and `downto`, `let _ = 0;` for `fun`,
`rec`, `and`, `begin`, `end`, `open` and `external`. Control-flow
keywords are thus colored as control flow, and the rest as keywords. The
`|` of a match arm or or-pattern gets a `|` operator (`0 | 0`), and each
`->` a `=>` (the generated match arms' own `=>` carry the arms' `->`).
Both constants are evaluated at compile time and generate no code; their
other tokens point at the first token of the code, so they do not seem to
contain it (clippy skips constants).

`(* comments *)` are not Rust comments: Rust reads them as a group of
tokens, which the macros drop, and editors color them as code. Prefer
`//` and `/* */`. They are real Rust comments, colored as comments
everywhere, and they may contain quotes and apostrophes, which can break
a `(* .. *)` comment (`(* say "hi *)` does not lex). rsml-fmt rewrites
`(* *)` comments into Rust ones with `comments = "rust"`, and Neovim (below)
and Vim ([Highlighting in Vim](#highlighting-in-vim)) can be taught to
color them.

### Neovim: coloring `(* *)` comments

rust-analyzer gives the tokens of a `(* .. *)` comment no semantic token
(the macros drop them), so tree-sitter's highlighting shows, and to
tree-sitter's Rust grammar the comment is a token tree that starts with
`(*` and ends with `*)`. `editors/nvim/queries/rust/highlights.scm`
extends Neovim's Rust highlights to color such token trees inside macro
calls as `@comment`, above rust-analyzer's semantic tokens (which would
still color a Rust keyword inside the comment), and leaves `( * )`,
`( ** )` and `( *x)` alone. Install it with either of:

```sh
mkdir -p ~/.config/nvim/after/queries/rust
cp editors/nvim/queries/rust/highlights.scm ~/.config/nvim/after/queries/rust/
```

```lua
vim.opt.runtimepath:append("/path/to/rsml/editors/nvim")
```

It needs the Rust tree-sitter parser (nvim-treesitter's, for example) and
highlighting by tree-sitter (`vim.treesitter.start()`, which
nvim-treesitter's highlight module does). `scripts/check_nvim_query.lua`
opens a file with `ml!` and `rsml!` blocks in a headless Neovim with the
query on the runtimepath and checks the captures at comment and
non-comment positions (with Neovim 0.12; both installations above were
checked):

```sh
nvim --headless -u NONE -l scripts/check_nvim_query.lua
```

Helix and Zed were not checked, and the query is not written for them.

## Big files

rust-analyzer expands a macro call again whenever its contents change, and
twice for each completion, so the time a completion takes grows with the
macro call it is in, not with the file. `scripts/check_latency.py`
measures it: the same functions as plain Rust, in one whole-file `rsml!`,
and in one `rsml!` per ten pairs of functions (about 110 lines), with the
median time of a completion after `xs.` in the last function (Apple M4
Pro, rust-analyzer of 2026-07-18):

| lines | plain Rust | one `rsml!` | one `rsml!`, macros optimized | one `rsml!` per ~110 lines |
|------:|-----------:|------------:|------------------------------:|---------------------------:|
|   500 |       8 ms |       37 ms |                         29 ms |                      14 ms |
|  1500 |      11 ms |       97 ms |                         72 ms |                      13 ms |
|  3300 |      16 ms |      203 ms |                        150 ms |                      21 ms |

So a whole file in one `rsml!` is fine up to about 1500 lines (under
100 ms a completion). Beyond that, use one `rsml!` per group of related
items (a type and its `impl`, a few functions that call each other): the
cost is then that of the block being edited, as for plain Rust. Functions of the same `rsml!` call
each other with their arity known (`add 1` is a partial application);
across blocks they are Rust functions of unknown arity, called with all
their arguments (`add 1 2` is `add(1, 2)` either way).

rust-analyzer builds procedural macros unoptimized, as `cargo check` does.
Optimizing rsml's crates takes about a quarter off the time of each
completion in a big `rsml!` (and speeds up `cargo build` the same way):

```toml
# In your Cargo.toml.
[profile.dev.package.rsml-macros]
opt-level = 3
[profile.dev.package.rsml-compiler]
opt-level = 3
[profile.dev.package.proc-macro2]
opt-level = 3
```

`python3 scripts/check_latency.py --opt` measures with these.

## Formatting on save

`rsml-fmt --rustfmt` formats a whole file: rustfmt for the Rust, then the
OCaml in `ml!` and `rsml!` (and, through rustfmt, the Rust inside
`rsml!`). It reads the file on stdin and writes the result to stdout, so
it can stand in for rustfmt wherever an editor runs rustfmt. If the file
is not valid Rust it writes nothing and exits with status 2, and the
editor keeps the buffer. rustfmt's edition comes from the nearest
`Cargo.toml` and its options from the nearest `rustfmt.toml`, found from
the current directory, or from `--stdin-filepath`'s when given (see
[rsml-fmt's README](../rsml-fmt/README.md)).

**rust-analyzer** (any editor that formats through it). rust-analyzer runs
`rustfmt.overrideCommand` with the file's contents on stdin, in the file's
directory, with no other arguments, and turns the output into edits.

VS Code, in `settings.json`:

```json
"rust-analyzer.rustfmt.overrideCommand": ["rsml-fmt", "--rustfmt"]
```

Neovim 0.11+, with the built-in client (`vim.lsp.buf.format()`, or
`format on save` from your config):

```lua
vim.lsp.config("rust-analyzer", {
    cmd = { "rust-analyzer" },
    filetypes = { "rust" },
    root_markers = { "Cargo.toml", "rust-project.json" },
    settings = {
        ["rust-analyzer"] = {
            rustfmt = { overrideCommand = { "rsml-fmt", "--rustfmt" } },
        },
    },
})
```

`scripts/check_format_lsp.py` checks this: it starts rust-analyzer with
that setting, asks it to format a messy file with `ml!` blocks and a
whole-file `rsml!`, and compares the result (it also checks that the
crate's edition was used and that a file that does not parse gets no
edits). `--nvim` also runs the Neovim configuration above in a headless
Neovim 0.12 and checks the buffer; `-v` prints how rust-analyzer ran the
command. The VS Code setting is the same rust-analyzer setting, but VS
Code itself was not run.

```sh
python3 scripts/check_format_lsp.py --nvim
```

**conform.nvim**, as a custom formatter (not checked: conform.nvim was not
run):

```lua
require("conform").setup({
    formatters_by_ft = { rust = { "rsml_fmt" } },
    formatters = {
        rsml_fmt = {
            command = "rsml-fmt",
            args = { "--rustfmt", "--stdin-filepath", "$FILENAME" },
            stdin = true,
        },
    },
})
```

**Helix**, in `languages.toml` (Helix 25.07 accepts it: `hx --health
rust` lists the formatter; formatting was not run in Helix):

```toml
[[language]]
name = "rust"
formatter = { command = "rsml-fmt", args = ["--rustfmt"] }
```

**Zed** was not checked. Through rust-analyzer, its
`"lsp": { "rust-analyzer": { "initialization_options": { "rustfmt": {
"overrideCommand": ["rsml-fmt", "--rustfmt"] } } } }` setting should
behave as above.

**Vim** runs rsml-fmt itself: see [Formatting in Vim](#formatting-in-vim).

`scripts/check_highlight.py` asks rust-analyzer for the semantic tokens of
a sample using every keyword and operator (`scripts/highlight_cases.txt`)
and checks each one's type:

```sh
python3 scripts/check_highlight.py           # all cases
python3 scripts/check_highlight.py -v match  # print every token of the `match` cases
```

## Vim

Vim has no LSP client of its own, so without a plugin it gets no
completion or hover; it does get formatting and highlighting. What is
said below about Vim was checked with Vim 9.1 and no plugins (`vim -Nu
NONE`), by `scripts/check_vim.sh`, except for the plugins of
[LSP plugins](#lsp-plugins), which were not run.

### Formatting in Vim

`rsml-fmt --rustfmt` works as a filter on the whole buffer:

```vim
:%!rsml-fmt --rustfmt --keep-on-error --stdin-filepath %:p 2>/dev/null
```

or, for `gq`, as `'formatprg'`, and `gggqG` formats the file:

```vim
setlocal formatprg=rsml-fmt\ --rustfmt\ --keep-on-error\ --stdin-filepath\ %:p\ 2>/dev/null
```

Both format the file exactly as `rsml-fmt --rustfmt` does on the command
line. `%:p` is the file's path, from which rsml-fmt finds its crate's
`Cargo.toml` (the edition), `rustfmt.toml` and `rsmlfmt.toml`. Without
`2>/dev/null`, rsml-fmt's warnings (a block left as written because it
does not parse) would end up in the buffer: Vim's filters read what a
command writes to stderr too (for sh-like shells, `'shellredir'`
defaults to `>%s 2>&1`; on Windows, write `2>nul`). Two things to know:

* If the file is not valid Rust, rsml-fmt exits with status 2 (Vim says
  "shell returned 2"), and `--keep-on-error` makes it write the file back
  as it was: Vim replaces the lines with whatever the command writes, so
  without the option they would be replaced with nothing (`u` brings them
  back).
* `gq` on part of the file (`gqip`) hands rsml-fmt only those lines, which
  it formats as a file of their own, or, when they are not valid Rust on
  their own, leaves as they are.

To format on save, put this in your vimrc. It runs rsml-fmt on the whole
buffer and, if that succeeds, replaces only the lines from the first to
the last that changed: the cursor stays on its text (on the same row of
the window), a file already formatted is left alone (not
even an undo step), and the formatting joins the undo step of the last
change, so `u` after saving undoes the change and its formatting
together. If the file does not parse, the buffer is saved as it is.
rsml-fmt's messages go to `:messages`, not into the buffer. `:RsmlFmt`
formats without saving, as an undo step of its own, and `:noautocmd w`
saves without formatting.

```vim
" rsml-fmt on the whole buffer, changing only the lines that differ.
function! RsmlFmt(on_write) abort
  let lines = getline(1, '$')
  let path = expand('%:p')
  let err = tempname()
  let out = systemlist('rsml-fmt --rustfmt'
        \ . (path ==# '' ? '' : ' --stdin-filepath ' . shellescape(path))
        \ . ' 2>' . shellescape(err), lines)
  let failed = v:shell_error
  for msg in filereadable(err) ? readfile(err) : []
    echohl WarningMsg | echomsg msg | echohl None
  endfor
  call delete(err)
  if failed || out ==# lines
    return
  endif
  " lines[a : -b - 1] become out[a : -b - 1].
  let a = 0
  while a < len(lines) && a < len(out) && lines[a] ==# out[a]
    let a += 1
  endwhile
  let b = 0
  while b < len(lines) - a && b < len(out) - a && lines[-1 - b] ==# out[-1 - b]
    let b += 1
  endwhile
  let old = len(lines) - a - b
  let new = len(out) - b > a ? out[a : len(out) - b - 1] : []
  let view = winsaveview()
  if a:on_write
    silent! undojoin
  endif
  let n = min([old, len(new)])
  if n > 0
    call setline(a + 1, new[: n - 1])
  endif
  if old > n
    silent execute (a + n + 1) . ',' . (a + old) . 'delete _'
  elseif len(new) > n
    call append(a + n, new[n :])
  endif
  " The cursor stays on its text (in what changed, on the line with the
  " same text nearest to where it was), at its row of the window.
  let lnum = view.lnum
  if lnum > a + old
    let lnum += len(new) - old
  elseif lnum > a
    let k = lnum - 1 - a
    let best = -1
    for i in range(len(new))
      if new[i] ==# lines[lnum - 1] && (best < 0 || abs(i - k) < abs(best - k))
        let best = i
      endif
    endfor
    let lnum = best >= 0 ? a + 1 + best : max([min([lnum, a + len(new)]), 1])
  endif
  let view.topline = max([view.topline + lnum - view.lnum, 1])
  let view.lnum = lnum
  call winrestview(view)
endfunction
command! -bar RsmlFmt call RsmlFmt(0)
augroup rsml_fmt
  autocmd!
  autocmd BufWritePre *.rs call RsmlFmt(1)
augroup END
```

### Highlighting in Vim

Vim's Rust syntax (`$VIMRUNTIME/syntax/rust.vim`) reads the code inside
the macros as Rust, and that is mostly right: strings, characters, `'a`
(colored as a lifetime), numbers, `//` and `/* */` comments, operators
and the keywords Rust shares with OCaml (`let`, `in`, `if`, `else`,
`match`, `for`, `while`, `as`, `ref`, `mod`) are colored as in Rust, and
the code after a block is colored correctly. The rest of OCaml is not:
`then`, `with`, `fun`, `done` and the like are left uncolored, `(* *)`
comments are colored as code, and `do` as an error (a word Rust
reserves).

`editors/vim/after/syntax/rust.vim` adds to Vim's Rust syntax, inside the
bodies of `ml!` and `rsml!` calls only (written with `{}`, `()` or `[]`):

* `(* .. *)` comments, nested ones included, as comments. A comment
  starts with `(*` and ends with the `*)` that closes its parentheses, as
  the macros read it, so `( * )` and `( ** )` (operator sections) and
  `( *x)` or `(*x)` (dereferences) are not comments (the macros also
  take `( * c * )`, spaced, for a comment; this does not);
* `then`, `when`, `with` and `function` as conditionals; `do`, `done`,
  `to` and `downto` as loops; `fun`, `rec`, `and`, `begin`, `end`, `open`,
  `external`, `not` and `assert` as keywords; and `land`, `lor`, `lxor`,
  `lsl`, `lsr`, `asr` and `or` as operators. A Rust name spelled the same
  stays as it is after `.`, `::` or `r#` (`c.then(f)`, `r#end`), as does a
  macro (`assert!`);
* the words Vim's Rust syntax colors as reserved (`final`, `priv`, ..)
  as names, which they are in rsml.

Outside the macros nothing changes: a Rust `fun`, `with`, `done`, `to` or
`end` is a name as before. Install it with either of:

```sh
mkdir -p ~/.vim/after/syntax
cp editors/vim/after/syntax/rust.vim ~/.vim/after/syntax/
```

```vim
set runtimepath+=/path/to/rsml/editors/vim/after
```

(the second at the end of `'runtimepath'`, so that it runs after Vim's
own Rust syntax). Limits, as it works line by line with patterns: inside
an `rsml!` it colors the Rust items too (a field named `to` would be a
keyword); a comment is recognized when it has at most two levels of
parentheses inside it and none in a string; and a whole file in one
`rsml!` (one that starts with a flush-left `rsml! {`) is highlighted from
its start, which costs a little time on large files. In Neovim the file
works too (`scripts/check_vim.sh` runs its checks in Neovim 0.12 as well),
when Vim's syntax highlighting is used; with tree-sitter's, use the query
of [Neovim: coloring `(* *)` comments](#neovim-coloring---comments).

`scripts/check_vim.sh` runs a headless Vim without plugins (and Neovim, if
installed): the filter, `'formatprg'` and the format-on-save function above
(taken from this page) on messy and broken files, and the highlighting of a
sample with and without the syntax file, at OCaml positions and at every
position outside the macros:

```sh
scripts/check_vim.sh
```

### LSP plugins

vim-lsp, coc.nvim and ALE start rust-analyzer as any other client does,
so completion, hover and diagnostics should work inside the macros as
described on this page, and formatting through rust-analyzer with
`rustfmt.overrideCommand` set to `["rsml-fmt", "--rustfmt"]`. None of them
was run. The settings, from their documentation (coc-rust-analyzer's
`package.json` lists the setting):

* vim-lsp: `'workspace_config': {'rust-analyzer': {'rustfmt':
  {'overrideCommand': ['rsml-fmt', '--rustfmt']}}}` in
  `lsp#register_server()`, and `:LspDocumentFormatSync` (on save:
  `autocmd BufWritePre *.rs LspDocumentFormatSync`);
* coc.nvim, with coc-rust-analyzer: `"rust-analyzer.rustfmt.overrideCommand":
  ["rsml-fmt", "--rustfmt"]` in `coc-settings.json`;
* ALE: `let g:ale_rust_analyzer_config = {'rustfmt': {'overrideCommand':
  ['rsml-fmt', '--rustfmt']}}` for its `analyzer` linter. ALE's
  documentation of rust-analyzer does not mention formatting through it
  (ALE formats with fixers, and its `rustfmt` fixer runs rustfmt); the
  function above formats with any of them.

## Errors

rustc's errors point at the OCaml code they are about: a type mismatch at
the expression that has the wrong type (`len` in `xs |> .len`, `"a"` in
`if c then 1 else "a"`), an unknown method at its name, a missing match
arm at the scrutinee, an over-applied function at the function. Rust code
that the macros generate (a call's parentheses, a closure's `|x|`, a
block's braces, `move`) is located at the OCaml token it stands for, and
keeps the macro's context, so lints treat it as generated, as before.
Where an expression starts and ends with a token written in the OCaml
code, the error covers the whole of it (`function Some x -> x | None ->
0`); otherwise it points at one token, its first. Only rustc's
expansion is located this way; rust-analyzer's own diagnostics on the
macro (not the ones it relays from `cargo check`) still point at the
whole call.

The locals the macros make up are named for what they stand for, because
rustc's errors can show them: in `xs |> .iter() |> .for_each(.push(1))`,
"`receiver` is a `&` reference, so it cannot be borrowed as mutable",
pointing at the `.` of `.push`. A placeholder's parameter is `_0`, `_1`,
.., a composition's is `argument`, and a value that a partial application
computes once is `value1` (the list is in [notation.md](notation.md));
each is located at the code it stands for. A generated binding that a
closure may mutate through is `mut`, since the user could not add the
`mut`; and rustc's suggestion for a local function that mutates what it
captures, `let mut f x = ..`, is valid rsml.

`scripts/check_spans.py` builds a crate of erroneous snippets
(`scripts/span_cases.txt`) with `--message-format=json` and checks the
text of each error's primary span, that none falls back to the whole
macro call, and that no message shows a name starting with `__rsml_`:

```sh
python3 scripts/check_spans.py
```

## Completion

### How it works

rust-analyzer completes inside a macro call by expanding the call twice:
as written, and with a made-up identifier, the marker, inserted at the
cursor. In the second expansion the marker says what is being completed: a
name, a method, a field, a path segment. In the first, rust-analyzer takes
the token at the marker's offset: its scope gives the names offered, and
its context the expected type, whose values are listed first. (The offsets
are those of the tokens laid end to end; see `expand` and `analyze` in
rust-analyzer's `crates/ide-completion/src/context/analysis.rs`.) So the two
expansions must line up up to the cursor: the first must reach as far as
the marker's offset, with code of the same scope there. Reading the same
up to the cursor is the simplest way to line up.

While you type, the code as written is usually unfinished: `xs |> .`,
`Vec::`, `let y = x in`, `if c`, `match o with Option::`. A parser that
stopped at the first error would produce only a `compile_error!`, and
nothing would complete. So when the macros run inside rust-analyzer (and
only then), the parser recovers:

* a missing name after `.` or `::`, a missing expression, pattern or type
  (also in the arguments of std macros such as `println!`), and a missing
  `then`, `do`, `done`, `end`, `in`, `with` or `->` at the end of the code
  become a stand-in (`__rsml_missing`), and parsing goes on;
* `a;` with nothing after it is given the shape `a; b` will have, and a
  Rust-style call keeps a trailing `,` (`f(a,)`), the shape `f(a, b)`
  will have;
* the expansion is always a block, and the errors are reported after the
  code, as `compile_error!`s, so they do not shift anything before the
  cursor;
* lists are built with `Vec::from([..])`, which reads the same whether or
  not an item is being added (`vec![]` and `vec![x]` take different macro
  arms, which rust-analyzer cannot follow);
* a function whose arity the macro knows, given fewer arguments than it
  takes, gets its arguments bound to locals before the closure is made
  (closures and constants stay in place): for a `plus` of two parameters,
  `plus n` expands to `{ let a1 = n; move |p2| plus(a1, p2) }`, and
  `plus` alone to `{ move |p1, p2| plus(p1, p2) }` (a function `f` of
  `()` alone to `{ move || f() }`). Nothing before an argument then
  depends on the arguments after it, and for `plus ‸` the marker, in
  `let a1 = marker`, lines up with `p1`: the names around `plus` are in
  scope, and values of the type of `plus`'s first parameter come first.
  Sections and compositions bind their operands the same way.

rustc never sees any of this: under `cargo build`, `cargo check` and
clippy, unfinished code is still a single error with the usual message.

### What to expect

* **Every name completes once you type a character** of it: a value, a
  function, a path segment, a method, a field, a constructor, a type.
* **Arguments complete before you type anything**, after any function:
  `add ‸`, `plus ‸` (for a local `let plus x y`, a closure-typed
  parameter, a function of the same `rsml!` or `Self::make` in its
  `impl`), `plus ‸ 2`, `plus 1 ‸`, `plus(‸)`. After a function whose arity
  the macro knows, at an argument written OCaml style with more missing
  after it (`plus ‸`), values of the type of that argument's parameter are
  listed first, when the type is declared.
* **`.` and `::` complete immediately**, as trigger characters: methods
  after `xs.`, `xs |> .` and `.‸ xs`; items after `Vec::`, `Vec.`,
  `std::cmp::` and `Shape::`; variants in patterns after `Option::` and
  `std::cmp::Ordering::`.
* **Unfinished constructs complete** at the end of what you have written:
  `if ‸`, `while n > 0 do ‸`, `for x in xs do ‸`, `begin ‸`, `fun x -> ‸`,
  `match o with ‸`, `let x = ‸`, `(n : ‸)`, `return ‸`, `add(1, ‸)`,
  `xs[‸]`, `0..‸`, `unsafe { ‸ }`, and in the arguments of std macros:
  `println!("{}", ‸)`, `assert!(xs.‸, "empty")`, `vec![n; ‸]`.
* **Bodies under `#[rust]`** are Rust, and complete as Rust does.
* **Methods complete from their receiver.** In `.name receiver` form the
  receiver comes after the name, so complete the name once the receiver is
  there (`.it‸ xs`, `.‸ xs 1`), or write it pipeline style (`xs |> .it‸`).
  As an argument, the receiver's type can also come from the function
  that takes the closure: `twice (.‸) 1`, `xs |> .iter |> .map _ (_.‸)`.

### The spacing rule

A name after `.` is written right after it (`.len`, `xs.len`), as is the
name after a path's `::` (`Vec::new`, `geometry::area`). This is what lets
an unfinished `.` or `::` be told apart from a finished one: in `.‸ xs`
the receiver is `xs`, and in `geometry::‸ &shape` the argument is
`&shape`. `. len` and `xs. len` are errors.

### Limitations

* **Generated names** (`__rsml_..`) are offered in a few places.
  rust-analyzer offers every local in scope at the cursor: macro hygiene
  keeps the macros' locals apart from yours when names are resolved (your
  `x` never means a generated one), but not from completion, which lists
  locals by name only. The macros keep their locals out of scope of your
  code where they can (the arguments above; `{ p with x = .. }` copies `p`
  into a local named `p` too, which your code does not see;
  `let a = .. and b = ..` binds both names from one tuple). They cannot
  inside a closure they make that contains your code: a case body of
  `function` (`function Some x -> ‸ | None -> 0`), where the value matched
  is the closure's parameter, and a closure passed to a section, a
  composition or a partial application. Also, in a partial application or
  a section that still lacks arguments, at an argument after a variable or
  computed one (`f n ‸`, for an `f` of three parameters), the arguments
  before it are locals of their own, so that the expansion lines up for
  completion; and at the last argument after constants, when the
  function's name is long, the marker falls inside the closure (see the
  next point).
* **The expected type at the last argument** of a known function
  (`plus 1 ‸`, `plus(1, ‸)`) or before written arguments (`plus ‸ 2`) is
  not reliably that of the argument's parameter, so its values are not
  always listed first (the names offered are right). There rust-analyzer
  lines up `plus 1`, a closure, with the full call `plus(1, marker)`, and
  no closure starts the way a call does: where the marker falls depends
  on the length of what comes before it, and after a variable argument it
  falls on that argument's local, whose type it takes. The macros read
  `plus(‸)` as `plus ()`, whose first argument is `()`.
* **A function used as a value is a closure** in rust-analyzer's view
  when its arity is known and is not one (see above), so hovering a
  variable bound to a function of the same `rsml!` (`let f = add3 in ..`)
  shows `impl Fn(i64, i64, i64) -> i64` rather than
  `fn add3(i64, i64, i64) -> i64`.
* Field names in a record *pattern* (`Point { ‸ } -> ..`) are not offered:
  OCaml record patterns may leave fields out, so the expansion always ends
  them with `..`, and rust-analyzer only offers missing fields.
* The types of an `external` declaration are not completed; only their
  arrows matter, so they are not part of the expansion.
* `a;` at the very end of a block is a value-less sequence in
  rust-analyzer's view (`{ a; () }`), while rustc keeps OCaml's meaning
  (the value of `a`). Only the types rust-analyzer shows for such a block
  differ.

### Checking it

`scripts/check_completion.py` runs rust-analyzer over LSP on every case in
`scripts/completion_cases.txt` and checks the labels it offers, and their
order where a case says so (`n` before `w`: `n` is listed first). It also
checks that no case offers a generated local (`__rsml_..`), except the few
listed in the script's `GENERATED_OFFERED`, each with its reason (the
places named in [Limitations](#limitations)):

```sh
python3 scripts/check_completion.py          # all cases
python3 scripts/check_completion.py match    # cases whose name contains "match"
python3 scripts/check_completion.py -v       # also print what was offered
```

It needs `rust-analyzer` on the `PATH` (or `RUST_ANALYZER=/path/to/it`)
and builds a small crate under `target/completion-check`. The first run
waits for rust-analyzer to build the macros.

### Every checked position

The code is the body of an `ml!` block in a function with the locals
`xs: Vec<i64>`, `s: &str`, `word: String`, `p: Point`, `o: Option<String>`,
`n: i64`, `shape: Shape` and `ord: std::cmp::Ordering`, next to
`Point { x, y }` (with a method `norm`), `enum Shape { Circle, Rect }`,
`fn add`, `fn twice` and `mod geometry { fn area, const ORIGIN }`; or, when
it starts with `rsml!`, an item.

**Names**

| Position | Code | Offers (at least) |
|---|---|---|
| local being typed | `let total = 1 in tot‸` | `total` |
| after `in` | `let total = 1 in ‸` | `total` |
| parameter of an rsml! function | `rsml! { fn f(items: Vec<i64>) -> usize { ite‸ } }` | `items` |
| empty rsml! body | `rsml! { fn f(items: Vec<i64>) -> usize { ‸ } }` | `items` |
| other rsml! functions | `rsml! { fn helper_one() -> i64 { 1 } fn f() -> i64 { help‸ } }` | `helper_one` |
| a raw identifier (labelled without its `r#`) | `let r#type = 1 in r#ty‸` | `type` |
| a word Rust reserves | `let gen = 1 in ge‸` | `gen` |

**Paths**

| Position | Code | Offers (at least) |
|---|---|---|
| `Type::` | `Vec::‸` | `new`, `with_capacity` |
| `Type.` | `Vec.‸` | `new`, `with_capacity` |
| module | `geometry::‸` | `area`, `ORIGIN` |
| nested std module | `std::cmp::‸` | `max`, `min`, `Ordering` |
| being typed | `std::cmp::ma‸` | `max` |
| leading `::` | `::std::‸` | `cmp` |
| enum variants | `Shape::‸` | `Circle`, `Rect` |
| enum variants, OCaml style | `Shape.‸` | `Circle`, `Rect` |
| after generic arguments | `Vec::<i64>::‸` | `new` |
| head of an application | `geometry::ar‸ &shape` | `area` |
| empty head of an application | `geometry::‸ &shape` | `area` |
| empty head with two arguments | `std::cmp::‸ 1 2` | `max`, `min` |

**Methods**

| Position | Code | Offers (at least) |
|---|---|---|
| after a value | `xs.‸` | `len`, `iter`, `push` |
| being typed | `xs.it‸` | `iter` |
| after a call | `xs.iter().‸` | `map`, `filter` |
| chain broken across lines | `xs .iter() .‸` | `map`, `filter` |
| name before Rust-style arguments | `xs.pu‸(1)` | `push` |
| fields and methods of a struct | `p.‸` | `x`, `y`, `norm` |
| on an Option | `o.‸` | `unwrap_or`, `is_some` |
| on a parenthesized expression | `(n + 1).‸` | `abs`, `pow` |
| on an annotated expression | `(n : i64).‸` | `abs` |

**Method refs**

| Position | Code | Offers (at least) |
|---|---|---|
| `.name receiver` | `.‸ xs` | `len`, `iter`, `push` |
| `.name receiver`, name being typed | `.it‸ xs` | `iter` |
| `.name receiver argument` | `.‸ xs 1` | `push`, `insert` |
| being typed, with an argument | `.pu‸ xs 1` | `push` |
| receiver in parentheses | `.‸ (s.trim())` | `len`, `trim` |
| struct receiver | `.‸ p` | `norm` |
| after `\|>` | `xs \|> .‸` | `len`, `iter` |
| after `\|>`, being typed | `xs \|> .it‸` | `iter` |
| further down a pipeline | `xs \|> .iter \|> .‸` | `map`, `filter` |
| in the middle of a pipeline | `xs \|> .iter \|> .‸ \|> .sum::<i64>` | `map`, `filter` |
| with placeholder arguments | `xs \|> .iter \|> .ma‸ _ (fun x -> x)` | `map` |
| as an argument whose type is known | `twice (.‸) 1` | `abs`, `pow` |
| Option in a pipeline | `o \|> .‸` | `is_some`, `unwrap_or` |
| composed | `xs \|> (.iter >> .‸)` | `map`, `filter` |
| glued to `\|>` | `xs \|>.‸` | `len`, `iter` |
| glued to `\|>`, being typed | `xs \|>.it‸` | `iter` |
| glued, further down a pipeline | `xs \|>.iter() \|>.‸` | `map`, `filter` |
| glued, in the middle of a pipeline | `xs \|>.iter \|>.‸ \|>.sum::<i64>` | `map`, `filter` |
| glued, before `in` | `let k = xs \|>.‸ in k` | `len`, `iter` |
| glued to `>>` | `xs \|> (.iter >>.‸)` | `map`, `filter` |
| turbofish type | `xs.into_iter() \|> .collect::<‸>` | `Vec` |
| after a match case arrow | `match o with Some x -> x \|> .‸ \| None -> 0` | `len`, `trim` |

**Application**

| Position | Code | Offers (at least) |
|---|---|---|
| function being typed | `ad‸ 1 2` | `add` |
| first argument | `add ‸` | `n`, `xs` |
| later argument | `add 1 ‸` | `n` |
| argument before others | `add ‸ 2` | `n` |
| argument in parentheses | `add (n‸ + 1) 2` | `n` |
| labeled argument | `add ~a:n‸ 2` | `n` |
| placeholder section | `add _ ‸` | `n` |
| numbered placeholders | `add _1 ‸` | `n` |
| operator section | `( + ) ‸` | `n` |
| Rust-style method arguments | `xs.push(‸)` | `n` |
| Rust macro arguments | `println!("{}", ‸)` | `n`, `word` |
| argument of a `.name` call | `.push xs ‸` | `n` |
| empty head before arguments | `‸ 1 2` | `add`, `twice` |
| `Type.name` head being typed | `Vec.wi‸ 10` | `with_capacity` |
| `Type::name` head being typed | `Vec::wi‸ 10` | `with_capacity` |
| trait method path in a pipeline | `xs.iter() \|> Iterator::‸ _ (fun x -> x)` | `map`, `filter` |
| constructor argument | `Some ‸` | `n` |
| constructor tuple argument | `Some (n, ‸)` | `n` |
| enum constructor argument | `Shape::Circle ‸` | `n` |
| `assert` | `assert ‸` | `n` |
| `incr` | `let r = ref 0 in incr ‸` | `r` |

**Operators**

| Position | Code | Offers (at least) |
|---|---|---|
| arithmetic | `n + ‸` | `n` |
| arithmetic, inner operand | `n + ‸ * 2` | `n` |
| `mod` | `n mod ‸` | `n` |
| `\|>` function | `xs \|> ‸` | `add`, `twice` |
| `\|>` argument | `‸ \|> add 1` | `n` |
| `@@` | `add 1 @@ ‸` | `n` |
| `>>` | `add 1 >> ‸` | `add`, `twice` |
| `::` | `n :: ‸` | `xs` |
| `@` | `[1] @ ‸` | `xs` |
| `^` | `word ^ ‸` | `word` |
| comparison | `n = ‸` | `n` |
| `&&` | `n > 0 && ‸` | `n` |
| `not` | `not ‸` | `n` |
| unary minus | `-‸` | `n` |
| `!` | `!‸` | `n` |
| `&` | `&‸` | `xs` |
| `&mut` | `&mut ‸` | `xs` |
| `:=` | `let r = ref 0 in r := ‸` | `n` |
| `<-` | `let mut m = 0 in m <- ‸` | `n` |
| `as` | `n as ‸` | `f64`, `i32` |

**Let**

| Position | Code | Offers (at least) |
|---|---|---|
| value at toplevel | `let x = ‸` | `n` |
| value before `in` | `let x = ‸ in x` | `n` |
| function body | `let f a = a + ‸ in f 1` | `a`, `n` |
| function body at toplevel | `let f a = ‸` | `a` |
| `let rec` body | `let rec go (k : i64) : i64 = ‸ in go 1` | `k`, `go` |
| `let rec` parameter type | `let rec go (k : ‸) : i64 = k in go 1` | `i64`, `String` |
| `let rec` result type | `let rec go (k : i64) : ‸ = k in go 1` | `i64`, `String` |
| annotated value type | `let x : ‸ = 1 in x` | `i64`, `String` |
| tuple pattern | `let (a, b) = (1, 2) in ‸` | `a`, `b` |
| `and` | `let a = 1 and b = ‸ in a + b` | `n` |
| next toplevel phrase | `let x = 1 let y = x‸` | `x` |
| next `let .. in` | `let x = 1 in let y = ‸ in y` | `x` |
| `let open` | `let open std::cmp in ‸` | `max`, `min` |
| `open` phrase | `open std::cmp ;; ‸` | `max`, `min` |
| module of `open` | `open std::‸` | `cmp` |
| module of `let open` | `let open std::‸ in max 1 2` | `cmp` |
| `external` name | `external add : i64 -> i64 -> i64 ;; ad‸ 1` | `add` |
| record pattern | `let Point { x; y } = p in ‸` | `x`, `y` |

**Fun**

| Position | Code | Offers (at least) |
|---|---|---|
| body | `fun x -> ‸` | `x` |
| operand in body | `fun x -> x + ‸` | `x`, `n` |
| annotated parameter | `fun (x : &str) -> x.‸` | `len`, `trim` |
| parameter type | `fun (x : ‸) -> x` | `str`, `String`, `i64` |
| `fun move` | `fun move x -> ‸` | `x` |
| closure argument of a method ref | `xs \|> .iter \|> .map _ (fun x -> x.‸)` | `abs`, `pow` |
| `function` body | `function Some x -> ‸ \| None -> 0` | `x` |
| `function` pattern | `function ‸` | `Some`, `None` |
| lambda applied with `\|>` | `n \|> fun y -> y.‸` | `abs`, `pow` |
| lambda applied directly | `(fun y -> y.‸) n` | `abs`, `pow` |
| `let rec .. = function` case body | `let rec go : &[i64] -> i64 = function [] -> 0 \| x :: rest -> ‸ in go &xs` | `rest`, `x` |

**Match**

| Position | Code | Offers (at least) |
|---|---|---|
| scrutinee | `match ‸ with Some x -> 1 \| None -> 0` | `o` |
| case body | `match o with Some x -> ‸ \| None -> 0` | `x` |
| method in a case body | `match o with Some x -> x.‸ \| None -> 0` | `len` |
| last case body | `match o with Some x -> 1 \| None -> ‸` | `n` |
| first pattern | `match o with ‸` | `Some`, `None` |
| path pattern | `match o with Option::‸` | `Some`, `None` |
| next pattern | `match o with Some x -> 1 \| ‸` | `None` |
| guard | `match o with Some x when ‸ -> 1 \| _ -> 0` | `x` |
| enum variant patterns | `match shape with Shape::‸` | `Circle`, `Rect` |
| enum variant patterns, OCaml style | `match shape with Shape.‸` | `Circle`, `Rect` |
| lowercase path pattern | `match ord with std::cmp::Ordering::‸` | `Less`, `Equal`, `Greater` |
| literal case body | `match n with 0 -> ‸ \| _ -> 1` | `n` |
| tuple pattern | `match (n, o) with (0, Some x) -> x.‸ \| _ -> 0` | `len`, `trim` |

**If**

| Position | Code | Offers (at least) |
|---|---|---|
| condition | `if ‸ then 1 else 0` | `n` |
| `then` branch | `if n > 0 then ‸ else 0` | `n` |
| `else` branch | `if n > 0 then 1 else ‸` | `n` |
| `then` without `else` | `if n > 0 then ‸` | `n` |
| condition, nothing after | `if ‸` | `n` |
| unfinished condition | `if n > ‸` | `n` |
| method in an unfinished condition | `if o.‸` | `is_some` |

**Sequence**

| Position | Code | Offers (at least) |
|---|---|---|
| after `;` | `println!("a"); ‸` | `n` |
| `begin .. end` | `begin ‸ end` | `n` |
| second item in `begin .. end` | `begin println!("a"); ‸ end` | `n` |
| `begin` without `end` | `begin ‸` | `n` |

**Loops**

| Position | Code | Offers (at least) |
|---|---|---|
| `for` bound | `for i = 0 to ‸ do () done` | `n` |
| `for` start | `for i = ‸ to n do () done` | `n` |
| `for` body | `for i = 0 to n do ‸ done` | `i` |
| `for` body without `done` | `for i = 0 to n do ‸` | `i` |
| `downto` body | `for i = n downto 0 do ‸ done` | `i` |
| `while` condition | `while ‸ do () done` | `n` |
| `while` body | `while n > 0 do ‸ done` | `n` |
| `while` body without `done` | `while n > 0 do ‸` | `n` |
| `while` condition, nothing after | `while ‸` | `n` |
| macro in a loop body | `for i = 0 to n do println!("{}", ‸) done` | `i` |
| after `done` | `while n > 0 do () done; ‸` | `n` |

**Collections**

| Position | Code | Offers (at least) |
|---|---|---|
| list item | `[n; ‸]` | `n` |
| first list item | `[‸]` | `n` |
| array item | `[\|n; ‸\|]` | `n` |
| tuple item | `(n, ‸)` | `n` |

**Records**

| Position | Code | Offers (at least) |
|---|---|---|
| field value | `Point { x = ‸; y = 0 }` | `n` |
| field names | `Point { x = 1; ‸ }` | `y` |
| field names of an empty record | `Point { ‸ }` | `x`, `y` |
| method in a field value | `Point { x = n; y = p.‸ }` | `x`, `y` |
| `with` value | `{ p with x = ‸ }` | `n` |
| `with` field names | `{ p with ‸ }` | `x`, `y` |

**Rust**

| Position | Code | Offers (at least) |
|---|---|---|
| closure in a Rust-style call | `xs.iter().map(fun x -> x.‸)` | `abs`, `pow` |
| turbofish type | `xs.iter().cloned().collect::<‸>()` | `Vec` |
| after a Rust-style chain with an OCaml closure | `xs.iter().map(fun x -> x + 1).‸` | `sum`, `collect` |

**Types**

| Position | Code | Offers (at least) |
|---|---|---|
| annotation | `(n : ‸)` | `i64`, `String` |
| annotated placeholder | `add (_ : ‸) 1` | `i64`, `String` |

**Items**

| Position | Code | Offers (at least) |
|---|---|---|
| `self.` in a method | `rsml! { impl Point { fn twice_x(&self) -> i64 { self.‸ } } }` | `x`, `y`, `norm` |
| `Self::` in a method | `rsml! { impl Point { fn twice_x(&self) -> i64 { Self::‸ } } }` | `norm` |
| `self \|> .` in a method | `rsml! { impl Point { fn twice_x(&self) -> i64 { self \|> .‸ } } }` | `norm` |
| trait default method | `rsml! { trait Describe { fn describe(&self) -> String; fn shout(&self) -> String { self.‸ } } }` | `describe` |
| closure-typed parameter | `rsml! { fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 { ‸ } }` | `f`, `x` |
| `Self { .. }` field value | `rsml! { impl Point { fn shifted(&self, by: i64) -> Point { Self { x = self.x + ‸; y = self.y } } } }` | `by`, `self` |
| closure parameter partially applied, argument being typed | `rsml! { fn apply(f: impl Fn(i64, i64) -> i64, n: i64) -> impl Fn(i64) -> i64 { f n‸ } }` | `n` |
| closure parameter, first argument with nothing after it | `rsml! { fn apply(g: impl Fn(i64, i64) -> i64, n: i64, w: String) -> i64 { g ‸ } }` | `n` before `w` |
| closure parameter, first argument with nothing after it, in a sequence | `rsml! { fn apply(g: impl Fn(i64, i64) -> i64, n: i64) -> i64 { g ‸; n } }` | `n` |

**Placeholders**

| Position | Code | Offers (at least) |
|---|---|---|
| method on a placeholder | `xs \|> .iter \|> .map _ (_.‸)` | `abs`, `pow` |

**Rust-style calls**

| Position | Code | Offers (at least) |
|---|---|---|
| first argument | `add(‸)` | `n`, `xs` |
| later argument | `add(1, ‸)` | `n` |
| later argument of a method, ranked by its parameter's type | `xs.insert(0, ‸)` | `n` before `xs` |
| argument being typed | `add(n‸, 1)` | `n` |
| result called again | `std::convert::identity(add)(1, ‸)` | `n` |

**Method sections**

| Position | Code | Offers (at least) |
|---|---|---|
| argument | `xs \|> .map(‸)` | `n` |
| further down a pipeline | `xs \|> .iter() \|> .‸` | `map`, `filter` |
| closure argument | `xs \|> .iter() \|> .map(fun x -> x.‸)` | `abs`, `pow` |
| name being typed | `xs \|> .iter() \|> .ma‸(fun x -> x)` | `map` |

**Index**

| Position | Code | Offers (at least) |
|---|---|---|
| inside the brackets | `xs[‸]` | `n` |
| method on an element | `xs[0].‸` | `abs`, `pow` |

**For in**

| Position | Code | Offers (at least) |
|---|---|---|
| iterated expression | `for x in ‸` | `xs` |
| body without `done` | `for x in xs do ‸` | `x` |
| body | `for x in xs do ‸ done` | `x` |
| tuple pattern | `for (i, x) in xs.iter().enumerate() do ‸ done` | `i`, `x` |

**Ranges**

| Position | Code | Offers (at least) |
|---|---|---|
| end | `0..‸` | `n` |
| inclusive end | `0..=‸` | `n` |
| method on a range | `(0..n).‸` | `rev`, `map` |

**Jumps and blocks**

| Position | Code | Offers (at least) |
|---|---|---|
| `return` value | `return ‸` | `n` |
| `break` value in an endless loop | `while true do break ‸ done` | `n` |
| `unsafe` | `unsafe { ‸ }` | `n` |
| `async` | `async { ‸ }` | `n` |

**Std macros**

| Position | Code | Offers (at least) |
|---|---|---|
| argument after a format string | `format!("{} {}", n, ‸)` | `n`, `word` |
| method section in an argument | `println!("{}", xs \|> .‸)` | `len` |
| argument of an application | `assert_eq!(add ‸, 3)` | `n` |
| name being typed | `format!("{}", n‸)` | `n` |
| named argument | `println!("{v}", v = ‸)` | `n` |
| `vec!` count | `vec![n; ‸]` | `n` |
| `vec!` items | `vec![n, ‸]` | `n` |
| assertion | `assert!(xs.‸, "empty")` | `is_empty` |

**Field sections**

| Position | Code | Offers (at least) |
|---|---|---|
| after `.map(.0)` | `let pairs = [(1, 2)] in pairs \|> .iter() \|> .map(.0) \|> .‸` | `sum`, `collect` |
| applied | `let q = (p, 1) \|> .0 in q.‸` | `x`, `y` |

**Deref**

| Position | Code | Offers (at least) |
|---|---|---|
| field of a dereferenced place | `*p.‸` | `x`, `y` |

**Functions of the same `rsml!`**

| Position | Code | Offers (at least) |
|---|---|---|
| argument after a sibling being typed | `rsml! { fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64) -> i64 { add2 n‸ 2 } }` | `n` |
| method on a sibling's result | `rsml! { fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64) -> i64 { (add2 n 2).‸ } }` | `abs`, `pow` |
| associated function | `rsml! { struct Q { x: i64 } impl Q { fn new_point(x: i64) -> Q { Q { x = x } } fn f() -> Q { Self::new_‸ 1 } } }` | `new_point` |
| later argument | `rsml! { fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64) -> i64 { add2 1 ‸ } }` | `n` |
| first argument before others | `rsml! { fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64) -> i64 { add2 ‸ 2 } }` | `n` |
| Rust-style arguments | `rsml! { fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64) -> i64 { add2(1, ‸) } }` | `n` |
| one parameter | `rsml! { fn inc(a: i64) -> i64 { a + 1 } fn f(n: i64) -> i64 { inc ‸ } }` | `n` |
| first argument with nothing after it | `rsml! { fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64, w: String) -> i64 { add2 ‸ } }` | `n` before `w` |
| first argument with nothing after it, in a sequence | `rsml! { fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64) -> i64 { add2 ‸; n } }` | `n` |
| first argument with nothing after it, as an argument | `rsml! { fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64) -> i64 { twice (add2 ‸) n } }` | `n` |
| associated function, first argument with nothing after it | `rsml! { struct Q { x: i64 } impl Q { fn make(a: i64, b: i64) -> Q { Q { x = a + b } } fn f(n: i64, w: String) -> Q { Self::make ‸ } } }` | `n` before `w` |
| associated function, first argument with nothing after it, as an argument | `rsml! { struct Q { x: i64 } impl Q { fn make(a: i64, b: i64) -> Q { Q { x = a + b } } fn f(n: i64) -> Option<Q> { Some (Self::make ‸) } } }` | `n` |

**Arity**

| Position | Code | Offers (at least) |
|---|---|---|
| local function, Rust-style first argument | `let plus x y = x + y in plus(‸)` | `n` |
| local function, Rust-style later argument | `let plus x y = x + y in plus(1, ‸)` | `n` |
| local function, first argument being typed | `let plus x y = x + y in plus n‸` | `n` |
| local function, first argument before others | `let plus x y = x + y in plus ‸ 2` | `n` |
| local function, later argument | `let plus x y = x + y in plus 1 ‸` | `n` |
| local function of one argument | `let inc x = x + 1 in inc ‸` | `n` |
| local function, first argument with nothing after it | `let plus x y = x + y in plus ‸` | `n` |
| local function, first argument with nothing after it, in a sequence | `let plus x y = x + y in plus ‸; n` | `n` |
| local function, first argument with nothing after it, as an argument | `let plus x y = x + y in twice (plus ‸) 1` | `n` |
| first argument, ranked by its parameter's type | `let f (a : &str) (b : i64) = b in f ‸` | `s` before `n` |
| middle argument, ranked by its parameter's type | `let f (a : &str) (b : i64) (c : &str) = b in f "a" ‸` | `n` before `s` |
| argument after a variable argument, three parameters (a generated local too: see [Limitations](#limitations)) | `let f a b c = a + b + c in f n ‸` | `n` |
| function of `()`, argument with nothing typed | `let f () = 1 in f ‸` | `n` |
| last argument after a constant, long function name (a generated local too: see [Limitations](#limitations)) | `let fairly_long_name (a : &str) (b : i64) = b in fairly_long_name "x" ‸` | `n` |

**`#[rust]`**

| Position | Code | Offers (at least) |
|---|---|---|
| body stays Rust | `rsml! { #[rust] fn f(xs: Vec<i64>) -> usize { let n = xs.‸; n } }` | `len` |

**Whole file**

| Position | Code | Offers (at least) |
|---|---|---|
| test using a sibling | `rsml! { fn double(n: i64) -> i64 { n * 2 } #[cfg(test)] mod tests { use super::*; #[test] fn doubles() { let n = 2 in assert_eq!(double n‸, 4) } } }` | `n` |

**Constructors**

| Position | Code | Offers (at least) |
|---|---|---|
| method of a constructed value | `Some(n).‸` | `unwrap`, `map` |

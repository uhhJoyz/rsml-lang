# rsml-fmt

Formats the rsml code in Rust files: the bodies of `ml! { .. }` blocks
and the function bodies inside `rsml! { .. }`, and the Rust inside
`rsml! { .. }` (which it hands to rustfmt). rustfmt itself never enters
a macro, so the two tools don't overlap. `--rustfmt` runs rustfmt on the
rest of the file first, so one command formats everything, which is
what an editor wants (see
[docs/editors.md](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md#formatting-on-save)). A block
that formatting breaks onto lines of its own changes how rustfmt lays out
the call around it (`assert_eq!(ml! { .. }, 3)`), so the two passes run in
turn until neither changes anything: formatting twice changes nothing.
Blocks inside a `macro_rules!` are templates (`ml! { $e + 1 }`) and are
left as written.

```sh
cargo run -p rsml-fmt -- src tests examples     # format in place
cargo run -p rsml-fmt -- --check src            # CI: exit 1 if anything would change
cargo run -p rsml-fmt -- --rustfmt src          # rustfmt and rsml-fmt in one go
cargo install rsml-fmt                          # then just `rsml-fmt ...`
echo 'x |> f |> g' | rsml-fmt --ml              # bare rsml code on stdin
rsml-fmt --rustfmt < src/main.rs                # a Rust file on stdin, for editors
```

With no path, a Rust file is read from stdin and written to stdout; if it
is not valid Rust (it does not lex, or rustfmt rejects it), nothing is
written and the exit status is 2, so an editor keeps its buffer.
`--keep-on-error` writes the input back unchanged instead (the status is
still 2), for filters that replace the text with whatever comes out, as
Vim's `:%!` and `gq` do.
`--stdin-filepath PATH` says which file stdin holds: `rsmlfmt.toml`,
`rustfmt.toml` / `.rustfmt.toml` and `Cargo.toml` are then looked for from
its directory instead of the current one, and warnings name it. The
edition passed to rustfmt is `--edition`'s, else the nearest `Cargo.toml`'s
(following `edition.workspace = true` to the workspace's
`[workspace.package]`; a `Cargo.toml` without one is 2015, as for Cargo),
else 2024.

It parses with the macros' own parser, so it accepts exactly what the
macros accept. A block that does not parse is left as it is (with a
warning). The test suite formats every block in this repository under a
dozen configurations and checks that the result expands to the same Rust as
the original and that formatting twice changes nothing. A fuzzer
(`tests/fuzz.rs`) does the same for the blocks' prefixes (what an editor
sends while you type), random mutations of them and generated code, and
checks that the macros never panic; `cargo test` runs a sample, and
`scripts/fuzz.sh [ITERS]` a long run. As a last guard, the formatter
expands a block before and after formatting, and leaves it as written,
with a warning, if the two would differ.

Tokens that would run together into another operator are kept apart:
`f & &xs` (where `f &&xs` would be `f && xs`), `Vec & ::new ()`,
`- ::x`, `& -5` (a pattern), `1 .2` (not the float `1.2`), `!1 .0`
(`(!1).0`), `t.0 .(i)`, `.0 .1`, `0. .. 1.` (`0...1.` is `0 ... 1.`).
The other way round, a section glued to a pipe or a composition
(`xs |>.iter()`, which Rust lexes with one operator `|>.`) gets its space,
`xs |> .iter()`, unless the operator `|>.` is defined.

A space before `(` or `[` is kept exactly as written, because it changes
the meaning: `f(a, b)` is a Rust-style call with two arguments and `f (a,
b)` passes one tuple; `v[0]` indexes and `f [0]` passes a list. So is the
space after a constructor, in expressions and patterns: `Some(x).unwrap()`
unwraps the option, `Some (x).unwrap()` unwraps `x`. The formatter never
prints `(*` for a dereference, which would read as an ML comment:
`(*x)` becomes `( *x)`.

Comments (`(* *)`, `//`, `/* */`) and blank lines are kept. A comment stays
with the code after it on its line, on a line of its own if it had one, or
at the end of the line it followed. A closing bracket, `end` or `done` is
not code a comment goes with: `f(x (* c *))` keeps the comment inside the
call. A comment between the tokens of a path (`Vec:: (* c *) <u8>::new`)
moves before the path. With `comments = "rust"`, `(* c *)`
becomes `// c` when it has a line to itself and `/* c */` elsewhere (a
space after `//` or `/*` keeps `(** c *)` from becoming a doc comment, and
a comment holding `/*` or `*/` stays as it is); `rsmlfmt:` comments keep
working in either spelling.

The arguments of std macros that take rsml code (`println!`, `assert_eq!`,
`format!`, `vec!`, ..) are formatted like a call's, the format string as
written: on one line if they fit, else one per line with a trailing
comma. Other macro calls and string literals are printed exactly as
written.

## The Rust inside `rsml!`

With `rust_items = "rustfmt"` (the default), the Rust parts of an `rsml!`
block (items, signatures, attributes, `where` clauses, `use`s, `#[rust]`
bodies) are formatted by rustfmt: the formatter replaces every rsml body
by `{}`, runs rustfmt on the result with the nearest rustfmt config and the
crate's edition, and puts the rsml bodies back, formatted as rsml code.
The items are at the indentation of the line with `rsml! {`, or one level
in from it with `indent_rsml_items = true` (see [Layout](#layout)), except
that items that were flush left (a whole file in one `rsml!`) stay flush
left; rustfmt's `max_width` counts that indentation. One blank line after
`{` and before `}` is kept if there was one. Comments in the Rust parts are
rustfmt's to keep. If rustfmt is missing (`rustfmt_path`) or rejects the
block (for example an ML comment between items, which is no Rust),
the block's Rust is kept as written, with a warning (once, for a missing
rustfmt). An `rsmlfmt: skip` comment at the top of the block leaves all of
it alone, and `rust_items = "preserve"` leaves the Rust of every block as
written.

## Configuring

Options live in `rsmlfmt.toml` (or `.rsmlfmt.toml`), found in the formatted
file's directory or the nearest parent that has one; `--config FILE` picks
one explicitly. The file only needs what you change, and a table can be
partial: `[pipe]` with only `indent = "align"` keeps the other pipe
defaults. Unknown keys and values are errors, so typos don't pass silently.

```sh
rsml-fmt --print-config            # every option, its default, and what it does
rsml-fmt --print-config current    # what applies in this directory
```

There are three ways to set an option, from broadest to narrowest:

```toml
# rsmlfmt.toml
max_width = 90

[pipe]
indent = "align"
```

```sh
rsml-fmt --set max_width=90 --set pipe.indent=align src    # one run
```

```rust
ml! {
    (* rsmlfmt: max_width = 60; pipe.indent = align *)
    xs |> .iter |> .map _ f |> .filter _ g
}
ml! {
    (* rsmlfmt: skip *)
    keep   this   exactly
}
rsml! {
    // rsmlfmt: let.body = fit       (applies to every function in this rsml!)
    fn f() -> i64 { let x = 1 in x + 1 }
}
```

A `rsmlfmt:` comment must come before the first token of its block. Values
that are single words need no quotes (`pipe.indent = align`), elsewhere
they are TOML.

### Presets

The defaults: one `let .. in` per line, sequences broken, pipes with a
leading `|>` under the subject, and a function whose body is one `ml!`
block opened on the function's line (`hug_fn_body`).

Compact, for short `ml!` blocks inside Rust expressions:

```toml
[let]
body = "fit"
[sequence]
break = "fit"
[toplevel]
break = "fit"
```

Aligned pipes, as in this repository's README:

```toml
[pipe]
indent = "align"
```

## Options

The examples show the default first.

### Layout

| Option | Values | |
|---|---|---|
| `max_width` | `100` | Line width, counted from column 0 (Rust indentation included). |
| `indent` | `2` | Spaces per nesting level in rsml code. |
| `block_indent` | `0` | Indentation of an `ml! { .. }` block's body relative to the line holding its `{` (`0`: the body and the `}` line up with that line). |
| `fn_body_indent` | `4` | Indentation of a function's body relative to its line: an `rsml!` function's, and a hugged one's (`hug_fn_body`). |
| `hug_fn_body` | `true` | Lay out a Rust function whose body is one `ml! { .. }` block as `fn f() -> T { ml! {`, the body, `}}` (below). |
| `indent_rsml_items` | `false` | Indent the items of an `rsml! { .. }` one level from the line with `rsml! {` (`false`: at that line's indentation). |
| `single_line_blocks` | `"when_fits"`, `"preserve"`, `"never"` | When `ml! { x \|> f }` stays on one line. `preserve`: only if it already was. The width check includes the rest of the Rust line (`}, 3);`). |
| `fn_single_line` | `false` | Allow `fn f() -> i64 { x \|> g }` in `rsml!`. |
| `preserve_blank_lines` | `true` | Keep blank lines between phrases, `let .. in` steps and sequence items. |
| `max_blank_lines` | `1` | |
| `op_section_spaces` | `"always"`, `"minimal"` | `( + )` or `(+)`. `( * )` keeps its spaces either way: `(*` opens a comment. |
| `colon_spacing` | `"both"`, `"after"` | `(x : int)` or `(x: int)`. |
| `begin_end` | `"preserve"`, `"parens"` | Turn `begin a; b end` into `(a; b)`. |
| `begin_end_hug` | `true` | Keep `begin`, `unsafe {` and `async {` on the line of the `then`, `else`, `->` or `do` before them when the block breaks (below). |
| `comments` | `"preserve"`, `"rust"` | `rust`: `(* c *)` becomes `// c` on a line of its own, else `/* c */`. |
| `rust_items` | `"rustfmt"`, `"preserve"` | Format the Rust inside `rsml!` with rustfmt (above), or keep it as written. |
| `rustfmt_path` | `"rustfmt"` | The rustfmt to run. |

A Rust function whose whole body is one `ml! { .. }` block gets `{ ml! {`
on its line and `}}` to close it, so that the block reads as the
function's body, `fn_body_indent` columns in (`hug_fn_body`). Any other
`ml!` block's body lines up with the line holding `ml! {`, and so does its
`}` (`block_indent = 0`), and the items of an `rsml!` line up with the line
holding `rsml! {` (`indent_rsml_items = false`):

```rust
fn total(xs: &[i64]) -> i64 { ml! {
    xs |> .iter() |> .sum()
}}

fn scaled(xs: &[i64], k: i64) -> i64 {
    let n = xs.len() as i64;
    ml! {
    let s = xs |> .iter() |> .sum::<i64>() in
    s * k / n
    }
}

mod shapes {
    rsml! {
    fn area(w: i64, h: i64) -> i64 {
        let a = w * h in
        a
    }
    }
}
```

With `hug_fn_body = false`, `block_indent = 4` and
`indent_rsml_items = true`, the same code is:

```rust
fn total(xs: &[i64]) -> i64 {
    ml! { xs |> .iter() |> .sum() }
}

fn scaled(xs: &[i64], k: i64) -> i64 {
    let n = xs.len() as i64;
    ml! {
        let s = xs |> .iter() |> .sum::<i64>() in
        s * k / n
    }
}

mod shapes {
    rsml! {
        fn area(w: i64, h: i64) -> i64 {
            let a = w * h in
            a
        }
    }
}
```

A function is hugged only when its body holds nothing but the block (no
comment, no `;` after it), and when `{ ml! {` fits in `max_width` after
the signature; otherwise the block goes on a line of its own in the body.
A hugged block always takes lines of its own, even one that would fit on
one line.

rustfmt does not keep the hug: it puts `ml! {` back on a line of its own
and indents the body under it, so `cargo fmt` undoes it, and
`cargo fmt --check` reports every hugged function. Format with
`rsml-fmt --rustfmt` instead (`rsml-fmt --rustfmt --check src` in CI),
which runs rustfmt and then hugs again, as the editor setups of
[docs/editors.md](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md#formatting-on-save)
do, or set `hug_fn_body = false`.

With `begin_end_hug`, a block that breaks keeps its opening on the
keyword's line and lines up its end with that line (inside a match case,
`end` lines up under the case's body, two columns in):

```ocaml
if c then begin                  (* begin_end_hug = false: *)
  a ();                          (* if c then              *)
  b ()                           (*   begin                *)
end                              (*     a ();              *)
else begin                       (*     b ()               *)
  c ()                           (*   end                  *)
end                              (* else ..                *)

match e with                     for i = 0 to n do begin
| A -> begin                       step i;
    a ();                          check i
    b ()                         end done
  end
| B -> ()
```

After a comment (`then (* why *) begin`) the block keeps lines of its own.

### `[toplevel]`: phrases of a block

| Option | Values | |
|---|---|---|
| `double_semi` | `"minimal"`, `"always"` | `;;` only where it is required (between an expression and another phrase), or after every phrase but the last. |
| `double_semi_position` | `"end_of_line"`, `"own_line"` | `let x = 1 ;;` or `;;` alone on a line. |
| `blank_lines` | `"preserve"`, `"always"`, `"never"` | Blank lines between phrases. |
| `break` | `"always"`, `"fit"` | `fit` keeps `external f : _ -> _ ;; f 1` on one line. |

```ocaml
external add : int -> int -> int        (* double_semi = "always": *)
let one = 1                             (* external add : int -> int -> int ;; *)
let two = add one one ;;                (* let one = 1 ;;                      *)
two                                     (* let two = add one one ;;            *)
```

### `[let]`

| Option | Values | |
|---|---|---|
| `in_position` | `"auto"`, `"end_of_line"` | `auto` puts `in` on its own line after a binding that spans several lines. |
| `body` | `"always_break"`, `"fit"` | `fit` keeps `let x = 1 in x + 1` on one line. |
| `binding_indent` | `2` | Indentation of a value that goes below `=`. |
| `fun_on_eq_line` | `true` | `let f = fun x ->` rather than `fun x ->` below the `=`. |
| `and_blank_line` | `false` | Blank line before each `and`. |

```ocaml
let area s =                            (* in_position = "end_of_line": *)
  match s with                          (* ...                          *)
  | Circle r -> 3.14 *. r *. r          (*   | Rect (w, h) -> w *. h in  *)
  | Rect (w, h) -> w *. h               (* area shape                   *)
in
area shape
```

### `[fun]`

| Option | Values | |
|---|---|---|
| `body_indent` | `2` | Indentation of a lambda body below `->`. |
| `hug_last_arg` | `true` | When a call's last argument (or a chain's last operand) is a lambda, keep `Iterator::map _ (fun x ->` on one line and put the body below. |

```ocaml
xs
|> Iterator::for_each _ (fun x ->
     print!("{}", x);
     println!())
```

### `[app]`: function application

| Option | Values | |
|---|---|---|
| `args` | `"wrap"`, `"fit_or_vertical"` | Arguments that don't fit: as many per line as fit, or one per line. |
| `indent` | `2` | Indentation of arguments on continuation lines. |

### `[match]` (also `function`)

| Option | Values | |
|---|---|---|
| `leading_bar` | `"when_broken"`, `"always"` | `\|` before the first case only on multi-line matches, or always. |
| `cases` | `"fit_or_vertical"`, `"vertical"` | Whether a short match may stay on one line. |
| `case_indent` | `0` | Indentation of `\| case` lines relative to `match`. |
| `body` | `"fit"`, `"next_line"` | Case body after `->` when it fits, or always below. |
| `body_indent` | `4` | Indentation of a case body below its pattern. |
| `align_arrows` | `false` | Align every `->` (when all patterns are one line). |

```ocaml
match n with                            (* align_arrows = true: *)
| 0 -> "zero"                           (* | 0            -> "zero"     *)
| n when n < 0 -> "negative"            (* | n when n < 0 -> "negative" *)
| _ -> "positive"                       (* | _            -> "positive" *)
```

### `[if]`

`style` is one of (each stays on one line when it fits, except `vertical`):

```ocaml
(* "fit_or_vertical" *)    (* "compact" *)             (* "keyword_first" *)
if x > 0 then              if x > 0 then f x           if x > 0
  f x                      else g x                    then f x
else                                                   else g x
  g x
```

`"vertical"` is `fit_or_vertical` without the one-line form. `else if`
chains stay flat.

### `[sequence]`: `a; b`

| Option | Values | |
|---|---|---|
| `style` | `"terminator"`, `"separator"` | `a;` ending the line, or `; b` starting the next. |
| `break` | `"always"`, `"fit"` | One item per line, or one line when it fits. |

### `[collections]`: lists, arrays and records

| Option | Values | |
|---|---|---|
| `style` | `"aligned"`, `"block"` | `[a;` / ` b]`, or brackets on their own lines with items indented. |
| `layout` | `"fit_or_vertical"`, `"wrap"` | One item per line, or as many as fit. |
| `separator_position` | `"after"`, `"before"` | `a;` / `b`, or `a` / `; b`. |
| `trailing_semicolon` | `"never"`, `"vertical"`, `"always"` | A `;` after the last item (`vertical`: only when broken). |
| `space_inside_lists` | `false` | `[ 1; 2 ]` |
| `space_inside_arrays` | `false` | `[\| 1; 2 \|]` (kept anyway when the first item starts with an operator: `[\|-1` would not lex). |
| `space_inside_records` | `true` | `{ x = 1 }` |

```ocaml
[aaaa;      [           [ aaaa            [aaaa; bbbb; cccc;
 bbbb;        aaaa;     ; bbbb             dddd]
 cccc]        bbbb;     ; cccc ]
              cccc;
            ]
(* default   block +     before +          wrap *)
(*           trailing    space_inside      *)
(*           vertical    lists            *)
```

### Infix operator chains

Each class of operators is a table with the same four keys:

| Table | Operators | Default |
|---|---|---|
| `[pipe]` | `\|>` | fit, leading, none |
| `[apply]` | `@@` | fit, trailing, indent |
| `[compose]` | `>>` `<<` | fit, leading, indent |
| `[boolean]` | `&&` `\|\|` | fit, leading, none |
| `[cons]` | `::` `@` `^` | fit, trailing, indent |
| `[arithmetic]` | `+ - * / ** mod land lor lxor lsl lsr asr` | wrap, leading, indent |
| `[comparison]` | `= <> < > <= >= == !=` | fit, leading, indent |
| `[custom_operators]` | operators you define (`>>=`, `+!`, ..) | fit, leading, none |

| Key | Values | |
|---|---|---|
| `layout` | `"fit"`, `"wrap"` | One line, else one operator per line; or as many per line as fit. |
| `position` | `"leading"`, `"trailing"` | Operators start lines, or end them. |
| `indent` | `"none"`, `"indent"`, `"align"` | Continuation lines at the chain's indentation, one level deeper, or aligned: under the first operator (leading; the subject stays on the first line) or under the first operand (trailing). |
| `force_break_at` | `0` | Chains with at least this many operators always break (`0`: never forced). |

A chain is a run of operators of one class and precedence: `a + b - c` is
one chain, `a + b * c` is two.

```ocaml
xs                                 xs |> map f               xs |>
|> map f                              |> filter g              map f |>
|> filter g                                                    filter g
(* default *)                      (* indent = "align" *)   (* position = "trailing",
                                                               indent = "indent" *)
```

## Limitations

* Comments are anchored to the token after them, and `then`, `else`,
  `with`, `in` and `->` don't count as tokens for this: a comment on its
  own line just before `then` ends up after it, before the branch.
* Code inside Rust macro calls in rsml code, other than the std macros
  above, is not formatted.
* Doc comments (`///`) inside blocks are not rsml code, and such blocks
  are left alone.

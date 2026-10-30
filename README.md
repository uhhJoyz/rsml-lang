# rsml

OCaml syntax inside Rust function bodies, with Rust's names and types.
`ml! { .. }` is a procedural macro that parses OCaml and lowers it, at
compile time, to the Rust you would have written by hand: pipelines become
nested calls, partial applications and compositions become ordinary
closures, and there is no runtime library behind them. There is no OCaml
standard library either: you call Rust's functions, methods and macros by
their Rust names, and every type is a Rust type.

```rust
use rsml::ml;

fn sum_of_odd_squares(xs: &[i64]) -> i64 {
    ml! {
        let square x = x * x in
        xs
        |> .iter
        |> .map _ (fun x -> square !x)
        |> .filter _ (fun x -> !x mod 2 = 1)
        |> .sum
    }
}

assert_eq!(sum_of_odd_squares(&[1, 2, 3, 4, 5]), 35);
```

This page is an overview. [docs/notation.md](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/notation.md) is the
reference for every functional notation, with its expansion and where an
editor completes it; [docs/editors.md](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md) covers editor
support.

## Quick start

rsml is on crates.io as `rsml-lang` (the name `rsml` was taken); the
library is still `rsml` in code:

```toml
[dependencies]
rsml-lang = "0.1"
```

```rust
use rsml::rsml;

rsml! {
    fn add(a: i64, b: i64) -> i64 { a + b }

    // `|>` passes the value last, `.name(..)` is a method, `add 10` a partial application.
    fn shifted_odds(xs: &[i64]) -> Vec<i64> {
        xs |> .iter() |> .copied() |> .filter(fun x -> x mod 2 = 1) |> .map(add 10) |> .collect()
    }
}

assert_eq!(shifted_odds(&[1, 2, 3]), vec![11, 13]);
```

[examples/toy](https://github.com/uhhJoyz/rsml-lang/blob/main/examples/toy/README.md) has a small commented program for
every notation, in reading order: `cargo run --example toy` runs them.

The formatter, and the editor setup:

```sh
cargo install rsml-fmt              # `rsml-fmt` on the PATH
```

* Format on save, OCaml included, wherever rust-analyzer formats (VS Code's
  `settings.json`; the same setting for Neovim and others is in
  [docs/editors.md](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md#formatting-on-save)):
  `"rust-analyzer.rustfmt.overrideCommand": ["rsml-fmt", "--rustfmt"]`
* Neovim, to color `(* *)` comments as comments (from the checkout;
  [details](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md#neovim-coloring---comments)):
  `mkdir -p ~/.config/nvim/after/queries/rust && cp editors/nvim/queries/rust/highlights.scm ~/.config/nvim/after/queries/rust/`
* Vim, to format the buffer
  ([format on save and highlighting](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md#vim)):
  `:%!rsml-fmt --rustfmt --stdin-filepath %:p 2>/dev/null`

Working on rsml itself: `scripts/check_all.sh` runs every check (`--quick`
leaves out the slow and editor ones; see [Checking changes](#checking-changes)).

## Pipes, composition and partial application

| OCaml in `ml!`          | Rust it becomes                                                 |
|-------------------------|-----------------------------------------------------------------|
| `x \|> f \|> g a`       | `g(a, f(x))`                                                    |
| `f @@ g @@ x`           | `f(g(x))`                                                       |
| `f >> g` (forward)      | `move \|argument\| g(f(argument))`                              |
| `f << g` (backward)     | `move \|argument\| f(g(argument))`                              |
| `(f >> g) x`            | `g(f(x))` (no closure at all)                                   |
| `add 1` (`add` takes 2) | `move \|arg2\| add(1, arg2)`                                    |
| `add (expensive ())`    | `{ let value1 = expensive(); move \|arg2\| add(value1, arg2) }` |
| `(fun x -> x + 1) y`    | `{ let x = y; x + 1 }`                                          |
| `( + )`, `( + ) 1`      | `\|lhs, rhs\| lhs + rhs`, `move \|rhs\| 1 + rhs`                |
| `f _ 1`, `f _1 _0`      | `move \|_0\| f(_0, 1)`, `move \|_0, _1\| f(_1, _0)`             |
| `x \|> f _ 1`           | `f(x, 1)`                                                       |

Arguments of a partial application are evaluated once, when it is created,
as in OCaml.

```rust
use rsml::ml;

let r = ml! {
    let add x y = x + y in
    let inc = add 1 in                 (* partial application *)
    let inc_then_double = inc >> ( * ) 2 in
    [1; 2; 3] |> .into_iter |> .map _ inc_then_double |> .collect::<Vec<_>>
};
assert_eq!(r, vec![4, 6, 8]);
```

`examples/zero_cost.rs` contains `ml!` functions next to hand-written
equivalents; in release builds each pair compiles to the same machine code
(LLVM even merges some into a single symbol). Check it with:

```sh
cargo rustc --release --example zero_cost -- --emit=asm
python3 scripts/compare_asm.py target/release/examples/zero_cost-*.s
```

## Placeholders

`_` and `_0`, `_1`, .. in an expression make it a function: each
placeholder is a parameter. `_N` is parameter `N` (from 0), and `_` is the
lowest one not written elsewhere in the expression, so `f _ _` is
`fun a b -> f a b` and `f _1 _ _` is `fun a b c -> f b a c`.

| OCaml in `ml!`                  | Rust it becomes                                |
|---------------------------------|------------------------------------------------|
| `Iterator::map _ (_ * 2)`       | `move \|_0\| Iterator::map(_0, \|_0\| _0 * 2)` |
| `s \|> str::starts_with _ "he"` | `str::starts_with(s, "he")`                    |
| `clamp 0 _ 10`                  | `move \|_0\| clamp(0, _0, 10)`                 |
| `f _1 _0`                       | `move \|_0, _1\| f(_1, _0)`                    |
| `( * ) _0 _0`                   | `move \|_0\| _0 * _0`                          |
| `map (f _1) xs`                 | `map(\|_, _1\| f(_1), xs)`                     |
| `map (_.len()) xs`              | `map(\|_0\| _0.len(), xs)`                     |
| `map (.len (_ : &str)) xs`      | `map(\|_0: &str\| _0.len(), xs)`               |

Rust functions take their data first, while `|>` passes it last, so
placeholders are how a pipeline uses them: `it |> Iterator::map _ f` is
`Iterator::map(it, f)`.

How far the function reaches: a placeholder belongs to the smallest *full*
expression around it, which is the inside of parentheses, a list or array
item, a method argument, a record field, a `let` value or body, a branch,
a match case, a sequence item, or a side of `|>`, `@@`, `>>` or `<<`.
Applications and other operators do not stop it: `(1 + _ * 2)` is
`fun a -> 1 + a * 2`, and `f (g _) x` is `f (fun a -> g a) x`. A
placeholder alone, possibly annotated, belongs to the expression around
it, so `f (_ : &str) 1` is `fun (a : &str) -> f a 1`.

Applied directly, a placeholder function is a plain call: `(f _1 _0) a b`
is `f(b, a)`. The rest of the expression is evaluated once, when the
function is made: `f (expensive ()) _` computes `expensive ()` once. Write
`(_ : t)` to give a parameter a type, which a closure parameter needs when
methods are called on it.

`_0`, `_1`, .. are not variable names. In patterns `_` is still the
wildcard.

## Methods as functions

Where an expression starts, `.name` is a method as a function of its
receiver and arguments: every argument after the receiver goes to the
method. The receiver alone is a call without arguments (`.iter xs`), so a
`()` after it is a unit argument. With pipes and placeholders, a method
chain reads left to right:

| OCaml in `ml!`             | Rust it becomes                             |
|----------------------------|---------------------------------------------|
| `.len s`                   | `s.len()`                                   |
| `.push xs 1`               | `xs.push(1)`                                |
| `.send tx ()`              | `tx.send(())`                               |
| `.collect::<Vec<_>> it`    | `it.collect::<Vec<_>>()`                    |
| `xs \|> .iter \|> .count`  | `xs.iter().count()`                         |
| `it \|> .map _ f`          | `it.map(f)`                                 |
| `.len`                     | `\|receiver\| receiver.len()`               |
| `map (.len (_ : &str)) xs` | `map(\|_0: &str\| _0.len(), xs)`            |
| `.iter >> .count`          | `move \|argument\| argument.iter().count()` |

With its arguments written Rust style, right after the name, `.name(args)`
is the method with those arguments, waiting for its receiver, so a
pipeline reads like a Rust method chain:

| OCaml in `ml!`                                              | Rust it becomes                           |
|-------------------------------------------------------------|-------------------------------------------|
| `xs \|> .map(f)`                                            | `xs.map(f)`                               |
| `.push(1) xs`                                               | `xs.push(1)`                              |
| `xs \|> .iter() \|> .map(fun x -> x * 2) \|> .sum::<i64>()` | `xs.iter().map(\|x\| x * 2).sum::<i64>()` |
| `.iter()`                                                   | `\|receiver\| receiver.iter()`            |

A tuple field works the same way: `.0` is `|tuple| tuple.0`, so `x |> .0`
is `x.0` and `pairs |> .iter() |> .map(.0)` is
`pairs.iter().map(|tuple| tuple.0)`.
Only numbered fields: `.name` is always a method.

After an expression, `.name` is still that expression's field or method
(`t.len`, `xs.iter()`, and chains broken across lines), so `f t.len (x + 1)`
passes the field `t.len`, and a method as an argument needs parentheses:
`map (.len) xs`. A closure that calls a method needs its receiver's type,
as in Rust; where nothing provides it, annotate the placeholder
(`.len (_ : &str)`) or call through the trait (`Iterator::map _ f`).

### Arity: what the macro can and cannot see

OCaml functions are curried; Rust functions take all their arguments at
once. The macro decides how to call something from the function's *shape*:

* Functions defined in the same `ml!` block, closures (`fun`), operator
  sections, placeholder functions, closure-typed parameters of `rsml!`
  functions and the functions defined in the same `rsml!` (see below) have
  known shapes, so `f a` is a direct call when it is complete and a
  closure when it is partial.
* A Rust function defined elsewhere is invisible to a macro. It is called
  with every argument it is given: `f a b` is `f(a, b)`. This is what you
  want for ordinary Rust calls, constructors and methods.
* In pipelines and compositions no arity is needed: `x |> f a` is always
  `f(a, x)` and `f a >> g` is always `move |argument| g(f(a, argument))`.
* To partially apply a Rust function, use a placeholder (`add 5 _`), or
  declare its shape with `external`, or constrain the result with a
  function type:

```rust
use rsml::ml;

fn add(a: i32, b: i32) -> i32 { a + b }
fn adder(n: i32) -> impl Fn(i32) -> i32 { move |x| x + n }

let add4 = ml! { add 4 _ };
assert_eq!(add4(1), 5);

let add5 = ml! { external add : i32 -> i32 -> i32 ;; add 5 };
assert_eq!(add5(1), 6);

let add6 = ml! { (add 6 : i32 -> i32) };
assert_eq!(add6(1), 7);

// Parentheses in the type mark a Rust function returning a closure.
assert_eq!(ml! { external adder : i32 -> (i32 -> i32) ;; adder 1 2 }, 3);
```

Only the number of arrows in an `external` type matters; the types
themselves can be `_` (`external add : _ -> _ -> _`). `= "path::to::fn"`
binds the name to a different Rust path.

## Whole functions: `rsml!`

`rsml! { .. }` holds Rust items whose function bodies are OCaml, so a
function needs no `ml! { }` inside it. Signatures, attributes and every
other item are Rust and pass through untouched; methods in `impl` and
`trait` blocks get OCaml bodies too.

```rust
use rsml::rsml;

rsml! {
    fn sum_of_odd_squares(xs: &[i64]) -> i64 {
        let square x = x * x in
        xs
        |> .iter
        |> .map _ (fun x -> square !x)
        |> .filter _ (fun x -> !x mod 2 = 1)
        |> .sum
    }

    fn add_all(add: impl Fn(i64, i64) -> i64, xs: Vec<i64>) -> Vec<i64> {
        xs |> .into_iter |> .map _ (add 10) |> .collect
    }
}

assert_eq!(sum_of_odd_squares(&[1, 2, 3, 4, 5]), 35);
assert_eq!(add_all(|a, b| a + b, vec![1, 2]), vec![11, 12]);
```

The macro sees the signature: parameters are bound names (so with a
parameter `h`, `h::t` is a cons rather than the path `h::t`), and a
parameter whose type is a closure (`impl Fn(..)`, `&dyn Fn(..)`, or a
generic with an `Fn(..)` bound) has a known arity, so `add 10` above is a
partial application.

It sees every other signature in the same `rsml!` too, so its functions
have known arities in each other's bodies, as local `let` functions do:
with `fn add(a: i64, b: i64)` defined there, `add 1` is a closure,
`add 1 2` and `add(1, 2)` are calls, and `xs |> .map(add 1)` needs no
`external` or `_`. A function's shape is its number of parameters (`()`
for none, so `now ()` is `now()`), plus one call for a returned closure
(`-> impl Fn(i64) -> i64`, `Box<dyn Fn(..)>`, `&dyn Fn(..)`): `adder 1 2`
is `adder(1)(2)`. Methods taking `self` are left out; associated functions
are known as `Self::name` inside their `impl`, and as `Type::name` in the
whole module. A `mod` knows its own functions, and its parent's when it
imports them (`use super::*` or `use super::name`). Parameters and local
bindings shadow them.

`#[rust]` on a `fn`, `impl`, `trait` or `mod` inside `rsml!` leaves every
function body under it as Rust (the attribute is removed), for code that
reads better that way; the formatter leaves those bodies alone too.

### Whole files

A file can be one flush-left `rsml::rsml! { .. }` holding all its items:
`use`s, types, `impl`s, functions, a `#[cfg(test)] mod tests` whose tests
assert on OCaml expressions, and `#[rust]` functions where Rust reads
better. Only the file's own `//!` comments and `#![..]` attributes stay
outside, as a macro call cannot hold them.
[examples/whole_file.rs](https://github.com/uhhJoyz/rsml-lang/blob/main/examples/whole_file.rs) is one; `cargo test` runs
its tests, and rustfmt and `rsml-fmt` leave it as it is.

```rust
rsml::rsml! {

use std::collections::HashMap;

pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn incremented(xs: &[i64]) -> Vec<i64> {
    xs |> .iter() |> .copied() |> .map(add 1) |> .collect()
}

pub fn counts(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new () in
    for w in text.split_whitespace() do
      let c = counts.entry(w).or_insert(0) in
      *c <- *c + 1
    done;
    counts
}

#[rust]
fn checksum(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0, |sum, b| sum.wrapping_mul(31).wrapping_add(u32::from(*b)))
}

// In a test as anywhere else, the arguments of `assert_eq!` are OCaml.
fn check() {
    assert_eq!(add 1 2, 3);
    assert_eq!(incremented &[|1; 2|], vec![2, 3])
}

}

check();
assert_eq!(counts("a b a")["a"], 2);
assert_ne!(checksum(b"ab"), checksum(b"ba"));
```

Inside OCaml code, the arguments of std's formatting, assertion and
panicking macros are OCaml expressions (next section), so tests read like
the code they test.

rust-analyzer expands a macro call again whenever its contents change, so
in an editor the cost of a completion grows with the `rsml!` it is in.
Measured, one `rsml!` around a file is fine up to about 1500 lines (under
100 ms a completion, against about 10 ms in plain Rust); beyond that, write
one `rsml!` per group of related items, and each completion costs about
what it costs in plain Rust
([docs/editors.md](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md#big-files) has the numbers).

This is a function-like macro rather than an attribute because rustc parses
the item under an attribute macro as Rust before the macro runs, and an
OCaml body is not Rust.

## Syntax

Expressions: `let .. in`, `let rec .. and ..`, `fun`, `function`, `match ..
with` (guards, or-patterns, `as`, constructors, tuples, lists, ranges
`'a'..='z'` and `0..10`), `if then else`, `;` sequencing, `begin .. end`,
`for .. to / downto / til .. do .. done`, `for p in e do .. done`, `while .. do
.. done` (`while true` is Rust's `loop`), `return e`, `break e`, `continue`,
ranges `a..b`, `a..=b`, `a..`, `..b`, `unsafe { .. }`, `async { .. }`,
tuples, lists `[a; b]`, arrays `[|a; b|]`, `::`, `@`, `^`, records `Point {
x = 1; y }` and `{ p with x = 2 }`, `ref` / `!` / `:=` / `incr` / `decr`,
`not`, `assert`, labels `~x:e` (matched by name), `(e : t)`, placeholders,
user-defined operators `let ( +! ) a b = ..`, `(* comments *)`.

The block itself is an OCaml toplevel: `let` without `in`, `external`,
`open` and `;;`-separated expressions are allowed, and the last expression
is the value. `open std::cmp` is `use std::cmp::*;` for the rest of the
block (`let open std::cmp in e` for one expression).

Operators follow OCaml's precedence and use OCaml's spelling: `=` / `<>`
compare, `mod`, `land`, `lor`, `lxor`, `lsl`, `lsr`, `asr`, `not`, `+.` (same
as `+`), `**` (`f64::powf`). `==` and `!=` are accepted as `==` and `!=`.
`>>` and `<<` are composition, not shifts.

Types are Rust types (`i64`, `&str`, `Vec<String>`, `&'a [u8]`) written with
OCaml's arrows and tuples: `i64 -> i64` is a function, `i64 * String` is
`(i64, String)`, and `'a` is a type variable, which in a `let rec`
signature becomes a generic parameter (`Vec<'a>` is `Vec<A>`). A `'name`
directly after `&` is a lifetime, as is `'static`.

### Rust inside OCaml

* `Module.item` is the path `Module::item` (`Vec.new ()` is `Vec::new()`),
  and Rust paths work as written when there is no space around `::`
  (`std::mem::swap a b`, `i32::MAX`, `Vec::<u8>::new ()`). `x :: xs` with
  spaces is cons; so is `h::t` when `h` is bound in the `ml!` block. In
  patterns, a path may start lowercase when it reaches a capitalized name
  (`std::cmp::Ordering::Less`, `i32::MAX`); `x::rest` is a cons.
* Calls and indexing written Rust style, with `(` or `[` right after the
  function or value: `add(1, 2)` is `add(1, 2)`, `f(a)(b)` calls the
  result of `f(a)`, `v[i]`, `m[&k]`, `xs[1..]`, `f(x)[0]`. Each item is
  one argument, and for a function whose arity the macro knows they count
  as that many: `add(1)` is a partial application. With a space it is
  OCaml: `f (a, b)` passes one tuple and `f [1; 2]` a list. A constructor
  applied this way is one value, as in Rust: `Some(x).unwrap()` and
  `Shape::Rect(w, h).area()` call methods of what they build, and
  `f Some(x)` passes one argument. Its argument still follows OCaml's
  rules (`Some(1, 2)` is `Some((1, 2))`). Spaced, it is OCaml again:
  `Some (x).unwrap()` is `Some (x.unwrap ())`.
* Method calls `xs.push(1)`, written without a space before the
  parenthesis (`x.f (a)` applies the field `f`), or OCaml style as
  `.push xs 1`. The name after `.` follows it directly (`. len` is an
  error). Turbofish, `e?`, `e.await`, `e as t`, `&x`, `&mut x`, `*x`.
* Macros. The arguments of std's `print!`, `println!`, `eprint!`,
  `eprintln!`, `format!`, `format_args!`, `write!`, `writeln!`, `panic!`,
  `unreachable!`, `todo!`, `unimplemented!`, `assert!`, `assert_eq!`,
  `assert_ne!`, their `debug_` forms, `dbg!` and `vec!` (also as
  `std::name!`) are OCaml expressions separated by `,`:
  `assert_eq!(add 1 2, 3)`, `println!("{}", xs |> .len)`, `vec![0; n]`. A
  format string stays as written, and after it `name = e` is a named
  argument (`(a = b)` compares). Every other macro gets Rust tokens, as
  written (`ml! { .. }` works inside them).
* `*e` is Rust's dereference, binding like Rust's: `*x.f` is `*(x.f)`,
  where OCaml's `!x.f` is `(!x).f`. `(*x)` reads as the start of an OCaml
  comment, so write `( *x)` (as the formatter does) or `!x`. After an
  argument `*` multiplies: `f *x` is `f * x`.
* `let mut x = ..` and `x <- v` for mutable locals, and `let mut f x = ..`
  for a local function that mutates what it captures (Rust's `FnMut`).
* `fun move x -> ..` for a `move` closure.
* `Some (a, b)` is `Some((a, b))`; other constructors applied to a tuple get
  one field per element (`Rect (w, h)` is `Rect(w, h)`); `Foo ((a, b))` keeps
  the tuple.
* Rust keywords are not names (`let loop = 1` is an error); write a raw
  identifier, `r#loop`, for one. A raw identifier is always a name, and
  stays raw: `r#match x` calls a function named `match`. Words that Rust
  only reserves (`gen`, `try`, `yield`, `box`, `macro`, `abstract`, ..)
  are ordinary names, emitted raw (`let gen = 3` is `let r#gen = 3`), so
  `rng.gen()` works in every edition. Rust habits get an error that says
  what to write instead: `|x| e`, `if let`, `match x { .. }`, `x += 1`,
  `{ a; b }`, `[1, 2]` (a list of one pair in OCaml) and `let x = e;`.
* `f ()` calls a function with no arguments (`f()`), and `f (())` passes
  the unit value (`f(())`); likewise `() |> f` and `(()) |> f`. With other
  arguments `()` is always a value: `f () x` is `f((), x)`. Methods have
  no such ambiguity: `.iter xs` takes no arguments and `.send tx ()`
  passes `()`.

```rust
use rsml::rsml;

rsml! {
    fn first_negative(xs: &[i64]) -> Option<usize> {
        for (i, x) in xs.iter().enumerate() do
          if !x < 0 then return Some(i)
        done;
        None
    }

    fn positive_total(xs: &[i64]) -> i64 {
        xs |> .iter() |> .copied() |> .filter(fun x -> !x > 0) |> .sum()
    }
}

assert_eq!(first_negative(&[3, -1, -2]), Some(1));
assert_eq!(positive_total(&[1, -2, 3]), 4);
```

### Semantics worth knowing

The macro changes syntax, not Rust's rules:

* Values move. `f xs` moves `xs`; `f &xs` borrows it.
* Lists are `Vec`. `x :: [y]` is `::std::vec![x, y]`, but `x :: xs` with a
  non-literal tail inserts at the front (O(n)).
* List patterns are slice patterns. `match xs with x :: rest -> ..` matches
  `xs` as a slice, so `x` is `&T` and `rest` is `&[T]` (use `!x` to copy the
  element out). Recursive list functions take slices: `(l : &[i64])`.
* Closures borrow what they capture, like Rust closures. A `fun` that is the
  result of the block (or of an enclosing closure) is made `move` so it can
  be returned, unless it assigns to something. Closures passed as arguments
  are never `move` implicitly; use `fun move` when one must outlive the
  function (for example in a returned iterator).
* `let f x = ..` is a closure, so its parameter types are inferred like a
  Rust closure's; annotate them when you call methods on them. Closures are
  not generic, so a local function is used at one type.
* `let rec` compiles to a nested `fn`, which needs annotations and cannot
  capture local variables:
  `let rec fact (n : i64) : i64 = ..` or `let rec f : i64 -> i64 = function ..`.
  One that calls itself only in tail position (`go (acc + 1) rest` as its
  value) compiles to a loop instead, which runs in constant stack space in
  debug builds too.
* `let r = ref e` is a `let mut` local; `!r` reads it, `r := v` and
  `incr r` write it, and passing `r` passes `&mut r`.
* `!` dereferences and `not` negates, in macro arguments too:
  `assert!(not (xs.is_empty()))`. `!` binds tighter than `.`, so Rust's
  `assert!(!xs.is_empty())` would assert the opposite; it is an error, as
  is `!` before a comparison or `true`.
* Ranges are Rust's, in patterns too: `0..n` stops before `n` and
  `'a'..='z'` includes `'z'` (OCaml's `'a' .. 'z'` would include it; here
  it does not).
* `assert false` is `::core::unreachable!("assert false")`. `try .. with` and `raise` are not
  supported; use `Result`, `match`, `?` and `panic!`.

```rust
use rsml::ml;

#[derive(Debug, PartialEq)]
enum Shape { Circle(f64), Rect(f64, f64) }

fn total_area(shapes: &[Shape]) -> f64 {
    ml! {
        let area (s : &Shape) =
          match s with
          | Shape::Circle r -> 3.14 *. !r *. !r
          | Shape::Rect (w, h) -> !w *. !h
        in
        shapes |> .iter |> .map _ area |> .sum
    }
}

fn length(l: &[i32]) -> i64 {
    ml! {
        let rec go (acc : i64) (l : &[i32]) : i64 =
          match l with [] -> acc | _ :: rest -> go (acc + 1) rest
        in
        go 0 l
    }
}

assert_eq!(total_area(&[Shape::Rect(2.0, 3.0)]), 6.0);
assert_eq!(length(&[1, 2, 3]), 3);
```

## No standard library

Everything is Rust's: `Iterator::map`, `Option::unwrap_or`, `str::len`,
`String::from`, `format!`, `println!`, `std::cmp::max`. Methods can be
called Rust style (`w.len()`), as functions (`.len w`, `it |> .filter _ p`)
or through their type (`s |> str::len`).

```rust
use rsml::ml;

let mut words = vec!["pear", "fig", "apple"];
let line = ml! {
    .sort words;
    words
    |> .iter
    |> .map _ (fun w -> format!("{:<6}({})", w, w.len()))
    |> .collect::<Vec<_>>
    |> .join _ " "
};
assert_eq!(line, "apple (5) fig   (3) pear  (4)");
```

## Editors

rust-analyzer understands the code inside both macros through their
expansion, so highlighting, hover, go to definition, diagnostics and
completion work inside OCaml code: methods after `xs.`, `xs |> .` and
`.‸ xs`, items after `Vec::` and `Vec.`, variants in patterns, locals,
fields and types, including in unfinished code (`if ‸`, `match o with ‸`).
Every OCaml keyword is colored as a keyword (`then`, `with`, `done`,
`fun`, ..) and every operator as an operator. rustc's errors point at the
OCaml code they are about. [docs/editors.md](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md) explains
how, with setup notes for Neovim and blink.cmp, Vim, and the list of
positions `scripts/check_completion.py` checks.

## Formatting

`rsml-fmt` formats the OCaml in `ml!` blocks and `rsml!` functions, which
rustfmt leaves alone, and has rustfmt format the Rust inside `rsml!`.
`rsml-fmt --rustfmt` runs rustfmt on the rest of the file too, so one
command formats everything; editors can run it in place of rustfmt
([docs/editors.md](https://github.com/uhhJoyz/rsml-lang/blob/main/docs/editors.md#formatting-on-save) has the setup for
rust-analyzer, VS Code and Neovim). Every layout decision is configurable
in `rsmlfmt.toml`; see [`rsml-fmt/README.md`](https://github.com/uhhJoyz/rsml-lang/blob/main/rsml-fmt/README.md).

```sh
cargo run -p rsml-fmt -- --rustfmt src tests
```

## Crates

* `rsml-lang` (imported as `rsml`): the macros, re-exported with the few
  helpers their expansions call. The only dependency you need.
* `rsml-macros`: the procedural macros, a thin layer over `rsml-compiler`.
* `rsml-compiler`: lexer, parser and code generator, as an ordinary library.
* `rsml-fmt`: the formatter, which uses `rsml-compiler`'s parser.

## Checking changes

`scripts/check_all.sh` runs every check in this repository and prints a
summary: the tests (also compiled as edition 2021, `tests/edition2021`);
the demos of the toy examples; the documentation (`scripts/check_docs.py`:
every expansion in a table or in a sentence such as "`x |> f` is `f(x)`"
against the macros, every completion position against the cases
rust-analyzer is checked on, the links, and the comments and index of the
toy examples); `cargo doc` with warnings denied; the files `cargo package`
would publish; rustfmt and `rsml-fmt --rustfmt` on the repository's own
code (the formatter's test fixtures stay messy); clippy on the generated
code; the check that `rsml!` expansion time grows linearly; the zero-cost
assembly comparison; the formatter's corpus test under every config and a
long fuzz run; and, where rust-analyzer, Neovim
and Vim are installed, the completion, highlighting, error-span, editor
formatting, latency and Vim checks. `--quick` leaves out the slow ones.
CI (`.github/workflows/ci.yml`) runs `--quick` on Linux and macOS, the
rust-analyzer and Vim checks, the tests on Rust 1.88 (the oldest
supported), and once a week the corpus and fuzz runs.

When a table in the documentation shows an expansion,
`python3 scripts/check_docs.py --expand 'x |> f a'` prints the one to
write.

## Limitations

* Rust's lexer runs before the macro, so OCaml `{|quoted strings|}`,
  identifiers with primes (`x'`) and comments containing unbalanced quotes
  or brackets do not work. Prefer `//` and `/* */` comments: they are Rust
  comments, which may contain anything and which editors color as
  comments, while `(* .. *)` is colored as code (except in Neovim with
  `editors/nvim`'s query and in Vim with `editors/vim`'s syntax file). `rsml-fmt` rewrites them with
  `comments = "rust"`.
* No modules, functors, objects, polymorphic variants, GADTs, `type`
  definitions, exceptions or `lazy`. Define types in Rust and match on them.
* `lsr` and `asr` are both Rust's `>>`: on a signed integer `lsr` is an
  arithmetic shift, not OCaml's logical one (cast to an unsigned type
  first for that).
* The expansions need edition 2018 or later: they name `::core::..`,
  which edition 2015 reads as a path from the crate root. Raw identifiers
  (`r#gen`) work in every edition.
* Clippy lints the generated Rust like any other code. Lints that OCaml
  code could not act on are avoided (`n <- n + 1` is allowed to be
  `n = n + 1`, as OCaml has no `+=`), and `scripts/check_clippy.sh` checks
  every notation for others.

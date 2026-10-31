# Functional notation

Every notation on this page parses inside `ml! { .. }` and in the bodies of
`rsml!` functions. Each entry shows the rsml code you write and the Rust
the macro turns it into (as rustc sees it; `scripts/check_docs.py` checks every
row against the macros). `f`, `g`, `xs` and so on stand for whatever is in
scope. The names of the locals the macros make up say what they stand for,
since rustc's errors can show them: `_0`, `_1` for placeholders,
`receiver` and `tuple` for the argument of a method or field section,
`argument` for that of a composition or `function`, `arg2` for a missing
second argument, `value1` for a value computed once, and so on. Macro
hygiene keeps them apart from your names. (Under rust-analyzer they start
with `__rsml_`: it offers them as completions in the few places where they
are in scope of your code, listed in [editors.md](editors.md#limitations).)

Every section has toy examples, small commented programs that use its
notations, in [examples/toy](../examples/toy/README.md).

Under each section, **Completes at** lists cursor positions (`‸`) where
rust-analyzer offers completions, and what it offers. Every one of them is
checked by `scripts/check_completion.py`; [editors.md](editors.md) explains
how completion works and lists all the checked positions.

* [Application](#application)
* [Arity and partial application](#arity-and-partial-application)
* [Pipelines: `|>` and `@@`](#pipelines--and-)
* [Composition: `>>` and `<<`](#composition--and-)
* [Operator sections and user-defined operators](#operator-sections-and-user-defined-operators)
* [Placeholders: `_` and `_0`, `_1`, ..](#placeholders-_-and-_0-_1-)
* [Methods as functions: `.name`](#methods-as-functions-name)
* [Rust-style methods, fields and calls](#rust-style-methods-fields-and-calls)
* [Rust-style calls and indexing: `f(a, b)`, `v[i]`](#rust-style-calls-and-indexing-fa-b-vi)
* [Macros](#macros)
* [Paths](#paths)
* [Lambdas: `fun` and `function`](#lambdas-fun-and-function)
* [Local functions and bindings](#local-functions-and-bindings)
* [Patterns](#patterns)
* [Operators that become calls](#operators-that-become-calls)
* [Ranges](#ranges)
* [Control flow](#control-flow)
* [Records and collections](#records-and-collections)
* [Types](#types)
* [Comments](#comments)
* [Rust habits](#rust-habits)

## Application

Juxtaposition applies a function to its arguments. A function the macro
cannot see (anything defined outside the block) is called with every
argument it is given.

| rsml                       | Rust                       |
|----------------------------|----------------------------|
| `f x y`                    | `f(x, y)`                  |
| `std::cmp::max a b`        | `std::cmp::max(a, b)`      |
| `Vec::with_capacity 10`    | `Vec::with_capacity(10)`   |
| `Vec.new ()`               | `Vec::new()`               |
| `String.from "a"`          | `String::from("a")`        |
| `f &xs`, `f &mut xs`       | `f(&xs)`, `f(&mut xs)`     |

**Labels.** `~x:e` gives `e` to the parameter labeled `x`, and `~x`
stands for `~x:x`. A function's labels are the names of its `~x` parameters (`let f
~x ~y = ..`, `let rec`, `let f = fun ~x ~y -> ..`; `~x:p` labels the
pattern `p`, and `~(x : t)` adds a type), and every parameter name of a
`fn` of the same `rsml!`. Labeled arguments go to their parameters in any
order; the others fill the parameters left, in order. The arguments are
evaluated in the order written. A parameter left out before one that is
given makes a function of it. A function the macro cannot see the
parameters of (a Rust function from elsewhere, a closure parameter) takes
no labels: `Option::unwrap_or None ~default:3` is an error.

| rsml                                           | Rust |
|------------------------------------------------|------|
| `let f ~x ~y = x - y in f ~y:1 ~x:3`           | `{ let f = \|x, y\| x - y; f(3, 1) }` |
| `let f ~x ~y = x - y in f ~y:1 2`              | `{ let f = \|x, y\| x - y; f(2, 1) }` |
| `let f ~x ~y = x - y in f ~y:(g a) ~x:(h b)`   | `{ let f = \|x, y\| x - y; { let value1 = g(a); let value2 = h(b); f(value2, value1) } }` |
| `let f ~x ~y = x - y in f ~y:1`                | `{ let f = \|x, y\| x - y; move \|_0\| f(_0, 1) }` |

**Unit.** `()` alone after a function means "no arguments"; anywhere else
it is the unit value (after a [method](#methods-as-functions-name)'s
receiver too), and `(())` is the unit value everywhere.

| rsml        | Rust          |
|-------------|---------------|
| `f ()`      | `f()`         |
| `f (())`    | `f(())`       |
| `f () x`    | `f((), x)`    |

**Constructors.** A constructor applied to a tuple gets one field per
element, except `Some`, which wraps the tuple. Double parentheses keep a
tuple as one field.

| rsml                   | Rust                   |
|------------------------|------------------------|
| `Some (1, 2)`          | `Some((1, 2))`         |
| `Shape::Rect (w, h)`   | `Shape::Rect(w, h)`    |
| `Pair ((1, 2))`        | `Pair((1, 2))`         |

**Completes at**

| Cursor                     | Offers                                   |
|----------------------------|------------------------------------------|
| `ad‸ 1 2`                  | functions in scope (`add`)               |
| `‸ 1 2`                    | functions in scope                       |
| `Vec.wi‸ 10`, `Vec::wi‸ 10`| `with_capacity`                          |
| `geometry::‸ &shape`       | the module's items (`area`)              |
| `std::cmp::‸ 1 2`          | `max`, `min`, ..                         |
| `add ‸`, `add 1 ‸`, `add ‸ 2` | values in scope                       |
| `add (n‸ + 1) 2`           | values in scope                          |
| `add ~a:n‸ 2`              | values in scope                          |
| `Some ‸`, `Some (n, ‸)`, `Shape::Circle ‸` | values in scope          |

## Arity and partial application

rsml functions are curried: applied to some of their arguments, they wait
for the rest. Rust functions take all their arguments at once. The macro
calls a function directly when it can see that all its
arguments are there, and makes a closure when some are missing. It can see
the arity of:

* functions defined in the same block (`let f x y = ..`),
* closures (`fun x -> e` and `|x| e`), sections and placeholder functions,
* closure-typed parameters of `rsml!` functions (`impl Fn(..)`,
  `&dyn Fn(..)`, generics bounded by `Fn(..)`),
* the functions of the same `rsml!`, in each other's bodies: by name at
  the same module level (a `mod` sees its own, and its parent's through
  `use super::*` or `use super::name`), and associated functions without
  `self` as `Self::name` in their `impl` and `Type::name` in the module,
* `external` declarations and `(e : t)` constraints with a function type.

| rsml                                                     | Rust |
|----------------------------------------------------------|------|
| `let add x y = x + y in add 1 2`                         | `{ let add = \|x, y\| x + y; add(1, 2) }` |
| `let add x y = x + y in add 1`                           | `{ let add = \|x, y\| x + y; move \|arg2\| add(1, arg2) }` |
| `let add x y = x + y in add (expensive ())`              | `{ let add = \|x, y\| x + y; { let value1 = expensive(); move \|arg2\| add(value1, arg2) } }` |
| `let add x y = x + y in add(1)`                          | `{ let add = \|x, y\| x + y; move \|arg2\| add(1, arg2) }` |
| `let add x y = x + y in add(1)(2)`                       | `{ let add = \|x, y\| x + y; add(1, 2) }` |
| `let add = \|x, y\| x + y in add 1`                       | `{ let add = \|x, y\| x + y; move \|arg2\| add(1, arg2) }` |
| `external add : i64 -> i64 -> i64 ;; add 1`              | `move \|arg2\| add(1, arg2)` |
| `external adder : i64 -> (i64 -> i64) ;; adder 1 2`      | `adder(1)(2)` |
| `external plus : _ -> _ -> _ = "std::ops::Add::add" ;; plus 1` | `move \|arg2\| std::ops::Add::add(1, arg2)` |
| `(add 6 : i64 -> i64)`                                   | `move \|arg2\| add(6, arg2)` |
| `rsml! { fn add(a: i64, b: i64) -> i64 { .. } fn f() { add 1 } }` | `.. fn f() { move \|arg2\| add(1, arg2) }` |
| `rsml! { fn now() -> i64 { .. } fn f() { now () } }`     | `.. fn f() { now() }` |
| `rsml! { fn adder(n: i64) -> impl Fn(i64) -> i64 { .. } fn f() { adder 1 2 } }` | `.. fn f() { adder(1)(2) }` |
| `rsml! { impl P { fn new(x: i64, y: i64) -> P { .. } fn o() -> P { Self::new 0 0 } } }` | `.. Self::new(0, 0) ..` |

Arguments of a partial application are evaluated once, when it is made.
In an `external` type only the arrows count: each `->` at the
top is one argument, and parentheses mark a function that returns a
closure.

The shape of a function of the same `rsml!` comes from its signature: one
group of its parameters (a pattern such as `(a, b): (i64, i64)` is one;
none is `()`), and one more for each closure it returns (`impl Fn(..)`,
`Box<dyn Fn(..)>`, `&dyn Fn(..)`, or a generic with an `Fn(..)` bound),
except that an `async fn` returns a future. Methods taking `self` are not
included, and a name defined twice with different shapes (under different
`#[cfg]`s) is left unknown. Functions under `#[rust]` count too.

```rust
use rsml::rsml;

rsml! {
    // `add` is a closure-typed parameter, so `add 10` is a partial application.
    fn add_all(add: impl Fn(i64, i64) -> i64, xs: Vec<i64>) -> Vec<i64> {
        xs |> .into_iter |> .map _ (add 10) |> .collect
    }
}

assert_eq!(add_all(|a, b| a + b, vec![1, 2]), vec![11, 12]);
```

**Completes at**: every argument position, as under
[Application](#application), with or without a character typed: `plus ‸`,
`plus n‸`, `plus ‸ 2`, `plus 1 ‸`, `plus(‸)`, `plus(1, ‸)`, `inc ‸` for a
`let inc x = x + 1`, `add ‸` for a Rust function the macro does not know.
At `plus ‸` (a local `let plus x y`, a closure-typed parameter, a function
of the same `rsml!`, `Self::make` in its `impl`), values of the type of
the first parameter are listed first when that type is declared. For
rust-analyzer the macro binds the arguments of a partial application to
locals ahead of its closure, so that `plus` and `plus x` line up
([editors.md](editors.md#how-it-works) explains why).

## Pipelines: `|>` and `@@`

`x |> f` is `f x` and `f @@ x` is `f x`. The piped value goes *last*, and
no arity is needed: the result is always a plain call.

| rsml                   | Rust                   |
|------------------------|------------------------|
| `x \|> f`              | `f(x)`                 |
| `x \|> f a`            | `f(a, x)`              |
| `x \|> f a \|> g b c`  | `g(b, c, f(a, x))`     |
| `f @@ g @@ x`          | `f(g(x))`              |
| `xs \|> Vec::len`      | `Vec::len(xs)`         |
| `() \|> f`             | `f()`                  |
| `(()) \|> f`           | `f(())`                |
| `x \|> f(_, a)`        | `f(x, a)`              |
| `x \|> f(a)`           | `f(a)(x)`              |

A Rust-style call `f(a)` is complete, so `x |> f(a)` passes `x` to its
result (with a function the macro knows, `add(1)` is a partial
application and `x |> add(1)` is `add(1, x)`).

Rust functions usually take their data first; with placeholders a pipeline
puts it anywhere: `x |> f _ 1` is `f(x, 1)`, and `it |> .map _ g` is
`it.map(g)` (see [methods as functions](#methods-as-functions-name)).

**Completes at**

| Cursor                 | Offers                           |
|------------------------|----------------------------------|
| `xs \|> ‸`             | functions and values in scope    |
| `‸ \|> add 1`          | values in scope                  |
| `add 1 @@ ‸`           | values in scope                  |
| `xs \|> .‸`            | methods of `xs`                  |

## Composition: `>>` and `<<`

`f >> g` applies `f`, then `g`; `f << g` applies `g`, then `f`. A
composition is a closure, unless it is applied right away.

| rsml                                        | Rust |
|---------------------------------------------|------|
| `f >> g`                                    | `move \|argument\| g(f(argument))` |
| `f << g`                                    | `move \|argument\| f(g(argument))` |
| `f >> g >> h`                               | `move \|argument\| h(g(f(argument)))` |
| `(f >> g) x`                                | `g(f(x))` |
| `x \|> (f >> g)`                            | `g(f(x))` |
| `let add x y = x + y in add 1 >> add 2`     | `{ let add = \|x, y\| x + y; move \|argument\| add(2, add(1, argument)) }` |
| `.iter >> .count`                           | `move \|argument\| argument.iter().count()` |

`>>` and `<<` are always composition, never shifts; shifts are `lsl`,
`lsr` and `asr`. Both `lsr` and `asr` are Rust's `>>`, which the operand's
type decides: arithmetic (sign-filling) for signed integers, logical for
unsigned ones. For a logical shift of a signed value, shift it as an
unsigned type (`((x as u64) lsr 2) as i64`).

**Completes at**: `add 1 >> ‸` (functions in scope), `xs |> (.iter >> .‸)`
(methods of the iterator).

## Operator sections and user-defined operators

An operator in parentheses is a function of its operands; applied to one
argument it is a partial application of the left operand.

| rsml         | Rust                         |
|--------------|------------------------------|
| `( + )`      | `\|lhs, rhs\| lhs + rhs`     |
| `( + ) 1`    | `move \|rhs\| 1 + rhs`       |
| `( - ) 10`   | `move \|rhs\| 10 - rhs`      |
| `( mod )`    | `\|lhs, rhs\| lhs % rhs`     |

`( * )` needs its spaces, since `(*` opens a comment. Keyword operators
(`mod`, `land`, `lor`, `lxor`, `lsl`, `lsr`, `asr`, `or`) work in sections
too.

Operators defined with `let ( op ) a b = ..` are ordinary two-argument
functions, with the precedence and associativity of their first character
(`+!` binds like `+`):

| rsml                                                   | Rust |
|--------------------------------------------------------|------|
| `let ( +! ) a b = a + 2 * b in 1 +! 2`                 | `{ let mut operator1 = \|a, b\| a + (2 * b); operator1(1, 2) }` |
| `let ( \|? ) o d = Option::unwrap_or o d in o \|? 0`   | `{ let mut operator1 = \|o, d\| Option::unwrap_or(o, d); operator1(o, 0) }` |

An operator starting with `!` (other than `!=`) cannot be defined: it
would be a prefix operator, and `!` before an expression is a dereference
(`!*p` is `*(*p)`), so it could never be used. `!=` is infix.

**Completes at**: `( + ) ‸` (values in scope) and both operands of any
operator (`n + ‸`, `n + ‸ * 2`, `n mod ‸`, `n = ‸`, `n > 0 && ‸`).

## Placeholders: `_` and `_0`, `_1`, ..

A placeholder in an expression makes the expression a function, with one
parameter per placeholder. `_N` is parameter `N` (from 0); `_` is the
lowest number not written elsewhere in the expression.

| rsml                         | Rust |
|------------------------------|------|
| `f _ 1`                      | `move \|_0\| f(_0, 1)` |
| `f _1 _0`                    | `move \|_0, _1\| f(_1, _0)` |
| `f _1 _ _`                   | `move \|_0, _1, _2\| f(_1, _0, _2)` |
| `( * ) _0 _0`                | `move \|_0\| _0 * _0` |
| `map (_ * 2) xs`             | `map(\|_0\| _0 * 2, xs)` |
| `map (f _1) xs`              | `map(\|_, _1\| f(_1), xs)` |
| `map (_.len()) xs`           | `map(\|_0\| _0.len(), xs)` |
| `map (f (_ : &str) 1) xs`    | `map(\|_0: &str\| f(_0, 1), xs)` |
| `(1 + _ * 2)`                | `move \|_0\| 1 + (_0 * 2)` |
| `filter (_ > 0 && ok ()) xs` | `filter(\|_0\| (_0 > 0) && ok(), xs)` |

**How far a placeholder reaches.** It belongs to the smallest *full*
expression around it: the inside of parentheses, a list or array item, a
method argument, a record field, a `let` value or body, a branch, a match
case, a sequence item, or a side of `|>`, `@@`, `>>` or `<<`. Applications
and other operators do not stop it, so `(1 + _ * 2)` is one function and
`f (g _) x` is `f (fun a -> g a) x`. A placeholder alone, possibly
annotated, belongs to the expression around it: `f (_ : &str) 1` is
`fun (a : &str) -> f a 1`.

**Applied directly, there is no closure**, and the rest of the expression
is evaluated once, when the function is made:

| rsml                   | Rust |
|------------------------|------|
| `x \|> f _ 1`          | `f(x, 1)` |
| `(g _1 _0) a b`        | `g(b, a)` |
| `(g _1 _0) a`          | `move \|_1\| g(_1, a)` |
| `f (expensive ()) _`   | `{ let value1 = expensive(); move \|_0\| f(value1, _0) }` |

In patterns `_` is still the wildcard, and `_0`, `_1`, .. are not variable
names anywhere.

**Completes at**: `add _ ‸`, `add _1 ‸` (values), `xs |> .iter |> .map _
(_.‸)` (methods of the element, whose type the call gives the closure),
`add (_ : ‸) 1` (types).

## Methods as functions: `.name`

Where an expression starts, `.name` is a method as a function of its
receiver and arguments. Every argument after the receiver goes to the
method, and the receiver alone is a call without arguments.

| rsml                                         | Rust |
|----------------------------------------------|------|
| `.len s`                                     | `s.len()` |
| `.push xs 1`                                 | `xs.push(1)` |
| `.send tx ()`                                | `tx.send(())` |
| `.get (f x) 0`                               | `f(x).get(0)` |
| `.collect::<Vec<_>> it`                      | `it.collect::<Vec<_>>()` |
| `.await fut`                                 | `fut.await` |
| `xs \|> .iter \|> .count`                    | `xs.iter().count()` |
| `it \|> .map _ f`                            | `it.map(f)` |
| `xs \|> .iter \|> .map _ (_ * 2) \|> .sum::<i64>` | `xs.iter().map(\|_0\| _0 * 2).sum::<i64>()` |
| `.len`                                       | `\|receiver\| receiver.len()` |
| `map (.len) xs`                              | `map(\|receiver\| receiver.len(), xs)` |
| `map (.len (_ : &str)) xs`                   | `map(\|_0: &str\| _0.len(), xs)` |
| `.iter >> .count`                            | `move \|argument\| argument.iter().count()` |
| `xs \|> .map(f)`                             | `xs.map(f)` |
| `.push(1) xs`                                | `xs.push(1)` |
| `xs \|> .iter() \|> .map(fun x -> x * 2) \|> .sum::<i64>()` | `xs.iter().map(\|x\| x * 2).sum::<i64>()` |
| `.iter()`                                    | `\|receiver\| receiver.iter()` |
| `.map(f)`, `.map(f) xs`                      | `move \|receiver\| receiver.map(f)`, `xs.map(f)` |
| `.0`, `.0 t`, `x \|> .1`                    | `\|tuple\| tuple.0`, `t.0`, `x.1` |
| `pairs \|> .iter() \|> .map(.0)`             | `pairs.iter().map(\|tuple\| tuple.0)` |
| `.0.1 t`                                     | `t.0.1` |
| `xs \|>.iter() \|>.count()`, `t \|>.0`        | `xs.iter().count()`, `t.0` |
| `.iter >>.count`                             | `move \|argument\| argument.iter().count()` |

**Field sections.** `.0`, `.1`, .. where an expression starts are tuple
fields as functions of the tuple, like methods: `.0` is `fun r -> r.0`,
applied as `x |> .0` it is `x.0`, and arguments after the tuple apply the
field (`.0 t x` is `(t.0)(x)`). Only numbered fields: `.name` is always a
method.

**Method sections.** With its arguments written Rust style, right after
the name (`.map(f)`, `.collect::<Vec<_>>()`), the method takes exactly
those arguments and waits for its receiver: `.map(f)` is `fun r ->
r.map(f)`, and applied, as in `x |> .map(f)`, it is `x.map(f)` with no
closure. Arguments after the receiver apply the result. With a space,
`.name (a)` is the method of `a`, as before.

Rules:

* The name is written right after the dot: `.len`, never `. len`. (While
  you type, `. xs` reads as a method still to be named, with `xs` its
  receiver.)
* The space before the dot can be left out after `|>`, `>>`, `<<` and
  `@@`. Rust lexes `xs |>.iter()` with one operator `|>.`, and unless a `( |>. )` is defined (as for `+=`, see
  [Rust habits](#rust-habits)) that is `|>` and the section:
  `xs |>.iter()` is `xs |> .iter()`, `t |>.0` is `t |> .0`, and
  `.iter >>.count` is `.iter >> .count`. rsml-fmt prints the space. Only
  these four operators are read this way, and only when a name or a
  number follows the dot: `x +.y` stays `+.`, and `xs |>. iter` is an
  unknown operator `|>.`.
* After an expression, `.name` is that expression's field or method
  (`t.len`, `xs.iter()`, and chains broken across lines), so as an argument
  a method needs parentheses: `map (.len) xs`.
* Because the receiver fills the "no arguments" role, `()` after it is a
  value: `.send tx ()` is `tx.send(())`.
* A closure that calls a method needs its receiver's type, as in Rust:
  annotate the placeholder (`.len (_ : &str)`) or call through the type
  (`Iterator::map _ f`) where nothing else provides it.

**Completes at**

| Cursor                                   | Offers                      |
|------------------------------------------|-----------------------------|
| `.‸ xs`, `.it‸ xs`                       | methods of `xs`             |
| `.‸ xs 1`, `.pu‸ xs 1`                   | methods of `xs`             |
| `.‸ (s.trim())`                          | methods of `&str`           |
| `.‸ p`                                   | methods of `Point`          |
| `xs \|> .‸`, `xs \|> .it‸`               | methods of `xs`             |
| `xs \|>.‸`, `xs \|>.it‸`                 | methods of `xs`             |
| `xs \|> .iter \|> .‸`                    | methods of the iterator     |
| `xs \|> .iter \|> .‸ \|> .sum::<i64>`    | methods of the iterator     |
| `xs \|> .iter \|> .ma‸ _ (fun x -> x)`   | `map`, ..                   |
| `xs \|> .iter() \|> .‸`, `xs \|> .iter() \|> .ma‸(fun x -> x)` | methods of the iterator |
| `xs \|> .map(‸)`                         | values in scope             |
| `xs \|> .iter() \|> .map(fun x -> x.‸)` | methods of `&i64`           |
| `twice (.‸) 1`                           | methods of `i64`, from `twice`'s `impl Fn(i64) -> i64` |
| `xs.into_iter() \|> .collect::<‸>`       | types                       |
| `.push xs ‸`                             | values in scope             |
| `pairs \|> .iter() \|> .map(.0) \|> .‸`   | methods of the iterator     |
| `let q = (p, 1) \|> .0 in q.‸`            | fields and methods of `Point` |

A method is completed from its receiver's type, so `.it‸` with no receiver
yet, and nothing around it that fixes one, has nothing to complete. Write
the receiver first (`xs |> .it‸`), or come back to the name once it is
there (`.it‸ xs`).

## Rust-style methods, fields and calls

Rust's own call syntax works in rsml code, with rsml expressions as
arguments.

| rsml                                                  | Rust |
|-------------------------------------------------------|------|
| `xs.push(1)`                                          | `xs.push(1)` |
| `xs.iter().map(fun x -> x + 1).collect::<Vec<_>>()`, `xs.iter().map(\|x\| x + 1).collect::<Vec<_>>()` | `xs.iter().map(\|x\| x + 1).collect::<Vec<_>>()` |
| `xs` / `.iter()` / `.count()` on separate lines       | `xs.iter().count()` |
| `p.0`                                                 | `p.0` |
| `r.f (x)`                                             | `(r.f)(x)` |
| `r.f ()`                                              | `(r.f)()` |
| `f t.len (x + 1)`                                     | `f(t.len, x + 1)` |
| `*self_`, `*x.f`, `!x.f`                              | `*self_`, `*x.f`, `(*x).f` |
| `&*s`, `**q`, `*p <- 3`                               | `&(*s)`, `*(*q)`, `*p = 3` |
| `f ( *x)`, `f *x`                                     | `f(*x)`, `f * x` |

A method call has no space before its parenthesis (`x.f(a)`); with a
space, `x.f (a)` applies the *field* `f` to `a`.

`*e` is Rust's dereference where an expression starts, and binds like
Rust's (`*x.f` is `*(x.f)`), while `!`, which dereferences too, binds
tighter than `.` (`!x.f` is `(!x).f`). Rust's glued prefix operators
(`&*s`, `!*p`, `**q`) are one operator each. `(*x)` opens an ML comment
(see [Comments](#comments)), so write `( *x)`, which is what rsml-fmt
prints, or `!x`. After an
argument, `*` multiplies: `f *x` is `f * x`.

**Completes at**

| Cursor                           | Offers                        |
|----------------------------------|-------------------------------|
| `xs.‸`, `xs.it‸`                 | methods of `xs`               |
| `xs.iter().‸`                    | methods of the iterator       |
| `xs` / `.iter()` / `.‸`          | methods of the iterator       |
| `xs.pu‸(1)`                      | `push`, ..                    |
| `p.‸`                            | fields and methods of `Point` |
| `o.‸`                            | methods of `Option<String>`   |
| `(n + 1).‸`, `(n : i64).‸`       | methods of `i64`              |
| `xs.push(‸)`                     | values in scope               |
| `xs.iter().map(fun x -> x.‸)`    | methods of `&i64`             |
| `*p.‸`                           | fields and methods of `Point` |

## Rust-style calls and indexing: `f(a, b)`, `v[i]`

A `(` written right after a function (a name, a path, its generic
arguments, a closing `)` or `]`, or the `?` of `x?`) is a Rust call: each item between the
parentheses is one argument, and a full expression of its own for
placeholders. A `[` written there is Rust indexing. With a space, both are
ML style: `f (a, b)` passes one tuple, `f [1; 2]` a list.

| rsml                                    | Rust |
|-----------------------------------------|------|
| `add(1, 2)`                             | `add(1, 2)` |
| `f()`, `f(())`                          | `f()`, `f(())` |
| `f(a)(b)`, `f(a) b`                     | `f(a)(b)` |
| `f (a, b)`, `f (a)(b)`                  | `f((a, b))`, `f(a(b))` |
| `(f >> g)(x)`                           | `g(f(x))` |
| `f(_, 1)`                               | `move \|_0\| f(_0, 1)` |
| `let add x y = x + y in add(1)`         | `{ let add = \|x, y\| x + y; move \|arg2\| add(1, arg2) }` |
| `let add x y = x + y in add(1, 2, 3)`   | `{ let add = \|x, y\| x + y; add(1, 2)(3) }` |
| `Some(1, 2)`, `Pair(1, 2)`              | `Some((1, 2))`, `Pair(1, 2)` |
| `Some(x).unwrap()`, `Shape::Rect(w, h).area()`, `Wrapper(v).0` | `Some(x).unwrap()`, `Shape::Rect(w, h).area()`, `Wrapper(v).0` |
| `f Some(x) y`, `f Some (x) y`           | `f(Some(x), y)`, `f(Some, x, y)` |
| `Some (x).unwrap()`                     | `Some(x.unwrap())` |
| `v[i]`, `m[&k]`, `f(x)[0]`, `xs[1..]`   | `v[i]`, `m[&k]`, `f(x)[0]`, `xs[1..]` |
| `f(x)?[0]`, `x?(1, 2)`, `x? (1, 2)`     | `f(x)?[0]`, `(x?)(1, 2)`, `(x?)((1, 2))` |
| `f [1; 2]`, `f[\|1; 2\|]`                | `f(::std::vec![1, 2])`, `f([1, 2])` |

* For a function whose arity the macro knows (a local `let`, a
  closure-typed `rsml!` parameter, an `external`), the items count as that
  many arguments, as if written ML style: `add(1)` is a partial
  application, `add(1)(2)` and `add(1, 2)` are the same call, and extra
  arguments apply the result. A Rust function's call `f(a)` is complete.
* `f()` is `f ()`: a call without arguments, or `()` passed to a function
  that takes one. `f(())` passes `()`.
* A constructor with its `(` written right after it is one value, like
  a Rust call: what follows (`.method()`, `.field`, `.0`, `[i]`, `?`,
  `.await`, another `(..)`) applies to the value it builds, and as an
  argument it is one argument. Its argument keeps the rules of
  constructors (see [Application](#application)): `Some(1, 2)` is
  `Some((1, 2))`, `Pair(1, 2)` is `Pair(1, 2)`. Spaced, it is ML-style
  application, where `.` binds tighter: `Some (x).unwrap()`
  is `Some (x.unwrap ())`. rsml-fmt keeps a constructor's argument
  against it or spaced, as written, so it never changes which is meant.
* `[|..|]` is always an array: `f[|1; 2|]` is `f [|1; 2|]`.
* `s.[i]` (the byte at `i` as a character, in OCaml) is an error: Rust
  strings are not indexed by position. Write `s.as_bytes()[i]` for the
  byte or `s.chars().nth(i)` for the character.
* rsml-fmt never adds or removes the space before such a `(` or `[`.

**Completes at**

| Cursor                                  | Offers                 |
|-----------------------------------------|------------------------|
| `add(‸)`, `add(1, ‸)`, `add(n‸, 1)`     | values in scope        |
| `std::convert::identity(add)(1, ‸)`     | values in scope        |
| `xs[‸]`                                 | values in scope        |
| `xs[0].‸`                               | methods of `i64`       |
| `Some(n).‸`                             | methods of `Option<i64>` |

## Macros

The arguments of std's formatting, assertion and panicking macros are
rsml expressions: `print!`, `println!`, `eprint!`, `eprintln!`,
`format!`, `format_args!`, `write!`, `writeln!`, `panic!`,
`unreachable!`, `todo!`, `unimplemented!`, `assert!`, `assert_eq!`,
`assert_ne!`, `debug_assert!`, `debug_assert_eq!`, `debug_assert_ne!`,
`dbg!` and `vec!`, called by name or as `std::`, `core::` or
`alloc::name!`. Every other macro (yours, `matches!`, ..) gets its tokens
as written, as Rust, and `ml! { .. }` works inside it.

| rsml                                         | Rust |
|----------------------------------------------|------|
| `assert_eq!(add 1 2, 3)`                     | `assert_eq!(add(1, 2), 3)` |
| `println!("{}", xs \|> .len)`                | `println!("{}", xs.len())` |
| `format!("{} {x}", f a, x = g b)`            | `format!("{} {x}", f(a), x = g(b))` |
| `assert!(a < b, "{:?}", HashMap::<K, V>::new ())` | `assert!(a < b, "{:?}", HashMap::<K, V>::new())` |
| `vec![0; n + 1]`, `vec![f 1, g 2]`           | `vec![0; n + 1]`, `vec![f(1), g(2)]` |
| `write!(out, "{}", f x)?`                    | `write!(out, "{}", f(x))?` |
| `println!("{}", _)`                          | `move \|_0\| println!("{}", _0)` |
| `assert!(not (xs.is_empty()))`, `assert!(a = b)` | `assert!(!xs.is_empty())`, `assert!(a == b)` |
| `matches!(x, Some(1) \| None)`               | `matches!(x, Some(1) \| None)` |

* Each argument ends at a `,` outside brackets, so generic arguments and
  comparisons need no parentheses. `vec![e; n]` (with `;`) repeats `e`.
* A format string stays as written. After it, `name = e` is a named
  argument, its value rsml code; write `(a = b)` for a comparison there.
* rsml's meanings apply: `!x` dereferences and `not x` negates. Rust's
  `assert!(!xs.is_empty())` would dereference `xs` and assert the
  opposite (`!` binds tighter than `.`), so in an assertion `!x.f()`
  without parentheses is an error that suggests `not (x.f())`, or
  `(!x).f()` for a dereference. `!` before a comparison, `not e` or
  `true` is an error anywhere.
* `assert e` is still the `assert` of [Control flow](#control-flow);
  `assert!(e)` is the macro.
* rsml-fmt prints macro calls as written.

**Completes at**

| Cursor                                   | Offers                  |
|------------------------------------------|-------------------------|
| `println!("{}", ‸)`, `format!("{} {}", n, ‸)` | values in scope    |
| `println!("{}", xs \|> .‸)`              | methods of `xs`         |
| `assert_eq!(add ‸, 3)`, `format!("{}", n‸)` | values in scope      |
| `println!("{v}", v = ‸)`                 | values in scope         |
| `vec![n; ‸]`, `vec![n, ‸]`               | values in scope         |
| `assert!(xs.‸, "empty")`                 | methods of `xs`         |

## Paths

Rust paths are written as in Rust when there is no space around `::`.
Capitalized names may also be joined with `.`, ML style.

| rsml                         | Rust                        |
|------------------------------|-----------------------------|
| `std::mem::swap a b`         | `std::mem::swap(a, b)`      |
| `::std::cmp::max a b`        | `::std::cmp::max(a, b)`     |
| `Vec.new ()`                 | `Vec::new()`                |
| `Vec::<i64>::new ()`         | `Vec::<i64>::new()`         |
| `i32::MAX`                   | `i32::MAX`                  |
| `Iterator::map _ f`          | `move \|_0\| Iterator::map(_0, f)` |

`x :: xs` with spaces is a cons, and so is `h::t` when `h` is bound in the
block (a name bound in rsml code is a value, never a module).

**Completes at**

| Cursor                                | Offers                       |
|---------------------------------------|------------------------------|
| `Vec::‸`, `Vec.‸`, `Vec::<i64>::‸`    | associated functions         |
| `Shape::‸`, `Shape.‸`                 | variants                     |
| `geometry::‸`                         | the module's items           |
| `std::cmp::‸`, `std::cmp::ma‸`        | the module's items           |
| `::std::‸`                            | `std`'s modules              |
| `xs.iter() \|> Iterator::‸ _ (fun x -> x)` | `Iterator`'s methods   |
| `use std::‸;`                         | modules                      |

## Lambdas: `fun` and `function`

| rsml                                 | Rust |
|--------------------------------------|------|
| `fun x y -> x + y`                   | `move \|x, y\| x + y` |
| `fun () -> 1`                        | `move \|\| 1` |
| `fun (x : &str) -> x.len()`          | `move \|x: &str\| x.len()` |
| `fun (a, b) -> a + b`                | `move \|(a, b)\| a + b` |
| `map (fun x -> x * 2) xs`            | `map(\|x\| x * 2, xs)` |
| `map (fun move x -> x * 2) xs`       | `map(move \|x\| x * 2, xs)` |
| `(fun x -> x + 1) 2`                 | `{ let x = 2; x + 1 }` |
| `3 \|> fun x -> x * x`               | `{ let x = 3; x * x }` |
| `function Some x -> x \| None -> 0`  | `move \|argument\| match argument { Some(x) => x, None => 0, }` |

A closure may also be written Rust style: `|x, y| e` makes the same
closure as `fun x y -> e`. The parameters are separated by `,` and typed
Rust's way (`x: i32`), `|| e` has no parameters, and `move |x| e` is
`fun move x -> e`.

| rsml                                 | Rust |
|--------------------------------------|------|
| `\|x: i32\| x + c`, `move \|x: i32\| x + c` | `move \|x: i32\| x + c` |
| `\|\| 1`                             | `move \|\| 1` |
| `\|(a, b)\| a + b`                   | `move \|(a, b)\| a + b` |
| `f (\|a, b\| a + b)`                 | `f(\|a, b\| a + b)` |
| `f (move \|x\| x)`                   | `f(move \|x\| x)` |
| `map (\|&x\| x * 2) xs`              | `map(\|&x\| x * 2, xs)` |
| `xs.map(\|x\| x + 1)`                | `xs.map(\|x\| x + 1)` |
| `(\|x\| x + 1) 2`                    | `{ let x = 2; x + 1 }` |
| `f (\|x\| a <- x; b <- x)`           | `f(\|x\| { a = x; b = x })` |
| `let add = \|x, y\| x + y in add 1`  | `{ let add = \|x, y\| x + y; move \|arg2\| add(1, arg2) }` |
| `a \|\| b`                           | `a \|\| b` |

* Both forms make the same closure, with the same known arity (see
  [Arity](#arity-and-partial-application)), and both are applied on the
  spot without a closure.
* The body reaches as far as a `fun` body does, over `;` too:
  `f (|x| a <- x; b <- x)` assigns both in the closure.
* As an argument, a closure goes in parentheses, as `fun` does:
  `f (|x| x)`. `f |x| x` is an error: "a closure here goes in parentheses, as `fun` does: `f (|x| e)`".
* The body comes right after the parameters: `|x| -> x` is an error: "a closure's body comes right after its parameters: `|x| e` (or `fun x -> e`)".
* `||` after an operand is still `or`: `a || b` is `a || b`.
* Among match cases, a `|` after `return` or after a trailing `;` is the
  next case, so a closure there needs parentheses: `return (|y| y)`.

A closure that is the result of the block (or of an enclosing closure) is
`move`, so it can be returned, unless it assigns to something. Closures
passed as arguments borrow what they capture, like Rust closures; write
`fun move` or `move |x| e` when one must outlive the function. A lambda
applied on the spot is not a closure at all.

**Completes at**

| Cursor                                          | Offers                  |
|-------------------------------------------------|-------------------------|
| `fun x -> ‸`, `fun x -> x + ‸`, `fun move x -> ‸` | the parameter and values |
| `fun (x : &str) -> x.‸`                         | methods of `&str`       |
| `fun (x : ‸) -> x`                              | types                   |
| `xs \|> .iter \|> .map _ (fun x -> x.‸)`        | methods of `&i64`       |
| `n \|> fun y -> y.‸`, `(fun y -> y.‸) n`        | methods of `i64`        |
| `function Some x -> ‸ \| None -> 0`             | `x` and values          |
| `function ‸`                                    | patterns (`Some`, `None`) |

## Local functions and bindings

| rsml | Rust |
|------|------|
| `let f x = x * 2 in f 3` | `{ let f = \|x\| x * 2; f(3) }` |
| `let c = ref 0 in let mut next () = incr c; !c in next ()` | `{ let mut c = 0; let mut next = \|\| { c += 1; c }; next() }` |
| `let rec fact (n : i64) : i64 = if n = 0 then 1 else n * fact (n - 1) in fact 5` | `{ fn fact(n: i64) -> i64 { if n == 0 { 1 } else { n * fact(n - 1) } } fact(5) }` |
| `let rec f : i64 -> i64 = function 0 -> 1 \| n -> n * f (n - 1) in f 5` | `{ fn f(argument: i64) -> i64 { match argument { 0 => 1, n => n * f(n - 1), } } f(5) }` |
| `let rec even (n : u32) : bool = .. odd (n - 1) and odd (n : u32) : bool = .. in even 10` | `{ fn even(n: u32) -> bool { .. } fn odd(n: u32) -> bool { .. } even(10) }` |
| `let rec id (x : 'a) : 'a = x in id 1` | `{ fn id<A>(x: A) -> A { x } id(1) }` |
| `let rec go (acc : i64) (k : i64) : i64 = if k = 0 then acc else go (acc + k) (k - 1) in go 0 n` | `{ fn go(mut arg1: i64, mut arg2: i64) -> i64 { loop { let acc = arg1; let k = arg2; break if k == 0 { acc } else { (arg1, arg2) = (acc + k, k - 1); continue }; } } go(0, n) }` |
| `let x = 1 let y = 2 ;; x + y` | `{ let x = 1; let y = 2; x + y }` |
| `use std::cmp::*; max a b` | `{ use std::cmp::*; max(a, b) }` |
| `use std::cmp::{max, min}; min (max a b) c` | `{ use std::cmp::{max, min}; min(max(a, b), c) }` |
| `let y = 2 in use std::cmp::max; max y 3` | `{ let y = 2; use std::cmp::max; max(y, 3) }` |
| `(use std::cmp::max; max a b) + 1` | `({ use std::cmp::max; max(a, b) }) + 1` |
| `let r = ref 0 in incr r; r := 10 - !r; !r` | `{ let mut r = 0; r += 1; r = 10 - r; r }` |
| `let mut n = 0 in n <- 10 - n; n` | `{ let mut n = 0; n = 10 - n; n }` |
| `n <- n + 1` | `{ #[allow(clippy::assign_op_pattern)] { n = n + 1; } }` |
| `let r = ref 0 in r := !r + 1` | `{ let mut r = 0; { #[allow(clippy::assign_op_pattern)] { r = r + 1; } } }` |

rsml has no `+=`, so an assignment that clippy would rewrite as one
(`x = x + e`, `x = e * x`, ..) is made a statement that allows the lint.

`let f x = ..` is a closure: its parameter types are inferred like a Rust
closure's and it is used at one type. A closure that mutates what it
captures (Rust's `FnMut`) is called through a `mut` binding, so such a
local function is `let mut f x = ..`, as rustc's error about it suggests.
`let rec` is a nested `fn`, which
needs annotations and cannot capture locals; `'a` in its signature is a
generic parameter. A function whose name is used only in calls of itself
in tail position (with all its arguments, as its value: a branch of `if`
or `match`, the body of `let`, the end of a sequence) is a loop instead:
such a call assigns the parameters and goes round again, so it runs in
constant stack space in every build. Rust does not guarantee that for a
recursive `fn`, and a debug build does not do it. Functions defined
together with `and` stay recursive. The block is a toplevel: `let`
without `in`, `external`, `use` and `;;`-separated expressions, the last
one being the value. `use` takes any Rust use tree and ends with its own
`;` (no `;;` after it); it can also start an expression, for that
expression only.

**Completes at**

| Cursor                                              | Offers               |
|-----------------------------------------------------|----------------------|
| `let total = 1 in tot‸`, `let total = 1 in ‸`       | `total`              |
| `let x = ‸`, `let x = ‸ in x`                       | values in scope      |
| `let f a = a + ‸ in f 1`, `let f a = ‸`             | `a` and values       |
| `let rec go (k : i64) : i64 = ‸ in go 1`            | `k`, `go`            |
| `let rec go (k : ‸) : i64 = ..`, `.. : ‸ = k in ..` | types                |
| `let x : ‸ = 1 in x`                                | types                |
| `let (a, b) = (1, 2) in ‸`, `let Point { x; y } = p in ‸` | the bound names |
| `let a = 1 and b = ‸ in a + b`                      | values in scope      |
| `let x = 1` / `let y = x‸`                          | `x`                  |
| `use std::cmp::*; ‸`, `let x = 1 in use std::cmp::*; ‸` | `max`, `min`, .. |
| `external add : i64 -> i64 -> i64 ;; ad‸ 1`         | `add`                |
| `let r = ref 0 in r := ‸`, `let mut m = 0 in m <- ‸`, `incr ‸` | values    |

## Patterns

`match`, `function` and `let` take patterns: constructors, tuples, lists
(as slice patterns), records, literals, ranges `'a'..='z'` and `0..10`,
or-patterns, `as`, and guards with `when` or `if` (`p if c -> e` means
the same as `p when c -> e`). Ranges are Rust's, in patterns as in
expressions (see [Ranges](#ranges)): `..=` includes its end and `..`
leaves it out, however it is spaced, so `'a' .. 'z'` leaves out `'z'`
(it would include it in OCaml; clippy's `almost_complete_range` lint
points at it, as in Rust).

| rsml | Rust |
|------|------|
| `match o with Some x when x > 0 -> x \| _ -> 0`, `match o with Some x if x > 0 -> x \| _ -> 0` | `match o { Some(x) if x > 0 => x, _ => 0, }` |
| `match x with A \| B if c -> 1 \| _ -> 2` | `match x { A \| B if c => 1, _ => 2, }` |
| `match xs with [] -> 0 \| x :: rest -> 1` | `match ::rsml::__rt::as_slice(&xs) { [] => 0, [x, rest @ ..] => 1, }` |
| `match ord with std::cmp::Ordering::Less -> -1 \| _ -> 0` | `match ord { std::cmp::Ordering::Less => -1, _ => 0, }` |
| `match n with i32::MAX -> 1 \| _ -> 0` | `match n { i32::MAX => 1, _ => 0, }` |
| `match p with Point { x; _ } -> x` | `match p { Point { x, .. } => x, }` |
| `match c with 'a'..='z' -> 1 \| _ -> 0` | `match c { 'a'..='z' => 1, _ => 0, }` |
| `match n with 0..10 -> 1 \| _ -> 0`, `match n with 0 .. 10 -> 1 \| _ -> 0` | `match n { 0..10 => 1, _ => 0, }` |
| `match n with 1..=3 -> 1 \| _ -> 0` | `match n { 1..=3 => 1, _ => 0, }` |

A path in a pattern may start with a lowercase module (`std::cmp::Ordering::Less`,
`i32::MAX`) when it is written without spaces and reaches a capitalized
name; `x::rest`, all lowercase, is a cons.

**Nested `match`.** A `match` in a case needs no parentheses: a case that
the inner `match` cannot take goes to the enclosing one, and the cases
after it go there too. The macro runs before types are known, but a
pattern shows part of its type: a path's type segment (`A` in `A::Y`),
`Some` and `None` (`Option`), `Ok` and `Err` (`Result`), a literal's kind
(integer, float, string, char, bool), a tuple's length, a list pattern.
A case goes to the nearest enclosing `match` (or `function`) that can take
it when

* its pattern is of another type than the inner cases', or
* a case before it in the inner `match` already matches everything it
  does: `_`, a variable, or the same constructor or literal again (a
  second `None`).

| rsml | Rust |
|------|------|
| `match a with A::X -> match b with B::P -> 1 \| B::Q -> 2 \| A::Y -> 3` | `match a { A::X => match b { B::P => 1, B::Q => 2, }, A::Y => 3, }` |
| `match r with Ok n -> match n with 0 -> 1 \| _ -> 2 \| Err e -> 3` | `match r { Ok(n) => match n { 0 => 1, _ => 2, }, Err(e) => 3, }` |
| `match a with A -> match b with X -> 1 \| _ -> 2 \| B -> 3` | `match a { A => match b { X => 1, _ => 2, }, B => 3, }` |
| `match a with Some x -> match x with Some y -> y \| None -> 0 \| None -> 3` | `match a { Some(x) => match x { Some(y) => y, None => 0, }, None => 3, }` |
| `function Some x -> match x with 0 -> 1 \| n -> n \| None -> 0` | `move \|argument\| match argument { Some(x) => match x { 0 => 1, n => n, }, None => 0, }` |
| `match a with A -> match b with X -> 1 \| Y -> 2 \| B -> 3` | `match a { A => match b { X => 1, Y => 2, B => 3, }, }` |
| `match a with A -> (match b with X -> 1 \| Y -> 2) \| B -> 3` | `match a { A => match b { X => 1, Y => 2, }, B => 3, }` |

Without such evidence (bare constructors such as `X` and `B`, a `_` or
a variable case that follows the inner cases, or the same type at both
levels: `A::P` and `A::Y`) a case stays with the inner `match`, and the
inner `match` needs parentheses or `begin .. end` for the case to go to
the enclosing one. `Self::A` shows no type: it could be any. A case never
leaves brackets, `begin .. end`, a `let` value (before `in`), an `if`
condition, a guard, a `match` scrutinee, or a loop's bounds or body. rsml-fmt puts each case of a nested `match` on its
own line, so the indentation shows which `match` a case belongs to.

**Completes at**

| Cursor                                             | Offers                 |
|----------------------------------------------------|------------------------|
| `match ‸ with ..`                                  | values in scope        |
| `match o with ‸`, `match o with Some x -> 1 \| ‸`  | patterns (`Some`, `None`, ..) |
| `match o with Option::‸`                           | `Some`, `None`         |
| `match shape with Shape::‸`, `Shape.‸`             | `Circle`, `Rect`       |
| `match ord with std::cmp::Ordering::‸`             | `Less`, `Equal`, `Greater` |
| `match o with Some x -> ‸ \| None -> 0`            | `x` and values         |
| `match o with Some x -> x.‸ \| ..`, `match o with Some x -> x \|> .‸ \| ..` | methods of `String` |
| `match o with Some x when ‸ -> ..`                 | `x` and values         |
| `match (n, o) with (0, Some x) -> x.‸ \| _ -> 0`   | methods of `String`    |
| `let rec go : &[i64] -> i64 = function [] -> 0 \| x :: rest -> ‸ in ..` | `x`, `rest` |

Field names inside a record pattern (`Point { ‸ }`) are not offered: a
record pattern in rsml code may leave fields out, so the expansion always
ends it with `..`, and rust-analyzer only offers the fields a pattern is
missing.

## Operators that become calls

| rsml                | Rust |
|---------------------|------|
| `x :: xs`           | `::rsml::__rt::cons(x, xs)` |
| `1 :: [2; 3]`       | `::std::vec![1, 2, 3]` |
| `xs @ ys`           | `::rsml::__rt::append(xs, ys)` |
| `a ^ b`             | `::rsml::__rt::concat(a, b)` |
| `2.0 ** 3.0`        | `::core::primitive::f64::powf(2.0, 3.0)` |
| `a mod b = 0 && not c` | `((a % b) == 0) && (!c)` |
| `n as f64 *. 2.0`   | `(n as f64) * 2.0` |

`=` and `<>` compare; `==` and `!=` are accepted as themselves. `+.`, `-.`,
`*.` and `/.` are the same as `+`, `-`, `*` and `/`.

**Completes at**: both operands of every operator (`n :: ‸`, `[1] @ ‸`,
`word ^ ‸`, `not ‸`, `-‸`, `!‸`, `&‸`, `&mut ‸`), and `n as ‸` (types).

## Ranges

Rust's ranges, with Rust's meaning: `a..b` stops before `b`.

| rsml           | Rust           |
|----------------|----------------|
| `a..b`, `a..=b` | `a..b`, `a..=b` |
| `a..`, `..b`, `..=b`, `..` | `a..`, `..b`, `..=b`, `..` |
| `0..n + 1`     | `0..(n + 1)`   |
| `0..n \|> .map(f)` | `(0..n).map(f)` |
| `xs[1..]`      | `xs[1..]`      |
| `for i in 0..n do .. done` | `for i in 0..n { .. }` |

A range binds looser than arithmetic, `::`, `@`, `^` and `@@`, and tighter
than comparisons, `|>`, `>>`, `&&` and `||`; ranges do not chain (`a..b..c` is
an error). The end is left out when what follows cannot start an
expression (`a..` before `)`, `do`, `|>`, ..). Patterns take the same
`..` and `..=` (see [Patterns](#patterns)).

**Completes at**: `0..‸`, `0..=‸` (values in scope), `(0..n).‸`
(methods of the range).

## Control flow

These are expressions, so they appear wherever a function body or argument
does.

| rsml | Rust |
|------|------|
| `if c then a else b` | `if c { a } else { b }` |
| `a; b` | `{ a; b }` |
| `begin a; b end` | `{ a; b }` |
| `for i = 0 to n do .. done` | `for i in 0..=n { .. }` |
| `for i = n downto 0 do .. done` | `for i in (0..=n).rev() { .. }` |
| `for i = 0 til n do .. done` | `for i in 0..n { .. }` |
| `for (k, v) in m do .. done`, `for k, v in m do .. done` | `for (k, v) in m { .. }` |
| `while c do .. done` | `while c { .. }` |
| `while true do .. done` | `loop { .. }` |
| `if n < 0 then return 0; n` | `{ if n < 0 { return 0 } ; n }` |
| `return`, `return e`, `break`, `break e`, `continue` | the same |
| `unsafe { e }`, `async { e }`, `async move { e }` | the same, with `e` translated |
| `assert c`, `assert false` | `::core::assert!(c)`, `::core::unreachable!("assert false")` |

* `for p in e do .. done` takes a pattern (see [Patterns](#patterns));
  its body is a unit block, like the other loops'.
* `while true` is the endless loop, Rust's `loop` (which rustc would ask
  for); `break e` gives it a value.
* The value of `return` and `break` extends like an `if` branch: up to a
  `;`, `in`, `then`, `else`, `with`, a match `|`, `done` or a closing
  bracket. It is left out when nothing that can start an expression
  follows. As an argument, `return` needs parentheses: `f (return x)`.
* The braces of `unsafe` and `async` hold rsml code, as `begin .. end`
  does.
* A `;` may end a sequence (`begin a; b; end`), but what follows it must
  then end something: `a; ; b` (two `;`s, which is not `;;`) and an
  operator right after a `;` (`a; + b`, `a; := b`, `a; , b`) are
  errors.

**Completes at**

| Cursor                                                  | Offers            |
|---------------------------------------------------------|-------------------|
| `if ‸ then ..`, `if ‸` (nothing after yet), `if n > ‸`  | values in scope   |
| `if o.‸`                                                | methods of `o`    |
| `if n > 0 then ‸ else 0`, `.. else ‸`, `if n > 0 then ‸` | values in scope  |
| `println!("a"); ‸`, `begin ‸ end`, `begin ‸`            | values in scope   |
| `for i = ‸ to n do ..`, `for i = 0 to ‸ do ..`          | values in scope   |
| `for i = 0 to n do ‸ done`, `.. do ‸` (no `done` yet)   | `i` and values    |
| `while ‸ do ..`, `while ‸`, `while n > 0 do ‸`          | values in scope   |
| `while n > 0 do () done; ‸`                             | values in scope   |
| `for x in ‸`                                            | values in scope   |
| `for x in xs do ‸ done`, `for x in xs do ‸`             | `x` and values    |
| `for (i, x) in xs.iter().enumerate() do ‸ done`         | `i`, `x`          |
| `return ‸`, `while true do break ‸ done`                | values in scope   |
| `unsafe { ‸ }`, `async { ‸ }`                           | values in scope   |
| `assert ‸`                                              | values in scope   |

## Records and collections

| rsml                  | Rust |
|-----------------------|------|
| `Point { x = 1; y }`  | `Point { x: 1, y }` |
| `{ p with x = 2 }`    | `{ let mut record = p; record.x = 2; record }` |
| `{ p with x = p.y }`  | `{ let value1 = p.y; let mut record = p; record.x = value1; record }` |
| `[a; b]`              | `::std::vec![a, b]` |
| `[]`                  | `::std::vec::Vec::new()` |
| `[\|a; b\|]`          | `[a, b]` |
| `(a, b)`              | `(a, b)` |

The new values of `{ p with .. }` are computed first, as in Rust's
`Point { x: p.y, ..p }`, so they may read the old record before it is
moved into the new one.

**Completes at**

| Cursor                                       | Offers                |
|----------------------------------------------|-----------------------|
| `Point { x = ‸; y = 0 }`                     | values in scope       |
| `Point { ‸ }`, `Point { x = 1; ‸ }`          | field names           |
| `Point { x = n; y = p.‸ }`                   | fields and methods    |
| `{ p with x = ‸ }`                           | values in scope       |
| `{ p with ‸ }`                               | field names           |
| `[‸]`, `[n; ‸]`, `[\|n; ‸\|]`, `(n, ‸)`      | values in scope       |
| `rsml! { impl Point { .. Self { x = self.x + ‸; .. } } }` | values      |

## Types

Types are Rust types written with ML arrows and tuples: `i64 -> i64`
is a function, `i64 * String` is `(i64, String)`, and `'a` is a type
variable (a generic parameter in a `let rec` signature). They appear in
`(e : t)`, `(x : t)` parameters, `let x : t = ..`, `let rec` signatures,
`external` declarations, `e as t` and turbofish arguments.

**Completes at**: `(n : ‸)`, `fun (x : ‸) -> ..`, `let x : ‸ = ..`,
`let rec go (k : ‸) : i64 = ..`, `let rec go (k : i64) : ‸ = ..`,
`add (_ : ‸) 1`, `n as ‸`, `xs.into_iter() |> .collect::<‸>`. The types
of an `external` declaration are not completed: only the number of arrows
matters there, so they are not part of the expansion.

## Comments

ML comments, `(* .. *)`, work in rsml code and nest, and so do Rust's
`//` and `/* .. */`.

| rsml                          | Rust      |
|-------------------------------|-----------|
| `1 + (* one (* two *) *) 2`   | `1 + 2`   |
| `1 + /* one */ 2`             | `1 + 2`   |

* Rust's lexer reads the code before the macro does, so a `(* *)` comment
  holds Rust tokens: an unbalanced `"` or bracket in it is an error from
  rustc, and so, from edition 2021 on, is an apostrophe inside a word
  (`don't`). `//` and `/* */` comments are Rust's own and may hold
  anything.
* Editors color `//` and `/* */` as comments; `(* *)` is colored as code,
  except in Neovim and Vim with the files of [editors.md](editors.md).
  rsml-fmt with `comments = "rust"` rewrites `(* *)` as `//` or `/* */`.
* `(*` always opens a comment: the multiplication section is `( * )`, and
  a dereference in parentheses `( *x)`.
* A `rsmlfmt:` comment before the first token of a block sets formatter
  options for it, and `rsmlfmt: skip` leaves the block as written (see
  [rsml-fmt](../rsml-fmt/README.md#configuring)).

## Rust habits

Rust code that means something else in rsml, or nothing, is an error
that says what to write instead. Rust closures (`|x| e`, `|| e`,
`move |x| e`) are not among them: they are accepted (see
[Lambdas](#lambdas-fun-and-function)), and so are `if` guards (see
[Patterns](#patterns)).

| Written                               | The error suggests                     |
|---------------------------------------|----------------------------------------|
| `if let Some(x) = o { .. }`           | `match e with p -> .. \| _ -> ..`      |
| `match x { p => e }`                  | `match x with p -> e \| ..`            |
| `x += e`, `x -= e`, `x *= e`, ..      | `x <- x + e` (`incr r` or `r := !r + e` for a `ref`) |
| `{ a; b }`, `{ f x }`                 | `begin .. end` or `( .. )`             |
| `[1, 2]`, `[\|1, 2\|]`, `function [a, b] -> a` | `[1; 2]`, or `[(1, 2)]` for a list of one pair |
| `let x = e; more`                     | `let x = e in ..`                      |
| `let loop = ..`, `fn x`, `p.type`, .. | a raw identifier (`r#loop`) for a name |
| `fn f x = ..`                         | `let f x = .. in ..` (a closure is `\|x\| e`) |
| `assert!(!xs.is_empty())`, `!(a = b)` | `not (x.f())`, `not e`                 |

A raw identifier is a name wherever one goes (values, parameters,
fields, methods, labels, paths, record fields, patterns) and is never a
keyword: `r#match x` calls a function named `match`, and `r#ref 1` a
function named `ref`. It stays raw in the expansion, and `r#x` is the same
name as `x`, as in Rust.

The words that Rust reserves without syntax rsml uses are ordinary names:
`abstract`, `become`, `box`, `final`, `gen`, `macro`, `override`, `priv`,
`try`, `typeof`, `unsized`, `virtual` and `yield`. They are emitted as raw
identifiers (`let gen = 3 in gen` is `{ let r#gen = 3; r#gen }`), which
every edition accepts: a macro cannot know its caller's edition, and `gen`
is a keyword from 2024 on, `try` from 2018 on. `try e with ..` still gets
the error that `try .. with` is not supported. `async`, `await` and `dyn`
stay keywords, as rsml uses them (`async { e }`, `e.await`, `dyn Trait` in
types); write `r#async` for the name.

Besides closures and `if` guards, one habit is read as meant, a method
chain's dot glued to the pipe before it: `xs |>.iter() |>.count()` is `xs |> .iter() |> .count()`
(see [methods as functions](#methods-as-functions-name)).

The rsml forms that look like these are not affected: `let () = a; b` and `let f x = a; b`
(a sequence as the value), `if let x = e in c then ..`, `x += e` with
a `let ( += ) a b = ..` in scope, and `x |>.y` with a
`let ( |>. ) a b = ..` in scope. Under rust-analyzer the parser recovers
from these as it did before (where the code still parses, the error comes
after it), so completion keeps working.

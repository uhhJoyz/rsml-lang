# Toy examples

One small program per notation of [docs/notation.md](../../docs/notation.md),
in reading order, simplest first. The header of each says what the syntax
is, when to use it and how to write it, and links the section of
notation.md it belongs to. Comments on the interesting lines say what they
mean, the Rust they become where that helps, and the gotchas. Each toy has
a `run()` that prints a short demo, and tests that check every value its
comments state.

They build as one example, `toy` ([main.rs](main.rs) registers them):

```sh
cargo run --example toy                # every toy, in order
cargo run --example toy -- 42 pipe     # toy 042, and the toys whose name has `pipe`
cargo run --example toy -- --list      # the number, name and summary of each
cargo test --example toy               # their tests (`cargo test` runs them too)
```

## Checked comments

A comment that says what code becomes is written in one form, and
`scripts/check_docs.py` checks each one with the macros, as it checks the
documentation:

```rust
// `x |> f a` is `f(a, x)`.          (the Rust it expands to)
// `1 :: [2; 3]` is `[1; 2; 3]`.     (or OCaml with the same expansion)
// Given `external add : _ -> _ -> _`, `add 1` is `move |arg2| add(1, arg2)`.
// `x += 1` is an error: "OCaml has no `+=`"
```

* A claim is two code spans joined by "is" (or "is always", "becomes",
  "would be", "stays", ..): the second is the expansion of the first as
  rustc sees it, or OCaml that expands to the same Rust, up to the names
  of closure parameters.
* "Given", a code span and a comma, before claims in the same comment,
  puts that code and `;;` before both of their sides: an `external`
  declaration there tells the check what the toy's code knows about a
  function.
* A code span, "is an error:" and a quoted text: the macro rejects the
  code with a message that contains the text.
* Other phrasings ("expands to", "turns into", an arrow between two code
  spans, ..) and an odd number of backticks in a comment are errors, so
  that no claim goes unchecked. `(* *)` comments hold no code spans, since
  a claim could span their lines unseen.

The same script checks that every toy's header links a section of
notation.md, that every section has a toy, that the index below lists
every toy once with its `SUMMARY`, and that main.rs registers each.

## Index

| #   | Toy | What it shows | Section |
|-----|-----|---------------|---------|
| 001 | [application](001_application.rs) | Juxtaposition: `area 3 4` calls `area` with two arguments | [Application](../../docs/notation.md#application) |
| 002 | [parenthesized_arguments](002_parenthesized_arguments.rs) | Arguments are atoms: `f (x + 1) (g y)`, `f (-1)`, `f s.len()` | [Application](../../docs/notation.md#application) |
| 003 | [associated_functions](003_associated_functions.rs) | Type functions: `Vec::with_capacity 10`, `Vec.new ()`, `String.from "a"` | [Application](../../docs/notation.md#application) |
| 004 | [borrows_and_labels](004_borrows_and_labels.rs) | Borrowing `f &xs`, `f &mut xs`; labels `f ~y ~x:1` in any order | [Application](../../docs/notation.md#application) |
| 005 | [unit_rules](005_unit_rules.rs) | `f ()` calls with no arguments, `f (())` passes `()`, `Ok ()` | [Application](../../docs/notation.md#application) |
| 006 | [let_in](006_let_in.rs) | `let x = e in body`: shadowing, patterns, types, `let _` | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 007 | [local_functions](007_local_functions.rs) | `let f x = .. in`: a local function is a Rust closure | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 008 | [let_rec](008_let_rec.rs) | `let rec f (n : t) : t = ..`: a recursive, nested `fn` (a loop for tail calls) | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 009 | [let_rec_and](009_let_rec_and.rs) | Mutual recursion `let rec .. and ..`; `let a = .. and b = ..` | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 010 | [let_mut_function](010_let_mut_function.rs) | `let mut next () = ..`: a local function that changes its captures | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 011 | [toplevel_phrases](011_toplevel_phrases.rs) | Toplevel phrases: `let` without `in`, `;;`, the last is the value | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 012 | [fun](012_fun.rs) | `fun x y -> x + y`: closures, as values and as arguments | [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function) |
| 013 | [fun_parameters](013_fun_parameters.rs) | `fun (x : t)`, `fun (a, b)`, `fun (&x)`, `fun ~x` parameters | [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function) |
| 014 | [fun_applied](014_fun_applied.rs) | `(fun x -> e) v` and `v \|> fun x -> e` are `let`s | [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function) |
| 015 | [fun_move](015_fun_move.rs) | `fun move`; a returned `fun` moves, an argument borrows | [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function) |
| 016 | [function](016_function.rs) | `function Some n -> n \| None -> 0`: a closure of cases | [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function) |
| 017 | [function_in_pipelines](017_function_in_pipelines.rs) | `.map(function 200 \| 204 -> .. \| c when .. -> ..)`, `n \|> function` | [Lambdas: fun and function](../../docs/notation.md#lambdas-fun-and-function) |
| 018 | [partial_application](018_partial_application.rs) | Partial application: `with_tax 20` waits for the price | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 019 | [partial_evaluated_once](019_partial_evaluated_once.rs) | `with_tax (lookup ())` computes `lookup ()` once, not per call | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 020 | [external](020_external.rs) | `external with_tax : u32 -> u32 -> u32` makes `with_tax 20` partial | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 021 | [external_paths](021_external_paths.rs) | `external at_least : _ -> _ -> _ = "std::cmp::max"`, `f -> (g -> h)` | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 022 | [type_constraint_arity](022_type_constraint_arity.rs) | `(std::cmp::max 0 : i64 -> i64)` is a partial application | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 023 | [closure_typed_parameters](023_closure_typed_parameters.rs) | A parameter `discount: impl Fn(u32, u32) -> u32` allows `discount 10` | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 024 | [pipe](024_pipe.rs) | `items \|> subtotal \|> discount 10 \|> add_shipping 5` | [Pipelines: \|> and @@](../../docs/notation.md#pipelines--and-) |
| 025 | [pipe_value_last](025_pipe_value_last.rs) | Data first: `x \|> i64::clamp _ 0 10`, `s \|> str::starts_with _ "he"` | [Pipelines: \|> and @@](../../docs/notation.md#pipelines--and-) |
| 026 | [apply_operator](026_apply_operator.rs) | `round_to 10 @@ with_tax 20 @@ subtotal items`, no parentheses | [Pipelines: \|> and @@](../../docs/notation.md#pipelines--and-) |
| 027 | [pipe_unit](027_pipe_unit.rs) | `() \|> f` calls `f` with no arguments, `(()) \|> f` passes `()` | [Pipelines: \|> and @@](../../docs/notation.md#pipelines--and-) |
| 028 | [pipe_into_rust_call](028_pipe_into_rust_call.rs) | `x \|> f(a)` is `f(a)(x)`; `x \|> f(_, a)` is `f(x, a)` | [Pipelines: \|> and @@](../../docs/notation.md#pipelines--and-) |
| 029 | [compose_forward](029_compose_forward.rs) | `add_tax 20 >> round_to 10 >> dollars`: one step, then the next | [Composition: >> and <<](../../docs/notation.md#composition--and-) |
| 030 | [compose_backward](030_compose_backward.rs) | `bracket << str::to_uppercase`: the right function runs first | [Composition: >> and <<](../../docs/notation.md#composition--and-) |
| 031 | [compose_applied](031_compose_applied.rs) | `(f >> g) x` and `x \|> (f >> g)` are plain nested calls | [Composition: >> and <<](../../docs/notation.md#composition--and-) |
| 032 | [compose_methods](032_compose_methods.rs) | `.trim >> .len`, `.split(',') >> .count()`, `add 1 >> add 2` | [Composition: >> and <<](../../docs/notation.md#composition--and-) |
| 033 | [operator_sections](033_operator_sections.rs) | An operator as a function: `( + )`, `( * )`, `( - )` | [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators) |
| 034 | [section_partial](034_section_partial.rs) | `( + ) 1` and `( - ) 10`: the argument is the left operand | [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators) |
| 035 | [keyword_operator_sections](035_keyword_operator_sections.rs) | Word operators as sections: `( lor )`, `( land )`, `( mod )`, `( lsl )` | [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators) |
| 036 | [custom_operators](036_custom_operators.rs) | Defining infix operators: `let ( +^ ) a b = ..`, precedence | [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators) |
| 037 | [custom_operator_rules](037_custom_operator_rules.rs) | Operator rules: grouping, no `!` operators, operators with state | [Operator sections and user-defined operators](../../docs/notation.md#operator-sections-and-user-defined-operators) |
| 038 | [placeholder](038_placeholder.rs) | `_` makes a function: `clamp 0 _ 100`, `_ * 2` | [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-) |
| 039 | [numbered_placeholders](039_numbered_placeholders.rs) | Numbered placeholders: `Ord::cmp _1 _0`, `( * ) _0 _0`, `f _1 _ _` | [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-) |
| 040 | [placeholder_reach](040_placeholder_reach.rs) | Where a `_` function ends: `(_ * 9 / 5 + 32)`, `f (g _) x` | [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-) |
| 041 | [placeholders_in_arguments](041_placeholders_in_arguments.rs) | Short closures: `.map(_ * 2)`, `.map(_.len())`, `(_0 > 0 && _0 < 10)` | [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-) |
| 042 | [typed_placeholders](042_typed_placeholders.rs) | `(_ : &str).trim().len()`: a placeholder with a type | [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-) |
| 043 | [placeholders_applied](043_placeholders_applied.rs) | `x \|> f _ 1` and `(g _1 _0) a b` are plain calls | [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-) |
| 044 | [placeholders_evaluate_once](044_placeholders_evaluate_once.rs) | `convert (rate ()) _` computes `rate ()` once, not per call | [Placeholders: _ and _0, _1, ..](../../docs/notation.md#placeholders-_-and-_0-_1-) |
| 045 | [method_refs](045_method_refs.rs) | `.len s`, `.push cart item`: the receiver comes first | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 046 | [method_pipelines](046_method_pipelines.rs) | `xs \|> .iter \|> .map _ f \|> .sum::<i64>`: a method chain | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 047 | [methods_as_values](047_methods_as_values.rs) | `.map(.len)`, `apply_all (.trim) xs`, `.len (_ : &str)` | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 048 | [method_sections](048_method_sections.rs) | `xs \|> .iter() \|> .map(f) \|> .sum::<i64>()`, `.push(1) xs` | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 049 | [field_sections](049_field_sections.rs) | Tuple fields as functions: `.map(.0)`, `x \|> .1`, `.0.1 t` | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 050 | [glued_pipe_dot](050_glued_pipe_dot.rs) | `xs \|>.iter() \|>.count()` reads as `xs \|> .iter() \|> .count()` | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 051 | [receiver_and_unit](051_receiver_and_unit.rs) | `.send tx ()` sends `()`; `.get (f x) 0`; `.collect::<Vec<_>> it` | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 052 | [rust_method_calls](052_rust_method_calls.rs) | Method calls `xs.push(1)`, chains, and `x.f (a)` calling a field | [Rust-style methods, fields and calls](../../docs/notation.md#rust-style-methods-fields-and-calls) |
| 053 | [rust_calls](053_rust_calls.rs) | Rust calls `add(1, 2)`, `f()`, `f(a)(b)`; spaced `f (a, b)` is a tuple | [Rust-style calls and indexing](../../docs/notation.md#rust-style-calls-and-indexing-fa-b-vi) |
| 054 | [known_arity_calls](054_known_arity_calls.rs) | With a known arity, `add(1)` is partial and `add(1)(2)` calls `add(1, 2)` | [Rust-style calls and indexing](../../docs/notation.md#rust-style-calls-and-indexing-fa-b-vi) |
| 055 | [indexing](055_indexing.rs) | Indexing `v[i]`, `m[&k]`, `f(x)[0]`, `xs[1..]`; `f [1; 2]` is a list | [Rust-style calls and indexing](../../docs/notation.md#rust-style-calls-and-indexing-fa-b-vi) |
| 056 | [constructors](056_constructors.rs) | Constructors: `Shape::Rect (w, h)`, `Some (1, 2)`, `Span ((a, b))` | [Application](../../docs/notation.md#application) |
| 057 | [constructor_values](057_constructor_values.rs) | `Some(x).unwrap()` vs `Some (x).unwrap()`; `f Some(x) y` | [Rust-style calls and indexing](../../docs/notation.md#rust-style-calls-and-indexing-fa-b-vi) |
| 058 | [rust_paths](058_rust_paths.rs) | Paths `std::mem::swap a b`, `::std::cmp::max`, `u8::MAX`, turbofish | [Paths](../../docs/notation.md#paths) |
| 059 | [dotted_paths](059_dotted_paths.rs) | Dotted paths `Vec.new ()`, `Level.High`; `h::t` with a bound `h` | [Paths](../../docs/notation.md#paths) |
| 060 | [open](060_open.rs) | `open std::cmp ;;`: Rust's `use std::cmp::*;` for the rest of the block | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 061 | [let_open](061_let_open.rs) | `let open std::f64::consts in PI *. r *. r`: names for one expression | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 062 | [cons](062_cons.rs) | `x :: xs` puts `x` in front of a `Vec`; `1 :: [2; 3]` is a literal | [Operators that become calls](../../docs/notation.md#operators-that-become-calls) |
| 063 | [append](063_append.rs) | `morning @ evening` joins two lists | [Operators that become calls](../../docs/notation.md#operators-that-become-calls) |
| 064 | [string_concat](064_string_concat.rs) | `"Dear " ^ name ^ ","` builds a `String` | [Operators that become calls](../../docs/notation.md#operators-that-become-calls) |
| 065 | [float_operators](065_float_operators.rs) | `+.` `-.` `*.` `/.` are `+ - * /`; `**` calls `f64::powf` | [Operators that become calls](../../docs/notation.md#operators-that-become-calls) |
| 066 | [integer_operators](066_integer_operators.rs) | `n mod 7`, `flags land MASK`, `1 lsl k`: integer word operators | [Operators that become calls](../../docs/notation.md#operators-that-become-calls) |
| 067 | [comparisons_and_not](067_comparisons_and_not.rs) | `a = b`, `a <> b`, `not (a = b)`; `!` is not negation | [Operators that become calls](../../docs/notation.md#operators-that-become-calls) |
| 068 | [match_variants](068_match_variants.rs) | Variants: `Some x`, `None`, `Shape::Rect (w, h)` in `match .. with` | [Patterns](../../docs/notation.md#patterns) |
| 069 | [tuple_patterns](069_tuple_patterns.rs) | Tuple patterns: `match (x, y) with (0, _) -> ..`, `let (h, m) = ..` | [Patterns](../../docs/notation.md#patterns) |
| 070 | [list_patterns](070_list_patterns.rs) | List patterns: `[]`, `[x]`, `[a; b]`, `x :: rest` on slices and Vecs | [Patterns](../../docs/notation.md#patterns) |
| 071 | [record_patterns](071_record_patterns.rs) | Record patterns: `Order { item; qty = 0; _ }`, `let Order { .. } = o` | [Patterns](../../docs/notation.md#patterns) |
| 072 | [or_patterns](072_or_patterns.rs) | Or-patterns: `'a' \| 'e' -> ..`, `Some 0 \| None -> ..`, `(0, k) \| (k, 0)` | [Patterns](../../docs/notation.md#patterns) |
| 073 | [as_patterns](073_as_patterns.rs) | `as` patterns: `Some n as found`, `x :: _ as all` name the whole | [Patterns](../../docs/notation.md#patterns) |
| 074 | [guards](074_guards.rs) | Guards: `s when s >= 90 -> "A"`, `(a, b) when a = b -> ..` | [Patterns](../../docs/notation.md#patterns) |
| 075 | [range_patterns](075_range_patterns.rs) | Range patterns: `'a'..='z'` includes its end, `0..10` leaves it out, as in Rust | [Patterns](../../docs/notation.md#patterns) |
| 076 | [reference_patterns](076_reference_patterns.rs) | Reference patterns: `for &x in ..`, `fun (&x) -> ..`, `let &x = r in` | [Patterns](../../docs/notation.md#patterns) |
| 077 | [literal_patterns](077_literal_patterns.rs) | Literal patterns: `"yes"` (on a String too), `-1`, `'\\n'`, `i32::MAX` | [Patterns](../../docs/notation.md#patterns) |
| 078 | [if_then_else](078_if_then_else.rs) | `if c then a else b`, `else if` chains, `if` without `else` | [Control flow](../../docs/notation.md#control-flow) |
| 079 | [sequences](079_sequences.rs) | `a; b`: steps in order, the last one is the value | [Control flow](../../docs/notation.md#control-flow) |
| 080 | [begin_end](080_begin_end.rs) | `begin a; b end` and `( a; b )` group steps into one | [Control flow](../../docs/notation.md#control-flow) |
| 081 | [for_to](081_for_to.rs) | `for i = 1 to n do .. done` includes `n`, `til` stops before it | [Control flow](../../docs/notation.md#control-flow) |
| 082 | [for_downto](082_for_downto.rs) | `for i = 3 downto 1 do .. done`: a countdown, ends included | [Control flow](../../docs/notation.md#control-flow) |
| 083 | [for_in](083_for_in.rs) | `for (k, v) in m do .. done`, `for (i, x) in xs.iter().enumerate()` | [Control flow](../../docs/notation.md#control-flow) |
| 084 | [while_loops](084_while_loops.rs) | `while n > 1 do .. done`: loop while a condition holds | [Control flow](../../docs/notation.md#control-flow) |
| 085 | [loop_break_value](085_loop_break_value.rs) | `while true do .. break e .. done`: Rust's `loop`, with a value | [Control flow](../../docs/notation.md#control-flow) |
| 086 | [return_break_continue](086_return_break_continue.rs) | Early `return e`, `continue` and `break` in loops | [Control flow](../../docs/notation.md#control-flow) |
| 087 | [assertions](087_assertions.rs) | `assert (n > 0)` and `assert false` for impossible cases | [Control flow](../../docs/notation.md#control-flow) |
| 088 | [ranges](088_ranges.rs) | `0..n` stops before `n`, `1..=n` includes it; `xs[1..]`, `0..n \|> ..` | [Ranges](../../docs/notation.md#ranges) |
| 089 | [refs](089_refs.rs) | Mutable cells: `ref 0`, `!r`, `r := e`, `incr r`, `decr r` | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 090 | [let_mut](090_let_mut.rs) | `let mut n = 0 in n <- n + 1`: assignment with `<-` | [Local functions and bindings](../../docs/notation.md#local-functions-and-bindings) |
| 091 | [deref_star](091_deref_star.rs) | Rust's `*`: `*c <- *c + 1`, `*w.cents`, `( *x)`, `**q`, `&*s` | [Rust-style methods, fields and calls](../../docs/notation.md#rust-style-methods-fields-and-calls) |
| 092 | [bang_dereference](092_bang_dereference.rs) | `!x` reads through a reference; `!m.age` is `(!m).age`; `not` negates | [Rust-style methods, fields and calls](../../docs/notation.md#rust-style-methods-fields-and-calls) |
| 093 | [records](093_records.rs) | Records: `Item { name = "tea"; price = 4 }`, `item.price` | [Records and collections](../../docs/notation.md#records-and-collections) |
| 094 | [record_punning](094_record_punning.rs) | Punning: `Size { width; height }` in expressions and patterns | [Records and collections](../../docs/notation.md#records-and-collections) |
| 095 | [record_update](095_record_update.rs) | Update: `{ p with price = 5 }`; new values read the old record | [Records and collections](../../docs/notation.md#records-and-collections) |
| 096 | [lists](096_lists.rs) | Lists are `Vec`s: `[a; b]`, `[]`, `[(1, 2)]`; `[1, 2]` is rejected | [Records and collections](../../docs/notation.md#records-and-collections) |
| 097 | [arrays](097_arrays.rs) | Arrays: `[\|a; b\|]` is `[a, b]`; array patterns `[\|r; g; b\|]` | [Records and collections](../../docs/notation.md#records-and-collections) |
| 098 | [tuples](098_tuples.rs) | Tuples: `(q, r)`, `p.0`, `let (q, r) = .. in`, `f (a, b)` | [Records and collections](../../docs/notation.md#records-and-collections) |
| 099 | [type_annotations](099_type_annotations.rs) | `(e : t)`, `let x : t = ..`, `fun (x : t) -> ..`: Rust types | [Types](../../docs/notation.md#types) |
| 100 | [function_and_tuple_types](100_function_and_tuple_types.rs) | Types with OCaml's `*` and `->`: `u32 * &str`, `u32 -> u32` | [Types](../../docs/notation.md#types) |
| 101 | [type_variables](101_type_variables.rs) | `let rec len (xs : &['a]) : usize`: `'a` becomes a generic parameter | [Types](../../docs/notation.md#types) |
| 102 | [lifetimes](102_lifetimes.rs) | Lifetimes: `(s : &'a str)` in a `fn f<'a>`, `&'static str` | [Types](../../docs/notation.md#types) |
| 103 | [turbofish](103_turbofish.rs) | Turbofish: `.parse::<i32>()`, `.sum::<i64>`, `Vec::<u8>::new ()` | [Types](../../docs/notation.md#types) |
| 104 | [println](104_println.rs) | `println!("{} items", items \|> .len)`: macro arguments are OCaml | [Macros](../../docs/notation.md#macros) |
| 105 | [format_named_arguments](105_format_named_arguments.rs) | `format!("{} {x}", f a, x = g b)`; `(a = b)` to compare | [Macros](../../docs/notation.md#macros) |
| 106 | [assert_macros](106_assert_macros.rs) | `assert_eq!(add 1 2, 3)`, `assert!(not (xs.is_empty()))` | [Macros](../../docs/notation.md#macros) |
| 107 | [vec_macro](107_vec_macro.rs) | `vec![0; n + 1]` repeats, `vec![f 1, g 2]` lists | [Macros](../../docs/notation.md#macros) |
| 108 | [write_macro](108_write_macro.rs) | `write!(out, "{}", f x)?`; other macros (`matches!`) take Rust | [Macros](../../docs/notation.md#macros) |
| 109 | [rsml_functions](109_rsml_functions.rs) | `rsml! { fn .. }`: Rust signatures, OCaml bodies | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 110 | [rsml_impl](110_rsml_impl.rs) | `impl` in `rsml!`: `&self`/`&mut self` methods with OCaml bodies | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 111 | [rsml_trait](111_rsml_trait.rs) | A `trait` with an OCaml default method, and its `impl`s | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 112 | [rsml_generics_where](112_rsml_generics_where.rs) | Generic `rsml!` functions: `<T: Display>` and `where` clauses | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 113 | [rsml_closure_parameters](113_rsml_closure_parameters.rs) | `&dyn Fn`, `F: Fn(..)`, `where F: Fn(..)` parameters, applied OCaml style | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 114 | [sibling_arity](114_sibling_arity.rs) | Functions of one `rsml!` are known: `with_tax 20` is partial | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 115 | [self_and_type_names](115_self_and_type_names.rs) | `Self::new 0 0` in the `impl`, `Money::new 5` partial outside | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 116 | [nested_modules](116_nested_modules.rs) | `mod` in `rsml!`: `use super::*` brings the parent's arities along | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 117 | [async_and_const_fn](117_async_and_const_fn.rs) | `async fn` and `const fn` in `rsml!`, OCaml bodies | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 118 | [rust_attribute](118_rust_attribute.rs) | `#[rust]` keeps a function's body Rust inside `rsml!` | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 119 | [question_mark](119_question_mark.rs) | Rust's `?`: `s.parse::<u32>()?`, `(parse s)?`, `words.first()?` | [Rust-style methods, fields and calls](../../docs/notation.md#rust-style-methods-fields-and-calls) |
| 120 | [await_futures](120_await_futures.rs) | `price(item).await`, `fut \|> .await`, `async move { e }` | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 121 | [unsafe_blocks](121_unsafe_blocks.rs) | `unsafe { e }` with OCaml inside: `unsafe { text_of bytes }` | [Control flow](../../docs/notation.md#control-flow) |
| 122 | [raw_identifiers](122_raw_identifiers.rs) | Raw names `r#match`, `t.r#type`; reserved words: `let gen = ..` | [Rust habits](../../docs/notation.md#rust-habits) |
| 123 | [as_casts](123_as_casts.rs) | Casts: `total as f64 /. count as f64`, `c as u8 - b'a'` | [Operators that become calls](../../docs/notation.md#operators-that-become-calls) |
| 124 | [whole_file](124_whole_file.rs) | A whole module in one flush-left `rsml::rsml! { .. }` | [Arity and partial application](../../docs/notation.md#arity-and-partial-application) |
| 125 | [whole_file_tests](125_whole_file_tests.rs) | A whole file whose tests are OCaml, with a `#[rust]` function | [Macros](../../docs/notation.md#macros) |
| 126 | [rust_comments](126_rust_comments.rs) | `//` and `/* */` in OCaml code; `///` documents `rsml!` items | [Comments](../../docs/notation.md#comments) |
| 127 | [ocaml_comments](127_ocaml_comments.rs) | `(* .. *)` comments, nested; `( * )`, `( *x)` and `(**)` are not | [Comments](../../docs/notation.md#comments) |
| 128 | [rsmlfmt_directives](128_rsmlfmt_directives.rs) | `// rsmlfmt: skip`, `(* rsmlfmt: max_width = 40 *)` per block | [Comments](../../docs/notation.md#comments) |
| 129 | [rust_habits_expressions](129_rust_habits_expressions.rs) | Rust habits rejected: `\|x\| e`, `if let`, `match {}`, `+=`, `!` for not | [Rust habits](../../docs/notation.md#rust-habits) |
| 130 | [rust_habits_blocks](130_rust_habits_blocks.rs) | Rust habits rejected: `{ a; b }`, `[1, 2]`, `let x = e;`, `fn f x` | [Rust habits](../../docs/notation.md#rust-habits) |
| 131 | [fizzbuzz](131_fizzbuzz.rs) | FizzBuzz: a `match` on a tuple, `mod`, a range and pipes | [Patterns](../../docs/notation.md#patterns) |
| 132 | [word_count](132_word_count.rs) | Word count: a cleaning pipeline, a counting loop, a sort | [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name) |
| 133 | [tree_fold](133_tree_fold.rs) | A search tree built with `fold`, and a fold over the tree | [Patterns](../../docs/notation.md#patterns) |
| 134 | [tokenizer](134_tokenizer.rs) | A tokenizer: slice patterns, or-patterns, guards and `?` | [Patterns](../../docs/notation.md#patterns) |
| 135 | [state_machine](135_state_machine.rs) | A turnstile: a `match` on (state, event), `{ t with .. }` | [Patterns](../../docs/notation.md#patterns) |
| 136 | [run_length_encoding](136_run_length_encoding.rs) | Run-length encoding: loops over chars, `last_mut`, `fun (&(c, n))` | [Patterns](../../docs/notation.md#patterns) |
| 137 | [expression_evaluator](137_expression_evaluator.rs) | An evaluator: `match` on an enum, `?`, `::` and `^` | [Patterns](../../docs/notation.md#patterns) |
| 138 | [matrix](138_matrix.rs) | Matrices: ranges into nested closures, `_[j]`, `[[1; 1]; [1; 0]]` | [Ranges](../../docs/notation.md#ranges) |
| 139 | [prime_sieve](139_prime_sieve.rs) | A prime sieve: `while`, `<-` on elements, `ref`, slice patterns | [Control flow](../../docs/notation.md#control-flow) |
| 140 | [sales_report](140_sales_report.rs) | A sales report: pipelines over records, `_.units`, `.1`, `@` | [Records and collections](../../docs/notation.md#records-and-collections) |

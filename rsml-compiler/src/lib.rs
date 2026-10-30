//! The translator behind [`rsml`](https://docs.rs/rsml): lexer, parser and
//! code generator for the OCaml subset accepted by `ml!` and `rsml!`.
//!
//! This is an ordinary library (proc-macro crates cannot export anything but
//! macros), so the macros in `rsml-macros` and the formatter in `rsml-fmt`
//! share one parser.

pub mod ast;
mod codegen;
pub mod items;
pub mod lexer;
pub mod parser;
pub mod types;

pub use codegen::{Scope, expand, expand_fn, expand_in, expand_with};
#[doc(hidden)]
pub use codegen::{force_editor_mode, without_markers};
pub use items::expand_items;

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;

    fn expand_raw(src: &str) -> String {
        let ts: TokenStream = src.parse().unwrap();
        crate::expand(ts).to_string().split_whitespace().collect()
    }

    /// The expansion without the editor markers (tested on their own).
    fn expand(src: &str) -> String {
        crate::codegen::without_markers(|| expand_raw(src))
    }

    fn expand_items(src: &str) -> String {
        let ts: TokenStream = src.parse().unwrap();
        crate::codegen::without_markers(|| crate::expand_items(ts))
            .to_string()
            .split_whitespace()
            .collect()
    }

    /// Token spacing differs between `quote!` and the parser; rustc ignores it.
    fn norm(src: &str) -> String {
        src.parse::<TokenStream>()
            .unwrap()
            .to_string()
            .split_whitespace()
            .collect()
    }

    #[track_caller]
    fn check(ml: &str, rust: &str) {
        assert_eq!(expand(ml), norm(rust), "\n  ml: {}", ml);
    }

    /// The expansion is a `compile_error!` saying `msg`.
    #[track_caller]
    fn check_error(ml: &str, msg: &str) {
        let out = expand(ml);
        let msg: String = msg.split_whitespace().collect();
        assert!(
            out.starts_with("::core::compile_error!") && out.contains(&msg),
            "\n  ml: {}\n  got: {}",
            ml,
            out
        );
    }

    #[track_caller]
    fn check_items(ml: &str, rust: &str) {
        assert_eq!(expand_items(ml), norm(rust), "\n  ml: {}", ml);
    }

    #[test]
    fn pipes_are_plain_calls() {
        check("x |> f |> g", "g(f(x))");
        check("x |> f a |> g b c", "g(b, c, f(a, x))");
        check("f @@ g @@ x", "f(g(x))");
        check("xs |> Vec::len", "Vec::len(xs)");
        check("Vec::<i64>::new ()", "Vec::<i64>::new()");
    }

    #[test]
    fn known_functions() {
        check(
            "let add x y = x + y in add 1 2",
            "{ let add = |x, y| x + y; add(1, 2) }",
        );
        check(
            "let add x y = x + y in let inc = add 1 in inc 5",
            "{ let add = |x, y| x + y; let inc = move |arg2| add(1, arg2); inc(5) }",
        );
        check(
            "let add x y = x + y in 5 |> add 1",
            "{ let add = |x, y| x + y; add(1, 5) }",
        );
    }

    #[test]
    fn partial_application_evaluates_arguments_once() {
        check(
            "let add x y = x + y in add (expensive ())",
            "{ let add = |x, y| x + y; { let value1 = expensive(); move |arg2| add(value1, arg2) } }",
        );
        check("( + ) 1", "move |rhs| 1 + rhs");
    }

    /// The expansion for rust-analyzer, without the markers.
    fn expand_editor_plain(src: &str) -> String {
        let ts: TokenStream = src.parse().unwrap();
        crate::codegen::without_markers(|| crate::codegen::expand_for(ts, &[], &[], true))
            .to_string()
            .split_whitespace()
            .collect()
    }

    /// rustc prints the names of generated locals in its errors, so they
    /// are named for what they stand for; rust-analyzer offers them as
    /// completions, so for it they keep a prefix no user name has.
    #[test]
    fn generated_names() {
        let cases = [
            (
                "f _ 1",
                "move |_0| f(_0, 1)",
                "move |__rsml_p1| f(__rsml_p1, 1)",
            ),
            (
                ".len",
                "|receiver| receiver.len()",
                "|__rsml_r1| __rsml_r1.len()",
            ),
            (".0", "|tuple| tuple.0", "|__rsml_r1| __rsml_r1.0"),
            (
                "f >> g",
                "move |argument| g(f(argument))",
                "{ let mut __rsml_a1 = f; let mut __rsml_a2 = g; \
                 move |__rsml_x3| __rsml_a2(__rsml_a1(__rsml_x3)) }",
            ),
            (
                "function Some x -> x | None -> 0",
                "move |argument| match argument { Some(x) => x, None => 0, }",
                "move |__rsml_arg1| match __rsml_arg1 { Some(x) => x, None => 0, }",
            ),
            (
                "external add : _ -> _ -> _ ;; add (g ())",
                "{ let value1 = g(); move |arg2| add(value1, arg2) }",
                "{ let __rsml_a1 = g(); move |__rsml_p2| add(__rsml_a1, __rsml_p2) }",
            ),
            (
                "( + )",
                "|lhs, rhs| lhs + rhs",
                "|__rsml_a1, __rsml_b2| __rsml_a1 + __rsml_b2",
            ),
            ("( ~- )", "|operand| -operand", "|__rsml_a1| -__rsml_a1"),
            (
                "{ p with x = 1 }",
                "{ let mut record = p; record.x = 1; record }",
                "{ let mut p = p; p.x = 1; p }",
            ),
            (
                "{ make () with x = 1 }",
                "{ let mut record = make(); record.x = 1; record }",
                "{ let mut __rsml_record = make(); __rsml_record.x = 1; __rsml_record }",
            ),
            (
                "(e : T)",
                "{ let typed: T = e; typed }",
                "{ let __rsml_typed: T = e; __rsml_typed }",
            ),
            (
                "let a = f () and b = g () in a + b",
                "{ let value1 = f(); let value2 = g(); let a = value1; let b = value2; a + b }",
                "{ let (a, b) = (f(), g()); a + b }",
            ),
        ];
        for (ml, rustc, editor) in cases {
            check(ml, rustc);
            // The editor's expansion is always a block.
            let editor = format!("{{{}}}", norm(editor));
            assert_eq!(expand_editor_plain(ml), editor, "\n  ml: {}", ml);
        }
    }

    /// A temporary that the generated code calls, or calls a method of, is
    /// `mut`: it may be a closure that mutates what it captures, or the
    /// method may take `&mut self`, and the user could not add the `mut`.
    #[test]
    fn called_temporaries_are_mut() {
        check(
            ".push (Vec::new ()) _",
            "{ let mut value1 = Vec::new(); move |_0| value1.push(_0) }",
        );
        check(
            "(Vec::new ()).push(_)",
            "{ let mut value1 = Vec::new(); move |_0| value1.push(_0) }",
        );
        check(
            "(_1.push(_0)) 1 (v ())",
            "{ let mut value1 = v(); value1.push(1) }",
        );
        check(
            "(_1 _0) 1 (make ())",
            "{ let mut value1 = make(); value1(1) }",
        );
        check(
            "(counter ()) _ 1",
            "{ let mut value1 = counter(); move |_0| value1(_0, 1) }",
        );
        check(
            "(fun a b -> a + b) 1",
            "{ let mut function1 = |a, b| a + b; move |arg2| function1(1, arg2) }",
        );
        check(
            "let ( +! ) a b = a + b in 1 +! 2",
            "{ let mut operator1 = |a, b| a + b; operator1(1, 2) }",
        );
        // An argument is passed by value: no `mut`.
        check(
            "add (g ()) _",
            "{ let value1 = g(); move |_0| add(value1, _0) }",
        );
    }

    /// `{ p with x = p.y }` reads `p` before the copy moves it, as Rust's
    /// `P { x: p.y, ..p }` does; rust-analyzer keeps the values in place,
    /// and gets a copy named `p` too: macro hygiene keeps the `p` of `p.y`
    /// the user's.
    #[test]
    fn record_update_values_come_first() {
        check(
            "{ p with x = p.y; y = 2 }",
            "{ let value1 = p.y; let mut record = p; record.x = value1; record.y = 2; record }",
        );
        assert_eq!(
            expand_editor_plain("{ p with x = p.y; y = 2 }"),
            norm("{{ let mut p = p; p.x = p.y; p.y = 2; p }}")
        );
    }

    /// `let mut f x = ..` is a closure that mutates what it captures.
    #[test]
    fn mutable_local_functions() {
        check(
            "let c = ref 0 in let mut next () = incr c; !c in next ()",
            "{ let mut c = 0; let mut next = || { c += 1; c }; next() }",
        );
        check(
            "let mut push x = v.push(x) in push 1",
            "{ let mut push = |x| v.push(x); push(1) }",
        );
        check_error(
            "let rec mut f (x : i64) : i64 = x in f 1",
            "a `let rec` function is a Rust `fn`, which cannot be `mut`",
        );
    }

    #[test]
    fn composition() {
        check("f >> g", "move |argument| g(f(argument))");
        check("f << g", "move |argument| f(g(argument))");
        check("f >> g >> h", "move |argument| h(g(f(argument)))");
        check("(f >> g) x", "g(f(x))");
        check("x |> (f >> g)", "g(f(x))");
        check("map (f >> g) xs", "map(move |argument| g(f(argument)), xs)");
        check(
            "let add x y = x + y in add 1 >> add 2",
            "{ let add = |x, y| x + y; move |argument| add(2, add(1, argument)) }",
        );
    }

    #[test]
    fn sections() {
        // A section is a function of its holes.
        check("f _ 1", "move |_0| f(_0, 1)");
        check("g _1 _0", "move |_0, _1| g(_1, _0)");
        // `_` takes the lowest number not written.
        check("f _1 _ _", "move |_0, _1, _2| f(_1, _0, _2)");
        check("map (_ * 2) xs", "map(|_0| _0 * 2, xs)");
        check("map (( * ) _0 _0) xs", "map(|_0| _0 * _0, xs)");
        check("mapi (f _1) xs", "mapi(|_, _1| f(_1), xs)");
        check("map (_.len()) xs", "map(|_0| _0.len(), xs)");
        check("map (Some _) xs", "map(|_0| Some(_0), xs)");
        check(
            "fold (cons _1 _0) [] xs",
            "fold(|_0, _1| cons(_1, _0), ::std::vec::Vec::new(), xs)",
        );
        // Applied directly, there is no closure.
        check("x |> f _ 1", "f(x, 1)");
        check("(g _1 _0) a b", "g(b, a)");
        check(
            "xs.iter() |> Iterator::map _ (_ + 1) |> Iterator::sum::<i64>",
            "Iterator::sum::<i64>(Iterator::map(xs.iter(), |_0| _0 + 1))",
        );
        // Arguments are evaluated once, in the order written.
        check(
            "(g _1 _0) (a ()) (b ())",
            "{ let value1 = a(); let value2 = b(); g(value2, value1) }",
        );
        check(
            "(f _0 _0) (a ())",
            "{ let value1 = a(); f(value1, value1) }",
        );
        check(
            "map (f (g ()) _) xs",
            "map({ let value1 = g(); move |_0| f(value1, _0) }, xs)",
        );
        // Partially applied.
        check("(g _1 _0) a", "move |_1| g(_1, a)");
        // `&&` and `||` short-circuit, so their operands stay in the body.
        check(
            "filter (_ > 0 && ok ()) xs",
            "filter(|_0| (_0 > 0) && ok(), xs)",
        );
        // Each side of `|>` is a whole expression.
        check("map (fun x -> x |> f _ 2) xs", "map(|x| f(x, 2), xs)");
    }

    #[test]
    fn annotated_holes_type_the_parameters() {
        check("map (f (_ : &str) 1) xs", "map(|_0: &str| f(_0, 1), xs)");
        check(
            "(_.len() + (_1 : i64)) |> keep",
            "keep(|_0, _1: i64| _0.len() + _1)",
        );
        check("x |> f (_ : i64) 1", "f(x, 1)");
    }

    #[test]
    fn lambdas_applied_directly_are_inlined() {
        check("(fun x -> x + 1) 2", "{ let x = 2; x + 1 }");
        check("3 |> fun x -> x * x", "{ let x = 3; x * x }");
    }

    #[test]
    fn closures() {
        check("fun x y -> x + y", "move |x, y| x + y");
        check("fun () -> 1", "move || 1");
        // Only closures that escape as the result are `move`, and never ones that mutate.
        check("map (fun x -> x * 2) xs", "map(|x| x * 2, xs)");
        check("iter (fun x -> n <- x * 2) xs", "iter(|x| n = x * 2, xs)");
        check("fun x -> n <- x", "|x| n = x");
        check("map (fun move x -> x * 2) xs", "map(move |x| x * 2, xs)");
        check("let f = fun x -> x in f", "{ let f = |x| x; f }");
        check("Some (fun x -> x)", "Some(move |x| x)");
    }

    #[test]
    fn unknown_functions_take_all_arguments() {
        check("f a b", "f(a, b)");
        check("f ()", "f()");
        check("f (())", "f(())");
        check("f () x", "f((), x)");
        check("Vec.new ()", "Vec::new()");
        check("String.from \"a\"", "String::from(\"a\")");
        check("std::mem::swap a b", "std::mem::swap(a, b)");
        check("Some (1, 2)", "Some((1, 2))");
        check("Pair (1, 2)", "Pair(1, 2)");
        check("Pair ((1, 2))", "Pair((1, 2))");
    }

    #[test]
    fn external_declares_arity() {
        check(
            "external add : i64 -> i64 -> i64 ;; add 1",
            "move |arg2| add(1, arg2)",
        );
        check(
            "external adder : i64 -> (i64 -> i64) ;; adder 1 2",
            "adder(1)(2)",
        );
        check("external now : () -> u64 ;; now ()", "now()");
        check("(add 1 : i64 -> i64)", "move |arg2| add(1, arg2)");
    }

    #[test]
    fn operators() {
        check("a + b * c", "a + (b * c)");
        check("(a + b) * c", "(a + b) * c");
        check("a mod b = 0", "(a % b) == 0");
        check("a <> b && not c", "(a != b) && (!c)");
        check("x :: [y; z]", "::std::vec![x, y, z]");
        check("x :: xs", "::rsml::__rt::cons(x, xs)");
        check("a ^ b", "::rsml::__rt::concat(a, b)");
        check("- x", "-x");
        check("2.0 ** 3.0", "::core::primitive::f64::powf(2.0, 3.0)");
    }

    #[test]
    fn match_and_lists() {
        check(
            "match xs with [] -> 0 | x :: rest -> 1",
            "match ::rsml::__rt::as_slice(&xs) { [] => 0, [x, rest @ ..] => 1, }",
        );
        check(
            "match o with Some x when x > 0 -> x | _ -> 0",
            "match o { Some(x) if x > 0 => x, _ => 0, }",
        );
        check(
            "match s with \"a\" -> 1 | _ -> 2",
            "match ::rsml::__rt::as_str(&s) { \"a\" => 1, _ => 2, }",
        );
        check(
            "match e with Add (a, b) -> a | Neg _ -> 0",
            "match e { Add(a, b) => a, Neg(..) => 0, }",
        );
        check(
            "match p with Point { x; _ } -> x",
            "match p { Point { x, .. } => x, }",
        );
    }

    #[test]
    fn types_are_rust() {
        check("let x : u8 = 1 in x", "{ let x: u8 = 1; x }");
        check(
            "let v : Vec<(i64, String)> = f () in v",
            "{ let v: Vec<(i64, String)> = f(); v }",
        );
        check(
            "let rec id (x : Vec<'a>) : Vec<'a> = x in id",
            "{ fn id<A>(x: Vec<A>) -> Vec<A> { x } id }",
        );
        check(
            "let rec go (x : i64 * &'b str) (f : i64 -> i64) : Option<'b> = None in go",
            "{ fn go<B>(x: (i64, &'b str), f: impl ::core::ops::Fn(i64) -> i64) -> Option<B> { None } go }",
        );
        check(
            "let x : Box<dyn Fn(i64) -> i64> = f () in x",
            "{ let x: Box<dyn Fn(i64) -> i64> = f(); x }",
        );
    }

    #[test]
    fn let_rec_is_a_fn_item() {
        check(
            "let rec fact (n : i64) : i64 = if n = 0 then 1 else n * fact (n - 1) in fact 5",
            "{ fn fact(n: i64) -> i64 { if n == 0 { 1 } else { n * fact(n - 1) } } fact(5) }",
        );
    }

    #[test]
    fn rust_interop() {
        check("xs.len()", "xs.len()");
        check(
            "xs.iter().map(fun x -> x + 1).collect::<Vec<_>>()",
            "xs.iter().map(|x| x + 1).collect::<Vec<_>>()",
        );
        check("n as f64 *. 2.0", "(n as f64) * 2.0");
        check("Some ()", "Some(())");
        check("a && b", "a && b");
        check("println!(\"{}\", x)", "println!(\"{}\", x)");
        check("f &x", "f(&x)");
        // `x.f (a)` and `x.f ()` apply the field `f`.
        check("r.f (x)", "(r.f)(x)");
        check("r.f ()", "(r.f)()");
        check("f t.len (x + 1)", "f(t.len, x + 1)");
        check("tx.send(())", "tx.send(())");
    }

    #[test]
    fn method_refs() {
        check(".iter xs", "xs.iter()");
        check(".push xs 1", "xs.push(1)");
        check(".collect::<Vec<_>> it", "it.collect::<Vec<_>>()");
        check(".get (f x) 0", "f(x).get(0)");
        check(".len", "|receiver| receiver.len()");
        check(".await fut", "fut.await");
        // The receiver takes the place of `()`, so `()` after it is an argument.
        check(".send tx ()", "tx.send(())");
        check("tx |> .send _ ()", "tx.send(())");
        check("f ()", "f()");
        // In pipelines, with placeholders for the receiver.
        check("xs |> .iter |> .count", "xs.iter().count()");
        check(
            "xs |> .iter |> .map _ (_ * 2) |> .sum::<i64>",
            "xs.iter().map(|_0| _0 * 2).sum::<i64>()",
        );
        check("xs |> .push _ 1", "xs.push(1)");
        check("map (.len) xs", "map(|receiver| receiver.len(), xs)");
        check("map (.len (_ : &str)) xs", "map(|_0: &str| _0.len(), xs)");
        check(".iter >> .count", "move |argument| argument.iter().count()");
        // After an expression, `.name` is that expression's field or method.
        check("xs\n  .iter()\n  .count()", "xs.iter().count()");
        check("a; .push v 1", "{ a; v.push(1) }");
    }

    #[test]
    fn rust_style_calls() {
        // `(` right after the function: one argument per item.
        check("add(1, 2)", "add(1, 2)");
        check("f()", "f()");
        check("f(())", "f(())");
        check("f(a)(b)", "f(a)(b)");
        check("f(a) b", "f(a)(b)");
        check("Vec::new()", "Vec::new()");
        check("std::mem::size_of::<u8>()", "std::mem::size_of::<u8>()");
        check("t.0(1, 2)", "(t.0)(1, 2)");
        check("xs.iter().map(fun x -> x)(1)", "xs.iter().map(|x| x)(1)");
        // With a space it is OCaml: one tuple argument.
        check("f (a, b)", "f((a, b))");
        check("f (a)(b)", "f(a(b))");
        // Constructors keep their rules in both spellings.
        check("Some(1, 2)", "Some((1, 2))");
        check("Pair(1, 2)", "Pair(1, 2)");
        check("(Some)(1)", "Some(1)");
        // Known shapes: the items are that many arguments.
        check(
            "let add x y = x + y in add(1, 2)",
            "{ let add = |x, y| x + y; add(1, 2) }",
        );
        check(
            "let add x y = x + y in add(1)",
            "{ let add = |x, y| x + y; move |arg2| add(1, arg2) }",
        );
        check(
            "let add x y = x + y in add(1)(2)",
            "{ let add = |x, y| x + y; add(1, 2) }",
        );
        check(
            "let add x y = x + y in add(1, 2, 3)",
            "{ let add = |x, y| x + y; add(1, 2)(3) }",
        );
        check(
            "let get x y = h in get(1, 2)(3)(4)",
            "{ let get = |x, y| h; get(1, 2)(3)(4) }",
        );
        check(
            "external add : i64 -> i64 -> i64 ;; add(1)",
            "move |arg2| add(1, arg2)",
        );
        check("(f >> g)(x)", "g(f(x))");
        check("( + )(1, 2)", "1 + 2");
        check("(fun x y -> x + y)(1)(2)", "{ let (x, y) = (1, 2); x + y }");
        check(
            "(fun x -> g x)(1)(2)",
            "{ let value1 = 2; let x = 1; g(x)(value1) }",
        );
        check("not(x)", "!x");
        check(
            "let r = ref(0) in r := 5; !r",
            "{ let mut r = 0; r = 5; r }",
        );
        // Pipes give the result of the call the piped value.
        check("x |> f(a)", "f(a)(x)");
        check(
            "let add x y = x + y in x |> add(1)",
            "{ let add = |x, y| x + y; add(1, x) }",
        );
        // Each item is a full expression for placeholders.
        check("f(_, 1)", "move |_0| f(_0, 1)");
        check("x |> f(_, 1)", "f(x, 1)");
        check("map(_ + 1, xs)", "map(|_0| _0 + 1, xs)");
        // A function type annotation: the call is complete, its result a function.
        check("(f(1) : i64 -> i64)", "f(1)");
        check(
            "(f(1) 2 : i64 -> i64)",
            "{ let mut function1 = f(1); move |arg2| function1(2, arg2) }",
        );
        // A Rust function's call is evaluated once in a composition.
        check(
            "mk(1) >> g",
            "{ let mut value1 = mk(1); move |argument| g(value1(argument)) }",
        );
        check(
            "let add x y = x + y in add(1) >> add(2)",
            "{ let add = |x, y| x + y; move |argument| add(2, add(1, argument)) }",
        );
        // A trailing comma is kept for an editor only.
        check("f(a,)", "f(a)");
        assert_eq!(expand_editor("f(a,)"), norm("{ f(a,) }"));
        check("xs.push(a,)", "xs.push(a)");
        assert_eq!(expand_editor("xs.push(a,)"), norm("{ xs.push(a,) }"));
    }

    #[test]
    fn adjacent_constructor_applications_are_atoms() {
        // Written against the constructor, it is applied first, as in Rust.
        check("Some(x).unwrap()", "Some(x).unwrap()");
        check("Shape::Rect(w, h).area()", "Shape::Rect(w, h).area()");
        check("Wrapper(v).0", "Wrapper(v).0");
        check("Wrapper(v).inner", "Wrapper(v).inner");
        check("Some(x)[0]", "Some(x)[0]");
        check("Ok(v)?", "Ok(v)?");
        check("Wrap(f).await", "Wrap(f).await");
        check("(Some)(x).is_some()", "Some(x).is_some()");
        check("f Some(x) y", "f(Some(x), y)");
        check("Pair(1, 2).first()", "Pair(1, 2).first()");
        check("Some(1, 2).is_some()", "Some((1, 2)).is_some()");
        // Spaced, it is OCaml: the method is the argument's.
        check("Some (x).unwrap()", "Some(x.unwrap())");
        check("f Some (x) y", "f(Some, x, y)");
        // Further arguments still go to the constructor, as before.
        check("Some(x) y", "Some(x, y)");
    }

    #[test]
    fn rust_style_index() {
        check("v[0]", "v[0]");
        check("m[&k]", "m[&k]");
        check("f(x)[0]", "f(x)[0]");
        check("x.f[1]", "x.f[1]");
        check("t.0[1]", "t.0[1]");
        check("xs[1..]", "xs[1..]");
        check("_[0]", "move |_0| _0[0]");
        check("v[0] <- 1", "v[0] = 1");
        // With a space, or `[|..|]`, it is a list or array argument.
        check("f [1; 2]", "f(::std::vec![1, 2])");
        check("f[|1; 2|]", "f([1, 2])");
        // After `?`, as in Rust; spaced, OCaml.
        check("f(x)?[0]", "f(x)?[0]");
        check("x?(1, 2)", "(x?)(1, 2)");
        check("x?.len()", "x?.len()");
        check("g x? [0]", "g(x?, ::std::vec![0])");
        check("x? (1, 2)", "(x?)((1, 2))");
    }

    #[test]
    fn method_sections() {
        check("xs |> .map(f)", "xs.map(f)");
        check("xs |> .iter() |> .count()", "xs.iter().count()");
        check(".push(1) xs", "xs.push(1)");
        check(
            "xs |> .iter() |> .map(fun x -> x * 2) |> .sum::<i64>()",
            "xs.iter().map(|x| x * 2).sum::<i64>()",
        );
        check(".map(f) xs ys", "xs.map(f)(ys)");
        check(".map(f)", "move |receiver| receiver.map(f)");
        check("g (.map(f))", "g(|receiver| receiver.map(f))");
        check(".iter()", "|receiver| receiver.iter()");
        check("it |> .collect::<Vec<_>>()", "it.collect::<Vec<_>>()");
        check(
            ".iter() >> .count()",
            "move |argument| argument.iter().count()",
        );
        check("map (.len() (_ : &str)) xs", "map(|_0: &str| _0.len(), xs)");
        // With a space the argument is the receiver, as before.
        check(".len (x)", "x.len()");
        check(".push xs 1", "xs.push(1)");
        check("(.len)(x)", "x.len()");
    }

    #[test]
    fn sections_glued_to_pipes() {
        // Rust lexes `|>.` as one operator: with none defined, it is `|>`
        // and the section.
        check("xs |>.iter() |>.count()", "xs.iter().count()");
        check("xs |>.iter |>.map _ f |>.sum", "xs.iter().map(f).sum()");
        check("xs|>.len", "xs.len()");
        check("t |>.0", "t.0");
        check("t |>.0.1", "t.0.1");
        check("fut |>.await", "fut.await");
        check("x |>.r#loop", "x.r#loop()");
        check(".iter >>.count", "move |argument| argument.iter().count()");
        check("f <<.len", "move |argument| f(argument.len())");
        check("f @@.len xs", "f(xs.len())");
        // A defined operator is the operator, where it is in scope.
        check(
            "let ( |>. ) a b = a + b in x |>.y",
            "{ let mut operator1 = |a, b| a + b; operator1(x, y) }",
        );
        check(
            "let ( |>. ) a b = a + b ;; x |>.y",
            "{ let mut operator1 = |a, b| a + b; operator1(x, y) }",
        );
        check(
            "let ( >>. ) a b = a in f >>.g",
            "{ let mut operator1 = |a, b| a; operator1(f, g) }",
        );
        check(
            r#"external ( |>. ) : _ -> _ -> _ = "op" ;; x |>.y"#,
            "op(x, y)",
        );
        check(
            "let f = (let ( |>. ) a b = a in 1) in x |>.y",
            "{ let f = { let mut operator1 = |a, b| a; 1 }; x.y() }",
        );
        // Never a float operator, nor without a name or number after the dot.
        check("x +.y", "x + y");
        check("1. *.2.", "1. * 2.");
        for src in ["xs |>. iter", "xs |>.(f)", r#"x |>."s""#] {
            check_error(
                src,
                "unknown operator `|>.`; for a method or field section after `|>`, write `|> .name`",
            );
        }
        check_error("f >>. g", "write `>> .name` or `>> .0`");
        // Unfinished, it completes like `xs |> .`.
        check_recovers("xs |>.$");
        check_recovers("xs |>.$ |>.sum");
        check_recovers("let k = xs |>.$ in k");
        check_completes("xs |>.it$");
        check_completes("xs |>.iter() |>.ma$(f)");
    }

    #[test]
    fn std_macro_arguments_are_ocaml() {
        check("assert_eq!(add 1 2, 3)", "assert_eq!(add(1, 2), 3)");
        check(
            r#"println!("{}", xs |> .len)"#,
            r#"println!("{}", xs.len())"#,
        );
        check(
            r#"format!("{} {x} {}", f x, g 1, x = h 2)"#,
            r#"format!("{} {x} {}", f(x), g(1), x = h(2))"#,
        );
        // Each argument ends at a `,` outside brackets, not at every `,`.
        check(
            r#"assert!(a < b, "{:?}", HashMap::<K, V>::new ())"#,
            r#"assert!(a < b, "{:?}", HashMap::<K, V>::new())"#,
        );
        check("vec![0; n + 1]", "vec![0; n + 1]");
        // A trailing comma is kept for an editor only.
        check("vec![f 1, g 2,]", "vec![f(1), g(2)]");
        assert_eq!(expand_editor("vec![f 1,]"), norm("{ vec![f(1),] }"));
        check("vec![]", "vec![]");
        check(
            r#"std::println!("{}", f x); ::core::assert_ne!(a, b); alloc::vec![a]"#,
            r#"{ std::println!("{}", f(x)); ::core::assert_ne!(a, b); alloc::vec![a] }"#,
        );
        check(r#"write!(out, "{}", f x)?"#, r#"write!(out, "{}", f(x))?"#);
        check(
            r#"println!("{}", format!("{}", f x))"#,
            r#"println!("{}", format!("{}", f(x)))"#,
        );
        check("dbg!(f x)", "dbg!(f(x))");
        check("todo!()", "todo!()");
        // `=` after the format string names an argument; before it, it compares.
        check(r#"println!("{}", a = b)"#, r#"println!("{}", a = b)"#);
        check(r#"println!("{}", (a = b))"#, r#"println!("{}", a == b)"#);
        check("assert!(a = b)", "assert!(a == b)");
        check("assert_eq!(a = b, c)", "assert_eq!(a == b, c)");
        // OCaml's `assert e` is still the keyword.
        check("assert!(x > 0)", "assert!(x > 0)");
        check("assert (x > 0)", "::core::assert!(x > 0)");
        check("assert !r", "::core::assert!(*r)");
        // Values escape from `vec!` and `dbg!` only.
        check("vec![fun x -> x]", "vec![move |x| x]");
        check(
            r#"println!("{:?}", Some(fun x -> x))"#,
            r#"println!("{:?}", Some(|x| x))"#,
        );
        // A `_` alone belongs to the expression around the macro.
        check(r#"println!("{}", _)"#, r#"move |_0| println!("{}", _0)"#);
        // Other macros keep Rust arguments.
        check("matches!(x, Some(1) | None)", "matches!(x, Some(1) | None)");
        check("my_macro!(a b; c)", "my_macro!(a b; c)");
        check("std::matches!(x, _)", "std::matches!(x, _)");
        check_error(r#"println!("{}", a; b)"#, "unexpected");
        check_error("vec![a; b; c]", "unexpected");
    }

    #[test]
    fn negation_is_not() {
        // `!` dereferences, so a Rust negation would assert the opposite.
        check_error(
            "assert!(!xs.is_empty())",
            "`!` dereferences in OCaml and binds tighter than `.`",
        );
        check_error("debug_assert!(!v[0], \"msg\")", "binds tighter than `.`");
        check("assert!(not (xs.is_empty()))", "assert!(!xs.is_empty())");
        check("assert!((!xs).is_empty())", "assert!((*xs).is_empty())");
        check(
            "let r = ref [1] in assert!(!r.is_empty())",
            "{ let mut r = ::std::vec![1]; assert!(r.is_empty()) }",
        );
        // `!` before something plainly `bool` is a mistake anywhere.
        for src in [
            "!(a = b)",
            "if !(n > 0) then 1 else 2",
            "!true",
            "f !(not x)",
            "!(a && b)",
        ] {
            check_error(src, "a `bool` cannot be dereferenced");
        }
        check("!x", "*x");
        check("!(f x)", "*f(x)");
        check(
            "let ( < ) a b = a in !(x < y)",
            "{ let mut operator1 = |a, b| a; *operator1(x, y) }",
        );
    }

    #[test]
    fn rust_deref() {
        check("*self_", "*self_");
        // `*` binds like Rust's, `!` like OCaml's.
        check("*x.f", "*x.f");
        check("!x.f", "(*x).f");
        check("*v.len()", "*v.len()");
        check("*f(x)", "*f(x)");
        // Rust glues prefix operators; each is its own.
        check("&*s", "&(*s)");
        check("**q", "*(*q)");
        check("!*p", "*(*p)");
        check("&mut *p", "&mut (*p)");
        check("*p <- 3", "*p = 3");
        check("a..*b", "a..(*b)");
        check("let r = ref 1 in *r + 1", "{ let mut r = 1; r + 1 }");
        // After an operand, `*` multiplies.
        check("f *x", "f * x");
        check("f ( *x)", "f(*x)");
        check_error("*(a = b)", "a `bool` cannot be dereferenced");
    }

    #[test]
    fn fuzz_regressions() {
        // A spaced `; ;` is two `;`s, not `;;`, and an operator cannot come
        // right after a `;`: before, the expression ended there and the
        // rest attached to the construct around it.
        for src in [
            "let mut n = 0 in n <- n + 1; ;  n",
            "f 1; ; g 2",
            "begin f 1; ; g 2 end",
        ] {
            check_error(src, "`; ;` is two `;`s");
        }
        for src in [
            "incr r; + c",
            "let r = ref 0 in incr r;  := !r + 1",
            "incr c; , !c",
        ] {
            check_error(src, "expected an expression after `;`, before this");
        }
        check("f 1; ;; g 2", "{ f(1); g(2) }");
        check("f 1; - 1", "{ f(1); -1 }");
        check("f 1; *x", "{ f(1); *x }");
        // Glued `&`s in types and patterns are that many references.
        check("(x : &&&str)", "{ let typed: &&&str = x; typed }");
        check("match x with &&&a -> a", "match x { &&&a => a, }");
        // `!` is never infix: a run starting with it is an argument.
        check("a !*!b", "a(*(*(*b)))");
        check("f !&x", "f(*(&x))");
        // A record's trailing `;` is a trailing `,` for an editor only.
        check("Point { x = 1; }", "Point { x: 1 }");
        assert_eq!(
            expand_editor("Point { x = 1; }"),
            norm("{ Point { x: 1, } }")
        );
        // `[T; ]` has no length (it expanded to `[T;]`, no Rust type).
        check_error("(l : [i32 ; ])", "expected the array's length after `;`");
        check("(l : [i32; 3])", "{ let typed: [i32; 3] = l; typed }");
    }

    #[test]
    fn raw_identifiers_are_names() {
        // In every name position (values, binders, parameters, fields,
        // methods, labels, paths, record fields, patterns), emitted raw.
        check(
            "let r#type = 1 in r#type + 1",
            "{ let r#type = 1; r#type + 1 }",
        );
        check("fun r#fn -> r#fn + 1", "move |r#fn| r#fn + 1");
        check("p.r#type + x.r#match()", "p.r#type + x.r#match()");
        check("r#mod::f x", "r#mod::f(x)");
        check("Point { r#type = 1; y = 2 }", "Point { r#type: 1, y: 2 }");
        check(
            "match x with Some r#in -> r#in | None -> 0",
            "match x { Some(r#in) => r#in, None => 0, }",
        );
        check(
            "let f ~r#in = r#in + 1 in f ~r#in:2",
            "{ let f = |r#in| r#in + 1; f(2) }",
        );
        // Never a keyword: `r#match x` applies a function named `match`, and
        // `r#ref`, `r#not` are Rust functions, not OCaml's.
        check("r#match x", "r#match(x)");
        check("r#ref 1 + r#not true", "r#ref(1) + r#not(true)");
        // `r#x` and `x` are one name, as in Rust.
        check(
            "let r#x = ref 0 in incr x; !r#x",
            "{ let mut r#x = 0; x += 1; r#x }",
        );
    }

    #[test]
    fn reserved_words_are_names() {
        // Rust reserves these (some only from 2018 or 2024 on) without
        // syntax rsml uses: names, emitted raw, which every edition accepts.
        check("let gen = 3 in gen + 1", "{ let r#gen = 3; r#gen + 1 }");
        check(
            "rng.gen() + rng.gen::<u32>()",
            "rng.r#gen() + rng.r#gen::<u32>()",
        );
        check("xs |> .gen", "xs.r#gen()");
        check(
            "p.gen + Point { gen = 1 }.gen",
            "p.r#gen + Point { r#gen: 1 }.r#gen",
        );
        check(
            "match x with Some gen -> gen | None -> 0",
            "match x { Some(r#gen) => r#gen, None => 0, }",
        );
        check(
            "let f ~gen = gen in f ~gen:1",
            "{ let f = |r#gen| r#gen; f(1) }",
        );
        check("gen::f x", "r#gen::f(x)");
        check(
            "let try = 1 in let yield = 2 in let box = 3 in let macro = 4 in try + yield + box + macro",
            "{ let r#try = 1; let r#yield = 2; let r#box = 3; let r#macro = 4; \
             ((r#try + r#yield) + r#box) + r#macro }",
        );
        check(
            "let abstract = 1 in let become = 2 in let final = 3 in let override = 4 in \
             let priv = 5 in let typeof = 6 in let unsized = 7 in let virtual = 8 in \
             abstract + become + final + override + priv + typeof + unsized + virtual",
            "{ let r#abstract = 1; let r#become = 2; let r#final = 3; let r#override = 4; \
             let r#priv = 5; let r#typeof = 6; let r#unsized = 7; let r#virtual = 8; \
             ((((((r#abstract + r#become) + r#final) + r#override) + r#priv) + r#typeof) \
             + r#unsized) + r#virtual }",
        );
        // OCaml's `try .. with` still gets its error; a `try` with no `with`
        // after it is a name.
        check_error("try f x with _ -> 0", "`try .. with` is not supported");
        check_error(
            "let y = try f x with _ -> 0 in y",
            "`try .. with` is not supported",
        );
        check("try x |> f", "f(r#try(x))");
        check("try (match x with _ -> 1)", "r#try(match x { _ => 1, })");
        // Rust syntax keeps its errors.
        check_error(
            "let dyn = 1 in dyn",
            "`dyn` is a Rust keyword, not a name (as a name: `r#dyn`)",
        );
        check_error("let async = 1 in async", "`async` is a Rust keyword");
        check_error("let fn = 1 in fn", "`fn` is a Rust keyword");
        check("let r#dyn = 1 in r#dyn", "{ let r#dyn = 1; r#dyn }");
        // In `rsml!`, a body sees a parameter `gen` as `r#gen`, the same name.
        let ts: TokenStream = "fn f(gen: i64) -> i64 { gen + 1 }".parse().unwrap();
        let out: String = crate::codegen::without_markers(|| crate::expand_items(ts))
            .to_string()
            .split_whitespace()
            .collect();
        assert_eq!(out, norm("fn f(gen: i64) -> i64 { r#gen + 1 }"));
    }

    #[test]
    fn bang_operators_cannot_be_defined() {
        // As in OCaml, an operator starting with `!` is prefix, and here
        // `!` before an expression is a dereference: it could never be used.
        let msg = "cannot be an operator: one starting with `!` is a prefix operator";
        check_error("let ( !* ) a b = a + b in 1 !* 2", msg);
        check_error("let ( !* ) a = a + 1 in !* 2", msg);
        check_error("let ( ! ) r = r + 1 in !2", msg);
        check_error("let rec ( !+ ) a = a and f x = x in 1", msg);
        check_error("external ( !! ) : i64 -> i64 = \"f\" ;; 1", msg);
        check_error("(!*)", msg);
        // `!=` is infix, in OCaml too.
        check(
            "let ( != ) a b = a + b in 1 != 2",
            "{ let mut operator1 = |a, b| a + b; operator1(1, 2) }",
        );
        check("!*p + !r", "(*(*p)) + (*r)");
    }

    #[test]
    fn field_sections() {
        check("x |> .0", "x.0");
        check(".1 t", "t.1");
        check(".0", "|tuple| tuple.0");
        check(
            "pairs |> .iter() |> .map(.0)",
            "pairs.iter().map(|tuple| tuple.0)",
        );
        check("pairs |> .map _ (.1)", "pairs.map(|tuple| tuple.1)");
        check(".0.1 t", "t.0.1");
        check("f x |> .0", "f(x).0");
        check(".0 t x", "(t.0)(x)");
        check("(.0 >> g) t", "g(t.0)");
        // After an expression `.0` is still that expression's field.
        check("f t.0 .1", "f(t.0.1)");
        check_error("x |> . 0", "no space is allowed between `.` and the field");
        check_error("x |> .0u8", "expected a tuple index");
    }

    #[test]
    fn for_in_loops() {
        check(
            "for (k, v) in m do f k v done",
            "for (k, v) in m { f(k, v); }",
        );
        check("for k, v in m do () done", "for (k, v) in m { (); }");
        check(
            "for x in xs do f x; g x done",
            "for x in xs { { f(x); g(x) }; }",
        );
        check("for i in 0..n do f i done", "for i in 0..n { f(i); }");
        // Counting loops: `to`/`downto` include the end (OCaml), `til` does not.
        check("for i = 0 to n do f i done", "for i in 0..=n { f(i); }");
        check(
            "for i = n downto 0 do f i done",
            "for i in (0..=n).rev() { f(i); }",
        );
        check("for i = 0 til n do f i done", "for i in 0..n { f(i); }");
        check_error("let til = 1 in til", "unexpected keyword `til`");
        // The pattern is bound in the body only.
        check(
            "let h::t = l in for h in t do h::t done",
            "{ let [h, t @ ..] = ::rsml::__rt::as_slice(&l) else { ::core::panic!(\"Match_failure\") }; for h in t { ::rsml::__rt::cons(h, t); } }",
        );
    }

    #[test]
    fn ranges() {
        check("a..b", "a..b");
        check("a..=b", "a..=b");
        check("a..", "a..");
        check("..b", "..b");
        check("..=b", "..=b");
        check("..", "..");
        check("xs[..]", "xs[..]");
        check("0..-1", "0..(-1)");
        // Looser than arithmetic, `::`, `@` and `^`; tighter than comparisons and `|>`.
        check("0..n + 1", "0..(n + 1)");
        check("a * 2..b", "(a * 2)..b");
        check("0..n |> .map(f)", "(0..n).map(f)");
        check("a..b = c", "(a..b) == c");
        check("a || b..c", "a || (b..c)");
        check("f (0..n)", "f(0..n)");
        check("(0..n).rev()", "(0..n).rev()");
        check("for i in 0.. do () done", "for i in 0.. { (); }");
        check_error("a..b..c", "ranges do not chain");
        check_error("0..=", "the end of the range");
    }

    #[test]
    fn tail_recursion_is_a_loop() {
        check(
            "let rec go (acc : i64) (n : i64) : i64 = if n = 0 then acc else go (acc + n) (n - 1) in go 0 10",
            "{ fn go(mut arg1: i64, mut arg2: i64) -> i64 { loop { let acc = arg1; let n = arg2; \
             break if n == 0 { acc } else { (arg1, arg2) = (acc + n, n - 1); continue }; } } go(0, 10) }",
        );
        // A list parameter is matched without borrowing the variable, so
        // that its tail can become the next value.
        check(
            "let rec go (acc : i64) (l : &[i32]) : i64 = match l with [] -> acc | _ :: rest -> go (acc + 1) rest in go 0 l",
            "{ fn go(mut arg1: i64, mut arg2: &[i32]) -> i64 { loop { let acc = arg1; let l = arg2; \
             break match ::rsml::__rt::as_slice(l) { [] => acc, [_, rest @ ..] => { (arg1, arg2) = (acc + 1, rest); continue } , }; } } go(0, l) }",
        );
        // Labeled self-calls assign in the order written; `function` too.
        check(
            "let rec f ~(x : i64) ~(y : i64) : i64 = if x = 0 then y else f ~y:(y + 1) ~x:(x - 1) in f 3 0",
            "{ fn f(mut arg1: i64, mut arg2: i64) -> i64 { loop { let x = arg1; let y = arg2; \
             break if x == 0 { y } else { (arg2, arg1) = (y + 1, x - 1); continue }; } } f(3, 0) }",
        );
        check(
            "let rec down : u32 -> u32 = function 0 -> 0 | k -> down (k - 1) in down 3",
            "{ fn down(mut arg1: u32) -> u32 { loop { let argument = arg1; \
             break match argument { 0 => 0, k => { arg1 = k - 1; continue } , }; } } down(3) }",
        );
        // Not a tail call, used as a value, or mutually recursive: a `fn`.
        check(
            "let rec fact (n : i64) : i64 = if n = 0 then 1 else n * fact (n - 1) in fact 5",
            "{ fn fact(n: i64) -> i64 { if n == 0 { 1 } else { n * fact(n - 1) } } fact(5) }",
        );
        check(
            "let rec go (n : i64) : i64 = let g = go in if n = 0 then 0 else go (n - 1) in go 3",
            "{ fn go(n: i64) -> i64 { let g = go; if n == 0 { 0 } else { go(n - 1) } } go(3) }",
        );
        check(
            "let rec a (n : i64) : i64 = if n = 0 then 0 else b (n - 1) and b (n : i64) : i64 = a n in a 3",
            "{ fn a(n: i64) -> i64 { if n == 0 { 0 } else { b(n - 1) } } fn b(n: i64) -> i64 { a(n) } a(3) }",
        );
    }

    #[test]
    fn range_patterns_are_rust_ranges() {
        check(
            "match n with 1..=3 -> 1 | -5..=-1 -> 2 | _ -> 0",
            "match n { 1..=3 => 1, -5..=-1 => 2, _ => 0, }",
        );
        // `..` leaves the end out, as in Rust, however it is spaced.
        check(
            "match c with 'a' .. 'z' -> 1 | 'A'..'Z' -> 2 | _ -> 0",
            "match c { 'a'..'z' => 1, 'A'..'Z' => 2, _ => 0, }",
        );
        check(
            "match n with 0..10 -> 1 | -10..0 -> 2 | _ -> 0",
            "match n { 0..10 => 1, -10..0 => 2, _ => 0, }",
        );
    }

    #[test]
    fn return_break_continue() {
        check("if n < 0 then return 0; n", "{ if n < 0 { return 0 } ; n }");
        check("return", "return");
        check("return (a, b)", "return (a, b)");
        check("return - 1", "return -1");
        check("x + return 1", "x + (return 1)");
        check(
            "match x with A -> return 1 | B -> 2",
            "match x { A => return 1, B => 2, }",
        );
        check("fun x -> return x", "move |x| return x");
        check(
            "while c do if d then break; continue done",
            "while c { { if d { break } ; continue }; }",
        );
        // `while true` is OCaml's endless loop, Rust's `loop`.
        check(
            "while true do if x then break; f () done",
            "loop { { if x { break } ; f() }; }",
        );
        check("while (true) do () done", "loop { (); }");
        check(
            "let r = while true do break 5 done in r",
            "{ let r = loop { break 5; }; r }",
        );
        check_error("f return", "put `return` in parentheses");
    }

    #[test]
    fn unsafe_and_async_blocks() {
        check("unsafe { f x }", "unsafe { f(x) }");
        check("unsafe {}", "unsafe {}");
        check("unsafe { let y = 1 in y }", "unsafe { let y = 1; y }");
        check("async { f x }", "async { f(x) }");
        check("async move { x }", "async move { x }");
        check("async { fun x -> x }", "async { move |x| x }");
        check("f unsafe { x }", "f(unsafe { x })");
    }

    #[test]
    fn rust_keywords_are_not_names() {
        for (src, msg) in [
            ("let loop = 1 in loop", "`loop` is a Rust keyword"),
            ("fn x", "a closure is `fun x -> e`"),
            (
                "fn f x = x",
                "a named local function is `let f x = .. in ..`, and a closure is `fun x -> e`",
            ),
            ("fn x = 1", "a named local function is `let x y = .. in ..`"),
            ("let f fn = 1 in f", "`fn` is a Rust keyword"),
            ("let fn x = x in 1", "`fn` is a Rust keyword"),
            ("xs.loop", "`loop` is a Rust keyword"),
            ("p.type", "`type` is a Rust keyword"),
            ("Foo::fn", "`fn` is a Rust keyword"),
            ("match o with Some(ref x) -> x", "`ref` is a Rust keyword"),
            ("for impl = 0 to 1 do () done", "`impl` is a Rust keyword"),
            ("P { struct = 1 }", "`struct` is a Rust keyword"),
            ("fun move -> 1", "`move` is a Rust keyword"),
            ("unsafe f", "a block is `unsafe { e }`"),
        ] {
            check_error(src, msg);
        }
        // As raw identifiers they are names, and `self`, `await`, `ref e` keep working.
        check(
            "let r#loop = 2 in r#loop + 1",
            "{ let r#loop = 2; r#loop + 1 }",
        );
        check("x.r#type", "x.r#type");
        check("self.x", "self.x");
        check("fut.await", "fut.await");
        check("crate::f x", "crate::f(x)");
    }

    #[test]
    fn rust_isms_get_helpful_errors() {
        for (src, msg) in [
            ("xs.map(|x| x + 1)", "written `fun x -> e`"),
            ("|| 1", "`fun () -> e` for `|| e`"),
            ("f (move |x| x)", "`fun move x -> e`"),
            ("if let Some(x) = o { x } else { 0 }", "Rust's `if let`"),
            ("if let Some(x) = o then x else 0", "Rust's `if let`"),
            (
                "match x { A => 1, B => 2 }",
                "write `match x with p -> e | ..`",
            ),
            (
                "x += 1",
                "write `x <- x + e` (for a `ref`: `r := !r + e`, or `incr r`)",
            ),
            ("x -= 1", "write `x <- x - e`"),
            ("x *= 2", "write `x <- x * e`"),
            ("x %= 2", "write `x <- x mod e`"),
            ("x |= 2", "write `x <- x lor e`"),
            ("x <<= 1", "write `x <- x lsl e`"),
            ("{ a; b }", "write `begin .. end` or `( .. )`"),
            ("{ f x }", "write `begin .. end` or `( .. )`"),
            (
                "{ x = 1; y = 2 }",
                "record expressions need the struct name",
            ),
            (
                "[1, 2]",
                "list items are separated by `;`: write `[1; 2]`, or `[(1, 2)]`",
            ),
            ("[|1, 2|]", "array items are separated by `;`"),
            (
                "s.[0]",
                "strings are not indexed by position in Rust: write `s.as_bytes()[i]`",
            ),
            (
                "match l with [a, b] -> a | _ -> 0",
                "list items are separated by `;`: write `[1; 2]`, or `[(1, 2)]`",
            ),
            (
                "match l with [|a, b|] -> a | _ -> 0",
                "array items are separated by `;`",
            ),
            ("let x = 1; x + 1", "write `let x = e in ..`"),
            ("f (let x = 1; x)", "write `let x = e in ..`"),
            ("let x = 1; let y = 2; x + y", "write `let x = e in ..`"),
        ] {
            check_error(src, msg);
        }
        // The OCaml these resemble is untouched.
        check(
            "if let x = 1 in x > 0 then 1 else 0",
            "if { let x = 1; x > 0 } { 1 } else { 0 }",
        );
        check(
            "match { p with x = 1 } with P { x } -> x",
            "match { let mut record = p; record.x = 1; record } { P { x, .. } => x, }",
        );
        check(
            "let ( += ) a b = a + b in x += 1",
            "{ let mut operator1 = |a, b| a + b; operator1(x, 1) }",
        );
        check("[(1, 2)]", "::std::vec![(1, 2)]");
        check(
            "match l with [(a, b)] -> a | _ -> 0",
            "match ::rsml::__rt::as_slice(&l) { [(a, b)] => a, _ => 0, }",
        );
        check("let () = a; b", "{ let () = { a; b }; () }");
        check(
            "let log msg = print msg; flush () ;; log 1",
            "{ let log = |msg| { print(msg); flush() }; log(1) }",
        );
        check(
            "let x = (a; b) let y = 1 ;; y",
            "{ let x = { a; b }; let y = 1; y }",
        );
        // Under rust-analyzer the code still parses as before, with the error last.
        for src in [
            "[1, 2]",
            "let x = 1; x + 1",
            "match l with [a, b] -> a | _ -> 0",
        ] {
            let out = expand_editor(src);
            assert!(out.contains("compile_error"), "\n  ml: {}", src);
            assert!(!out.starts_with("::core::compile_error"), "\n  ml: {}", src);
        }
    }

    #[test]
    fn assignments_clippy_would_rewrite_are_allowed() {
        // OCaml has no `+=`, so `x = x + e` is not linted.
        let allowed = |place: &str, value: &str| {
            format!(
                "{{ #[allow(clippy::assign_op_pattern)] {{ {} = {}; }} }}",
                place, value
            )
        };
        check("x <- x + 1", &allowed("x", "x + 1"));
        check("x <- 1 + x", &allowed("x", "1 + x"));
        check("p.x <- p.x * 2", &allowed("p.x", "p.x * 2"));
        check("a.(i) <- a.(i) lsl 1", &allowed("a[i]", "a[i] << 1"));
        check(
            "let r = ref 0 in r := !r + 1; !r",
            &format!("{{ let mut r = 0; {}; r }}", allowed("r", "r + 1")),
        );
        check(
            "let p = &mut n in p := !p + 1",
            &format!("{{ let p = &mut n; {} }}", allowed("*p", "(*p) + 1")),
        );
        // Nothing to rewrite: kept as it is.
        check("x <- 1 - x", "x = 1 - x");
        check("x <- y + 1", "x = y + 1");
        check("x <- x + 1 + 2", "x = (x + 1) + 2");
        check("x <- x = 1", "x = x == 1");
        check(
            "let ( +! ) a b = a + b in x <- x +! 1",
            "{ let mut operator1 = |a, b| a + b; x = operator1(x, 1) }",
        );
    }

    #[test]
    fn refs() {
        check(
            "let r = ref 0 in r := 10 - !r; !r",
            "{ let mut r = 0; r = 10 - r; r }",
        );
        check(
            "let r = ref 0 in incr r; !r",
            "{ let mut r = 0; r += 1; r }",
        );
    }

    #[test]
    fn toplevel_phrases() {
        check(
            "let x = 1 let y = 2 ;; x + y",
            "{ let x = 1; let y = 2; x + y }",
        );
    }

    #[test]
    fn open_is_a_glob_import() {
        check("open std::cmp ;; max a b", "{ use std::cmp::*; max(a, b) }");
        check(
            "let open std::cmp in max a b",
            "{ use std::cmp::*; max(a, b) }",
        );
    }

    #[test]
    fn operators_lowered_to_calls_get_a_marker() {
        // One `+` per character, so editors color `|>` like `+`.
        assert_eq!(
            expand_raw("x |> f"),
            norm("{ const _: usize = 1 + 1 + 1; f(x) }")
        );
        assert_eq!(
            expand_raw("1 :: [2] @ xs"),
            norm("{ const _: usize = 1 + 1 + 1 + 1; ::rsml::__rt::append(::std::vec![1, 2], xs) }"),
        );
        // `.` of `+.` is dropped; `+` itself is in the expansion.
        assert_eq!(
            expand_raw("a +. b"),
            norm("{ const _: usize = 1 + 1; a + b }")
        );
        // Operators that stay operators need nothing.
        assert_eq!(expand_raw("a + b * c"), norm("a + (b * c)"));
        assert_eq!(expand_raw("a <= b && c"), norm("(a <= b) && c"));
    }

    #[test]
    fn keywords_without_rust_tokens_get_a_marker() {
        // `then` is colored like the `if` that stands for it, and so on.
        assert_eq!(
            expand_raw("if a then b else c"),
            norm("{ const _: () = { if true {} }; if a { b } else { c } }")
        );
        assert_eq!(
            expand_raw("fun x -> x"),
            norm("{ const _: () = { let _ = 0; match () { _ => {} } }; move |x| x }")
        );
        assert_eq!(
            expand_raw("match o with | A -> 1 | B when c -> 2"),
            norm(
                "{ const _: () = { match () { _ => {} } let _ = 0 | 0; let _ = 0 | 0; if true {} }; \
                 match o { A => 1, B if c => 2, } }"
            )
        );
        assert_eq!(
            expand_raw("for i = 1 to n do f i done"),
            norm("{ const _: () = { loop { break } loop { break } }; for i in 1..=n { f(i); } }")
        );
        // A block takes the markers as its first items.
        assert_eq!(
            expand_raw("begin f x end; g y"),
            norm("{ const _: () = { let _ = 0; let _ = 0; }; f(x); g(y) }")
        );
        // Keywords Rust has too need nothing.
        assert_eq!(expand_raw("let x = 1 in x"), norm("{ let x = 1; x }"));
        assert_eq!(
            expand_raw("while c do () done |> ignore"),
            norm(
                "{ const _: usize = 1 + 1 + 1; const _: () = { loop { break } }; ignore(while c { (); }) }"
            )
        );
    }

    #[test]
    fn errors_are_compile_errors() {
        assert!(expand("let rec f x = x in f").contains("compile_error"));
        assert!(expand("try x with _ -> y").contains("compile_error"));
        assert!(expand("1 +").contains("compile_error"));
        assert!(expand("_").contains("compile_error"));
    }

    /// The expansion rust-analyzer sees.
    fn expand_editor(src: &str) -> String {
        let ts: TokenStream = src.parse().unwrap();
        crate::codegen::expand_for(ts, &[], &[], true)
            .to_string()
            .split_whitespace()
            .collect()
    }

    /// rust-analyzer completes at the cursor (`$`) by expanding the code as
    /// written and with an identifier at the cursor; the two must agree up
    /// to that identifier. Returns the expansion as written.
    #[track_caller]
    fn check_completes(src: &str) -> String {
        let written = expand_editor(&src.replace('$', ""));
        let filled = expand_editor(&src.replace('$', crate::parser::MISSING));
        let cut = filled.find(crate::parser::MISSING).unwrap();
        assert_eq!(written.get(..cut), Some(&filled[..cut]), "\n  ml: {}", src);
        written
    }

    /// Like [`check_completes`] for code that is unfinished as written.
    #[track_caller]
    fn check_recovers(src: &str) {
        let written = check_completes(src);
        assert!(written.contains("compile_error"), "\n  ml: {}", src);
    }

    /// The tokens of an expansion as rust-analyzer lays them out, each with
    /// its offset: end to end, with a space only after a punctuation
    /// character that is alone and followed by another (rust-analyzer's
    /// `syntax-bridge`, `TtTreeSink::token`).
    fn ra_layout(ts: TokenStream) -> Vec<(usize, String)> {
        use proc_macro2::{Delimiter, Spacing, TokenTree};
        fn walk(ts: TokenStream, out: &mut Vec<(usize, String)>, pos: &mut usize) {
            let tts: Vec<TokenTree> = ts.into_iter().collect();
            for (i, tt) in tts.iter().enumerate() {
                let mut push = |text: String| {
                    let n = text.len();
                    out.push((*pos, text));
                    *pos += n;
                };
                match tt {
                    TokenTree::Group(g) => {
                        let (open, close) = match g.delimiter() {
                            Delimiter::Parenthesis => ("(", ")"),
                            Delimiter::Brace => ("{", "}"),
                            Delimiter::Bracket => ("[", "]"),
                            Delimiter::None => ("", ""),
                        };
                        if !open.is_empty() {
                            push(open.into());
                        }
                        walk(g.stream(), out, pos);
                        if !close.is_empty() {
                            out.push((*pos, close.into()));
                            *pos += 1;
                        }
                    }
                    TokenTree::Punct(p) => {
                        push(p.as_char().to_string());
                        let spaced = p.spacing() == Spacing::Alone
                            && p.as_char() != ';'
                            && matches!(tts.get(i + 1),
                                Some(TokenTree::Punct(q)) if q.as_char() != '\'');
                        if spaced {
                            *pos += 1;
                        }
                    }
                    TokenTree::Ident(id) => push(id.to_string()),
                    TokenTree::Literal(l) => push(l.to_string()),
                }
            }
        }
        let mut out = Vec::new();
        walk(ts, &mut out, &mut 0);
        out
    }

    /// rust-analyzer completes at the cursor (`$`) in the expansion of the
    /// code with an identifier there, and takes the scope and the expected
    /// type from the token at that identifier's offset in the expansion of
    /// the code as written (`crates/ide-completion/src/context/analysis.rs`,
    /// `expand` and `analyze`). Returns that token (left-biased: the one
    /// that ends at the offset or contains it).
    #[track_caller]
    fn token_at_marker(src: &str, items: bool) -> String {
        let expand = |src: String| {
            let ts: TokenStream = src.parse().unwrap();
            crate::codegen::force_editor_mode(Some(true));
            let out = if items {
                crate::expand_items(ts)
            } else {
                crate::expand(ts)
            };
            crate::codegen::force_editor_mode(None);
            ra_layout(out)
        };
        let marker = "intellijRulezz";
        let written = expand(src.replace('$', ""));
        let filled = expand(src.replace('$', marker));
        let at = filled.iter().find(|(_, t)| t == marker).unwrap().0;
        let hit = written.iter().find(|(s, t)| *s < at && at <= s + t.len());
        hit.unwrap_or_else(|| panic!("\n  ml: {}\n  offset {} is past the end", src, at))
            .1
            .clone()
    }

    /// The marker of a first argument with nothing after it falls on the
    /// closure parameter that stands for that argument, outside the
    /// closure's scope; rust-analyzer then offers the names in scope there
    /// and ranks values by that parameter's type (see `Cx::eta`).
    #[test]
    fn editor_partial_applications_line_up() {
        let ml = [
            ("let plus x y = x + y in plus $", "__rsml_p1"),
            ("let plus x y = x + y in plus $; n", "__rsml_p1"),
            ("let plus x y = x + y in twice (plus $) 1", "__rsml_p1"),
            ("let f a b c = a + b + c in f $", "__rsml_p1"),
            ("let f a b c = a + b + c in f n $", "__rsml_p2"),
            ("let f a b c = a + b + c in f 1 $", "__rsml_p1"),
            ("let f a b c = a + b + c in f (g n) $", "__rsml_p2"),
            ("let f a b c = a + b + c in f (fun x -> x) $", "__rsml_p1"),
            // A function of `()`: on its `||`.
            ("let f () = 1 in f $", "|"),
            ("add _ $", "__rsml_p1"),
            ("add _ n $", "__rsml_p2"),
        ];
        for (src, want) in ml {
            assert_eq!(token_at_marker(src, false), want, "\n  ml: {}", src);
        }
        let items = [
            "fn add2(a: i64, b: i64) -> i64 { a + b } fn f(n: i64) -> i64 { add2 $ }",
            "struct Q { x: i64 } impl Q { \
             fn make(a: i64, b: i64) -> Q { Q { x = a + b } } fn f(n: i64) -> Q { Self::make $ } }",
            "fn apply(g: impl Fn(i64, i64) -> i64, n: i64) -> i64 { g $ }",
        ];
        for src in items {
            assert_eq!(token_at_marker(src, true), "__rsml_p1", "\n  rsml: {}", src);
        }
    }

    #[test]
    fn editors_get_unfinished_code_expanded() {
        check_recovers("xs.$");
        check_recovers("xs |> .$");
        check_recovers("xs |> .iter |> .$ |> .sum");
        check_recovers("xs |> $");
        check_recovers("let square x = x * x in $");
        check_recovers("let y = $ in y + 1");
        check_recovers("Vec::$");
        check_recovers("Vec.$");
        check_recovers("::std::$");
        check_recovers("f (fun (x : &str) -> x.$) xs");
        check_recovers("match o with Some x -> x.$ | None -> 0");
        check_recovers("let n = xs.$\nlet m = 2 ;; m");
        check_recovers("match o with Option::$");
        check_recovers("match o with Some x -> x | Option.$");
        check_recovers("match o with Some x -> x | $");
        check_recovers("match ord with std::cmp::Ordering::$");
        // The name is written right after `.` and `::`, so what follows a
        // space is the receiver or argument.
        check_recovers(".$ xs");
        check_recovers(".$ xs 1");
        check_recovers("geometry::$ &shape");
        check_recovers("Vec::<i64>::$");
        check_recovers("n :: $");
        // Keywords still to come.
        check_recovers("if $");
        check_recovers("if n > $");
        check_recovers("begin $");
        check_recovers("while n > 0 do $");
        check_recovers("for i = 0 to n do $");
        check_recovers("fun x -> x + $");
        check_recovers("let open std::$ in max 1 2");
        check_recovers("(n : $)");
        check_recovers("for x in $");
        check_recovers("for x in xs do $");
        check_completes("f($)");
        check_completes("f(1, $)");
        check_completes("xs |> .map($)");
        check_recovers("xs |> .iter() |> .$");
        check_recovers("v[$]");
        check_recovers("0..=$");
        // Finished as written, and still read the same up to the cursor.
        check_completes("[$]");
        check_completes("Point { $ }");
        check_completes("Point { x = 1; $ }");
        check_completes("f (); $");
        check_completes("return $");
        check_completes("0..$");
        check_completes("unsafe { $ }");
        check_completes("xs |> .iter() |> .map(fun x -> x.$)");
        check_recovers("pairs |> .iter() |> .map(.0) |> .$");
        check_recovers("Some(n).$");
        check_recovers("Shape::Rect(1.0, 2.0).$ + 1");
        check_completes(r#"println!("{}", $)"#);
        check_recovers(r#"println!("{}", xs |> .$)"#);
        check_completes(r#"format!("{}", n$)"#);
        check_recovers("vec![n; $]");
        check_recovers(r#"assert!(xs.$, "msg")"#);
        check_recovers("pairs |> .map(.0) |> .$ |> f");
        check_completes("for x in xs do f x; $ done");
        // Finished code expands as usual, in a block.
        for src in [
            "A :: [b]",
            "x :: xs",
            "f t.len (x + 1)",
            "Vec.new ()",
            "xs |> .len",
            "h::t",
        ] {
            let raw = expand_raw(src);
            let block = if raw.starts_with("{const_:usize=") {
                raw
            } else {
                format!("{{{}}}", raw)
            };
            assert_eq!(expand_editor(src), block, "\n  ml: {}", src);
        }
    }

    #[test]
    fn editors_see_trailing_semicolons_as_unfinished_sequences() {
        // `a;` keeps its value for rustc; an editor sees the shape `a; b` will have.
        assert_eq!(expand("f (); "), norm("f()"));
        assert!(expand_editor("f (); ").starts_with("{{f();"));
        assert!(!expand_editor("f (); ").contains("compile_error"));
    }

    #[test]
    fn no_space_after_a_dot() {
        assert!(expand(". iter xs").contains("compile_error"));
        assert!(expand("xs. len").contains("compile_error"));
        check(".iter xs", "xs.iter()");
        check("xs\n  .iter()", "xs.iter()");
    }

    #[test]
    fn lowercase_path_patterns() {
        check(
            "match o with std::cmp::Ordering::Less -> 1 | _ -> 0",
            "match o { std::cmp::Ordering::Less => 1, _ => 0, }",
        );
        check(
            "match n with i32::MAX -> 1 | _ -> 0",
            "match n { i32::MAX => 1, _ => 0, }",
        );
        // Without a capitalized segment it is still a cons.
        check(
            "match l with x::rest -> 1 | [] -> 0",
            "match ::rsml::__rt::as_slice(&l) { [x, rest @ ..] => 1, [] => 0, }",
        );
    }

    #[test]
    fn items_get_ocaml_bodies() {
        check_items(
            "fn f(x: i64) -> i64 { x |> g |> h }",
            "fn f(x: i64) -> i64 { h(g(x)) }",
        );
        check_items(
            "pub(crate) async fn f() -> i64 { let x = 1 in x }",
            "pub(crate) async fn f() -> i64 { let x = 1; x }",
        );
        check_items(
            "#[inline] fn a() -> i64 { 1 } fn b() -> i64 { a () }",
            "#[inline] fn a() -> i64 { 1 } fn b() -> i64 { a() }",
        );
        // Methods, default trait methods and modules; declarations and types pass through.
        check_items(
            "impl S { fn get(&self) -> i64 { self.x |> g } }",
            "impl S { fn get(&self) -> i64 { g(self.x) } }",
        );
        check_items(
            "trait T { fn a(&self) -> i64; fn b(&self) -> i64 { self.a() + 1 } }",
            "trait T { fn a(&self) -> i64; fn b(&self) -> i64 { self.a() + 1 } }",
        );
        check_items(
            "mod m { pub fn f() -> i64 { 1 + 1 } }",
            "mod m { pub fn f() -> i64 { 1 + 1 } }",
        );
        check_items(
            "struct S { f: fn(i64) -> i64 } const N: S = S { f: g };",
            "struct S { f: fn(i64) -> i64 } const N: S = S { f: g };",
        );
        check_items(
            "fn f() { m! { fn g() { a |> b } } }",
            "fn f() { m! { fn g() { a |> b } } }",
        );
    }

    #[test]
    fn sibling_functions_have_known_arity() {
        let add = "fn add(a: i64, b: i64) -> i64 { a + b } ";
        check_items(
            &format!("{}fn f() -> impl Fn(i64) -> i64 {{ add 1 }}", add),
            &format!(
                "{}fn f() -> impl Fn(i64) -> i64 {{ move |arg2| add(1, arg2) }}",
                add
            ),
        );
        check_items(
            &format!(
                "{}fn f(xs: Vec<i64>) -> Vec<i64> {{ xs |> .into_iter() |> .map(add 1) |> .collect() }}",
                add
            ),
            &format!(
                "{}fn f(xs: Vec<i64>) -> Vec<i64> {{ xs.into_iter().map(move |arg2| add(1, arg2)).collect() }}",
                add
            ),
        );
        check_items(
            &format!("{}fn f() -> i64 {{ add 1 2 + add(1, 2) }}", add),
            &format!("{}fn f() -> i64 {{ add(1, 2) + add(1, 2) }}", add),
        );
        // Zero parameters, a tuple pattern, returned closures.
        check_items(
            "fn now() -> i64 { 1 } fn pair((a, b): (i64, i64)) -> i64 { a + b } \
             fn adder(n: i64) -> impl Fn(i64) -> i64 { fun m -> n + m } \
             fn boxed() -> Box<dyn Fn(i64, i64) -> i64 + Send> { Box::new (fun a b -> a + b) } \
             fn by_ref<'a>(f: &'a dyn Fn(i64) -> i64) -> &'a dyn Fn(i64) -> i64 { f } \
             fn g<F>(f: F) -> F where F: Fn(i64) -> i64 { f } \
             fn f() -> i64 { now () + pair (1, 2) + adder 1 2 + boxed () 1 2 + g (fun x -> x) 1 + (adder 1 |> by_ref) 2 }",
            "fn now() -> i64 { 1 } fn pair((a, b): (i64, i64)) -> i64 { a + b } \
             fn adder(n: i64) -> impl Fn(i64) -> i64 { move |m| n + m } \
             fn boxed() -> Box<dyn Fn(i64, i64) -> i64 + Send> { Box::new(|a, b| a + b) } \
             fn by_ref<'a>(f: &'a dyn Fn(i64) -> i64) -> &'a dyn Fn(i64) -> i64 { f } \
             fn g<F>(f: F) -> F where F: Fn(i64) -> i64 { f } \
             fn f() -> i64 { ((((now() + pair((1, 2))) + adder(1)(2)) + boxed()(1, 2)) + g(|x| x)(1)) + by_ref(adder(1))(2) }",
        );
        // An `async fn` returns a future, whatever its output type says.
        check_items(
            "async fn later(n: i64, m: i64) -> impl Fn(i64) -> i64 { fun k -> k } fn f() { later 1 }",
            "async fn later(n: i64, m: i64) -> impl Fn(i64) -> i64 { move |k| k } fn f() { move |arg2| later(1, arg2) }",
        );
        // Parameters and locals shadow them.
        check_items(
            &format!("{}fn f(add: impl Fn(i64) -> i64) -> i64 {{ add 1 }}", add),
            &format!("{}fn f(add: impl Fn(i64) -> i64) -> i64 {{ add(1) }}", add),
        );
        check_items(
            &format!(
                "{}fn f(add: i64) -> i64 {{ add }} fn g() -> i64 {{ let add x = x in add 1 }}",
                add
            ),
            &format!(
                "{}fn f(add: i64) -> i64 {{ add }} fn g() -> i64 {{ let add = |x| x; add(1) }}",
                add
            ),
        );
        // Two definitions with different shapes (under `#[cfg]`s): unknown.
        check_items(
            "#[cfg(a)] fn t(a: i64) -> i64 { a } #[cfg(b)] fn t(a: i64, b: i64) -> i64 { a } fn f() -> i64 { t 1 }",
            "#[cfg(a)] fn t(a: i64) -> i64 { a } #[cfg(b)] fn t(a: i64, b: i64) -> i64 { a } fn f() -> i64 { t(1) }",
        );
    }

    #[test]
    fn associated_functions_have_known_arity() {
        let p = "struct P { x: i64 } impl P { fn new(x: i64, y: i64) -> P { P { x = x + y } } \
                 fn get(&self, by: i64) -> i64 { self.x + by } ";
        let pr = "struct P { x: i64 } impl P { fn new(x: i64, y: i64) -> P { P { x: x + y } } \
                  fn get(&self, by: i64) -> i64 { self.x + by } ";
        check_items(
            &format!("{}fn mk() -> impl Fn(i64) -> P {{ Self::new 1 }} }}", p),
            &format!(
                "{}fn mk() -> impl Fn(i64) -> P {{ move |arg2| Self::new(1, arg2) }} }}",
                pr
            ),
        );
        check_items(
            &format!("{}}} fn mk() -> impl Fn(i64) -> P {{ P::new 1 }}", p),
            &format!(
                "{}}} fn mk() -> impl Fn(i64) -> P {{ move |arg2| P::new(1, arg2) }}",
                pr
            ),
        );
        // In a trait impl for `P`, `Self` is `P`; `self` methods stay unknown.
        check_items(
            &format!(
                "{}}} impl Default for P {{ fn default() -> P {{ let f = Self::new 1 in f 2 }} }} fn g(p: P) -> i64 {{ P::get p 1 }}",
                p
            ),
            &format!(
                "{}}} impl Default for P {{ fn default() -> P {{ let f = move |arg2| Self::new(1, arg2); f(2) }} }} fn g(p: P) -> i64 {{ P::get(p, 1) }}",
                pr
            ),
        );
        check_items(
            "trait T { fn make(n: i64, m: i64) -> Self; fn twice() -> impl Fn(i64) -> Self { Self::make 2 } }",
            "trait T { fn make(n: i64, m: i64) -> Self; fn twice() -> impl Fn(i64) -> Self { move |arg2| Self::make(2, arg2) } }",
        );
    }

    #[test]
    fn modules_know_their_own_functions() {
        let add = "fn add(a: i64, b: i64) -> i64 { a + b } ";
        // A nested module sees its own items, not its parent's...
        check_items(
            &format!(
                "{}mod m {{ fn sub(a: i64, b: i64) -> i64 {{ a - b }} fn f() {{ (sub 1, add 1) }} }}",
                add
            ),
            &format!(
                "{}mod m {{ fn sub(a: i64, b: i64) -> i64 {{ a - b }} fn f() {{ (move |arg2| sub(1, arg2), add(1)) }} }}",
                add
            ),
        );
        // ...unless it imports them with `use super::*` or `use super::name`.
        check_items(
            &format!("{}mod tests {{ use super::*; fn f() {{ add 1 }} }}", add),
            &format!(
                "{}mod tests {{ use super::*; fn f() {{ move |arg2| add(1, arg2) }} }}",
                add
            ),
        );
        check_items(
            &format!(
                "{}mod tests {{ use super::{{add as plus}}; fn f() {{ (plus 1, add 1) }} }}",
                add
            ),
            &format!(
                "{}mod tests {{ use super::{{add as plus}}; fn f() {{ (move |arg2| plus(1, arg2), add(1)) }} }}",
                add
            ),
        );
        // An explicit import of another `add` wins over the glob.
        check_items(
            &format!(
                "{}mod tests {{ use super::*; use other::add; fn f() {{ add 1 }} }}",
                add
            ),
            &format!(
                "{}mod tests {{ use super::*; use other::add; fn f() {{ add(1) }} }}",
                add
            ),
        );
    }

    #[test]
    fn rust_attribute_keeps_bodies_rust() {
        check_items(
            "#[rust] fn r(x: i64) -> i64 { let y = x + 1; y * 2 } fn o(x: i64) -> i64 { r x |> r }",
            "fn r(x: i64) -> i64 { let y = x + 1; y * 2 } fn o(x: i64) -> i64 { r(r(x)) }",
        );
        check_items(
            "#[inline] #[rust] pub(crate) fn r() -> impl Fn(i64) -> i64 { |x| x + 1 }",
            "#[inline] pub(crate) fn r() -> impl Fn(i64) -> i64 { |x| x + 1 }",
        );
        check_items(
            "#[rust] impl P { fn a(&self) -> i64 { if let Some(x) = self.o { x } else { 0 } } } \
             #[rust] mod m { pub fn b() -> i64 { let x = 1; x + 1 } } \
             #[rust] unsafe trait T { fn d(&self) -> i64 { 1 } }",
            "impl P { fn a(&self) -> i64 { if let Some(x) = self.o { x } else { 0 } } } \
             mod m { pub fn b() -> i64 { let x = 1; x + 1 } } \
             unsafe trait T { fn d(&self) -> i64 { 1 } }",
        );
        // Signatures under `#[rust]` are still known.
        check_items(
            "impl Q { #[rust] fn e(a: i64, b: i64) -> i64 { a } fn f() -> impl Fn(i64) -> i64 { Self::e 1 } }",
            "impl Q { fn e(a: i64, b: i64) -> i64 { a } fn f() -> impl Fn(i64) -> i64 { move |arg2| Self::e(1, arg2) } }",
        );
        for src in [
            "#[rust] struct S;",
            "#[rust] const C: i64 = 1;",
            "#[rust] use std::fmt;",
        ] {
            let out = expand_items(src);
            assert!(
                out.starts_with("::core::compile_error!") && out.contains("`#[rust]`goeson"),
                "{}",
                out
            );
        }
        // Formatting leaves them alone too.
        let bodies = crate::items::fn_bodies("#[rust] fn a() { 1 } fn b() { 2 }".parse().unwrap());
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].body.to_string(), "{ 2 }");
    }

    #[test]
    fn item_parameters_are_in_scope() {
        // A parameter is a bound name: `h::t` is a cons, not a path.
        check_items(
            "fn f(h: i64, t: Vec<i64>) -> Vec<i64> { h::t }",
            "fn f(h: i64, t: Vec<i64>) -> Vec<i64> { ::rsml::__rt::cons(h, t) }",
        );
        // Closure-typed parameters have a known arity.
        check_items(
            "fn f(add: impl Fn(i64, i64) -> i64) -> impl Fn(i64) -> i64 { add 1 }",
            "fn f(add: impl Fn(i64, i64) -> i64) -> impl Fn(i64) -> i64 { move |arg2| add(1, arg2) }",
        );
        check_items(
            "fn f<'a, F: Clone + FnMut(i64, &'a str) -> i64, const N: usize>(mut g: F) -> i64 { g 1 }",
            "fn f<'a, F: Clone + FnMut(i64, &'a str) -> i64, const N: usize>(mut g: F) -> i64 { move |arg2| g(1, arg2) }",
        );
        check_items(
            "fn f<F>(g: &dyn Fn(i64, i64) -> i64, h: F) -> i64 where F: Fn() -> i64 { g (h ()) }",
            "fn f<F>(g: &dyn Fn(i64, i64) -> i64, h: F) -> i64 where F: Fn() -> i64 { let value1 = h(); move |arg2| g(value1, arg2) }",
        );
    }
}

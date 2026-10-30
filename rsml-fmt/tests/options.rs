//! What each option does, on small inputs.

use rsml_fmt::{Config, format_ml, format_source};

#[track_caller]
fn check(overrides: &str, input: &str, expected: &str) {
    let cfg = Config::default().with_overrides(overrides).unwrap();
    let got = format_ml(input, &cfg).unwrap();
    let expected = expected.strip_prefix('\n').unwrap_or(expected);
    assert_eq!(got, expected, "\n[{}] gave:\n{}\n", overrides, got);
}

#[track_caller]
fn check_file(overrides: &str, input: &str, expected: &str) {
    let cfg = Config::default().with_overrides(overrides).unwrap();
    let got = format_source(input, &cfg).unwrap();
    assert!(got.warnings.is_empty(), "{:?}", got.warnings);
    assert_eq!(
        got.text, expected,
        "\n[{}] gave:\n{}\n",
        overrides, got.text
    );
}

const PIPE: &str =
    "xs |> List.map (fun x -> x * 2) |> List.filter (fun x -> x > 10) |> List.fold_left ( + ) 0";

#[test]
fn pipes() {
    check(
        "max_width = 40",
        PIPE,
        "
xs
|> List.map (fun x -> x * 2)
|> List.filter (fun x -> x > 10)
|> List.fold_left ( + ) 0",
    );
    check(
        "max_width = 40; pipe.indent = align",
        PIPE,
        "
xs |> List.map (fun x -> x * 2)
   |> List.filter (fun x -> x > 10)
   |> List.fold_left ( + ) 0",
    );
    check(
        "max_width = 40; pipe.indent = indent",
        PIPE,
        "
xs
  |> List.map (fun x -> x * 2)
  |> List.filter (fun x -> x > 10)
  |> List.fold_left ( + ) 0",
    );
    check(
        "max_width = 40; pipe.position = trailing; pipe.indent = indent",
        PIPE,
        "
xs |>
  List.map (fun x -> x * 2) |>
  List.filter (fun x -> x > 10) |>
  List.fold_left ( + ) 0",
    );
    check(
        "max_width = 40; pipe.layout = wrap",
        PIPE,
        "
xs |> List.map (fun x -> x * 2)
|> List.filter (fun x -> x > 10)
|> List.fold_left ( + ) 0",
    );
    // Short chains stay on one line unless forced.
    check("", "x |> f |> g", "x |> f |> g");
    check("pipe.force_break_at = 2", "x |> f |> g", "\nx\n|> f\n|> g");
}

#[test]
fn lambdas_keep_the_line_open() {
    check(
        "max_width = 40",
        "xs |> List.iter (fun x -> print_int x; print_newline ())",
        "
xs
|> List.iter (fun x ->
     print_int x;
     print_newline ())",
    );
    check(
        "max_width = 40",
        "List.iter (fun x -> print_int x; print_newline ()) xs",
        "
List.iter
  (fun x ->
    print_int x;
    print_newline ())
  xs",
    );
    check(
        "",
        "let f = fun x -> let y = x in y in f",
        "
let f = fun x ->
  let y = x in
  y
in
f",
    );
    check(
        "let.fun_on_eq_line = false",
        "let f = fun x -> let y = x in y in f",
        "
let f =
  fun x ->
    let y = x in
    y
in
f",
    );
    check(
        "",
        "m >>= fun x -> let y = x + 1 in return y",
        "
m >>= fun x ->
  let y = x + 1 in
  return y",
    );
}

#[test]
fn let_in() {
    check(
        "",
        "let x = 1 in let y = 2 in x + y",
        "\nlet x = 1 in\nlet y = 2 in\nx + y",
    );
    check(
        "let.body = fit",
        "let x = 1 in let y = 2 in x + y",
        "let x = 1 in let y = 2 in x + y",
    );
    let area = "let area s = match s with Circle r -> 3.14 *. r *. r | Rect (w, h) -> w *. h in area shape";
    check(
        "max_width = 50",
        area,
        "
let area s =
  match s with
  | Circle r -> 3.14 *. r *. r
  | Rect (w, h) -> w *. h
in
area shape",
    );
    check(
        "max_width = 50; let.in_position = end_of_line",
        area,
        "
let area s =
  match s with
  | Circle r -> 3.14 *. r *. r
  | Rect (w, h) -> w *. h in
area shape",
    );
    check(
        "",
        "let x = 1 and y = 2 in x + y",
        "\nlet x = 1\nand y = 2 in\nx + y",
    );
}

const MATCH: &str = r#"match n with 0 -> "zero" | n when n < 0 -> "negative" | _ -> "positive""#;

#[test]
fn match_cases() {
    check("", MATCH, MATCH);
    check(
        "match.leading_bar = always",
        MATCH,
        r#"match n with | 0 -> "zero" | n when n < 0 -> "negative" | _ -> "positive""#,
    );
    check(
        "match.cases = vertical",
        MATCH,
        r#"
match n with
| 0 -> "zero"
| n when n < 0 -> "negative"
| _ -> "positive""#,
    );
    check(
        "match.cases = vertical; match.align_arrows = true; match.case_indent = 2",
        MATCH,
        r#"
match n with
  | 0            -> "zero"
  | n when n < 0 -> "negative"
  | _            -> "positive""#,
    );
    check(
        "match.cases = vertical; match.body = next_line",
        MATCH,
        r#"
match n with
| 0 ->
    "zero"
| n when n < 0 ->
    "negative"
| _ ->
    "positive""#,
    );
}

const IF: &str = "if x > 0 then compute_something x else compute_something_else x";

#[test]
fn if_styles() {
    check(
        "max_width = 40",
        IF,
        "\nif x > 0 then\n  compute_something x\nelse\n  compute_something_else x",
    );
    check(
        "max_width = 40; if.style = compact",
        IF,
        "\nif x > 0 then compute_something x\nelse compute_something_else x",
    );
    check(
        "max_width = 40; if.style = keyword_first",
        IF,
        "\nif x > 0\nthen compute_something x\nelse compute_something_else x",
    );
    check(
        "if.style = vertical",
        "if a then b else c",
        "\nif a then\n  b\nelse\n  c",
    );
    check(
        "",
        "if a then b else if c then d else e",
        "if a then b else if c then d else e",
    );
}

#[test]
fn placeholders_and_rust_names() {
    check(
        "",
        "xs |> .iter |> .map _ (_ * 2)",
        "xs |> .iter |> .map _ (_ * 2)",
    );
    check("", ".push xs 1", ".push xs 1");
    check("", "f () (())", "f () (())");
    // A `.` after an operator character would lex as part of it.
    check("", "- .len xs", "- .len xs");
    check("", "g ~n: .len", "g ~n: .len");
    check("", "[| .len xs |]", "[| .len xs |]");
    check("", "f _1 _0 (_ : &str)", "f _1 _0 (_ : &str)");
    check(
        "",
        "open std::cmp ;; max 1 2",
        "\nopen std::cmp ;;\nmax 1 2",
    );
    check(
        "colon_spacing = after",
        "let rec go (l : Vec<'a>) (m : Option<(i64, &'b str)>) : Vec<'a> = l in go",
        "\nlet rec go (l: Vec<'a>) (m: Option<(i64, &'b str)>): Vec<'a> = l in\ngo",
    );
}

#[test]
fn sequences() {
    let s = r#"print_string "a"; print_string "b""#;
    check("", s, "\nprint_string \"a\";\nprint_string \"b\"");
    check(
        "sequence.style = separator",
        s,
        "\nprint_string \"a\"\n; print_string \"b\"",
    );
    check("sequence.break = fit", s, s);
}

const LIST: &str = "[aaaa; bbbb; cccc; dddd; eeee; ffff]";

#[test]
fn collections() {
    check(
        "max_width = 20",
        LIST,
        "\n[aaaa;\n bbbb;\n cccc;\n dddd;\n eeee;\n ffff]",
    );
    check(
        "max_width = 20; collections.style = block; collections.trailing_semicolon = vertical",
        LIST,
        "\n[\n  aaaa;\n  bbbb;\n  cccc;\n  dddd;\n  eeee;\n  ffff;\n]",
    );
    check(
        "max_width = 20; collections.separator_position = before; collections.space_inside_lists = true",
        LIST,
        "\n[ aaaa\n; bbbb\n; cccc\n; dddd\n; eeee\n; ffff ]",
    );
    check(
        "max_width = 25; collections.layout = wrap",
        LIST,
        "\n[aaaa; bbbb; cccc; dddd;\n eeee; ffff]",
    );
    check(
        "collections.space_inside_lists = true",
        "[1; 2]",
        "[ 1; 2 ]",
    );
    check(
        "collections.space_inside_arrays = true",
        "[|1; 2|]",
        "[| 1; 2 |]",
    );
    // `[|-1|]` would lex as `[` `|-` ..., so a space is kept.
    check("", "[| -1; 2 |]", "[| -1; 2 |]");
    check(
        "collections.space_inside_records = false",
        "P { x = 1 }",
        "P {x = 1}",
    );
}

#[test]
fn applications() {
    let a = "f first_arg second_arg third_arg fourth_arg";
    check(
        "max_width = 30",
        a,
        "\nf first_arg second_arg\n  third_arg fourth_arg",
    );
    check(
        "max_width = 30; app.args = fit_or_vertical; app.indent = 4",
        a,
        "\nf\n    first_arg\n    second_arg\n    third_arg\n    fourth_arg",
    );
    check(
        "max_width = 30",
        "xs.iter().map(fun x -> x + 1).collect::<Vec<_>>()",
        "\nxs.iter()\n  .map(fun x -> x + 1)\n  .collect::<Vec<_>>()",
    );
}

#[test]
fn spacing() {
    check(
        "",
        "List.fold_left (+) 0 (List.map (( * ) 2) xs)",
        "List.fold_left ( + ) 0 (List.map (( * ) 2) xs)",
    );
    check(
        "op_section_spaces = minimal",
        "List.fold_left ( + ) 0 (List.map (( * ) 2) xs)",
        "List.fold_left (+) 0 (List.map (( * ) 2) xs)",
    );
    check(
        "colon_spacing = after; let.body = fit",
        "let f (x : int) : int = x in (f 1 : int)",
        "let f (x: int): int = x in (f 1: int)",
    );
    check("begin_end = parens", "f (begin a; b end)", "\nf (a;\n   b)");
    check("", "- (-x) + -. 1.0", "-(-x) + -. 1.0");
    // A section glued to `|>`, `>>`, `<<` or `@@` (one operator `|>.` to
    // Rust) gets its space, unless the operator is defined.
    check(
        "",
        "xs |>.iter() |>.map _ (.0) |>.sum",
        "xs |> .iter() |> .map _ (.0) |> .sum",
    );
    check("", "t |>.0 |>.1", "t |> .0 |> .1");
    check(
        "",
        "f @@.len (.iter >>.count <<.g)",
        "f @@ .len (.iter >> .count << .g)",
    );
    check(
        "let.body = fit",
        "let ( |>. ) a b = a + b in 1 |>.2",
        "let ( |>. ) a b = a + b in 1 |>. 2",
    );
    check("", "x +.y *.2.", "x +. y *. 2.");
}

const PHRASES: &str =
    "external add : int -> int -> int ;; let one = 1 let two = add one one ;; two";

#[test]
fn toplevel() {
    check(
        "",
        PHRASES,
        "\nexternal add : int -> int -> int\nlet one = 1\nlet two = add one one ;;\ntwo",
    );
    check(
        "toplevel.double_semi = always",
        PHRASES,
        "\nexternal add : int -> int -> int ;;\nlet one = 1 ;;\nlet two = add one one ;;\ntwo",
    );
    check(
        "toplevel.double_semi_position = own_line",
        PHRASES,
        "\nexternal add : int -> int -> int\nlet one = 1\nlet two = add one one\n;;\ntwo",
    );
    check(
        "toplevel.blank_lines = always",
        "let a = 1 let b = 2",
        "\nlet a = 1\n\nlet b = 2",
    );
    check(
        "toplevel.break = fit",
        "external add : int -> int -> int ;; add 5",
        "external add : int -> int -> int ;; add 5",
    );
}

#[test]
fn comments_and_blank_lines() {
    check(
        "",
        "let x = 1 in (* why *)\n\n\n(* own line *)\nlet y = x (* trailing *) in\n// line comment\nx + y",
        "
let x = 1 in (* why *)

(* own line *)
let y = x (* trailing *) in
// line comment
x + y",
    );
    check(
        "preserve_blank_lines = false",
        "let x = 1 in\n\nx",
        "\nlet x = 1 in\nx",
    );
    check("max_blank_lines = 2", "a;\n\n\n\nb", "\na;\n\n\nb");
    check("", "f (* c *) x", "f (* c *) x");
}

#[test]
fn blocks_in_rust_files() {
    let src =
        "fn f() {\n    let a = ml! {   x |> f  };\n    let b = ml! {\n        x |> f\n    };\n}\n";
    check_file(
        "",
        src,
        "fn f() {\n    let a = ml! { x |> f };\n    let b = ml! { x |> f };\n}\n",
    );
    check_file(
        "single_line_blocks = preserve",
        src,
        "fn f() {\n    let a = ml! { x |> f };\n    let b = ml! {\n        x |> f\n    };\n}\n",
    );
    check_file(
        "single_line_blocks = never; block_indent = 2",
        "fn f() {\n    ml!(x |> f)\n}\n",
        "fn f() {\n    ml!(\n      x |> f\n    )\n}\n",
    );
    // Too long for one line once the rest of the Rust line is counted.
    check_file(
        "max_width = 40",
        "fn f() {\n    assert_eq!(ml! { xs |> List.map g |> h }, 3);\n}\n",
        "fn f() {\n    assert_eq!(ml! {\n        xs |> List.map g |> h\n    }, 3);\n}\n",
    );
    // Breaking the first block moves the second one left, where it fits on
    // one line: the second is laid out at its new column, in the same run.
    check_file(
        "max_width = 40; block_indent = 2",
        "fn f() {\n    assert_eq!(ml! { 1 :: [2; 3] }, ml! { [1; 2; 3] });\n}\n",
        "fn f() {\n    assert_eq!(ml! {\n      1 :: [2; 3]\n    }, ml! { [1; 2; 3] });\n}\n",
    );
}

#[test]
fn rsml_function_bodies() {
    let src = "rsml! {\n    fn f(x: i64) -> i64 { x |> g }\n\n    impl S {\n        fn m(&self) -> i64 { let y = self.x in y }\n    }\n}\n";
    check_file(
        "",
        src,
        "rsml! {\n    fn f(x: i64) -> i64 {\n        x |> g\n    }\n\n    impl S {\n        fn m(&self) -> i64 {\n            let y = self.x in\n            y\n        }\n    }\n}\n",
    );
    check_file(
        "fn_single_line = true",
        "rsml! {\n    fn f(x: i64) -> i64 {\n        x |> g\n    }\n}\n",
        "rsml! {\n    fn f(x: i64) -> i64 { x |> g }\n}\n",
    );
}

#[test]
fn rust_bodies_are_left_alone() {
    // Under `#[rust]` the bodies are Rust (and not OCaml): rustfmt's, or
    // kept as written with `rust_items = "preserve"`.
    let rust = "    #[rust]\n    fn r(x: i64) -> i64 {   let y = x+1;  if let Some(z) = f(y) { z } else { 0 } }\n\n    #[rust]\n    impl S {\n        fn m(&self) -> i64 {  self.x+1 }\n    }\n";
    check_file(
        "rust_items = preserve",
        &format!(
            "rsml! {{\n{}\n    fn o(x: i64) -> i64 {{ x |> r }}\n}}\n",
            rust
        ),
        &format!(
            "rsml! {{\n{}\n    fn o(x: i64) -> i64 {{\n        x |> r\n    }}\n}}\n",
            rust
        ),
    );
    check_file(
        "",
        &format!(
            "rsml! {{\n{}\n    fn o(x: i64) -> i64 {{ x |> r }}\n}}\n",
            rust
        ),
        "rsml! {
    #[rust]
    fn r(x: i64) -> i64 {
        let y = x + 1;
        if let Some(z) = f(y) { z } else { 0 }
    }

    #[rust]
    impl S {
        fn m(&self) -> i64 {
            self.x + 1
        }
    }

    fn o(x: i64) -> i64 {
        x |> r
    }
}
",
    );
}

#[test]
fn rust_items_by_rustfmt() {
    // A whole file: flush left, comments and blank lines kept.
    let messy = "//! File docs.\nrsml::rsml! {\n\nuse std::fmt;\nuse std::collections::HashMap;\n\n/// A point.\npub struct   Point{ x:i64,y :i64 }   // trailing\n\n// before add\npub fn   add(a:i64,b:i64)->i64 { a+b |> fun x -> x }\n\n/* block */\nimpl Point {\n    pub fn norm(&self) -> i64 { self.x |> abs }\n\n\n    fn long_name(first_argument: i64, second_argument: i64, third_argument: i64) -> i64 where i64: Copy { first_argument }\n}\n\n}\n";
    let tidy = "//! File docs.
rsml::rsml! {

use std::collections::HashMap;
use std::fmt;

/// A point.
pub struct Point {
    x: i64,
    y: i64,
} // trailing

// before add
pub fn add(a: i64, b: i64) -> i64 {
    a + b |> fun x -> x
}

/* block */
impl Point {
    pub fn norm(&self) -> i64 {
        self.x |> abs
    }

    fn long_name(first_argument: i64, second_argument: i64, third_argument: i64) -> i64
    where
        i64: Copy,
    {
        first_argument
    }
}

}
";
    check_file("", messy, tidy);
    check_file("", tidy, tidy);
    // Indented one level from the line with `rsml! {`, which rustfmt's
    // width counts.
    check_file(
        "",
        "mod m {\n    rsml! {\n  fn   f(a:i64)->i64 { a }\n      fn g() {}\n    }\n}\n",
        "mod m {\n    rsml! {\n        fn f(a: i64) -> i64 {\n            a\n        }\n        fn g() {}\n    }\n}\n",
    );
    // `rsmlfmt: skip` leaves the whole block alone; so does `preserve`.
    let skipped = "rsml! {\n    // rsmlfmt: skip\n    fn   f(a:i64)->i64 { a  |>  g }\n}\n";
    check_file("", skipped, skipped);
    let kept = "rsml! {\n    fn   f(a:i64)->i64 {\n        a |> g\n    }\n}\n";
    check_file("rust_items = preserve", kept, kept);
}

#[test]
fn directives() {
    let skipped =
        "fn f() {\n    ml! {\n        (* rsmlfmt: skip *)\n        x   |>   f\n    };\n}\n";
    check_file("", skipped, skipped);
    check_file(
        "",
        "fn f() {\n    ml! {\n        (* rsmlfmt: pipe.force_break_at = 1 *)\n        x |> f\n    };\n}\n",
        "fn f() {\n    ml! {\n        (* rsmlfmt: pipe.force_break_at = 1 *)\n        x\n        |> f\n    };\n}\n",
    );
    // At the top of `rsml!`, a directive covers every function.
    check_file(
        "",
        "rsml! {\n    // rsmlfmt: let.body = fit\n    fn f() -> i64 { let x = 1 in x }\n}\n",
        "rsml! {\n    // rsmlfmt: let.body = fit\n    fn f() -> i64 {\n        let x = 1 in x\n    }\n}\n",
    );
    let bad = Config::default();
    let out = format_source("fn f() { ml! { (* rsmlfmt: nope = 1 *) x } }", &bad).unwrap();
    assert_eq!(out.warnings.len(), 1);
}

#[test]
fn rust_style_syntax() {
    // A space before `(` or `[` changes the meaning, so each is kept.
    check("", "f(a, b) (c) + f (a, b)(c)", "f(a, b) (c) + f (a, b)(c)");
    check("", "v[0] + f [0] + f[|0|]", "v[0] + f [0] + f [|0|]");
    check(
        "",
        "xs |> .iter() |> .map(f) |> .sum::<i64>()",
        "xs |> .iter() |> .map(f) |> .sum::<i64>()",
    );
    check("", ".len (s) + .len() s", ".len (s) + .len() s");
    check(
        "max_width = 30",
        "some_function(first_argument, second_argument)",
        "
some_function(
  first_argument,
  second_argument,
)",
    );
    check(
        "max_width = 40",
        "register(fun event -> handle event; log event)",
        "
register(fun event ->
  handle event;
  log event)",
    );
    check(
        "max_width = 40; fun.hug_last_arg = false",
        "register(fun event -> handle event; log event)",
        "
register(
  fun event ->
    handle event;
    log event,
)",
    );
    check(
        "",
        "(0..n, a..=b, a.., ..b, .., xs[1..])",
        "(0..n, a..=b, a.., ..b, .., xs[1..])",
    );
    // Spaces where the characters would run together.
    check(
        "",
        "(0.. -1, x? .., [| ..b |])",
        "(0 .. -1, x? .., [| ..b |])",
    );
    check(
        "",
        "match n with 1..=3 -> a | 'a' .. 'z' -> b",
        "match n with 1..=3 -> a | 'a' .. 'z' -> b",
    );
}

#[test]
fn constructors_keep_their_spelling() {
    // `Some(1, 2)` and `Some (1, 2)` mean the same; each stays as written.
    check(
        "",
        "Some(1, 2) :: Some (1, 2) :: []",
        "Some(1, 2) :: Some (1, 2) :: []",
    );
    check(
        "",
        "Shape::Rect(2.0, 3.0) + Shape.Rect (w, h)",
        "Shape::Rect(2.0, 3.0) + Shape.Rect (w, h)",
    );
    check(
        "",
        "f Some(x) (Some)(y) Ok(()) Some[1; 2] Some [3]",
        "f Some(x) (Some)(y) Ok(()) Some[1; 2] Some [3]",
    );
    check(
        "",
        "match o with Some(x, y) -> 1 | Some (x) -> 2 | Shape::Rect(w, _) -> 3 | Ok(()) -> 4",
        "match o with Some(x, y) -> 1 | Some (x) -> 2 | Shape::Rect(w, _) -> 3 | Ok(()) -> 4",
    );
    check(
        "max_width = 30",
        "let s = Shape::Rect(width_of first, height_of second) in s",
        "
let s =
  Shape::Rect(width_of first,
              height_of second)
in
s",
    );
}

#[test]
fn rust_deref() {
    check(
        "",
        "*self_ + **q + !*p + &*s + *x.len + !r",
        "*self_ + **q + !*p + &*s + *x.len + !r",
    );
    // `(*` would read as an OCaml comment.
    check(
        "",
        "(*x) + f ( *a) + g(*a, b) + v.(*i) + .m(*x) y",
        "( *x) + f ( *a) + g( *a, b) + v.( *i) + .m( *x) y",
    );
    // Where the next character would merge with `*`, `!`, `&` or `:`.
    check(
        "",
        "- *x + [| *x |] + f ~k: !r ~i: *r",
        "- *x + [| *x |] + f ~k: !r ~i: *r",
    );
}

#[test]
fn constructor_atoms_keep_their_reading() {
    // `Some(x).f()` calls the option's `f`, `Some (x).f()` the argument's:
    // formatting must never turn one into the other.
    check(
        "",
        "Some(x).unwrap() + Some (x).unwrap()",
        "Some(x).unwrap() + Some (x).unwrap()",
    );
    check("", "f Some(x).y Some (x).y", "f Some(x).y Some (x).y");
    check(
        "max_width = 30",
        "Shape::Rect(width_of first, height_of second).area()",
        "
Shape::Rect(width_of first,
            height_of second).area()",
    );
    check(
        "max_width = 30",
        "Shape::Rect (width_of first, height_of second).area()",
        "
Shape::Rect
  (width_of first,
   height_of second).area()",
    );
}

#[test]
fn comments_as_rust_comments() {
    let src = "let x = 1 in (* inline *) x + (* own *)\n  y (* trailing *)";
    check(
        "",
        src,
        "let x = 1 in\n(* inline *) x + (* own *) y (* trailing *)",
    );
    // Only a comment with a line to itself becomes `//`, so the layout
    // does not change.
    check(
        "comments = rust",
        src,
        "let x = 1 in\n/* inline */ x + /* own */ y /* trailing */",
    );
    check(
        "comments = rust",
        "f x;\n(* step two *)\ng y",
        "f x;\n// step two\ng y",
    );
    // Several lines, doc-like and unconvertible comments.
    check(
        "comments = rust",
        "(** doc *)\nf (* one\n   two *) x\n; g (*a*/b*) y (*! bang *)",
        "
// * doc
f
  /* one
   two */
  x;
g (*a*/b*) y /* ! bang */",
    );
    // Directives keep working, and the result is stable.
    let file = "fn f() {\n    ml! {\n        (* rsmlfmt: pipe.force_break_at = 1 *)\n        x |> f\n    };\n}\n";
    let rust = "fn f() {\n    ml! {\n        // rsmlfmt: pipe.force_break_at = 1\n        x\n        |> f\n    };\n}\n";
    check_file("comments = rust", file, rust);
    check_file("comments = rust", rust, rust);
}

#[test]
fn trailing_spaces_before_a_comment_line() {
    // Pre-existing: `x + ` kept its space before a comment's own line.
    check(
        "",
        "x +\n  (* own line *)\n  y",
        "\nx +\n    (* own line *)\n    y",
    );
}

#[test]
fn std_macro_arguments() {
    // OCaml arguments are formatted; the format string is kept as written.
    check(
        "",
        "println!(\"{} {x}\",   f   x,x = xs |> .len)",
        "println!(\"{} {x}\", f x, x = xs |> .len)",
    );
    check(
        "",
        "vec![0;n+1] @ vec![ f 1 , g 2 , ]",
        "vec![0; n + 1] @ vec![f 1, g 2]",
    );
    check(
        "max_width = 40",
        "assert_eq!(compute_the_total first second, expected_total, \"totals differ for {}\", name)",
        "
assert_eq!(
  compute_the_total first second,
  expected_total,
  \"totals differ for {}\",
  name,
)",
    );
    check(
        "max_width = 40",
        "assert!(xs |> .iter() |> .all(fun x -> !x > 0), \"positive\")",
        "
assert!(
  xs
  |> .iter()
  |> .all(fun x -> !x > 0),
  \"positive\",
)",
    );
    // Other macros, and `{ .. }` calls, stay as written.
    check(
        "",
        "matches!(x,Some(1)|None) && my_macro!(a   b)",
        "matches!(x,Some(1)|None) && my_macro!(a   b)",
    );
    check("", "println!{\"{}\",   x}", "println!{\"{}\",   x}");
}

#[test]
fn begin_end_hugs_its_keyword() {
    let src = "if c then begin a (); b () end else begin c (); d () end";
    check(
        "max_width = 30",
        src,
        "
if c then begin
  a ();
  b ()
end
else begin
  c ();
  d ()
end",
    );
    check(
        "max_width = 30; begin_end_hug = false",
        src,
        "
if c then
  begin
    a ();
    b ()
  end
else
  begin
    c ();
    d ()
  end",
    );
    // A block that fits stays on the line.
    check(
        "",
        "if c then begin a () end else b",
        "if c then begin a () end else b",
    );
    check(
        "max_width = 20",
        "match x with A -> begin a (); b () end | B -> c",
        "
match x with
| A -> begin
    a ();
    b ()
  end
| B -> c",
    );
    check(
        "max_width = 20",
        "fun x -> begin a x; b x end",
        "
fun x -> begin
  a x;
  b x
end",
    );
    check(
        "max_width = 20",
        "while c do begin a (); b () end done",
        "
while c do begin
  a ();
  b ()
end done",
    );
    check(
        "max_width = 20",
        "if c then unsafe { a (); b () } else async move { c (); d () }",
        "
if c then unsafe {
  a ();
  b ()
}
else async move {
  c ();
  d ()
}",
    );
    // After a comment, the block keeps lines of its own.
    check(
        "max_width = 20",
        "if c then (* why *) begin a (); b () end",
        "
if c then
  (* why *) begin
    a ();
    b ()
  end",
    );
}

#[test]
fn field_sections() {
    check("", "x |> .0", "x |> .0");
    check(
        "",
        "pairs |> .iter() |> .map(.0) |> .sum::<i64>()",
        "pairs |> .iter() |> .map(.0) |> .sum::<i64>()",
    );
    check("", ".1 t + .0.1 u", ".1 t + .0.1 u");
    // Where `!`, `&` or `:` would merge with the `.`.
    check(
        "",
        "f ~k: .0 (! (.1)) [| .0 |] (& .0)",
        "f ~k: .0 (!(.1)) [| .0 |] (& .0)",
    );
}

#[test]
fn rust_style_control() {
    check(
        "",
        "for (k, v) in m do f k v done",
        "for (k, v) in m do f k v done",
    );
    check(
        "indent = 4",
        "for x in xs.iter() do total <- total + !x; count <- count + 1 done",
        "
for x in xs.iter() do
    total <- total + !x;
    count <- count + 1
done",
    );
    check(
        "",
        "if n < 0 then return 0; n",
        "\nif n < 0 then return 0;\nn",
    );
    check(
        "",
        "match x with A -> return 1 | B -> break 2 | C -> continue",
        "match x with A -> return 1 | B -> break 2 | C -> continue",
    );
    check(
        "",
        "while true do step () done",
        "while true do step () done",
    );
    check(
        "",
        "unsafe { f x } + unsafe {}",
        "unsafe { f x } + unsafe {}",
    );
    check(
        "",
        "async move { let body = fetch url in parse body }",
        "
async move {
  let body = fetch url in
  parse body
}",
    );
    check(
        "let.body = fit",
        "async move { let body = fetch url in parse body }",
        "async move { let body = fetch url in parse body }",
    );
}

/// Formats `input`, checks the result, and that formatting it again
/// changes nothing.
#[track_caller]
fn check_stable(overrides: &str, input: &str, expected: &str) {
    check(overrides, input, expected);
    let expected = expected.strip_prefix('\n').unwrap_or(expected);
    check(overrides, expected, expected);
}

#[test]
fn fuzz_glued_tokens() {
    // Two `&`s at the start of an argument stay apart: `f &&xs` is `f && xs`.
    check_stable("", "f & & xs", "f & &xs");
    check_stable("", "map & & (_ * 2) &xs", "map & &(_ * 2) &xs");
    check_stable(
        "",
        "std::mem::swap & & mut a &mut b",
        "std::mem::swap & &mut a &mut b",
    );
    check_stable("", "(.iter >> .count) & & v", "(.iter >> .count) & &v");
    check_stable("", "f (& & x) & *y", "f (&&x) & *y");
    // `!` is never infix, so its runs may stay glued; and where `!` is
    // written, against which side, does not change what it applies to.
    check_stable("", "a! * !b", "a !*!b");
    check_stable("", "incr r! r", "incr r !r");
    check_stable("", "x <- x ! 1", "x <- x !1");
    // The trailing `;` of a record is not part of its expansion (as the
    // trailing `,` of a call is not).
    check_stable("", "Point { x = 1; }", "Point { x = 1 }");
    // `&` or `!` before a path that starts with `::`.
    check_stable("", "Vec & ::new ()", "Vec & ::new ()");
    check_stable("", "f ! ::x", "f ! ::x");
    // A `.` after a number, or after `.0`.
    check_stable("", "1 . 2 + f 10 . (i)", "1 .2 + f 10 .(i)");
    check_stable(
        "",
        "(0. . 4).sum::<i64>() + 0. .len()",
        "(0. .4).sum::<i64>() + 0. .len()",
    );
    check_stable("", ".0 .1 t", ".0 .1 t");
    // Operator characters that would run together: `-::`, `&-`, `:&`;
    // and `t.0.(i)`, where `0.` would be a float.
    check_stable("", "- ::cmp + - !x", "- ::cmp + - !x");
    check_stable(
        "",
        "match n with & -5..=-1 -> 2 | _ -> 0",
        "match n with & -5..=-1 -> 2 | _ -> 0",
    );
    check_stable(
        "",
        "let f ~y: &(y : i64) = y in f",
        "let f ~y: &(y : i64) = y in\nf",
    );
    check_stable("", "t.0 .(1) + t.0 .[1]", "t.0 .(1) + t.0 .[1]");
    // `!1 .0` is `(!1).0`; `!1.0` would dereference the float.
    check_stable("", "!1 .0 + !1. .abs()", "!1 .0 + !1. .abs()");
    // `0...1.` would be `0 ... 1.`: a float ending in `.` keeps its space.
    check_stable("", "0. .. 1. + (0. ..= x)", "0. .. 1. + (0. ..= x)");
    check_stable(
        "",
        "match x with 0. ..= 1. -> a | _ -> b",
        "match x with 0. ..=1. -> a | _ -> b",
    );
    check_stable("", "1.0 .. 2.0", "1.0..2.0");
    // Glued `&`s in types and patterns read as that many references.
    check_stable("", "(x : & & &str)", "(x : &&&str)");
    check_stable("", "match x with & & &a -> a", "match x with &&&a -> a");
}

#[test]
fn fuzz_comments_before_closers() {
    check_stable(
        "",
        "f((* inside *) a, (* after b *)) (* after *)",
        "f((* inside *) a (* after b *)) (* after *)",
    );
    check_stable("", "f((* inside *) )", "f((* inside *))");
    check_stable(
        "",
        "eprintln!(\"{:?}\", (* why *) )",
        "eprintln!(\"{:?}\" (* why *))",
    );
    check_stable("", "xs.map((* c *) f (* d *))", "xs.map((* c *) f (* d *))");
    check_stable(
        "",
        "xs |> .map((* c *) f (* d *))",
        "xs |> .map((* c *) f (* d *))",
    );
    check_stable("", "xs[(* c *) 0 (* d *)]", "xs[(* c *) 0 (* d *)]");
    check_stable(
        "",
        "unsafe { a (); x (* last *) }",
        "
unsafe {
  a ();
  x (* last *)
}",
    );
    check_stable(
        "",
        "unsafe { a ();\n x\n (* last *) }",
        "
unsafe {
  a ();
  x
  (* last *)
}",
    );
    check_stable("", "begin a (* c *) end", "begin a (* c *) end");
    check_stable(
        "",
        "if c then return (* c *); x",
        "if c then return (* c *);\nx",
    );
    // A `//` after a case's `|` ends the case before.
    check_stable(
        "",
        "match x with A -> 1\n| // c\n  B -> 2",
        "
match x with
| A ->
    1 // c
| B -> 2",
    );
    // Before an array's `|]`, after a `//` comment.
    check_stable(
        "",
        "([| f x | // c\n ], [|a|])",
        "([|f x // c\n |],\n [|a|])",
    );
    // A `//` inside a `(* *)` belongs to it.
    check_stable("", "x (* a // b\n   c *)", "x\n(* a // b\n c *)");
    // A comment inside generic arguments is printed once.
    check_stable("", "b.parse::< (* c *) i32>()", "b.parse::< (* c *) i32>()");
    check_stable(
        "",
        "Vec::<i64 (* c *) >::new ()",
        "Vec::<i64 (* c *) >::new ()",
    );
    check_stable(
        "",
        "Vec::<i64 // c\n >::new ()",
        "Vec::<i64 // c\n >::new ()",
    );
    check_stable(
        "",
        "let f (x : (* c *) 'a) = x in f",
        "let f (x : (* c *) 'a) = x in\nf",
    );
}

#[test]
fn fuzz_comments_after_openers() {
    // A comment between the tokens of a path goes before it; an
    // application it heads starts the next line, as it is read back. A `//`
    // comment has a line of its own, wherever the application is.
    check_stable(
        "",
        "Vec:: // c\n <u8>::new () y",
        "// c\nVec::<u8>::new () y",
    );
    check_stable("", "-Vec. // c\n new x", "-\n// c\nVec::new x");
    check_stable("", "return Vec. // c\n new x", "return\n// c\nVec::new x");
    check_stable(
        "",
        "a || -Vec. // c\n new () < 1",
        "a\n|| -\n     // c\n     Vec::new ()\n     < 1",
    );
    check_stable("", "a + Vec. // c\n new x", "a +\n    // c\n    Vec::new x");
    check_stable("", "Vec:: (* c *) <u8>::new", "(* c *) Vec::<u8>::new");
    check_stable(
        "",
        "f (Vec:: (* c *) <u8>::new)",
        "f ((* c *) Vec::<u8>::new)",
    );
    // After `(` and a `//` comment, the contents start where they would
    // after the `(`.
    check_stable("", "f (// c\n x y)", "f (// c\n   x y)");
    check_stable(
        "",
        "f (Vec:: // c\n <u8>::new () y)",
        "f (\n   // c\n   Vec::<u8>::new () y)",
    );
    // A comment before code on its line, after other code, is spaced
    // from both and stays on its line.
    check_stable("", "f (* c *) .0", "f (* c *) .0");
    check_stable("", "let x = 1 in (* c *) x", "let x = 1 in\n(* c *) x");
    // `?` and `|]` are kept apart.
    check_stable("", "[| fun x ->  f x? |]", "[| fun x -> f x? |]");
    // A `//` moved to after a case's body or a list's last item is placed
    // as it is read back: inside them (it breaks what it ends).
    check_stable(
        "",
        "match a with\n| Some x -> begin match b with Some y -> y | None -> x end\n| // c\n  None -> 0",
        "
match a with
| Some x -> begin
    match b with Some y -> y | None -> x
  end // c
| None -> 0",
    );
    check_stable("", "[a; f x y; // c\n ]", "[a;\n f x y // c\n]");
    // In a record, a comment before `}` stays inside.
    check_stable(
        "",
        "Point { x = f a b; // c\n}",
        "Point { x = f a b // c\n}",
    );
    check_stable("", "Point { x = 1 (* c *) }", "Point { x = 1 (* c *) }");
    // In an operator section, inside its parentheses.
    check_stable(
        "",
        "Iterator::fold _ 0 ( + (* c *)  ) xs",
        "Iterator::fold _ 0 ( + (* c *) ) xs",
    );
    check_stable(
        "",
        "xs |> .fold _ 0 (+ // c\n )",
        "xs\n|> .fold _ 0 ( + // c\n      )",
    );
    // In `()`, inside it.
    check_stable("", "Some((* c *)) .map f", "Some((* c *)).map f");
    // With `;` leading, a `//` before or after the `;` ends the item's line.
    check_stable(
        "sequence.style = separator",
        "begin x <- y + 1; // c\n 1 end",
        "\nbegin\n  x <- y + 1 // c\n  ; 1\nend",
    );
    check_stable(
        "sequence.style = separator",
        "begin x <- y + 1 // c\n ; 1 end",
        "\nbegin\n  x <- y + 1 // c\n  ; 1\nend",
    );
    // `begin fun .. end` printed as `(fun ..)` is laid out as that.
    check_stable(
        "begin_end = parens",
        "add3 (begin fun x -> calls <- calls + 1; 10 end)",
        "add3 (fun x ->\n  calls <- calls + 1;\n  10)",
    );
    check_stable(
        "begin_end = parens",
        "f begin // c\n a (); b () end",
        "f (// c\n   a ();\n   b ())",
    );
    check_stable(
        "begin_end = parens",
        "f (begin // c\n a (); g x y end)",
        "f (// c\n   a ();\n   g x y)",
    );
}

#[test]
fn fuzz_toplevel_double_semicolons() {
    // `;;` is needed after an expression that another phrase follows.
    check_stable(
        "",
        "external f : i64 -> i64 ;; f 1 ;; external g : i64 -> i64 ;; g 2",
        "
external f : i64 -> i64 ;;
f 1 ;;
external g : i64 -> i64 ;;
g 2",
    );
    // A spaced `; ;` is not `;;`: rejected, as by the macros.
    let cfg = Config::default();
    assert!(format_ml("let mut n = 0 in n <- n + 1; ;  n", &cfg).is_err());
    assert!(format_ml("incr r; + c", &cfg).is_err());
}

#[test]
fn fuzz_begin_end_as_parens() {
    check_stable(
        "begin_end = parens; max_width = 40",
        "begin fun move x -> return true; for i = 0 to n do step i done end",
        "
(fun move x ->
  return true;
  for i = 0 to n do step i done)",
    );
    check_stable("begin_end = parens", "f begin *x end", "f ( *x)");
}

#[test]
fn raw_identifiers_and_reserved_words() {
    // `r#` is kept; a word Rust reserves stays as written (the macros make
    // it raw).
    check_stable(
        "",
        "let r#type = 1 in r#type+gen",
        "let r#type = 1 in\nr#type + gen",
    );
    check_stable(
        "",
        "match x with Some r#in -> r#in | None->0",
        "match x with Some r#in -> r#in | None -> 0",
    );
    check_stable("", "rng.gen()  + p.r#type", "rng.gen() + p.r#type");
    check_stable("", "try x |> f", "try x |> f");
}

#[test]
fn fuzz_phase5_findings() {
    // A comment after a path's leading `::` goes before the path, at its
    // indentation.
    check_stable(
        "",
        ":: // c\n std::cmp::max 1 2",
        "// c\n::std::cmp::max 1 2",
    );
    check_stable(
        "",
        "f (:: // c\n std::cmp::max 1 2)",
        "f (// c\n   ::std::cmp::max 1 2)",
    );
    // `;;` stays after a `let` that would take the next phrase: `x..` would
    // read `let b = 1` as its upper bound.
    check_stable(
        "",
        "let a = x .. ;;\nlet b = 1 ;;\nb",
        "let a = x.. ;;\nlet b = 1 ;;\nb",
    );
    check_stable(
        "",
        "let a = 1 ;;\nlet b = 2 ;;\nb",
        "let a = 1\nlet b = 2 ;;\nb",
    );
    // A `//` after a `;;` that is not printed ends the phrase before, as it
    // is read back: inside its groups.
    check_stable(
        "",
        "let it = f () ;; // c\n\nopen std::cmp ;;\ngo 1",
        "let it =\n  f () // c\n\nopen std::cmp ;;\ngo 1",
    );
}

#[test]
fn mutable_local_functions() {
    check_stable(
        "",
        "let mut next ()=incr c; !c in next ()",
        "let mut next () =\n  incr c;\n  !c\nin\nnext ()",
    );
    check_stable(
        "let.body = fit",
        "let mut push (x : i64)=v.push(x) in push 1",
        "let mut push (x : i64) = v.push(x) in push 1",
    );
}

#[test]
fn fuzz_phase6_findings() {
    // A comment after a final `;;`, which is not printed, ends the last
    // line of code and is read back as part of the last phrase: it counts
    // in that phrase's width.
    check_stable(
        "max_width = 16",
        "xs |> .filter _ g ;;  (* filter *)",
        "xs\n|> .filter _\n     g (* filter *)",
    );
    check_stable("", "f () ;; (* c *)", "f () (* c *)");
}

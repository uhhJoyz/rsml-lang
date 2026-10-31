#![deny(warnings)]

use rsml::ml;

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn double(x: i32) -> i32 {
    x * 2
}

fn adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

mod util {
    pub fn triple(x: i32) -> i32 {
        x * 3
    }
}

#[test]
fn pipe() {
    assert_eq!(ml! { 3 |> double |> double }, 12);
    assert_eq!(ml! { 3 |> add 1 |> double }, 8);
    assert_eq!(ml! { double @@ add 1 @@ 2 }, 6);
    assert_eq!(ml! { 2 |> util::triple }, 6);
    let xs = [1, 2, 3];
    assert_eq!(
        ml! { xs |> .iter |> .map _ (double !_) |> .fold _ 0 ( + ) },
        12
    );
}

#[test]
fn composition() {
    let f = ml! { double >> add 1 };
    assert_eq!(f(5), 11);
    let g = ml! { double << add 1 };
    assert_eq!(g(5), 12);
    assert_eq!(ml! { 5 |> (double >> add 1 >> double) }, 22);
    assert_eq!(ml! { (double >> double) 3 }, 12);
    let h = ml! {
    let add x y = x + y in
    let inc = add 1 in
    inc >> inc >> inc
    };
    assert_eq!(h(0), 3);
}

#[test]
fn partial_application() {
    let add5 = ml! {
    external add : _ -> _ -> _ ;;
    add 5
    };
    assert_eq!(add5(1), 6);
    assert_eq!(add5(2), 7);
    let r = ml! {
    let mul x y = x * y in
    let triple = mul 3 in
    triple 4 + triple 5
    };
    assert_eq!(r, 27);
    assert_eq!(
        ml! { [1; 2; 3] |> .into_iter |> .map _ (( * ) 10) |> .collect::<Vec<_>> },
        vec![10, 20, 30]
    );
    assert_eq!(ml! { (add 1 : i32 -> i32) 2 }, 3);
    // A placeholder gives the arity without `external`.
    let add6 = ml! { add 6 _ };
    assert_eq!(add6(1), 7);
    // Rust functions returning closures are declared with a parenthesized result.
    assert_eq!(
        ml! {
        external adder : _ -> (_ -> _) ;;
        adder 1 2
        },
        3
    );
    let add1 = ml! {
    external adder : _ -> (_ -> _) ;;
    adder 1
    };
    assert_eq!(add1(2), 3);
}

#[test]
fn partial_application_evaluates_arguments_once() {
    let mut calls = 0;
    let p = ml! {
    let add3 a b c = a + b + c in
    add3
      (begin
         calls <- calls + 1;
         10
       end)
    };
    assert_eq!(p(1, 2) + p(3, 4), 30);
    assert_eq!(calls, 1);
}

#[test]
fn mutating_closures_mutate_the_original() {
    let mut total = 0;
    ml! { [1; 2; 3] |> .into_iter |> .for_each _ (fun x -> total <- total + x) };
    assert_eq!(total, 6);
    let mut count = 0;
    let mut bump = ml! { fun () -> count <- count + 1 };
    bump();
    bump();
    assert_eq!(count, 2);
}

#[test]
fn closures_capture() {
    let n = 10;
    let f = ml! { fun x -> x + n };
    assert_eq!(f(1), 11);
    let k = ml! { fun x y -> x * y + n };
    assert_eq!(k(2, 3), 16);
    let unit = ml! { fun () -> 42 };
    assert_eq!(unit(), 42);
}

fn returns_closure(n: i32) -> impl Fn(i32) -> i32 { ml! {
    fun x -> x + n
}}

// Closures passed as arguments borrow their environment, as in Rust; one
// that outlives the function says so with `fun move`.
fn returns_iterator(xs: &[i32], n: i32) -> impl Iterator<Item = i32> + '_ { ml! {
    xs.iter().map(fun move x -> x + n).filter(fun x -> x % 2 = 0)
}}

#[test]
fn escaping_closures_move() {
    assert_eq!(returns_closure(2)(3), 5);
    assert_eq!(
        returns_iterator(&[1, 2, 3, 4], 1).collect::<Vec<_>>(),
        vec![2, 4]
    );
}

#[test]
fn let_forms() {
    assert_eq!(
        ml! {
        let x = 1 in
        let y = x + 1 in
        x + y
        },
        3
    );
    assert_eq!(
        ml! {
        let x = 1
        and y = 2 in
        x + y
        },
        3
    );
    let x = 10;
    assert_eq!(
        ml! {
        let x = 1
        and y = x in
        x + y
        },
        11
    );
    assert_eq!(
        ml! {
        let (a, b) = (1, 2) in
        a * 10 + b
        },
        12
    );
    assert_eq!(
        ml! {
        let f () = 7 in
        f ()
        },
        7
    );
    assert_eq!(
        ml! {
        let x : u8 = 5 in
        x
        },
        5u8
    );
    assert_eq!(
        ml! {
        let a = 1
        let b = a + 1 ;;
        a + b
        },
        3
    );
}

#[test]
fn let_rec() {
    let r = ml! {
    let rec fact (n : i64) : i64 = if n <= 1 then 1 else n * fact (n - 1) in
    fact 10
    };
    assert_eq!(r, 3628800);
    let r = ml! {
    let rec sum : &[i64] -> i64 = function [] -> 0 | x :: rest -> x + sum rest in
    sum &[1; 2; 3; 4]
    };
    assert_eq!(r, 10);
    let r = ml! {
    let rec even (n : i64) : bool = if n = 0 then true else odd (n - 1)
    and odd (n : i64) : bool = if n = 0 then false else even (n - 1) in
    (even 10, odd 7)
    };
    assert_eq!(r, (true, true));
    let r = ml! {
    let rec len (l : &['a]) : i64 = match l with [] -> 0 | _ :: t -> 1 + len t in
    len &["a"; "b"]
    };
    assert_eq!(r, 2);
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Shape {
    Circle(f64),
    Rect(f64, f64),
    Dot,
}

#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn match_patterns() {
    let area = |s: Shape| -> f64 {
        ml! {
        match s with
        | Shape::Circle r -> 3.0 *. r *. r
        | Shape::Rect (w, h) -> w *. h
        | Shape::Dot -> 0.0
        }
    };
    assert_eq!(area(Shape::Rect(2.0, 3.0)), 6.0);
    assert_eq!(area(Shape::Circle(1.0)), 3.0);
    assert_eq!(area(Shape::Dot), 0.0);

    let classify = |n: i32| -> &'static str {
        ml! {
        match n with 0 -> "zero" | 1 | 2 | 4 -> "small" | n when n < 0 -> "negative" | _ -> "big"
        }
    };
    assert_eq!(classify(0), "zero");
    assert_eq!(classify(2), "small");
    assert_eq!(classify(-5), "negative");
    assert_eq!(classify(50), "big");

    let opt = Some((1, 2));
    assert_eq!(ml! { match opt with Some (a, b) -> a + b | None -> 0 }, 3);

    let p = Point { x: 1, y: 2 };
    assert_eq!(
        ml! { match p with Point { x; y = 2 } -> x | Point { x = _; _ } -> 0 },
        1
    );

    // Spaced or not, `..` leaves its end out, as in Rust.
    let c = 'm';
    assert_eq!(
        ml! { match c with 'a'..'m' -> 1 | 'm'..'z' -> 2 | _ -> 3 },
        2
    );

    let s = String::from("hi");
    assert_eq!(ml! { match s with "hi" -> 1 | _ -> 0 }, 1);
}

#[test]
fn list_patterns() {
    let xs = vec![1, 2, 3];
    let r = ml! { match xs with [] -> 0 | [x] -> !x | x :: y :: _ -> !x + !y };
    assert_eq!(r, 3);
    let ys: Vec<i32> = vec![];
    assert!(ml! { match (xs, ys) with (_ :: _, []) -> true | _ -> false });
    let f = ml! { function [] -> "empty" | _ -> "nonempty" };
    assert_eq!(f(vec![1]), "nonempty");
    assert_eq!(f(Vec::new()), "empty");
    let zs = [5, 6];
    assert_eq!(
        ml! {
        let [a; b] = zs in
        !a * !b
        },
        30
    );
}

#[test]
fn lists_and_cons() {
    assert_eq!(ml! { [1; 2; 3] }, vec![1, 2, 3]);
    assert_eq!(ml! { 0 :: [1; 2] }, vec![0, 1, 2]);
    let tail = vec![2, 3];
    assert_eq!(ml! { 1 :: tail }, vec![1, 2, 3]);
    assert_eq!(ml! { [1; 2] @ [3] }, vec![1, 2, 3]);
    assert_eq!(ml! { [|1; 2|] }, [1, 2]);
    assert_eq!(ml! { [(1, 'a'); (2, 'b')] }, vec![(1, 'a'), (2, 'b')]);
}

#[test]
fn operators() {
    assert_eq!(ml! { 1 + 2 * 3 }, 7);
    assert_eq!(ml! { (1 + 2) * 3 }, 9);
    assert_eq!(ml! { 7 mod 3 }, 1);
    assert!(ml! { 1 = 1 && 2 <> 3 });
    let no = false;
    assert!(ml! { not (1 > 2) || no });
    assert_eq!(ml! { 1.5 +. 2.0 *. 2.0 }, 5.5);
    assert_eq!(ml! { 2.0 ** 10.0 }, 1024.0);
    assert_eq!(ml! { 6 land 3 lor 8 }, 10);
    assert_eq!(ml! { 1 lsl 4 }, 16);
    assert_eq!(ml! { "ab" ^ "cd" ^ "!" }, "abcd!");
    assert_eq!(ml! { -3 + 1 }, -2);
    assert_eq!(
        ml! {
        let ( +! ) a b = a * 10 + b in
        1 +! 2 +! 3
        },
        123
    );
    assert_eq!(ml! { [1; 2] |> .into_iter |> Iterator::fold _ 10 ( - ) }, 7);
}

#[test]
fn control_flow() {
    let mut out = Vec::new();
    ml! {
    for i = 1 to 3 do out.push(i) done;
    for i = 3 downto 1 do out.push(i * 10) done
    };
    assert_eq!(out, vec![1, 2, 3, 30, 20, 10]);
    let r = ml! {
    let i = ref 0 in
    let total = ref 0 in
    while !i < 5 do
      total := !total + !i;
      incr i
    done;
    !total
    };
    assert_eq!(r, 10);
    assert_eq!(
        ml! { if 1 > 2 then "a" else if 2 > 1 then "b" else "c" },
        "b"
    );
}

#[test]
fn refs() {
    fn bump(r: &mut i32) {
        *r += 1
    }
    let r = ml! {
    let count = ref 0 in
    .for_each (.into_iter [1; 2; 3]) (fun x -> count := !count + x);
    bump count;
    !count
    };
    assert_eq!(r, 7);
}

#[test]
fn records_and_methods() {
    let p = ml! { Point { x = 1; y = 2 } };
    assert_eq!(p, Point { x: 1, y: 2 });
    let q = ml! { { p with y = 5 } };
    assert_eq!(q, Point { x: 1, y: 5 });
    assert_eq!(ml! { q.x + q.y }, 6);
    let v = [1, 2, 3];
    assert_eq!(
        ml! { v.iter().map(fun x -> x * 2).collect::<Vec<i32>>() },
        vec![2, 4, 6]
    );
    assert_eq!(ml! { v.len() }, 3);
    let t = ((1, 2), 3);
    assert_eq!(ml! { t.0.1 + t.1 }, 5);
    let mut arr = [1, 2, 3];
    ml! { arr.(0) <- 10 };
    assert_eq!(ml! { arr.(0) + arr.(2) }, 13);
    assert_eq!(ml! { .len v + (v |> .iter |> .count) }, 6);
    assert_eq!(ml! { String::from "x" }, "x");
    assert_eq!(ml! { String.from "y" }, "y");
    let v1: Vec<u8> = ml! { Vec::with_capacity 4 };
    assert!(v1.capacity() >= 4);
    let v2: Vec<i32> = ml! { Vec::new () };
    assert!(v2.is_empty());
}

#[test]
fn method_refs() {
    let mut v = vec![1, 2];
    ml! { .push v 3 };
    assert_eq!(v, vec![1, 2, 3]);
    assert_eq!(ml! { .len v }, 3);
    assert_eq!(ml! { (.iter >> .count) &v }, 3);
    let words = ["a", "bb"];
    assert_eq!(ml! { words |> .iter |> .map _ (.len) |> .sum::<usize> }, 3);
    assert_eq!(ml! { v |> .get _ 0 |> .copied }, Some(1));
    // After an expression, `.name` is still its method or field.
    let t = (5, 6);
    assert_eq!(ml! { std::cmp::max t.0 (t.1) }, 6);
}

#[test]
fn sections() {
    // Placeholders make the expression around them a function.
    let xs = [1, 2, 3];
    assert_eq!(
        ml! { xs |> .iter |> .map _ (!_ * 10) |> .collect::<Vec<i32>> },
        vec![10, 20, 30]
    );
    let sub = |a: i32, b: i32| a - b;
    assert_eq!(ml! { (sub _1 _0) 1 10 }, 9);
    let flipped = ml! { sub _1 _0 };
    assert_eq!(flipped(1, 10), 9);
    let clamp = |lo: i32, x: i32, hi: i32| x.max(lo).min(hi);
    assert_eq!(
        ml! { [-5; 5; 50] |> .into_iter |> .map _ (clamp 0 _ 10) |> .collect::<Vec<_>> },
        vec![0, 5, 10]
    );
    // Data-first Rust functions in a pipeline.
    assert_eq!(ml! { "hello" |> str::len }, 5);
    assert!(ml! { "hello" |> str::starts_with _ "he" });
    // An annotated placeholder types the closure parameter.
    assert_eq!(
        ml! { ["a"; "bcd"] |> .into_iter |> .map _ (.len (_ : &str)) |> .sum::<usize> },
        4
    );
    let square = ml! { ( * ) _0 _0 };
    assert_eq!(square(7), 49);
    // The rest of the expression is evaluated once, when the function is made.
    let mut calls = 0;
    let f = ml! {
    add
      (begin
         calls <- calls + 1;
         1
       end)
      _
    };
    assert_eq!(f(1) + f(2), 5);
    assert_eq!(calls, 1);
}

#[test]
fn rust_interop() {
    let s = String::from("hello");
    assert_eq!(ml! { String::len &s }, 5);
    assert_eq!(s, "hello");
    let parse = |t: &str| -> Result<i32, std::num::ParseIntError> {
        ml! { Ok (t.parse::<i32>()? + 1) }
    };
    assert_eq!(parse("41"), Ok(42));
    assert!(parse("x").is_err());
    assert_eq!(ml! { format!("{}{}", 1, 2) }, "12");
    assert_eq!(ml! { std::cmp::max 3 4 }, 4);
    assert_eq!(ml! { i32::MAX }, i32::MAX);
    // A Rust keyword names a value only as a raw identifier.
    let r#fn = 3;
    assert_eq!(
        ml! {
        let r#loop = 2 in
        r#loop + 1
        },
        r#fn
    );
}

#[test]
fn comments() {
    assert_eq!(ml! { (* a comment *) 1 + (* another *) 2 }, 3);
    assert_eq!(ml! { ( * ) 2 3 }, 6);
}

#[test]
fn use_items() {
    assert_eq!(
        ml! {
        use std::cmp::*;
        max 3 4
        },
        4
    );
    assert_eq!(
        ml! {
        let x = 3 in
        use std::cmp::min;
        min x 4
        },
        3
    );
    let open = 5;
    assert_eq!(ml! { open + 1 }, 6);
}

#[test]
fn assert_false_is_unreachable() {
    let f = |x: i32| -> i32 {
        ml! { match x with 0 -> 1 | _ -> assert false }
    };
    assert_eq!(f(0), 1);
    assert!(std::panic::catch_unwind(|| f(1)).is_err());
}

#[test]
fn iterator_pipelines_are_lazy() {
    let r: Vec<i64> = ml! {
    std::iter::successors (Some 1) (fun x -> Some (x + 1))
    |> Iterator::map _ (fun x -> x * x)
    |> Iterator::filter _ (fun x -> !x mod 2 = 1)
    |> Iterator::take _ 4
    |> Iterator::collect
    };
    assert_eq!(r, vec![1, 9, 25, 49]);
}

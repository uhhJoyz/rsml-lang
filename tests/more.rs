#![deny(warnings)]

use rsml::ml;

fn add(a: i32, b: i32) -> i32 {
    a + b
}

macro_rules! twice_plus {
    ($e:expr) => {
        ml! { let x = $e in x + x }
    };
}

#[test]
fn macro_rules_fragments_are_rust() {
    assert_eq!(twice_plus!(add(1, 2)), 6);
    assert_eq!(twice_plus!(vec![1, 2].len()), 4);
}

#[test]
fn let_rec_forms() {
    // Annotated binding with `fun`.
    let r = ml! {
        let rec pow : i64 -> i64 -> i64 = fun b e -> if e = 0 then 1 else b * pow b (e - 1) in
        pow 2 10
    };
    assert_eq!(r, 1024);
    // Type variables become generic parameters.
    let r = ml! {
        let rec count (l : &['a]) (acc : i64) : i64 =
          match l with [] -> acc | _ :: t -> count t (acc + 1)
        in
        (count &[1; 2; 3] 0, count &["a"] 0)
    };
    assert_eq!(r, (3, 1));
    // Unit parameter.
    let r = ml! {
        let rec answer () : i64 = 42 in
        answer ()
    };
    assert_eq!(r, 42);
    // Partially applied `let rec` function.
    let r = ml! {
        let rec add3 (a : i64) (b : i64) (c : i64) : i64 = a + b + c in
        [1; 2] |> .into_iter |> .map _ (add3 10 20) |> .collect::<Vec<_>>
    };
    assert_eq!(r, vec![31, 32]);
}

#[test]
fn nested_matches_and_begin_end() {
    let f = |a: Option<i32>, b: Option<i32>| -> i32 {
        ml! {
            match a with Some x -> begin match b with Some y -> x + y | None -> x end | None -> 0
        }
    };
    assert_eq!(f(Some(1), Some(2)), 3);
    assert_eq!(f(Some(1), None), 1);
    assert_eq!(f(None, Some(2)), 0);
}

#[test]
fn as_patterns_and_or_patterns() {
    let xs = vec![1, 2, 3];
    let r = ml! { match xs with (x :: _) as all -> !x + .len all as i32 | [] -> 0 };
    assert_eq!(r, 4);
    let n = 5;
    assert_eq!(ml! { match n with (1 | 2) as k -> k | k -> k * 10 }, 50);
}

#[test]
fn sequencing_and_unit() {
    let mut log = Vec::new();
    let r = ml! {
        log.push(1);
        log.push(2);
        if log.len() > 1 then log.push(3);
        .len log
    };
    assert_eq!(r, 3);
    assert_eq!(log, vec![1, 2, 3]);
    assert_eq!(ml! { () }, ());
}

#[test]
fn labeled_arguments() {
    assert_eq!(
        ml! {
            let f ~x ~y = x - y in
            f ~x:5 ~y:2
        },
        3
    );
    // In any order, evaluated in the order written.
    let mut log = Vec::new();
    assert_eq!(
        ml! {
            let f ~x ~y = x - y in
            let mut note c v =
              log.push(c);
              v
            in
            f ~y:(note 'y' 2) ~x:(note 'x' 5)
        },
        3
    );
    assert_eq!(log, ['y', 'x']);
    // `~y` is `~y:y`; unlabeled arguments fill the parameters left.
    let y = 2;
    assert_eq!(
        ml! {
            let f ~x ~y = x - y in
            f ~y 5
        },
        3
    );
    // Leaving out a parameter before a given one makes a function of it.
    let minus_two = ml! {
        let f ~x ~y = x - y in
        f ~y:2
    };
    assert_eq!(minus_two(5), 3);
    // `let f = fun ~x ~y -> ..` has labels too.
    assert_eq!(
        ml! {
            let f = fun ~x ~y -> x - y in
            f ~y:2 ~x:5
        },
        3
    );
}

#[test]
fn strings() {
    let owned = String::from("a");
    let s = ml! { owned ^ "b" ^ "c" };
    assert_eq!(s, "abc");
    let name = "world";
    assert_eq!(ml! { "hello, " ^ name }, "hello, world");
    assert_eq!(ml! { "abc".as_bytes()[1] }, b'b');
    assert_eq!(
        ml! { format!("{:>5}|{:<3}|{:x}", "ab", 7, 255) },
        "   ab|7  |ff"
    );
}

#[test]
fn deref_and_borrow() {
    let v = vec![1, 2, 3];
    let r = &v;
    assert_eq!(ml! { .len r }, 3);
    let mut n = 1;
    ml! {
        let p = &mut n in
        p := !p + 1
    };
    assert_eq!(n, 2);
    let boxed = Box::new(5);
    assert_eq!(ml! { !boxed + 1 }, 6);
}

#[test]
fn tuples_and_nested_data() {
    let pairs = [(1, "one"), (2, "two")];
    let r = ml! {
        pairs
        |> .iter
        |> .map _ (fun (n, s) -> format!("{}={}", n, s))
        |> .collect::<Vec<_>>
        |> .join _ ","
    };
    assert_eq!(r, "1=one,2=two");
    assert_eq!(
        ml! {
            let (a, (b, c)) = (1, (2, 3)) in
            a + b + c
        },
        6
    );
    assert_eq!(ml! { (1, (2, 3)).1.0 }, 2);
}

#[derive(Debug, PartialEq)]
enum Expr {
    Num(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

fn eval(e: &Expr) -> i64 {
    ml! {
        match e with
        | Expr::Num n -> !n
        | Expr::Add (a, b) -> eval a + eval b
        | Expr::Mul (a, b) -> eval a * eval b
        | Expr::Neg a -> -(eval a)
    }
}

#[test]
fn interpreter() {
    let e = Expr::Add(
        Box::new(Expr::Num(2)),
        Box::new(Expr::Mul(
            Box::new(Expr::Num(3)),
            Box::new(Expr::Neg(Box::new(Expr::Num(4)))),
        )),
    );
    assert_eq!(eval(&e), -10);
    assert_eq!(ml! { Expr::Num 3 }, Expr::Num(3));
    assert_eq!(ml! { Expr.Num 3 }, Expr::Num(3));
}

#[test]
fn point_free_style() {
    let r = ml! {
        let sum = Iterator::fold _ 0 ( + ) in
        let squares = Iterator::map _ (fun x -> x * x) in
        let sum_of_squares = squares >> sum in
        sum_of_squares (.into_iter (vec![1, 2, 3]))
    };
    assert_eq!(r, 14);
    let is_even = |x: &i32| x % 2 == 0;
    let r = ml! {
        [1; 2; 3; 4; 5; 6]
        |> .into_iter
        |> Iterator::filter _ is_even
        |> Iterator::map _ (( * ) 3 >> ( + ) 1)
        |> Iterator::rev
        |> Iterator::collect::<Vec<_>>
    };
    assert_eq!(r, vec![19, 13, 7]);
    let k = compose3(|x: i32| x + 1, |x: i32| x * 2, |x: i32| x - 3);
    assert_eq!(k(4), 7);
}

fn compose3<A, B, C, D>(
    f: impl Fn(A) -> B,
    g: impl Fn(B) -> C,
    h: impl Fn(C) -> D,
) -> impl Fn(A) -> D {
    ml! { f >> g >> h }
}

#[test]
fn shadowing() {
    let r = ml! {
        let add x y = x + y in
        let add = add 10 in
        add 1
    };
    assert_eq!(r, 11);
    let r = ml! {
        let x = 1 in
        let x = x + 1 in
        let x = x * 10 in
        x
    };
    assert_eq!(r, 20);
}

#[test]
fn unit_functions() {
    let mut calls = 0;
    {
        let mut tick = ml! { fun () -> calls <- calls + 1 };
        tick();
        tick();
    }
    assert_eq!(calls, 2);
    let r = ml! {
        let f () = 5 in
        let g = f in
        g () + f ()
    };
    assert_eq!(r, 10);
    // `f ()` calls with no arguments; `f (())` passes the unit value.
    fn zero() -> i32 {
        1
    }
    fn takes_unit(u: ()) -> i32 {
        let () = u;
        2
    }
    fn unit_and_more(u: (), n: i32) -> i32 {
        let () = u;
        n
    }
    assert_eq!(ml! { zero () + takes_unit (()) + unit_and_more () 4 }, 7);
    // The same rule in a pipeline: `() |> f` is `f ()`.
    assert_eq!(ml! { () |> zero }, 1);
    assert_eq!(ml! { (()) |> takes_unit }, 2);
}

struct Callbacks {
    on_value: fn(i32) -> i32,
}

#[test]
fn whitespace_sensitive_forms() {
    // `h::t` is cons when `h` is bound by the ml code, a path otherwise.
    // (Written glued on purpose: the formatter would space it.)
    let r = ml! {
        // rsmlfmt: skip
        let h = 0 in let t = [1; 2] in h::t
    };
    assert_eq!(r, vec![0, 1, 2]);
    let r = ml! {
        // rsmlfmt: skip
        match [1; 2; 3] with h::t -> !h + 10 * (.len t as i32) | [] -> 0
    };
    assert_eq!(r, 21);
    let tail = vec![2];
    assert_eq!(ml! { 1 :: tail }, vec![1, 2]);
    assert_eq!(ml! { std::cmp::min 3 4 }, 3);
    // `x.f(a)` is a method call; `x.f (a)` applies the field `f`.
    let cb = Callbacks {
        on_value: |x| x + 1,
    };
    assert_eq!(ml! { cb.on_value (41) }, 42);
    let v = [1, 2];
    assert_eq!(ml! { v.len() + .len v }, 4);
    // `name!(..)` is a Rust macro; `f !r` dereferences.
    let r = &5;
    assert_eq!(ml! { ( + ) 1 !r }, 6);
}

#[test]
fn early_return_with_question_mark() {
    fn parse_sum(a: &str, b: &str) -> Result<i32, std::num::ParseIntError> {
        ml! {
            let x = a.parse::<i32>()? in
            let y = b.parse::<i32>()? in
            Ok (x + y)
        }
    }
    assert_eq!(parse_sum("1", "2"), Ok(3));
    assert!(parse_sum("1", "x").is_err());
}

#[test]
fn tail_recursion_runs_in_constant_stack() {
    // A million calls deep, in a test thread's small stack: each
    // function is a loop, not a recursive `fn`.
    let xs: Vec<i32> = (0..1_000_000).collect();
    let n = ml! {
        let rec go (acc : i64) (l : &[i32]) : i64 =
          match l with [] -> acc | _ :: rest -> go (acc + 1) rest
        in
        go 0 &xs
    };
    assert_eq!(n, 1_000_000);
    let s = ml! {
        let rec go ~(acc : u64) ~(k : u64) : u64 =
          if k = 0 then acc else go ~k:(k - 1) ~acc:(acc + k)
        in
        go ~k:1_000_000 ~acc:0
    };
    assert_eq!(s, 500_000_500_000);
}

#![deny(warnings)]

//! Every notation of docs/notation.md, as code that runs. Besides testing
//! the notations, this file is what `scripts/check_clippy.sh` lints: clippy
//! sees the Rust the macros generate, so a lint that OCaml code could not
//! act on shows up here.

use rsml::{ml, rsml};

fn f(a: i64, b: i64) -> i64 {
    a * 10 + b
}

fn g(a: i64) -> i64 {
    a + 1
}

fn h(a: i64) -> i64 {
    a * 2
}

fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn adder(n: i64) -> impl Fn(i64) -> i64 {
    move |x| x + n
}

fn zero() -> i64 {
    0
}

fn takes_unit(u: ()) -> i64 {
    let () = u;
    1
}

fn unit_and(u: (), n: i64) -> i64 {
    let () = u;
    n
}

fn clamp(lo: i64, x: i64, hi: i64) -> i64 {
    x.max(lo).min(hi)
}

fn swap(p: (i64, i64)) -> (i64, i64) {
    (p.1, p.0)
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Point {
    x: i64,
    y: i64,
}

#[derive(Debug, PartialEq)]
struct Pair((i64, i64));

/// A record that is not `Copy`.
#[derive(Debug, PartialEq)]
struct Labelled {
    label: String,
    n: i64,
}

#[derive(Debug, PartialEq)]
enum Shape {
    Circle(f64),
    Rect(f64, f64),
}

struct Callbacks {
    f: fn(i64) -> i64,
}

#[test]
fn application() {
    assert_eq!(ml! { f 1 2 }, 12);
    assert_eq!(ml! { std::cmp::max 1 2 }, 2);
    let v: Vec<u8> = ml! { Vec::with_capacity 10 };
    assert!(v.capacity() >= 10);
    let v: Vec<u8> = ml! { Vec.new () };
    assert!(v.is_empty());
    assert_eq!(ml! { String.from "a" }, "a");
    let mut xs = vec![1, 2];
    assert_eq!(ml! { Vec::len &xs }, 2);
    ml! { Vec::push &mut xs 3 };
    assert_eq!(xs, [1, 2, 3]);
    let y = 2;
    assert_eq!(ml! { f ~x:1 ~y }, 12);
    // Unit.
    assert_eq!(ml! { zero () + takes_unit (()) + unit_and () 2 }, 3);
    // Constructors.
    assert_eq!(ml! { Some (1, 2) }, Some((1, 2)));
    assert_eq!(ml! { Shape::Rect (1.0, 2.0) }, Shape::Rect(1.0, 2.0));
    assert_eq!(ml! { Pair ((1, 2)) }, Pair((1, 2)));
}

#[test]
fn arity_and_partial_application() {
    assert_eq!(
        ml! {
            let add x y = x + y in
            add 1 2
        },
        3
    );
    let inc = ml! {
        let add x y = x + y in
        add 1
    };
    assert_eq!(inc(2), 3);
    let expensive = || 5;
    let add5 = ml! {
        let add x y = x + y in
        add (expensive ())
    };
    assert_eq!(add5(1), 6);
    let inc = ml! {
        external add : i64 -> i64 -> i64 ;;
        add 1
    };
    assert_eq!(inc(1), 2);
    assert_eq!(
        ml! {
            external adder : i64 -> (i64 -> i64) ;;
            adder 1 2
        },
        3
    );
    let plus1 = ml! {
        external plus : _ -> _ -> _ = "std::ops::Add::add" ;;
        plus 1i64
    };
    assert_eq!(plus1(2), 3);
    let add6 = ml! { (add 6 : i64 -> i64) };
    assert_eq!(add6(1), 7);
}

rsml! {
    fn add_all(add: impl Fn(i64, i64) -> i64, xs: Vec<i64>) -> Vec<i64> {
        xs |> .into_iter |> .map _ (add 10) |> .collect
    }
}

#[test]
fn closure_typed_parameters() {
    assert_eq!(add_all(|a, b| a + b, vec![1, 2]), [11, 12]);
}

#[test]
fn pipelines() {
    let x = 3;
    assert_eq!(ml! { x |> g }, 4);
    assert_eq!(ml! { x |> f 1 }, 13);
    assert_eq!(ml! { x |> f 1 |> f 2 }, 33);
    assert_eq!(ml! { g @@ h @@ x }, 7);
    let xs = vec![1, 2];
    assert_eq!(ml! { &xs |> Vec::len }, 2);
    assert_eq!(ml! { () |> zero }, 0);
    assert_eq!(ml! { (()) |> takes_unit }, 1);
    assert_eq!(ml! { x |> f _ 1 }, 31);
}

#[test]
fn composition() {
    let gh = ml! { g >> h };
    assert_eq!(gh(1), 4);
    let hg = ml! { g << h };
    assert_eq!(hg(1), 3);
    let ghg = ml! { g >> h >> g };
    assert_eq!(ghg(1), 5);
    assert_eq!(ml! { (g >> h) 1 }, 4);
    assert_eq!(ml! { 1 |> (g >> h) }, 4);
    let add3 = ml! {
        let add x y = x + y in
        add 1 >> add 2
    };
    assert_eq!(add3(0), 3);
    let count: fn(&Vec<i64>) -> usize = ml! { .iter >> .count };
    assert_eq!(count(&vec![1, 2]), 2);
}

#[test]
fn operator_sections_and_user_operators() {
    let plus = ml! { ( + ) };
    assert_eq!(plus(1, 2), 3);
    let inc = ml! { ( + ) 1 };
    assert_eq!(inc(1), 2);
    let from10 = ml! { ( - ) 10 };
    assert_eq!(from10(1), 9);
    let modulo = ml! { ( mod ) };
    assert_eq!(modulo(7, 3), 1);
    assert_eq!(
        ml! {
            let ( +! ) a b = a + 2 * b in
            1 +! 2
        },
        5
    );
    let o: Option<i64> = None;
    assert_eq!(
        ml! {
            let ( |? ) o d = Option::unwrap_or o d in
            o |? 0
        },
        0
    );
    // An operator that mutates what it captures.
    assert_eq!(
        ml! {
            let mut calls = 0 in
            let ( +! ) a b =
              calls <- calls + 1;
              a + b
            in
            let a = 1 +! 2 in
            let _ = a +! 3 in
            calls
        },
        2
    );
}

#[test]
fn placeholders() {
    let f1 = ml! { f _ 1 };
    assert_eq!(f1(2), 21);
    let flipped = ml! { f _1 _0 };
    assert_eq!(flipped(1, 2), 21);
    let three = |a: i64, b: i64, c: i64| a * 100 + b * 10 + c;
    let mixed = ml! { three _1 _ _ };
    assert_eq!(mixed(1, 2, 3), 213);
    let square = ml! { ( * ) _0 _0 };
    assert_eq!(square(4), 16);
    let xs = [1, 2, 3];
    let map = |m: &dyn Fn(i64) -> i64, xs: &[i64]| xs.iter().map(|x| m(*x)).collect::<Vec<_>>();
    assert_eq!(ml! { map &(_ * 2) &xs }, [2, 4, 6]);
    let mapi = |m: &dyn Fn(usize, i64) -> i64, xs: &[i64]| {
        xs.iter()
            .enumerate()
            .map(|(i, x)| m(i, *x))
            .collect::<Vec<_>>()
    };
    assert_eq!(ml! { mapi &(g _1) &xs }, [2, 3, 4]);
    let words = ["a", "bcd"];
    assert_eq!(
        ml! { words |> .iter |> .map _ (_.len()) |> .sum::<usize> },
        4
    );
    let lens = |m: &dyn Fn(&str, i64) -> i64| m("ab", 1);
    assert_eq!(ml! { lens &(fun s n -> (s : &str).len() as i64 + n) }, 3);
    let affine = ml! { (1 + _ * 2) };
    assert_eq!(affine(3), 7);
    let ok = || true;
    let pos: Vec<i64> = ml! { xs |> .into_iter |> .filter _ (_ > &1 && ok ()) |> .collect };
    assert_eq!(pos, [2, 3]);
    // Applied directly, no closure.
    assert_eq!(ml! { (f _1 _0) 1 2 }, 21);
    let one = ml! { (f _1 _0) 1 };
    assert_eq!(one(2), 21);
    let expensive = || 4;
    let ex = ml! { f (expensive ()) _ };
    assert_eq!(ex(2), 42);
    assert_eq!(ml! { clamp 0 _ 10 }(20), 10);
}

#[test]
fn methods_as_functions() {
    let s = "abc";
    assert_eq!(ml! { .len s }, 3);
    let mut xs = vec![1, 2];
    ml! { .push xs 3 };
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    ml! { .send tx () |> .unwrap };
    assert!(rx.try_recv().is_ok());
    let make = |n: i64| vec![n];
    assert_eq!(ml! { .get (make 2) 0 |> .copied }, Some(2));
    let it = xs.iter().copied();
    assert_eq!(ml! { .collect::<Vec<_>> it }, [1, 2, 3]);
    assert_eq!(ml! { xs |> .iter |> .count }, 3);
    assert_eq!(ml! { xs |> .iter |> .map _ (_ * 2) |> .sum::<i64> }, 12);
    let len: fn(&str) -> usize = ml! { .len };
    assert_eq!(len("ab"), 2);
    let words = ["a", "bb"];
    assert_eq!(
        ml! { words |> .into_iter |> .map _ (.len (_ : &str)) |> .sum::<usize> },
        3
    );
    // With Rust-style arguments: method sections.
    assert_eq!(
        ml! { xs |> .iter() |> .map(fun x -> x * 2) |> .sum::<i64>() },
        12
    );
    ml! { .push(4) xs };
    assert_eq!(xs, [1, 2, 3, 4]);
    let count: fn(&Vec<i64>) -> usize = ml! { .iter() >> .count() };
    assert_eq!(count(&xs), 4);
    // A receiver evaluated once, which the method mutates.
    let firsts = ml! {
        let mut next = .nth (0..10) _ in
        (next 0, next 0)
    };
    assert_eq!(firsts, (Some(0), Some(1)));
}

#[test]
fn rust_style_methods_fields_and_calls() {
    let mut xs = vec![1, 2];
    ml! { xs.push(3) };
    assert_eq!(
        ml! { xs.iter().map(fun x -> x + 1).collect::<Vec<_>>() },
        [2, 3, 4]
    );
    let n = ml! { xs.iter().count() };
    assert_eq!(n, 3);
    let p = (1, 2);
    assert_eq!(ml! { p.0 }, 1);
    let r = Callbacks { f: g };
    assert_eq!(ml! { r.f (1) }, 2);
    let t = Point { x: 1, y: 2 };
    assert_eq!(ml! { f t.x (t.y + 1) }, 13);
    // Calls and indexing written without a space.
    assert_eq!(ml! { f(1, 2) + adder(1)(2) }, 15);
    assert_eq!(ml! { swap (1, 2) }, (2, 1));
    assert_eq!(
        ml! {
            let add x y = x + y in
            add(1)(2) + add(1, 2)
        },
        6
    );
    assert_eq!(ml! { 3 |> f(_, 1) }, 31);
    assert_eq!(ml! { xs[0] + xs[1..].len() as i64 }, 3);
}

#[test]
fn paths() {
    let mut a = 1;
    let mut b = 2;
    ml! { std::mem::swap &mut a &mut b };
    assert_eq!((a, b), (2, 1));
    assert_eq!(ml! { ::std::cmp::max 1 2 }, 2);
    let v: Vec<i64> = ml! { Vec::<i64>::new () };
    assert!(v.is_empty());
    assert_eq!(ml! { i32::MAX }, i32::MAX);
    let doubled: Vec<i64> = ml! { [1; 2].into_iter() |> Iterator::map _ h |> Iterator::collect };
    assert_eq!(doubled, [2, 4]);
}

#[test]
fn lambdas() {
    let sum = ml! { fun x y -> x + y };
    assert_eq!(sum(1, 2), 3);
    let one = ml! { fun () -> 1 };
    assert_eq!(one(), 1);
    let len = ml! { fun (x : &str) -> x.len() };
    assert_eq!(len("ab"), 2);
    let pair = ml! { fun (a, b) -> a + b };
    assert_eq!(pair((1, 2)), 3);
    let xs = [1, 2];
    assert_eq!(
        ml! { xs.iter().map(fun x -> x * 2).collect::<Vec<_>>() },
        [2, 4]
    );
    assert_eq!(
        ml! { xs.iter().map(fun move x -> x * 2).collect::<Vec<_>>() },
        [2, 4]
    );
    assert_eq!(ml! { (fun x -> x + 1) 2 }, 3);
    assert_eq!(ml! { 3 |> fun x -> x * x }, 9);
    let unwrap = ml! { function Some x -> x | None -> 0 };
    assert_eq!(unwrap(Some(4)), 4);
}

#[test]
fn local_functions_and_bindings() {
    assert_eq!(
        ml! {
            let f x = x * 2 in
            f 3
        },
        6
    );
    assert_eq!(
        ml! {
            let rec fact (n : i64) : i64 = if n = 0 then 1 else n * fact (n - 1) in
            fact 5
        },
        120
    );
    assert_eq!(
        ml! {
            let rec f : i64 -> i64 = function 0 -> 1 | n -> n * f (n - 1) in
            f 5
        },
        120
    );
    assert!(ml! {
        let rec even (n : u32) : bool = if n = 0 then true else odd (n - 1)
        and odd (n : u32) : bool = if n = 0 then false else even (n - 1) in
        even 10
    });
    assert_eq!(
        ml! {
            let rec id (x : 'a) : 'a = x in
            id 1
        },
        1
    );
    assert_eq!(
        ml! {
            let x = 1
            let y = 2 ;;
            x + y
        },
        3
    );
    assert_eq!(
        ml! {
            open std::cmp ;;
            max 1 2
        },
        2
    );
    assert_eq!(
        ml! {
            let open std::cmp in
            min 1 2
        },
        1
    );
    assert_eq!(
        ml! {
            let r = ref 0 in
            incr r;
            r := !r + 1;
            !r
        },
        2
    );
    assert_eq!(
        ml! {
            let mut n = 0 in
            n <- n + 1;
            n
        },
        1
    );
    // A function that mutates what it captures.
    assert_eq!(
        ml! {
            let c = ref 0 in
            let mut next () =
              incr c;
              !c
            in
            next () + next ()
        },
        3
    );
    let mut v = Vec::new();
    ml! {
        let mut push x = v.push(x) in
        push 1;
        push 2
    };
    assert_eq!(v, [1, 2]);
}

#[test]
fn patterns() {
    let o = Some(3);
    assert_eq!(ml! { match o with Some x when x > 0 -> x | _ -> 0 }, 3);
    let xs = vec![1, 2];
    assert_eq!(
        ml! { match xs with [] -> 0 | _ :: rest -> 1 + .len rest },
        2
    );
    let ord = 1.cmp(&2);
    assert_eq!(
        ml! { match ord with std::cmp::Ordering::Less -> -1 | _ -> 0 },
        -1
    );
    let n = i32::MAX;
    assert_eq!(ml! { match n with i32::MAX -> 1 | _ -> 0 }, 1);
    let p = Point { x: 1, y: 2 };
    assert_eq!(ml! { match p with Point { x; _ } -> x }, 1);
    let area = |s: &Shape| -> f64 {
        ml! { match s with Shape::Circle r -> 3.0 *. !r *. !r | Shape::Rect (w, h) -> !w *. !h }
    };
    assert_eq!(
        area(&Shape::Circle(1.0)) + area(&Shape::Rect(2.0, 3.0)),
        9.0
    );
    let c = 'q';
    assert_eq!(
        ml! { match c with 'a' .. 'm' -> 1 | 'n'..='z' -> 2 | _ -> 3 },
        2
    );
}

#[test]
fn operators_that_become_calls() {
    let xs = vec![2, 3];
    assert_eq!(ml! { 1 :: xs }, [1, 2, 3]);
    assert_eq!(ml! { 1 :: [2; 3] }, [1, 2, 3]);
    let ys = vec![4];
    assert_eq!(ml! { [1] @ ys }, [1, 4]);
    let a = String::from("a");
    assert_eq!(ml! { a ^ "b" }, "ab");
    assert_eq!(ml! { 2.0 ** 3.0 }, 8.0);
    let (a, b, c) = (4, 2, false);
    assert!(ml! { a mod b = 0 && not c });
    let n = 3;
    assert_eq!(ml! { n as f64 *. 2.0 }, 6.0);
}

#[test]
fn control_flow() {
    let c = true;
    assert_eq!(ml! { if c then 1 else 2 }, 1);
    let mut out = Vec::new();
    ml! {
        out.push(1);
        out.push(2)
    };
    ml! {
        begin
          out.push(3);
          out.push(4)
        end
    };
    let n = 2;
    ml! { for i = 0 to n do out.push(i) done };
    ml! { for i = n downto 0 do out.push(i) done };
    let mut k = 3;
    ml! { while k > 0 do k <- k - 1 done };
    ml! { assert (k = 0) };
    assert_eq!(out, [1, 2, 3, 4, 0, 1, 2, 2, 1, 0]);
    let never = |x: i64| ml! { match x with 0 -> 1 | _ -> assert false };
    assert_eq!(never(0), 1);
    // Rust's loops and jumps.
    let mut total = 0;
    ml! { for (i, x) in [10; 20].iter().enumerate() do total <- total + i as i64 * !x done };
    assert_eq!(total, 20);
    let first_big = |xs: &[i64]| -> Option<i64> {
        ml! {
            for x in xs do if !x > 10 then return Some(!x) done;
            None
        }
    };
    assert_eq!(first_big(&[1, 20, 30]), Some(20));
    let r = ml! {
        let i = ref 0 in
        while true do
          incr i;
          if !i mod 2 = 1 then continue;
          if !i > 5 then break
        done;
        !i
    };
    assert_eq!(r, 6);
    assert_eq!(ml! { (0..4).sum::<i64>() + (1..=3).sum::<i64>() }, 12);
    let v = [1, 2, 3];
    assert_eq!(ml! { v[1..].len() + v[..2].len() + v[..].len() }, 7);
    unsafe fn unchecked(x: i64) -> i64 {
        x
    }
    assert_eq!(ml! { unsafe { unchecked 1 } }, 1);
    let fut = ml! { async move { 1 + 1 } };
    drop(fut);
}

#[test]
fn records_and_collections() {
    let y = 2;
    let p = ml! { Point { x = 1; y } };
    assert_eq!(p, Point { x: 1, y: 2 });
    let q = ml! { { p with x = 2 } };
    assert_eq!(q, Point { x: 2, y: 2 });
    // The new values are computed before the old record is moved.
    let l = Labelled {
        label: String::from("a"),
        n: 1,
    };
    let l = ml! { { l with n = l.n + 1 } };
    assert_eq!(l.n, 2);
    let (a, b) = (1, 2);
    assert_eq!(ml! { [a; b] }, [1, 2]);
    let empty: Vec<i64> = ml! { [] };
    assert!(empty.is_empty());
    assert_eq!(ml! { [|a; b|] }, [1, 2]);
    assert_eq!(ml! { (a, b) }, (1, 2));
}

#[test]
fn types() {
    let n = 3;
    assert_eq!(ml! { (n : i64) + 1 }, 4);
    let s = ml! {
        let t : (i64 * String) = (1, String::from("a")) in
        t
    };
    assert_eq!(s, (1, String::from("a")));
    let c = ml! { (fun x -> x : i64 -> i64) };
    assert_eq!(c(5), 5);
    assert_eq!(ml! { [1u8; 2].into_iter() |> .collect::<Vec<u8>> }, [1, 2]);
}

#[test]
fn field_sections() {
    let pairs = [(1, 'a'), (2, 'b')];
    assert_eq!(ml! { pairs |> .iter() |> .map(.0) |> .sum::<i64>() }, 3);
    assert_eq!(ml! { (1, 'z') |> .1 }, 'z');
    let first: fn((i64, char)) -> i64 = ml! { .0 };
    assert_eq!(first((7, 'x')), 7);
    assert_eq!(ml! { .0.1 ((1, 2), 3) }, 2);
}

#[test]
fn std_macros_take_ocaml() {
    let xs = [1, 2, 3];
    ml! { assert_eq!(add 1 2, 3) };
    ml! { assert!(xs |> .len = 3, "{} items", xs |> .len) };
    ml! { assert_ne!(xs |> .iter() |> .sum::<i64>(), 0) };
    ml! { debug_assert!(not (xs.is_empty())) };
    assert_eq!(ml! { format!("{} {n}", g 1, n = h 2) }, "2 4");
    assert_eq!(ml! { vec![zero (); xs |> .len] }, [0, 0, 0]);
    assert_eq!(ml! { vec![g 1, h 2] }, [2, 4]);
    let mut out = String::new();
    ml! { std::fmt::Write::write_fmt(&mut out, format_args!("{}", g 0)) |> .unwrap() };
    assert_eq!(out, "1");
    let m = ml! { std::collections::HashMap::<i64, i64>::new () };
    assert!(ml! { format!("{:?}", m) } == "{}");
}

rsml! {
    fn sum3(a: i64, b: i64, c: i64) -> i64 {
        a + b + c
    }

    fn adder3(n: i64) -> impl Fn(i64) -> i64 {
        fun m -> n + m
    }

    fn sibling_partial() -> Vec<i64> {
        [|1; 2|] |> .iter() |> .copied() |> .map(sum3 10 20) |> .collect()
    }

    fn sibling_groups() -> i64 {
        adder3 1 2
    }

    struct Meters(f64);

    impl Meters {
        fn new(a: f64, b: f64) -> Meters {
            Meters(a +. b)
        }

        fn from_parts() -> Meters {
            let half = Self::new 0.5 in
            half 0.5
        }
    }

    #[rust]
    fn rust_body(xs: &[i64]) -> i64 {
        let mut total = 0;
        for x in xs {
            total += *x;
        }
        if let Some(first) = xs.first() {
            total * first
        } else {
            0
        }
    }

    fn derefs(p: &mut i64, q: &&i64) -> i64 {
        *p <- *p + **q;
        *p
    }
}

#[test]
fn functions_of_the_same_rsml() {
    assert_eq!(sibling_partial(), [31, 32]);
    assert_eq!(sibling_groups(), 3);
    assert_eq!(Meters::from_parts().0 as i64, 1);
    assert_eq!(rust_body(&[2, 3]), 10);
    let mut n = 1;
    assert_eq!(derefs(&mut n, &&2), 3);
}

#![deny(warnings)]

//! Rust's own syntax inside OCaml: calls and indexing written without a
//! space, method sections, `for .. in`, ranges, `return` / `break` /
//! `continue` and `unsafe` / `async` blocks.

use rsml::{ml, rsml};

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn swap(p: (i32, i32)) -> (i32, i32) {
    (p.1, p.0)
}

#[derive(Debug, PartialEq)]
struct Pair(i32, i32);

#[test]
fn calls_written_rust_style() {
    assert_eq!(ml! { add(1, 2) }, 3);
    assert_eq!(ml! { add(add(1, 2), 3) }, 6);
    // `f(a)(b)` calls the result of `f(a)`.
    assert_eq!(ml! { adder(1)(2) }, 3);
    assert_eq!(ml! { adder(1) 2 }, 3);
    // With a space, it is OCaml: `swap (1, 2)` passes one tuple.
    assert_eq!(ml! { swap (1, 2) }, (2, 1));
    assert_eq!(ml! { swap((1, 2)) }, (2, 1));
    // Constructors read the same either way.
    assert_eq!(ml! { Some(1, 2) }, Some((1, 2)));
    assert_eq!(ml! { Pair(1, 2) }, Pair(1, 2));
    let v: Vec<i32> = ml! { Vec::with_capacity(4) };
    assert!(v.capacity() >= 4);
    assert_eq!(ml! { std::cmp::max(add(1, 2), 4) }, 4);
}

#[test]
fn rust_style_calls_of_known_functions() {
    let r = ml! {
        let add3 x y z = x + y + z in
        let add1 = add3(0, 1) in
        add3(1, 2, 3) + add1(10) + add3(1)(1)(1)
    };
    assert_eq!(r, 6 + 11 + 3);
    let curried = ml! {
        let mul x y = x * y in
        mul(3)
    };
    assert_eq!(curried(4), 12);
    // Extra arguments apply the result.
    let r = ml! {
        let make x = fun y -> x + y in
        make(1, 2)
    };
    assert_eq!(r, 3);
    assert_eq!(
        ml! {
            external add : _ -> _ -> _ ;;
            (add(5) >> adder(1))(1)
        },
        7
    );
    assert_eq!(ml! { (fun x y -> x * y)(6, 7) }, 42);
}

#[test]
fn rust_style_calls_in_pipelines() {
    // A placeholder puts the piped value into the call.
    assert_eq!(ml! { 5 |> add(_, 1) }, 6);
    assert_eq!(ml! { 5 |> add(10, _) |> adder(1) }, 16);
    let xs = [1, 2, 3];
    assert_eq!(
        ml! { xs |> .iter() |> .map(fun x -> add(!x, 1)) |> .sum::<i32>() },
        9
    );
}

#[test]
fn method_sections() {
    let xs = [1, 2, 3];
    assert_eq!(ml! { xs |> .iter() |> .count() }, 3);
    assert_eq!(
        ml! { xs |> .iter() |> .map(fun x -> x * 2) |> .sum::<i32>() },
        12
    );
    assert_eq!(
        ml! { xs |> .iter() |> .copied() |> .filter(fun x -> !x > 1) |> .collect::<Vec<_>>() },
        vec![2, 3]
    );
    let mut v = vec![1];
    ml! {
        .push(2) v;
        v |> .push(3)
    };
    assert_eq!(v, [1, 2, 3]);
    // As a value, a function of the receiver.
    let len: fn(&str) -> usize = ml! { .len() };
    assert_eq!(len("abc"), 3);
    let words = ["a", "bb"];
    assert_eq!(
        ml! { words |> .iter() |> .map(.len() (_ : &&str)) |> .sum::<usize>() },
        3
    );
    let count: fn(&Vec<i32>) -> usize = ml! { .iter() >> .count() };
    assert_eq!(count(&v), 3);
}

#[test]
fn sections_glued_to_pipes() {
    // Rust lexes `|>.` as one operator; none is defined, so it is `|>` and a
    // section. (Glued on purpose: the formatter would space them.)
    let xs = [1, 2, 3];
    let pairs = [(1, 'a'), (2, 'b')];
    let r = ml! {
        // rsmlfmt: skip
        xs |>.iter() |>.map(fun x -> x * 2) |>.sum::<i32>()
    };
    assert_eq!(r, 12);
    let r = ml! {
        // rsmlfmt: skip
        pairs |>.iter |>.map _ (.0) |>.sum::<i32>
    };
    assert_eq!(r, 3);
    let r = ml! {
        // rsmlfmt: skip
        (pairs[1] |>.1, (.iter >>.count) xs, Some @@.len xs)
    };
    assert_eq!(r, ('b', 3, Some(3)));
    // A defined `( |>. )` is that operator.
    let r = ml! {
        // rsmlfmt: skip
        let ( |>. ) a b = a * 10 + b in 1 |>.2
    };
    assert_eq!(r, 12);
}

#[test]
fn rust_style_indexing() {
    let v = [10, 20, 30];
    assert_eq!(ml! { v[0] + v[2] }, 40);
    assert_eq!(ml! { v[1..] }, [20, 30]);
    assert_eq!(ml! { v[..=1] }, [10, 20]);
    let m = std::collections::HashMap::from([("k", 7)]);
    assert_eq!(ml! { m[&"k"] }, 7);
    let grid = [[1, 2], [3, 4]];
    assert_eq!(ml! { grid[1][0] }, 3);
    let rows = || vec![vec![5, 6]];
    assert_eq!(ml! { rows()[0][1] }, 6);
    let mut a = [0, 0];
    ml! { a[1] <- 5 };
    assert_eq!(a, [0, 5]);
    // With a space, a list argument.
    let first = |l: Vec<i32>| l[0];
    assert_eq!(ml! { first [4; 5] }, 4);
}

#[test]
fn for_in_loops() {
    let mut total = 0;
    ml! { for x in [1; 2; 3] do total <- total + x done };
    assert_eq!(total, 6);
    let pairs = [(1, 'a'), (2, 'b')];
    let mut seen = Vec::new();
    ml! { for (n, c) in pairs do seen.push(format!("{}{}", n, c)) done };
    assert_eq!(seen, ["1a", "2b"]);
    let mut keys = Vec::new();
    let map = std::collections::BTreeMap::from([(2, "two"), (1, "one")]);
    ml! { for k, _ in &map do keys.push(!k) done };
    assert_eq!(keys, [1, 2]);
}

#[test]
fn ranges() {
    let n = 4;
    assert_eq!(ml! { 0..n |> .sum::<i32>() }, 6);
    assert_eq!(ml! { (1..=n).product::<i32>() }, 24);
    assert_eq!(ml! { 0..n + 1 }, 0..5);
    assert_eq!(ml! { .. }, ..);
    assert_eq!(ml! { ..n }, ..4);
    assert_eq!(ml! { 2.. }, 2..);
    let mut squares = Vec::new();
    ml! { for i in 0..n do squares.push(i * i) done };
    assert_eq!(squares, [0, 1, 4, 9]);
    // In patterns `..` is OCaml's inclusive range; `..=` is accepted too.
    // Both spacings of `..` are on purpose: the formatter would make them one.
    let classify = |c: char, n: i32| {
        ml! {
            // rsmlfmt: skip
            match (c, n) with
            | ('a' .. 'z', 1..=3) -> "lower, small"
            | ('a'..'z', _) -> "lower"
            | _ -> "other"
        }
    };
    assert_eq!(classify('z', 3), "lower, small");
    assert_eq!(classify('q', 9), "lower");
    assert_eq!(classify('Q', 1), "other");
}

rsml! {
    fn first_negative(xs: &[i32]) -> Option<usize> {
        for (i, x) in xs.iter().enumerate() do if !x < 0 then return Some(i) done;
        None
    }

    fn sum_until_zero(xs: &[i32]) -> i32 {
        let mut total = 0 in
        let mut i = 0 in
        while true do
          if i = xs.len() || xs[i] = 0 then break;
          i <- i + 1;
          if xs[i - 1] < 0 then continue;
          total <- total + xs[i - 1]
        done;
        total
    }

    fn find_power(base: u32, limit: u32) -> u32 {
        let mut p = 1 in
        while true do
          if p * base > limit then return p;
          p <- p * base
        done
    }

    fn first_even(xs: &[i32]) -> i32 {
        let mut it = xs.iter() in
        while true do
          match it.next() with
          | Some x when !x mod 2 = 0 -> break !x
          | Some _ -> continue
          | None -> break -1
        done
    }
}

#[test]
fn return_break_continue() {
    assert_eq!(first_negative(&[1, -2, 3]), Some(1));
    assert_eq!(first_negative(&[1]), None);
    assert_eq!(sum_until_zero(&[1, -5, 2, 0, 9]), 3);
    assert_eq!(find_power(2, 100), 64);
    assert_eq!(first_even(&[1, 3, 4, 6]), 4);
    assert_eq!(first_even(&[1]), -1);
    let early = |n: i32| -> i32 {
        ml! {
            if n < 0 then return 0;
            n * 2
        }
    };
    assert_eq!(early(-1), 0);
    assert_eq!(early(2), 4);
}

unsafe fn unchecked_double(x: i32) -> i32 {
    x * 2
}

#[test]
fn unsafe_and_async_blocks() {
    assert_eq!(ml! { unsafe { unchecked_double 21 } }, 42);
    assert_eq!(
        ml! {
            unsafe {
              let y = 2 in
              unchecked_double(y)
            }
        },
        4
    );
    let n = 5;
    let fut = ml! { async { n + 1 } };
    assert_eq!(block_on(fut), 6);
    let fut = ml! {
        async move {
          let m = n * 2 in
          m
        }
    };
    assert_eq!(block_on(fut), 10);
    let nested = ml! { async { (async { 1 }).await + 1 } };
    assert_eq!(block_on(nested), 2);
}

/// Polls a future that is ready without waiting.
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    let mut fut = std::pin::pin!(fut);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    match fut.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(v) => v,
        std::task::Poll::Pending => panic!("not ready"),
    }
}

#[test]
fn assignments_like_rust_compound_ones() {
    // `x <- x + e` is `x = x + e`, which clippy accepts here (see
    // scripts/check_clippy.sh): OCaml has no `+=`.
    let mut x = 1;
    let r = ml! {
        let c = ref 10 in
        x <- x + 1;
        x <- 2 * x;
        c := !c - 1;
        incr c;
        !c + x
    };
    assert_eq!((r, x), (14, 4));
}

#[derive(Debug, PartialEq)]
struct Wrapper(i64);

impl Wrapper {
    fn doubled(&self) -> i64 {
        self.0 * 2
    }
}

#[test]
fn adjacent_constructor_applications() {
    let x = 3;
    assert_eq!(
        ml! { Some(x).unwrap() + Wrapper(x).doubled() + Wrapper(x).0 },
        12
    );
    assert_eq!(ml! { Some(x).map(fun n -> n + 1) }, Some(4));
    // Spaced, the method is the argument's.
    let s = "ab";
    assert_eq!(ml! { Some (s).len() }, Some(2));
}

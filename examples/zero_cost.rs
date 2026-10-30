//! Pairs of functions computing the same thing, once with `ml!` and once in
//! hand-written Rust. Compare the generated code with
//!
//! ```sh
//! cargo rustc --release --example zero_cost -- --emit=asm
//! ```
//!
//! and look for the `ml_*` / `rs_*` symbols in
//! `target/release/examples/zero_cost-*.s`: each pair compiles to identical
//! machine code.

use rsml::ml;

#[inline]
fn add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

#[inline]
fn double(x: i32) -> i32 {
    x.wrapping_mul(2)
}

// Pipeline: `|>` is a direct call.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_pipe(x: i32) -> i32 {
    ml! { x |> double |> add 1 |> double }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_pipe(x: i32) -> i32 {
    double(add(1, double(x)))
}

// Composition and partial application inside an iterator pipeline.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_compose(xs: &[i32], k: i32) -> i32 {
    ml! {
        let step = double >> add k _ in
        xs
        |> .iter
        |> .map _ (fun x -> step !x)
        |> .filter _ (fun x -> !x land 1 = 1)
        |> .fold _ 0 add
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_compose(xs: &[i32], k: i32) -> i32 {
    let step = move |x| add(k, double(x));
    xs.iter()
        .map(|x| step(*x))
        .filter(|x| *x & 1 == 1)
        .fold(0, add)
}

// A local curried function, partially applied and passed to `Iterator::map`.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_partial(xs: &[i32], k: i32) -> Vec<i32> {
    ml! {
        let scale a b = a * b + 1 in
        xs |> .iter |> .map _ (scale k) |> .collect
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_partial(xs: &[i32], k: i32) -> Vec<i32> {
    let scale = |a: i32, b: &i32| a * b + 1;
    xs.iter().map(move |b| scale(k, b)).collect()
}

// Pattern matching on a list and recursion: a tail call is a loop.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_sum(xs: &[i64]) -> i64 {
    ml! {
        let rec go (acc : i64) (l : &[i64]) : i64 =
          match l with [] -> acc | x :: rest -> go (acc + !x) rest
        in
        go 0 xs
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_sum(xs: &[i64]) -> i64 {
    fn go(mut acc: i64, mut l: &[i64]) -> i64 {
        loop {
            match l {
                [] => return acc,
                [x, rest @ ..] => {
                    acc += *x;
                    l = rest;
                }
            }
        }
    }
    go(0, xs)
}

#[inline]
fn sub(a: i32, b: i32) -> i32 {
    a.wrapping_sub(b)
}

// A section: `sub _ k` is `|x| sub(x, k)`, and `(sub _1 _0) a b` is `sub(b, a)`.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_section(xs: &[i32], k: i32) -> i32 {
    ml! { xs |> .iter |> .map _ (!_) |> .map _ (sub _ k) |> .fold _ 0 (add _1 _0) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_section(xs: &[i32], k: i32) -> i32 {
    xs.iter()
        .copied()
        .map(|x| sub(x, k))
        .fold(0, |a, b| add(b, a))
}

// A function whose whole body is OCaml.
rsml::rsml! {
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn ml_item(x: i32) -> i32 {
        x |> double |> add 1
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_item(x: i32) -> i32 {
    add(1, double(x))
}

// Rust-style calls: a local function called with all its arguments is a
// direct call, and `add(_, k)` in a pipeline is `add(x, k)`.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_rust_call(x: i32, k: i32) -> i32 {
    ml! {
        let scale (a : i32) b = a.wrapping_mul(b) in
        add(scale(k, x), double(x)) |> add(_, k) |> double
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_rust_call(x: i32, k: i32) -> i32 {
    let scale = |a: i32, b: i32| a.wrapping_mul(b);
    double(add(add(scale(k, x), double(x)), k))
}

// Method sections: `xs |> .map(f)` is `xs.map(f)`.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_method_section(xs: &[i32], k: i32) -> i32 {
    ml! { xs |> .iter() |> .copied() |> .map(add(_, k)) |> .fold(0, add) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_method_section(xs: &[i32], k: i32) -> i32 {
    xs.iter().copied().map(|x| add(x, k)).fold(0, add)
}

// A range in a pipeline.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_range(n: i32) -> i32 {
    ml! { 0..n |> .map(fun i -> double i) |> .filter(fun i -> !i land 3 = 0) |> .fold(0, add) }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_range(n: i32) -> i32 {
    (0..n).map(double).filter(|i| *i & 3 == 0).fold(0, add)
}

// `for .. in` over a slice, with a `ref` accumulator.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_for_in(xs: &[i32]) -> i32 {
    ml! {
        let total = ref 0 in
        for x in xs do total := add !total (double !x) done;
        !total
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_for_in(xs: &[i32]) -> i32 {
    let mut total = 0;
    for x in xs {
        total = add(total, double(*x));
    }
    total
}

// `while true` is `loop`, with `break`.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn ml_loop(limit: u32) -> u32 {
    ml! {
        let p = ref 1 in
        while true do
          if !p > limit / 2 then break;
          p := !p * 2
        done;
        !p
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn rs_loop(limit: u32) -> u32 {
    let mut p = 1;
    loop {
        if p > limit / 2 {
            break;
        }
        p *= 2;
    }
    p
}

fn main() {
    let xs = [1, 2, 3, 4, 5];
    assert_eq!(ml_pipe(3), rs_pipe(3));
    assert_eq!(ml_compose(&xs, 3), rs_compose(&xs, 3));
    assert_eq!(ml_partial(&xs, 3), rs_partial(&xs, 3));
    assert_eq!(ml_sum(&[1, 2, 3]), rs_sum(&[1, 2, 3]));
    assert_eq!(ml_section(&xs, 3), rs_section(&xs, 3));
    assert_eq!(ml_item(3), rs_item(3));
    assert_eq!(ml_rust_call(3, 4), rs_rust_call(3, 4));
    assert_eq!(ml_method_section(&xs, 3), rs_method_section(&xs, 3));
    assert_eq!(ml_range(10), rs_range(10));
    assert_eq!(ml_for_in(&xs), rs_for_in(&xs));
    assert_eq!(ml_loop(100), rs_loop(100));
    println!(
        "pipe {} | compose {} | partial {:?} | sum {}",
        ml_pipe(3),
        ml_compose(&xs, 3),
        ml_partial(&xs, 3),
        ml_sum(&[1, 2, 3])
    );
}

//! `rsml!` expansion time must grow linearly with the number of functions
//! (it used to grow quadratically: every body got its own copy of the
//! functions it could call). A timing test, so it is ignored by default:
//!
//! ```sh
//! scripts/bench_expand.sh      # cargo test --release -p rsml-compiler --test scaling -- --ignored --nocapture
//! ```

use std::time::{Duration, Instant};

/// `pairs` pairs of functions, about 11 lines each, calling each other by
/// name, as a whole file in one `rsml!` would.
fn block(pairs: usize) -> String {
    let mut s = String::new();
    for i in 0..pairs {
        let h = i.saturating_sub(1);
        s += &format!(
            "fn f{i}(xs: Vec<i64>, k: i64) -> i64 {{
    let ys = xs |> .iter() |> .map(fun x -> x * k + {i}) |> .collect::<Vec<_>>() in
    match ys.first() with
    | Some y when !y > 0 -> !y + f{h}_helper k
    | _ -> ys.len() as i64
}}

fn f{i}_helper(k: i64) -> i64 {{
    if k > 10 then k else k * 2
}}

"
        );
    }
    s
}

/// The fastest of a few expansions of `src`.
fn time(src: &str) -> Duration {
    let ts: proc_macro2::TokenStream = src.parse().unwrap();
    (0..5)
        .map(|_| {
            let input = ts.clone();
            let t = Instant::now();
            let out = rsml_compiler::expand_items(input);
            let d = t.elapsed();
            assert!(!out.to_string().contains("compile_error"));
            d
        })
        .min()
        .unwrap()
}

#[test]
#[ignore = "timing; run with --release (scripts/bench_expand.sh)"]
fn expansion_is_linear_in_the_number_of_functions() {
    let small = time(&block(150));
    let big = time(&block(1200));
    let ratio = big.as_secs_f64() / small.as_secs_f64();
    println!("300 functions: {small:?}, 2400 functions: {big:?}, ratio {ratio:.1} (linear: 8)");
    // Linear is ×8; the quadratic version gave ×30 and more.
    assert!(
        ratio < 14.0,
        "expansion grows faster than linearly: ×{ratio:.1} for ×8 the functions"
    );
}

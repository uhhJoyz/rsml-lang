#![deny(warnings)]

use rsml::rsml;

rsml! {
    fn sum_of_odd_squares(xs: &[i64]) -> i64 {
        let square x = x * x in
        xs |> .iter |> .map _ (fun x -> square !x) |> .filter _ (fun x -> !x mod 2 = 1) |> .sum
    }

    pub fn length(l: &[i32]) -> i64 {
        let rec go (acc : i64) (l : &[i32]) : i64 =
          match l with [] -> acc | _ :: rest -> go (acc + 1) rest
        in
        go 0 l
    }

    fn minus(x: i64, y: i64) -> i64 {
        x - y
    }

    /// The parameters of a function of the same `rsml!` are labels.
    fn labeled() -> i64 {
        minus ~y:2 ~x:5
    }

    /// Toplevel phrases work as in `ml!`.
    fn describe(n: i64) -> String {
        let sign = if n < 0 then "negative" else "non-negative"
        let parity = if n mod 2 = 0 then "even" else "odd" ;;
        format!("{} and {}", sign, parity)
    }

    // Parameters are bound names, so `h::t` is a cons, not the path `h::t`.
    // (Written glued on purpose: the formatter would space it.)
    fn prepend(h: i64, t: Vec<i64>) -> Vec<i64> {
        // rsmlfmt: skip
        h::t
    }

    // Closure-typed parameters can be partially applied.
    fn add_all(add: impl Fn(i64, i64) -> i64, xs: Vec<i64>) -> Vec<i64> {
        xs |> .into_iter |> .map _ (add 10) |> .collect
    }

    fn twice<F>(f: F) -> impl Fn(i64) -> i64
    where
        F: Fn(i64) -> i64 + Copy,
    {
        f >> f
    }

    struct Counter {
        n: i64,
    }

    impl Counter {
        fn bumped(&self, by: i64) -> Counter {
            Counter { n = self.n + by }
        }

        fn get(&self) -> i64 {
            self.n
        }
    }

    trait Describe {
        fn name(&self) -> String;

        fn shout(&self) -> String {
            self |> .name |> .to_uppercase
        }
    }

    impl Describe for Counter {
        fn name(&self) -> String {
            format!("counter {}", self.n)
        }
    }
}

#[test]
fn functions_with_ocaml_bodies() {
    assert_eq!(sum_of_odd_squares(&[1, 2, 3, 4, 5]), 35);
    assert_eq!(length(&[1, 2, 3]), 3);
    assert_eq!(describe(-3), "negative and odd");
    assert_eq!(prepend(1, vec![2]), vec![1, 2]);
    assert_eq!(labeled(), 3);
    assert_eq!(add_all(|a, b| a + b, vec![1, 2]), vec![11, 12]);
    assert_eq!(twice(|x| x * 3)(2), 18);
}

#[test]
fn methods_and_traits() {
    let c = Counter { n: 1 }.bumped(2);
    assert_eq!(c.get(), 3);
    assert_eq!(c.shout(), "COUNTER 3");
}

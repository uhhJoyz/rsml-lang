// Toy 138: matrices.
//
// What it is: square matrices of integers as lists of rows: the identity,
// the transpose, the product, powers, and Fibonacci numbers as a power.
//
// When to use it: an example of ranges piped into nested closures (toys
// 088 and 012), of a placeholder that indexes (`_[j]`, toy 041), and of
// nested list literals (toy 096).
//
// How to write it: each entry is computed by a pipeline over a range of
// indices; `(0..n) |> .map(fun i -> ..)` makes the rows.
//
// Reference: [Ranges](../../docs/notation.md#ranges)

use rsml::rsml;

pub const SUMMARY: &str = "Matrices: ranges into nested closures, `_[j]`, `[[1; 1]; [1; 0]]`";

rsml! {
pub type Matrix = Vec<Vec<i64>>;

/// The `n` by `n` identity matrix.
pub fn identity(n: usize) -> Matrix {
    (0..n)
    |> .map(fun i -> (0..n) |> .map(fun j -> if i = j then 1 else 0) |> .collect())
    |> .collect()
}

/// The transpose: the rows become the columns.
pub fn transpose(a: &Matrix) -> Matrix {
    // `.map(_[j])` is `.map(fun row -> row[j])` (toy 040).
    (0..a[0].len()) |> .map(fun j -> a |> .iter() |> .map(_[j]) |> .collect()) |> .collect()
}

/// The product: entry (i, j) is row `i` of `a` times column `j` of `b`.
pub fn mul(a: &Matrix, b: &Matrix) -> Matrix {
    let n = a.len() in
    // The indices are annotated: rustc must know a closure
    // parameter's type before it indexes with it.
    let entry (i : usize) (j : usize) = (0..n) |> .map(fun k -> a[i][k] * b[k][j]) |> .sum::<i64>() in
    (0..n) |> .map(fun i -> (0..n) |> .map(fun j -> entry i j) |> .collect()) |> .collect()
}

/// `m` to the power `k`.
pub fn power(m: &Matrix, k: u32) -> Matrix {
    let mut acc = identity m.len() in
    // `for _ = 1 to k` runs `k` times.
    for _ = 1 to k do acc <- mul &acc m done;
    acc
}

/// The `n`th Fibonacci number, a corner of `[[1; 1]; [1; 0]]` to the
/// power `n`.
pub fn fibonacci(n: u32) -> i64 {
    // `(power m n)[0][1]`: indexing right after the parentheses (toy 055).
    (power &[[1; 1]; [1; 0]] n)[0][1]
}

/// The rows as lines of numbers.
pub fn to_text(m: &Matrix) -> String {
    let row (r : &Vec<i64>) =
      r |> .iter() |> .map(_.to_string()) |> .collect::<Vec<_>>() |> .join(" ")
    in
    m |> .iter() |> .map(fun r -> row r) |> .collect::<Vec<_>>() |> .join("\n")
}
}

pub fn run() {
    let m = vec![vec![1, 2], vec![3, 4]];
    println!("m:\n{}", to_text(&m));
    println!("transpose m:\n{}", to_text(&transpose(&m)));
    println!("m * m:\n{}", to_text(&mul(&m, &m)));
    println!("fibonacci 10 = {}", fibonacci(10));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operations() {
        let m = vec![vec![1, 2], vec![3, 4]];
        assert_eq!(identity(2), [[1, 0], [0, 1]]);
        assert_eq!(transpose(&m), [[1, 3], [2, 4]]);
        assert_eq!(mul(&m, &m), [[7, 10], [15, 22]]);
        assert_eq!(mul(&m, &identity(2)), m);
        assert_eq!(power(&m, 0), identity(2));
    }

    #[test]
    fn fibonacci_numbers() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
    }

    #[test]
    fn text() {
        assert_eq!(to_text(&vec![vec![1, 2], vec![3, 4]]), "1 2\n3 4");
    }
}

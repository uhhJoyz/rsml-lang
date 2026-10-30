// Toy 139: the sieve of Eratosthenes.
//
// What it is: the primes up to a limit, found by crossing out the
// multiples of each prime in a table of booleans; also prime factors and
// twin primes.
//
// When to use it: an example of `while` loops (toy 084) that assign to
// elements of a table (`is_prime[m] <- false`, toy 090), of `ref`
// counters (toy 089), and of slice patterns in a `function` (toy 070).
//
// How to write it: the table is `vec![true; limit + 1]` (toy 107); the
// primes are the indices still `true`, read with a filter over a range.
//
// Reference: [Control flow](../../docs/notation.md#control-flow)

use rsml::rsml;

pub const SUMMARY: &str = "A prime sieve: `while`, `<-` on elements, `ref`, slice patterns";

rsml! {
    /// The primes up to `limit`.
    pub fn primes(limit: usize) -> Vec<usize> {
        let mut is_prime = vec![true; limit + 1] in
        let mut p = 2 in
        while p * p <= limit do
          if is_prime[p] then begin
            // Cross out `p * p`, `p * p + p`, ..: the smaller multiples
            // of `p` are already crossed out.
            let m = ref (p * p) in
            while !m <= limit do
              is_prime[!m] <- false;
              m := !m + p
            done
          end;
          p <- p + 1
        done;
        // The filter's closure gets a reference: `!n` reads the index.
        2..=limit |> .filter(fun n -> is_prime[!n]) |> .collect()
    }

    /// The prime factors of `n`, smallest first.
    pub fn factors(n: u64) -> Vec<u64> {
        let mut n = n in
        let mut d = 2 in
        let mut out = Vec.new () in
        while d * d <= n do
          if n mod d = 0 then begin
            out.push(d);
            n <- n / d
          end
          else
            d <- d + 1
        done;
        if n > 1 then out.push(n);
        out
    }

    /// The pairs of primes two apart, up to `limit`.
    pub fn twins(limit: usize) -> Vec<(usize, usize)> {
        // `.windows(2)` gives slices of two primes; the `function` takes
        // one apart with the list pattern `[a; b]` (toy 070).
        primes limit
        |> .windows(2)
        |> .filter_map(function [a; b] when !b - !a = 2 -> Some (!a, !b) | _ -> None)
        |> .collect()
    }
}

pub fn run() {
    println!("primes 30 = {:?}", primes(30));
    println!("factors 360 = {:?}", factors(360));
    println!("twins 20 = {:?}", twins(20));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sieve() {
        assert_eq!(primes(30), [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
        assert!(primes(1).is_empty());
        assert_eq!(primes(2), [2]);
    }

    #[test]
    fn factors_and_twins() {
        assert_eq!(factors(360), [2, 2, 2, 3, 3, 5]);
        assert_eq!(factors(97), [97]);
        assert_eq!(twins(20), [(3, 5), (5, 7), (11, 13), (17, 19)]);
    }
}

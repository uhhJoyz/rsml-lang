// Toy 046: method chains as pipelines, `xs |> .iter |> .map _ f`.
//
// What it is: a method as a function (toy 045) takes its receiver first,
// and `|>` passes the value last, so a `_` marks the receiver's place
// when the method has other arguments: `it |> .map _ f` is `it.map(f)`.
// Without other arguments, `xs |> .iter` is `xs.iter()`. A chain of such
// steps is a Rust method chain, read left to right (or top to bottom,
// one step per line, when it is long).
//
// When to use it: for iterator chains.
//
// How to write it:
// * `xs |> .iter |> .count` is `xs.iter().count()`.
// * `it |> .filter _ p` is `it.filter(p)`: `_` is the receiver.
// * Generic arguments go after the name: `it |> .sum::<i64>` is
//   `it.sum::<i64>()`, and `.collect::<Vec<_>>` likewise.
// * Toy 048 writes the same steps Rust style: `|> .map(f)`.
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name)

use rsml::ml;

pub const SUMMARY: &str = "`xs |> .iter |> .map _ f |> .sum::<i64>`: a method chain";

/// The total of the scores that passed, doubled.
pub fn bonus_total(scores: &[i64]) -> i64 {
    ml! {
        // `scores |> .iter |> .copied |> .filter _ f |> .map _ g |> .sum::<i64>`
        // is `scores.iter().copied().filter(f).map(g).sum::<i64>()`.
        scores |> .iter |> .copied |> .filter _ (fun s -> !s >= 50) |> .map _ (_ * 2) |> .sum::<i64>
    }
}

/// The names in capitals, as a list.
pub fn shouted(names: &[&str]) -> Vec<String> {
    ml! {
        // `it |> .collect::<Vec<_>>` is `it.collect::<Vec<_>>()`.
        names |> .iter |> .map _ (fun n -> n.to_uppercase()) |> .collect::<Vec<_>>
    }
}

/// The names joined: `.join _ ", "` puts the list in the `_`.
pub fn listed(names: &[&str]) -> String {
    ml! {
        // `names |> .join _ ", "` is `names.join(", ")`.
        names |> .join _ ", "
    }
}

pub fn run() {
    println!("bonus_total [40; 50; 90] = {}", bonus_total(&[40, 50, 90]));
    println!(
        "shouted [\"ada\"; \"alan\"] = {:?}",
        shouted(&["ada", "alan"])
    );
    println!(
        "listed [\"ada\"; \"alan\"] = {:?}",
        listed(&["ada", "alan"])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chains() {
        assert_eq!(bonus_total(&[40, 50, 90]), 280);
        assert_eq!(shouted(&["ada", "alan"]), ["ADA", "ALAN"]);
        assert_eq!(listed(&["ada", "alan"]), "ada, alan");
    }

    #[test]
    fn the_header_example() {
        let xs = [1, 2, 3];
        assert_eq!(ml! { xs |> .iter |> .count }, 3);
    }
}

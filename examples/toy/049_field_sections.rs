// Toy 049: field sections, `.0`, `.1`.
//
// What it is: where an expression starts, `.0`, `.1`, .. are the fields
// of a tuple as functions of the tuple, like methods (toy 047): `.0` is
// `|tuple| tuple.0`, and applied, `x |> .1` is `x.1`, with no closure.
//
// When to use it: to take one column out of pairs in a pipeline.
//
// How to write it:
// * `pairs |> .iter() |> .map(.0)` is
//   `pairs.iter().map(|tuple| tuple.0)`.
// * `.1 t` is `t.1`, and `.0.1 t` is `t.0.1`.
// * More arguments apply the field: `.0 t x` is `(t.0)(x)`.
// * Only numbered fields: `.name` is always a method, `.name` is
//   `|receiver| receiver.name()`.
//
// Reference: [Methods as functions: .name](../../docs/notation.md#methods-as-functions-name)

use rsml::ml;

pub const SUMMARY: &str = "Tuple fields as functions: `.map(.0)`, `x |> .1`, `.0.1 t`";

/// The names of the people, from (name, age) pairs.
pub fn names<'a>(people: &[(&'a str, u32)]) -> Vec<&'a str> {
    ml! {
        // `.map(.0)` takes each pair's first field.
        people |> .iter() |> .map(.0) |> .collect()
    }
}

/// The age of the oldest person, if there is anyone.
pub fn oldest(people: &[(&str, u32)]) -> Option<u32> {
    ml! {
        // `p |> .1` is `p.1`: a field section applied.
        people |> .iter() |> .map(fun p -> p |> .1) |> .max()
    }
}

/// The city of a record (name, (city, zip)).
pub fn city<'a>(record: &(&str, (&'a str, u32))) -> &'a str {
    ml! {
        // `.1.0 record` is `record.1.0`.
        .1.0 record
    }
}

pub fn run() {
    let people = [("Ada", 36), ("Alan", 41)];
    println!("names {people:?} = {:?}", names(&people));
    println!("oldest {people:?} = {:?}", oldest(&people));
    println!(
        "city (\"Ada\", (\"London\", 1)) = {}",
        city(&("Ada", ("London", 1)))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_as_functions() {
        let people = [("Ada", 36), ("Alan", 41)];
        assert_eq!(names(&people), ["Ada", "Alan"]);
        assert_eq!(oldest(&people), Some(41));
        assert_eq!(oldest(&[]), None);
        assert_eq!(city(&("Ada", ("London", 1))), "London");
    }

    #[test]
    fn a_field_that_is_a_function() {
        let t = (|n: i32| n + 1, "inc");
        assert_eq!(ml! { .0 t 41 }, 42);
    }
}

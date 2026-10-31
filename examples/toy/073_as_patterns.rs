// Toy 073: `as` patterns, `p as name`.
//
// What it is: `p as name` matches `p` and also binds the whole matched
// value to `name`; Rust writes it `name @ p`.
//
// When to use it: when a case needs both the parts and the whole: the
// first reading and the list of all of them, a coupon and its value, a
// pair and its sides.
//
// How to write it:
// * `as` binds loosest of all pattern operators, so it names the whole:
//   `match xs with x :: _ as all -> all | _ -> xs` is
//   `match xs with (x :: _) as all -> all | _ -> xs`.
// * So to name a part, put the `as` in parentheses with it:
//   `match e with (((a, b) as size), name) -> name` is
//   `match e { (size @ (a, b), name) => name, }`, while
//   `match e with (name, (a, b) as size) -> name` is
//   `match e { size @ (name, (a, b)) => name, }`.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::ml;

pub const SUMMARY: &str = "`as` patterns: `Some n as found`, `x :: _ as all` name the whole";

/// The first reading and how many there are.
pub fn summary(readings: &[i32]) -> String { ml! {
    // `match xs with x :: _ as all -> all.len() | [] -> 0` is
    // `match ::rsml::__rt::as_slice(&xs) { all @ [x, ..] => all.len(), [] => 0, }`.
    match readings with
    | first :: _ as all -> format!("{} readings, starting at {first}", all.len())
    | [] -> String.from "no readings"
}}

/// A coupon worth keeping: one for at least 10%, handed back whole.
pub fn keep(coupon: Option<u32>) -> Option<u32> { ml! {
    // `match o with Some n as found -> found | None -> None` is
    // `match o { found @ Some(n) => found, None => None, }`.
    match coupon with Some n as found when n >= 10 -> found | _ -> None
}}

/// A pair with its larger side, naming the pair inside a bigger pattern.
pub fn labelled(entry: ((u32, u32), &str)) -> String { ml! {
    match entry with (((a, b) as size), name) -> format!("{name} {size:?}: {}", std::cmp::max a b)
}}

pub fn run() {
    println!("summary [18; 21] = {:?}", summary(&[18, 21]));
    println!("summary [] = {:?}", summary(&[]));
    println!(
        "keep (Some 15) = {:?}, keep (Some 5) = {:?}",
        keep(Some(15)),
        keep(Some(5))
    );
    println!(
        "labelled ((3, 8), \"box\") = {:?}",
        labelled(((3, 8), "box"))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_and_parts() {
        assert_eq!(summary(&[18, 21]), "2 readings, starting at 18");
        assert_eq!(summary(&[]), "no readings");
        assert_eq!(keep(Some(15)), Some(15));
        assert_eq!(keep(Some(5)), None);
        assert_eq!(keep(None), None);
        assert_eq!(labelled(((3, 8), "box")), "box (3, 8): 8");
    }
}

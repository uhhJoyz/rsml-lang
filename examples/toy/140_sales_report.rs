// Toy 140: a sales report.
//
// What it is: a day's sales as records, and a report computed from them:
// the revenue of each region, the best-selling item and the average
// price.
//
// When to use it: an example of iterator pipelines over records (toy 046)
// with method sections, placeholders (`_.units`, toy 041) and field
// sections (`.1`, toy 049), of a local function that builds records with
// punning (toys 007 and 094), and of `@` (toy 063).
//
// How to write it: each figure is one pipeline; maps from `BTreeMap`
// keep the regions in order, so the report is the same every time.
//
// Reference: [Records and collections](../../docs/notation.md#records-and-collections)

use rsml::rsml;
use std::collections::BTreeMap;

pub const SUMMARY: &str = "A sales report: pipelines over records, `_.units`, `.1`, `@`";

rsml! {
/// One sale: what, where, how many, at what price in cents.
#[derive(Debug, Clone)]
pub struct Sale {
    pub item: &'static str,
    pub region: &'static str,
    pub units: u32,
    pub cents: u32,
}

/// The sales of the day.
pub fn sales() -> Vec<Sale> {
    // `Sale { item; region; units; cents }` takes each field from the
    // variable of its name.
    let sale item region units cents = Sale { item; region; units; cents } in
    [sale "tea" "north" 3 250;
     sale "coffee" "north" 5 300;
     sale "tea" "south" 10 250;
     sale "cake" "south" 2 400;
     sale "coffee" "south" 1 300]
}

pub fn revenue(s: &Sale) -> u32 {
    s.units * s.cents
}

/// The revenue of each region, in cents.
pub fn by_region(sales: &[Sale]) -> BTreeMap<&'static str, u32> {
    let mut totals = BTreeMap.new () in
    for s in sales do
      let c = totals.entry(s.region).or_insert(0) in
      *c <- *c + revenue s
    done;
    totals
}

/// The item that sold the most units, with the units.
pub fn best_seller(sales: &[Sale]) -> Option<(&'static str, u32)> {
    let mut units = BTreeMap.new () in
    for s in sales do
      let c = units.entry(s.item).or_insert(0) in
      *c <- *c + s.units
    done;
    // `.1` is a function of a pair, its second field.
    units |> .into_iter() |> .max_by_key(.1)
}

/// The average price of the units sold, in cents.
pub fn average_price(sales: &[Sale]) -> f64 {
    let total = sales |> .iter() |> .map(revenue) |> .sum::<u32>() in
    let units = sales |> .iter() |> .map(_.units) |> .sum::<u32>() in
    // `total as f64 /. units as f64` is `(total as f64) / (units as f64)`.
    total as f64 /. units as f64
}

/// The report, one line per region, then the best seller.
pub fn report(sales: &[Sale]) -> Vec<String> {
    let lines =
      by_region sales
      |> .iter()
      |> .map(fun (region, cents) -> format!("{:<6}{:>8.2}", region, !cents as f64 /. 100.0))
      |> .collect::<Vec<_>>()
    in
    match best_seller sales with
    | Some (item, units) -> lines @ [format!("best seller: {} ({} units)", item, units)]
    | None -> lines
}
}

pub fn run() {
    for line in report(&sales()) {
        println!("{line}");
    }
    println!("average price: {:.2} cents", average_price(&sales()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures() {
        let day = sales();
        assert_eq!(
            by_region(&day).into_iter().collect::<Vec<_>>(),
            [("north", 2250), ("south", 3600)]
        );
        assert_eq!(best_seller(&day), Some(("tea", 13)));
        assert!((average_price(&day) - 5850.0 / 21.0).abs() < 1e-9);
        assert_eq!(best_seller(&[]), None);
    }

    #[test]
    fn the_report() {
        assert_eq!(
            report(&sales()),
            [
                "north    22.50",
                "south    36.00",
                "best seller: tea (13 units)"
            ]
        );
    }
}

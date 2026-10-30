// Toy 110: methods with OCaml bodies, `impl` in `rsml!`.
//
// What it is: in an `impl` (or `trait`, toy 111) inside `rsml!`, every
// method body is OCaml, `&self` and `&mut self` methods included. `self`
// is an ordinary name there: `self.balance` reads a field and
// `self.balance <- e` assigns it.
//
// When to use it: for a type's methods, written next to the type.
//
// How to write it:
// * The signature is Rust: `pub fn deposit(&mut self, cents: i64)`.
// * `Self { owner = ..; balance = 0 }` builds the type (toy 093).
// * Call a method with no space before its parenthesis:
//   `acct.deposit(5)` stays `acct.deposit(5)`, while
//   `acct.deposit (5)` is `(acct.deposit)(5)`, a call of a field
//   `deposit`, which rustc rejects. `.deposit acct 5` is the OCaml spelling
//   (toy 045).
//
// Reference: [Arity and partial application](../../docs/notation.md#arity-and-partial-application),
// [Whole functions: rsml!](../../README.md#whole-functions-rsml)

use rsml::rsml;

pub const SUMMARY: &str = "`impl` in `rsml!`: `&self`/`&mut self` methods with OCaml bodies";

rsml! {
    /// A bank account; amounts are in cents.
    #[derive(Debug)]
    pub struct Account {
        owner: String,
        balance: i64,
    }

    impl Account {
        /// A new, empty account.
        pub fn open(owner: &str) -> Self {
            Self { owner = String.from owner; balance = 0 }
        }

        /// `&self`: reads a field.
        pub fn balance(&self) -> i64 {
            self.balance
        }

        /// `&mut self`: `self.balance <- self.balance + cents` assigns.
        pub fn deposit(&mut self, cents: i64) {
            self.balance <- self.balance + cents
        }

        /// Takes money out, if there is enough.
        pub fn withdraw(&mut self, cents: i64) -> Result<(), String> {
            if cents > self.balance then
              Err (format!("{} has only {}", self.owner, self.balance))
            else begin
              self.balance <- self.balance - cents;
              Ok ()
            end
        }
    }

    /// A month of an account, its methods called both ways.
    pub fn month(owner: &str) -> (i64, Result<(), String>) {
        let mut acct = Account::open owner in
        acct.deposit(1200);
        // `.deposit acct 300` is `acct.deposit(300)`.
        .deposit acct 300;
        let refused = acct.withdraw(5000) in
        (acct.balance(), refused)
    }
}

pub fn run() {
    println!("month \"Ada\" = {:?}", month("Ada"));
    let mut acct = Account::open("Bo");
    acct.deposit(100);
    println!(
        "Bo after depositing 100 and withdrawing 40: {:?}",
        (acct.withdraw(40), acct.balance())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods() {
        assert_eq!(month("Ada"), (1500, Err("Ada has only 1500".to_string())));
        let mut acct = Account::open("Bo");
        acct.deposit(100);
        assert_eq!(acct.withdraw(40), Ok(()));
        assert_eq!(acct.balance(), 60);
    }
}

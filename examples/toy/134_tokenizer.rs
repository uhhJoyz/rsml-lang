// Toy 134: a tokenizer.
//
// What it is: turning the text of an arithmetic expression into tokens:
// numbers, names, operators and parentheses, or an error that names the
// character it does not know.
//
// When to use it: the shape of most hand-written scanners: a recursive
// function over a slice of characters whose `match` uses list patterns
// (toy 070), literal and or-patterns (toys 077 and 072), `as` (toy 073)
// and guards (toy 074), and `?` to stop at the first error (toy 119).
//
// How to write it: each case of the `match` handles what the slice starts
// with, then calls the function again on the rest. A case's body runs to
// the next `|`, so `out.push(..); scan rest out` is one case.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::rsml;

pub const SUMMARY: &str = "A tokenizer: slice patterns, or-patterns, guards and `?`";

rsml! {
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Num(i64),
    Name(String),
    Op(char),
    Open,
    Close,
}

/// How many characters at the start of `cs` satisfy `keep`.
fn span(cs: &[char], keep: impl Fn(char) -> bool) -> usize {
    cs |> .iter() |> .copied() |> .take_while(fun c -> keep !c) |> .count()
}

/// The characters as a string.
fn text(cs: &[char]) -> String {
    cs |> .iter() |> .collect()
}

/// Adds the tokens of `cs` to `out`, or says which character is wrong.
fn scan(cs: &[char], out: &mut Vec<Token>) -> Result<(), String> {
    // In a slice's list patterns the elements are references: `op`
    // has type `&char`, so the token takes `!op` (toy 092).
    match cs with
    | [] -> Ok ()
    | c :: rest when c.is_whitespace() -> scan rest out
    | '(' :: rest ->
        out.push(Token::Open);
        scan rest out
    | ')' :: rest ->
        out.push(Token::Close);
        scan rest out
    | (('+' | '-' | '*' | '/') as op) :: rest ->
        out.push(Token::Op !op);
        scan rest out
    | c :: _ when c.is_ascii_digit() ->
        let n = span cs (fun c -> c.is_ascii_digit()) in
        out.push(Token::Num (text &cs[..n] |> .parse() |> .unwrap()));
        scan &cs[n..] out
    | c :: _ when c.is_alphabetic() ->
        let n = span cs char::is_alphanumeric in
        out.push(Token::Name (text &cs[..n]));
        scan &cs[n..] out
    | c :: _ -> Err (format!("unexpected {:?}", c))
}

/// The tokens of `source`.
pub fn tokens(source: &str) -> Result<Vec<Token>, String> {
    let cs = source.chars() |> .collect::<Vec<_>>() in
    let mut out = Vec.new () in
    // `scan &cs &mut out?` would be `scan(&cs, &mut out?)`: `?` binds
    // to the last argument, so the call goes in parentheses.
    (scan &cs &mut out)?;
    Ok out
}
}

pub fn run() {
    println!(
        "tokens \"width * (height + 12)\" = {:?}",
        tokens("width * (height + 12)")
    );
    println!("tokens \"3 % 4\" = {:?}", tokens("3 % 4"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn an_expression() {
        let name = |s: &str| Name(s.to_string());
        assert_eq!(
            tokens("width * (height + 12)"),
            Ok(vec![
                name("width"),
                Op('*'),
                Open,
                name("height"),
                Op('+'),
                Num(12),
                Close
            ])
        );
        assert_eq!(tokens("x2-7"), Ok(vec![name("x2"), Op('-'), Num(7)]));
        assert_eq!(tokens(""), Ok(vec![]));
    }

    #[test]
    fn an_unknown_character() {
        assert_eq!(tokens("3 % 4"), Err("unexpected '%'".to_string()));
    }
}

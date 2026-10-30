// Toy 137: an expression evaluator.
//
// What it is: arithmetic expressions with variables and `let`, as a Rust
// enum; a function computing their value, which fails on an unknown
// variable or a division by zero; and a printer.
//
// When to use it: the classic first interpreter. `match` on the
// constructors (toy 068), `?` to pass errors up (toy 119), `::` to extend
// the environment (toy 062) and `^` to build text (toy 064).
//
// How to write it: one case per constructor. The gotcha: `eval a env?` is
// `eval(a, env?)`, since `?` binds to the last argument; write
// `(eval a env)?`.
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::rsml;

pub const SUMMARY: &str = "An evaluator: `match` on an enum, `?`, `::` and `^`";

rsml! {
    #[derive(Debug, Clone)]
    pub enum Expr {
        Num(i64),
        Var(String),
        Add(Box<Expr>, Box<Expr>),
        Mul(Box<Expr>, Box<Expr>),
        Div(Box<Expr>, Box<Expr>),
        Let(String, Box<Expr>, Box<Expr>),
    }

    pub fn num(n: i64) -> Expr {
        Expr::Num n
    }
    pub fn var(name: &str) -> Expr {
        Expr::Var (String.from name)
    }
    pub fn plus(a: Expr, b: Expr) -> Expr {
        Expr::Add (Box::new a, Box::new b)
    }
    pub fn times(a: Expr, b: Expr) -> Expr {
        Expr::Mul (Box::new a, Box::new b)
    }
    pub fn over(a: Expr, b: Expr) -> Expr {
        Expr::Div (Box::new a, Box::new b)
    }
    pub fn bind(name: &str, v: Expr, body: Expr) -> Expr {
        Expr::Let (String.from name, Box::new v, Box::new body)
    }

    /// The value of `e`; `env` holds the variables, innermost first.
    pub fn eval(e: &Expr, env: &[(String, i64)]) -> Result<i64, String> {
        match e with
        | Expr::Num n -> Ok !n
        | Expr::Var name ->
            env
            |> .iter()
            |> .find(fun (n, _) -> n = name)
            |> .map(fun (_, v) -> !v)
            |> .ok_or_else(fun () -> format!("unbound {}", name))
        | Expr::Add (a, b) -> Ok ((eval a env)? + (eval b env)?)
        | Expr::Mul (a, b) -> Ok ((eval a env)? * (eval b env)?)
        | Expr::Div (a, b) ->
            let d = (eval b env)? in
            if d = 0 then Err (String.from "division by zero") else Ok ((eval a env)? / d)
        | Expr::Let (name, value, body) ->
            // `x :: xs` puts `x` in front of a copy of the list.
            let v = (eval value env)? in
            let inner = (name.clone(), v) :: env.to_vec() in
            eval body &inner
    }

    /// The expression as text, every operation in parentheses.
    pub fn show(e: &Expr) -> String {
        // `^` groups to the right, `a ^ b ^ c` is `a ^ (b ^ c)`, and
        // builds `String`s (toy 064).
        match e with
        | Expr::Num n -> n.to_string()
        | Expr::Var name -> name.clone()
        | Expr::Add (a, b) -> "(" ^ show a ^ " + " ^ show b ^ ")"
        | Expr::Mul (a, b) -> "(" ^ show a ^ " * " ^ show b ^ ")"
        | Expr::Div (a, b) -> "(" ^ show a ^ " / " ^ show b ^ ")"
        | Expr::Let (name, v, b) -> "(let " ^ name ^ " = " ^ show v ^ " in " ^ show b ^ ")"
    }

    /// `let x = 6 in x * (1 + x)`.
    pub fn example() -> Expr {
        bind "x" (num 6) (times (var "x") (plus (num 1) (var "x")))
    }
}

pub fn run() {
    println!("{} = {:?}", show(&example()), eval(&example(), &[]));
    println!("y = {:?}", eval(&var("y"), &[]));
    println!(
        "1 / (0 * x) = {:?}",
        eval(&over(num(1), times(num(0), var("x"))), &[("x".into(), 3)])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        assert_eq!(eval(&example(), &[]), Ok(42));
        assert_eq!(
            eval(&plus(var("x"), num(1)), &[("x".to_string(), 2)]),
            Ok(3)
        );
    }

    #[test]
    fn errors() {
        assert_eq!(eval(&var("y"), &[]), Err("unbound y".to_string()));
        let zero = times(num(0), var("x"));
        let env = [("x".to_string(), 3)];
        assert_eq!(
            eval(&over(num(1), zero), &env),
            Err("division by zero".to_string())
        );
    }

    #[test]
    fn text() {
        assert_eq!(show(&example()), "(let x = 6 in (x * (1 + x)))");
    }
}

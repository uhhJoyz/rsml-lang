//! The command line, as editors run it: `rsml-fmt --rustfmt` with the file
//! on stdin.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn run(args: &[&str], dir: &Path, input: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rsml-fmt"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rsml-fmt-cli-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

const MESSY: &str = "use x::{a9, a10};\nfn   main( ) { let n = ml!{ 1 |>   f }; }\nrsml! {\nfn   f(a:i64)->i64 { a+1 }\n}\n";

#[test]
fn rustfmt_then_ocaml() {
    let d = temp("both");
    let (code, out, err) = run(&["--rustfmt"], &d, MESSY);
    assert_eq!(code, 0, "{}", err);
    assert_eq!(
        out,
        "use x::{a9, a10};\nfn main() {\n    let n = ml! { 1 |> f };\n}\nrsml! {\nfn f(a: i64) -> i64 {\n    a + 1\n}\n}\n"
    );
    // Without `--rustfmt`, the Rust outside the macros is left alone.
    let (_, out, _) = run(&[], &d, MESSY);
    assert!(
        out.starts_with("use x::{a9, a10};\nfn   main( ) { let n = ml!{ 1 |> f }; }\n"),
        "{}",
        out
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn bad_input_writes_nothing() {
    let d = temp("bad");
    for input in ["fn main( { ml! { 1 } \n", "fn main() { let s = \"open; }\n"] {
        let (code, out, err) = run(&["--rustfmt"], &d, input);
        assert_eq!(code, 2, "{}", err);
        assert_eq!(out, "");
        let (code, out, _) = run(&[], &d, input);
        assert_ne!(code, 0);
        assert_eq!(out, "");
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn keep_on_error_writes_the_input_back() {
    let d = temp("keep");
    for (args, input) in [
        (
            &["--rustfmt", "--keep-on-error"][..],
            "fn main( { ml! { 1 } \n",
        ),
        (&["--keep-on-error"][..], "fn main() { let s = \"open; }\n"),
        (&["--ml", "--keep-on-error"][..], "let x = in\n"),
    ] {
        let (code, out, err) = run(args, &d, input);
        assert_eq!(code, 2, "{:?}: {}", args, err);
        assert_eq!(out, input, "{:?}", args);
        assert!(!err.is_empty(), "{:?}", args);
    }
    // Input that formats is formatted as without the option.
    let (code, out, _) = run(&["--rustfmt", "--keep-on-error"], &d, MESSY);
    assert_eq!(code, 0);
    assert_eq!(out, run(&["--rustfmt"], &d, MESSY).1);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn stdin_filepath_finds_the_configs() {
    let d = temp("configs");
    std::fs::write(
        d.join("Cargo.toml"),
        "[package]\nname = \"p\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(d.join("rustfmt.toml"), "max_width = 40\n").unwrap();
    std::fs::write(d.join("rsmlfmt.toml"), "max_width = 30\n").unwrap();
    std::fs::create_dir_all(d.join("src")).unwrap();
    let input = "use x::{a9, a10};\nfn long_function_name(first: i64, second: i64) -> i64 { ml!{ first_argument |> some_function |> another } }\n";
    // From another directory, as an editor may run it.
    let elsewhere = temp("elsewhere");
    let file = d.join("src/main.rs");
    let (code, out, err) = run(
        &["--rustfmt", "--stdin-filepath", file.to_str().unwrap()],
        &elsewhere,
        input,
    );
    assert_eq!(code, 0, "{}", err);
    assert_eq!(
        out,
        "use x::{a10, a9};\nfn long_function_name(\n    first: i64,\n    second: i64,\n) -> i64 { ml! {\n    first_argument\n    |> some_function\n    |> another\n}}\n"
    );
    // Inside `rsml!` too, rustfmt's options apply.
    let (_, out, _) = run(
        &["--stdin-filepath", file.to_str().unwrap()],
        &elsewhere,
        "rsml! {\nfn long_function_name(first: i64, second: i64) -> i64 { first }\n}\n",
    );
    assert_eq!(
        out,
        "rsml! {\nfn long_function_name(\n    first: i64,\n    second: i64,\n) -> i64 {\n    first\n}\n}\n"
    );
    // `--edition` wins over Cargo.toml.
    let (_, out, _) = run(
        &[
            "--rustfmt",
            "--edition",
            "2024",
            "--stdin-filepath",
            file.to_str().unwrap(),
        ],
        &elsewhere,
        input,
    );
    assert!(out.starts_with("use x::{a9, a10};\n"), "{}", out);
    // Warnings name the file.
    let (_, _, err) = run(
        &["--stdin-filepath", file.to_str().unwrap()],
        &elsewhere,
        "fn f() { ml! { let x = in } }\n",
    );
    assert!(err.contains("src/main.rs:1"), "{}", err);
    let _ = std::fs::remove_dir_all(&d);
    let _ = std::fs::remove_dir_all(&elsewhere);
}

#[test]
fn rustfmt_and_ocaml_agree() {
    // The OCaml pass breaks the block onto lines of its own, which changes
    // how rustfmt lays out the call around it: one run settles both, and a
    // second run changes nothing.
    let d = temp("agree");
    let src =
        "#[test]\nfn t() {\n    assert_eq!(ml! { let x = 1 in let y = x + 1 in x + y }, 3);\n}\n";
    let (code, once, err) = run(&["--rustfmt"], &d, src);
    assert_eq!(code, 0, "{}", err);
    assert_eq!(
        once,
        "#[test]\nfn t() {\n    assert_eq!(\n        ml! {\n        let x = 1 in\n        let y = x + 1 in\n        x + y\n        },\n        3\n    );\n}\n"
    );
    let (_, twice, _) = run(&["--rustfmt"], &d, &once);
    assert_eq!(twice, once);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn macro_rules_templates_are_left_alone() {
    let d = temp("templates");
    let src =
        "macro_rules! twice {\n    ($e:expr) => {\n        ml! { let x = $e in x+x }\n    };\n}\n";
    let (code, out, err) = run(&["--rustfmt"], &d, src);
    assert_eq!(code, 0, "{}", err);
    assert_eq!(out, src);
    assert!(err.is_empty(), "{}", err);
    let _ = std::fs::remove_dir_all(&d);
}

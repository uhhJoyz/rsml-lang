//! Running rustfmt: on whole files (`--rustfmt`), and on the Rust inside
//! `rsml! { .. }`, which rustfmt itself never enters.
//!
//! For an `rsml!` block, the formatter builds a Rust skeleton of its
//! contents in which every OCaml function body is `{}`, formats that with
//! rustfmt, and puts the OCaml bodies back where the `{}`s are (finding
//! them as `rsml!` does, so `#[rust]` bodies and other empty bodies are
//! never confused with them). The bodies are then formatted as OCaml like
//! any other.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use proc_macro2::{Delimiter, TokenStream, TokenTree};

use crate::config::Config;
use crate::source::range;

/// Where a file is, as rustfmt needs to know it.
#[derive(Clone, Debug, PartialEq)]
pub struct Env {
    /// The Rust edition (`2015`, `2018`, `2021`, `2024`).
    pub edition: String,
    /// The `rustfmt.toml` or `.rustfmt.toml` that applies, if any.
    pub rustfmt_config: Option<PathBuf>,
}

impl Default for Env {
    fn default() -> Env {
        Env {
            edition: "2024".to_string(),
            rustfmt_config: None,
        }
    }
}

impl Env {
    /// The environment of files in `dir`: the edition of the nearest
    /// `Cargo.toml` and the nearest rustfmt config.
    pub fn for_dir(dir: &Path) -> Env {
        Env {
            edition: edition_for(dir),
            rustfmt_config: find_upward(dir, &["rustfmt.toml", ".rustfmt.toml"]),
        }
    }
}

/// The nearest of `names` in `dir` or a parent.
pub fn find_upward(dir: &Path, names: &[&str]) -> Option<PathBuf> {
    let mut d = Some(dir);
    while let Some(dir) = d {
        for name in names {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
        d = dir.parent();
    }
    None
}

/// The edition of the crate holding `dir`: `package.edition` of the
/// nearest `Cargo.toml`, or the workspace's for `edition.workspace = true`.
/// A `Cargo.toml` without an edition is 2015, as for Cargo; with no
/// `Cargo.toml` at all, 2024.
pub fn edition_for(dir: &Path) -> String {
    let Some(manifest) = find_upward(dir, &["Cargo.toml"]) else {
        return "2024".to_string();
    };
    let Some(toml) = read_toml(&manifest) else {
        return "2024".to_string();
    };
    let edition = toml.get("package").and_then(|p| p.get("edition"));
    match edition {
        Some(toml::Value::String(e)) => e.clone(),
        Some(toml::Value::Table(t)) if t.get("workspace") == Some(&toml::Value::Boolean(true)) => {
            workspace_edition(&manifest).unwrap_or_else(|| "2015".to_string())
        }
        // A workspace root without a package: its members' default.
        None if toml.get("package").is_none() => toml
            .get("workspace")
            .and_then(|w| w.get("package"))
            .and_then(|p| p.get("edition"))
            .and_then(|e| e.as_str())
            .map_or_else(|| "2024".to_string(), str::to_string),
        _ => "2015".to_string(),
    }
}

/// `workspace.package.edition` of the workspace above `manifest`.
fn workspace_edition(manifest: &Path) -> Option<String> {
    let mut d = manifest.parent();
    while let Some(dir) = d {
        let p = dir.join("Cargo.toml");
        if let Some(t) = read_toml(&p)
            && let Some(w) = t.get("workspace")
        {
            return w
                .get("package")
                .and_then(|p| p.get("edition"))
                .and_then(|e| e.as_str())
                .map(str::to_string);
        }
        d = dir.parent();
    }
    None
}

fn read_toml(path: &Path) -> Option<toml::Table> {
    std::fs::read_to_string(path).ok()?.parse().ok()
}

/// Why rustfmt gave no output.
#[derive(Debug)]
pub enum Failure {
    /// It could not be run.
    Missing(String),
    /// It ran and rejected the input; its first error line.
    Rejected(String),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Missing(m) => write!(f, "rustfmt could not be run: {}", m),
            Failure::Rejected(m) => write!(f, "rustfmt failed: {}", m),
        }
    }
}

/// Runs rustfmt on `text` (stdin to stdout), with the edition and config
/// of `env` and the extra `--config` options given.
pub fn run(cfg: &Config, env: &Env, text: &str, options: &[String]) -> Result<String, Failure> {
    let mut cmd = Command::new(&cfg.rustfmt_path);
    cmd.arg("--edition").arg(&env.edition);
    if let Some(c) = &env.rustfmt_config {
        cmd.arg("--config-path").arg(c);
    }
    if !options.is_empty() {
        cmd.arg("--config").arg(options.join(","));
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| Failure::Missing(format!("`{}`: {}", cfg.rustfmt_path, e)))?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let input = text.to_string();
    // Written from a thread, so that a large output cannot block the input.
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let out = child
        .wait_with_output()
        .map_err(|e| Failure::Missing(e.to_string()))?;
    let _ = writer.join();
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("no message")
            .to_string();
        return Err(Failure::Rejected(first));
    }
    String::from_utf8(out.stdout).map_err(|e| Failure::Rejected(e.to_string()))
}

/// rustfmt's `tab_spaces` for `env`.
fn tab_spaces(env: &Env) -> usize {
    env.rustfmt_config
        .as_deref()
        .and_then(read_toml)
        .and_then(|t| t.get("tab_spaces").and_then(|v| v.as_integer()))
        .map_or(4, |n| n.max(1) as usize)
}

/// The new text of an `rsml! { .. }` group (`src[group range]`), its Rust
/// formatted by rustfmt and its OCaml bodies as written; `Ok(None)` when
/// nothing would change.
pub fn rsml_block(
    src: &str,
    group: &proc_macro2::Group,
    cfg: &Config,
    env: &Env,
) -> Result<Option<String>, Failure> {
    let open = range(group.span_open());
    let close = range(group.span_close());
    let inner = &src[open.end..close.start];
    if inner.trim().is_empty() {
        return Ok(None);
    }
    // The OCaml bodies, in order, and the skeleton with `{}` for each.
    let bodies = rsml_compiler::items::fn_bodies(group.stream());
    let mut skeleton = String::new();
    let mut at = open.end;
    let mut texts = Vec::new();
    for b in &bodies {
        let r = range(b.body.span());
        skeleton.push_str(&src[at..r.start]);
        skeleton.push_str("{}");
        texts.push(&src[r.clone()]);
        at = r.end;
    }
    skeleton.push_str(&src[at..close.start]);

    // The items' indentation: flush left if they were, else that of the
    // line with `rsml! {`, or one level in with `indent_rsml_items`.
    let line_start = src[..open.start].rfind('\n').map_or(0, |i| i + 1);
    let tab = tab_spaces(env);
    let line_indent: usize = src[line_start..open.start]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { tab } else { 1 })
        .sum();
    let first = inner.trim_start();
    let first_at = open.end + (inner.len() - first.len());
    let first_col = first_at - src[..first_at].rfind('\n').map_or(0, |i| i + 1);
    let multiline = inner[..inner.len() - first.len()].contains('\n');
    let levels = if multiline && first_col == 0 {
        0
    } else {
        line_indent / tab + usize::from(cfg.indent_rsml_items)
    };
    // Formatted inside `levels` nested modules, the items get their
    // indentation from rustfmt itself, and fit its width.
    let mut wrapped = String::new();
    for _ in 0..levels {
        wrapped.push_str("mod __rsml_items {\n");
    }
    wrapped.push_str(&skeleton);
    wrapped.push('\n');
    for _ in 0..levels {
        wrapped.push_str("}\n");
    }
    let formatted = run(cfg, env, &wrapped, &[])?;
    let lines: Vec<&str> = formatted.trim_end_matches('\n').split('\n').collect();
    if lines.len() < 2 * levels {
        return Err(Failure::Rejected("unexpected output".to_string()));
    }
    // Without the blank lines rustfmt keeps at the start and end.
    let mut items: &[&str] = &lines[levels..lines.len() - levels];
    while items.first().is_some_and(|l| l.trim().is_empty()) {
        items = &items[1..];
    }
    while items.last().is_some_and(|l| l.trim().is_empty()) {
        items = &items[..items.len() - 1];
    }
    let items = items.join("\n");

    // Put the OCaml bodies back where the `{}`s are.
    let ts: TokenStream = items
        .parse()
        .map_err(|_| Failure::Rejected("output does not lex".to_string()))?;
    let stubs = rsml_compiler::items::fn_bodies(ts);
    if stubs.len() != texts.len() {
        return Err(Failure::Rejected("function bodies moved".to_string()));
    }
    let mut out = items.clone();
    for (stub, text) in stubs.iter().zip(&texts).rev() {
        let r = range(stub.body.span());
        let empty = stub.body.delimiter() == Delimiter::Brace && stub.body.stream().is_empty();
        if !empty {
            return Err(Failure::Rejected("function bodies moved".to_string()));
        }
        out.replace_range(r, text);
    }

    // Blank lines after `{` and before `}` are kept, at most one.
    let blank = |s: &str| s.matches('\n').count() >= 2;
    let lead = if !multiline {
        "\n"
    } else if blank(&inner[..inner.len() - first.len()]) {
        "\n\n"
    } else {
        "\n"
    };
    let last = inner.trim_end();
    let trail_blank = blank(&inner[last.len()..]) && multiline;
    let indent: String = src[line_start..open.start]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    let new = format!(
        "{{{}{}\n{}{}}}",
        lead,
        out,
        if trail_blank { "\n" } else { "" },
        indent
    );
    let old = &src[open.start..close.end];
    if new == old {
        return Ok(None);
    }
    Ok(Some(new))
}

/// Every outermost `rsml! { .. }` group of a Rust file.
pub fn rsml_groups(ts: TokenStream, out: &mut Vec<proc_macro2::Group>) {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut i = 0;
    while i < toks.len() {
        if let (TokenTree::Ident(name), Some(TokenTree::Punct(bang)), Some(TokenTree::Group(g))) =
            (&toks[i], toks.get(i + 1), toks.get(i + 2))
            && bang.as_char() == '!'
            && name == "rsml"
        {
            out.push(g.clone());
            i += 3;
            continue;
        }
        if let TokenTree::Group(g) = &toks[i] {
            rsml_groups(g.stream(), out);
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh directory under the system's temporary one.
    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("rsml-fmt-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn editions() {
        let d = temp("editions");
        write(
            &d.join("Cargo.toml"),
            "[workspace]\nmembers = [\"a\", \"b\", \"c\"]\n\n[workspace.package]\nedition = \"2018\"\n",
        );
        write(
            &d.join("a/Cargo.toml"),
            "[package]\nname = \"a\"\nedition.workspace = true\n",
        );
        write(
            &d.join("b/Cargo.toml"),
            "[package]\nname = \"b\"\nedition = \"2021\"\n",
        );
        write(&d.join("c/Cargo.toml"), "[package]\nname = \"c\"\n");
        assert_eq!(edition_for(&d.join("a/src")), "2018");
        assert_eq!(edition_for(&d.join("b/src/bin")), "2021");
        // No edition: Cargo's default.
        assert_eq!(edition_for(&d.join("c")), "2015");
        // The workspace root itself.
        assert_eq!(edition_for(&d), "2018");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn rustfmt_config_is_found() {
        let d = temp("config");
        write(&d.join(".rustfmt.toml"), "max_width = 60\n");
        write(&d.join("x/y/z.rs"), "");
        let env = Env::for_dir(&d.join("x/y"));
        assert_eq!(env.rustfmt_config, Some(d.join(".rustfmt.toml")));
        let _ = std::fs::remove_dir_all(&d);
    }
}

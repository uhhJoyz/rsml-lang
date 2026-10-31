//! `rsml-fmt`: formats the rsml code in `ml!` / `rsml!` blocks of Rust files.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rsml_fmt::{Config, DEFAULT_TOML, Env};

const USAGE: &str = "\
rsml-fmt: formats the rsml code in `ml! { .. }` blocks and `rsml! { .. }` functions

USAGE:
    rsml-fmt [OPTIONS] [PATH ...]

PATH is a .rs file or a directory (searched recursively, skipping `target`
and hidden directories). Without a PATH, a Rust file is read from stdin and
the result written to stdout; if it is not valid Rust, nothing is written
and the exit status is 2.

OPTIONS:
    --rustfmt             Run rustfmt on the whole file first, so that one
                          command formats Rust and rsml (for editors: use
                          `rsml-fmt --rustfmt` where rustfmt would be run)
    --stdin-filepath PATH The file that stdin holds: its directory, not the
                          current one, is where rsmlfmt.toml, rustfmt.toml
                          and Cargo.toml are looked for
    --edition EDITION     The Rust edition, instead of the one in the
                          nearest Cargo.toml (default 2024)
    --check               Write nothing; list files that would change and
                          exit with status 1 if there are any
    --stdout              Print formatted files instead of writing them
    --keep-on-error       With stdin: if it cannot be formatted, write it
                          back unchanged (the exit status is still 2), for
                          filters that replace the text with whatever comes
                          out (Vim's `:%!` and `gq`)
    --ml                  Stdin is bare rsml code (a block's contents), not Rust
    --config FILE         Use FILE instead of the nearest rsmlfmt.toml
    --set KEY=VALUE       Override an option, e.g. --set max_width=80
                          --set pipe.indent=align (repeatable)
    --print-config [default|current]
                          Print every option as TOML: the documented
                          defaults, or the config in effect here
    -h, --help            Show this message

Options are read from `rsmlfmt.toml` or `.rsmlfmt.toml` in the file's
directory or the closest parent that has one.";

struct Args {
    rustfmt: bool,
    stdin_filepath: Option<PathBuf>,
    edition: Option<String>,
    check: bool,
    stdout: bool,
    keep_on_error: bool,
    ml: bool,
    config: Option<PathBuf>,
    sets: Vec<String>,
    print_config: Option<String>,
    paths: Vec<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        rustfmt: false,
        stdin_filepath: None,
        edition: None,
        check: false,
        stdout: false,
        keep_on_error: false,
        ml: false,
        config: None,
        sets: Vec::new(),
        print_config: None,
        paths: Vec::new(),
    };
    let mut it = std::env::args().skip(1).peekable();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{}", USAGE);
                std::process::exit(0);
            }
            "--check" => args.check = true,
            "--rustfmt" => args.rustfmt = true,
            "--stdin-filepath" => {
                args.stdin_filepath =
                    Some(it.next().ok_or("--stdin-filepath needs a path")?.into());
            }
            "--edition" => args.edition = Some(it.next().ok_or("--edition needs an edition")?),
            s if s.starts_with("--edition=") => {
                args.edition = Some(s["--edition=".len()..].to_string())
            }
            s if s.starts_with("--stdin-filepath=") => {
                args.stdin_filepath = Some(s["--stdin-filepath=".len()..].into());
            }
            "--stdout" => args.stdout = true,
            "--keep-on-error" => args.keep_on_error = true,
            "--ml" => args.ml = true,
            "--config" => {
                args.config = Some(it.next().ok_or("--config needs a file")?.into());
            }
            "--set" => args.sets.push(it.next().ok_or("--set needs KEY=VALUE")?),
            "--print-config" => {
                let which = match it.peek().map(String::as_str) {
                    Some("default" | "current") => it.next().unwrap(),
                    _ => "default".to_string(),
                };
                args.print_config = Some(which);
            }
            s if s.starts_with("--set=") => args.sets.push(s["--set=".len()..].to_string()),
            s if s.starts_with('-') => return Err(format!("unknown option `{}`\n\n{}", s, USAGE)),
            _ => args.paths.push(a.into()),
        }
    }
    Ok(args)
}

/// The config for files under `dir`.
fn load_config(args: &Args, dir: &Path) -> Result<Config, String> {
    let file = match &args.config {
        Some(f) => Some(f.clone()),
        None => find_config(dir),
    };
    let mut cfg = match &file {
        Some(f) => {
            let text = std::fs::read_to_string(f).map_err(|e| format!("{}: {}", f.display(), e))?;
            Config::from_toml(&text).map_err(|e| format!("{}: {}", f.display(), e))?
        }
        None => Config::default(),
    };
    if !args.sets.is_empty() {
        cfg = cfg
            .with_overrides(&args.sets.join("\n"))
            .map_err(|e| format!("--set: {}", e))?;
    }
    Ok(cfg)
}

fn find_config(dir: &Path) -> Option<PathBuf> {
    rsml_fmt::rustfmt::find_upward(dir, &["rsmlfmt.toml", ".rsmlfmt.toml"])
}

/// The edition and rustfmt config for files in `dir`.
fn env_for(args: &Args, dir: &Path) -> Env {
    let mut env = Env::for_dir(dir);
    if let Some(e) = &args.edition {
        env.edition = e.clone();
    }
    env
}

/// rustfmt on the whole file (with `--rustfmt`), then the OCaml. A block
/// that the OCaml pass breaks onto lines of its own changes how rustfmt lays
/// out the code around it (`assert_eq!(ml! { .. }, 3)`), and that the
/// block's indentation: the two run in turn until neither changes anything.
fn format_file(
    args: &Args,
    cfg: &Config,
    env: &Env,
    src: &str,
) -> Result<rsml_fmt::Formatted, String> {
    if !args.rustfmt {
        return rsml_fmt::format_source_in(src, cfg, env);
    }
    let mut text = src.to_string();
    let mut out = None;
    for _ in 0..4 {
        let rust = rsml_fmt::rustfmt::run(cfg, env, &text, &[]).map_err(|e| e.to_string())?;
        let formatted = rsml_fmt::format_source_in(&rust, cfg, env)?;
        let done = formatted.text == text;
        text = formatted.text.clone();
        out = Some(formatted);
        if done {
            break;
        }
    }
    Ok(out.unwrap())
}

/// Formats `input` (stdin) and prints it; on an error, prints nothing.
fn format_stdin(args: &Args, cwd: &Path, input: &str) -> Result<bool, String> {
    let (name, dir) = match &args.stdin_filepath {
        Some(p) => {
            let p = cwd.join(p);
            let dir = p.parent().map_or(cwd.to_path_buf(), Path::to_path_buf);
            (p.display().to_string(), dir)
        }
        None => ("<stdin>".to_string(), cwd.to_path_buf()),
    };
    let cfg = load_config(args, &dir)?;
    if args.ml {
        print!("{}", rsml_fmt::format_ml(input.trim_end(), &cfg)?);
        println!();
        return Ok(true);
    }
    let env = env_for(args, &dir);
    let out = format_file(args, &cfg, &env, input).map_err(|e| format!("{}: {}", name, e))?;
    for w in &out.warnings {
        eprintln!("{}:{}: {}", name, w.line, w.message);
    }
    print!("{}", out.text);
    Ok(true)
}

fn rust_files(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    let mut entries: Vec<_> = std::fs::read_dir(path)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if name != "target" && !name.starts_with('.') {
                rust_files(&p, out)?;
            }
        } else if name.ends_with(".rs") {
            out.push(p);
        }
    }
    Ok(())
}

fn run() -> Result<bool, String> {
    let args = parse_args()?;
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    if let Some(which) = &args.print_config {
        if which == "default" {
            print!("{}", DEFAULT_TOML);
        } else {
            print!("{}", load_config(&args, &cwd)?.to_toml());
        }
        return Ok(true);
    }

    if args.paths.is_empty() {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())?;
        let result = format_stdin(&args, &cwd, &input);
        if result.is_err() && args.keep_on_error {
            print!("{}", input);
        }
        return result;
    }

    let mut files = Vec::new();
    for p in &args.paths {
        rust_files(p, &mut files).map_err(|e| format!("{}: {}", p.display(), e))?;
    }
    let mut ok = true;
    for f in files {
        let dir = f.parent().map_or(cwd.clone(), Path::to_path_buf);
        let dir = cwd.join(dir);
        let cfg = load_config(&args, &dir)?;
        let env = env_for(&args, &dir);
        let src = std::fs::read_to_string(&f).map_err(|e| format!("{}: {}", f.display(), e))?;
        let out = match format_file(&args, &cfg, &env, &src) {
            Ok(out) => out,
            Err(e) => {
                eprintln!("{}: {}", f.display(), e);
                ok = false;
                continue;
            }
        };
        for w in &out.warnings {
            eprintln!("{}:{}: {}", f.display(), w.line, w.message);
        }
        if args.stdout {
            print!("{}", out.text);
        } else if out.text != src {
            if args.check {
                println!("{}", f.display());
                ok = false;
            } else {
                std::fs::write(&f, &out.text).map_err(|e| format!("{}: {}", f.display(), e))?;
            }
        }
    }
    Ok(ok)
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("rsml-fmt: {}", e);
            ExitCode::from(2)
        }
    }
}

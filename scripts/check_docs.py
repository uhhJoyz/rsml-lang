#!/usr/bin/env python3
"""Checks what the documentation says about expansions and completion.

    python3 scripts/check_docs.py [-v] [NAME_FILTER]
    python3 scripts/check_docs.py --expand 'OCAML' ..   # print expansions
    python3 scripts/check_docs.py --toys FILE ..        # only these toys' claims

In README.md, docs/notation.md, docs/editors.md, rsml-fmt/README.md and
examples/toy/README.md:

* Every row of a table with an OCaml and a Rust column: the OCaml is
  expanded by the real compiler (as rustc sees it, without the editor
  markers) and compared token by token with the Rust. Each code span of the
  OCaml cell goes with the code span of the Rust cell at the same position
  (or with the only one). In the rows listed in ELIDED, `..` in the Rust
  stands for any tokens.
* Every inline claim "`a` is `b`" (also "is always", "becomes"): `b` is
  either the expansion of `a`, or OCaml with the same expansion up to the
  names of closure parameters.
* Every relative link: the file exists, and so does the heading its
  `#anchor` names.
* Every code span with a cursor (`‸`): a case of
  scripts/completion_cases.txt has that code (`..` standing for anything),
  and the rows of docs/editors.md's "Every checked position" are exactly
  the cases, with the same labels.
* The toy examples (examples/toy/NNN_name.rs, see its README.md): in
  their comments (and code), every claim "`a` is `b`" as above (also "is
  always", "becomes", "would be", ..; "Given `c`, " before claims in the
  same comment puts `c ;;` before both sides), and every quoted error
  "`a` is an error: "message"" (the message contains the quote); other
  phrasings of a claim, odd backticks in a comment and code spans in
  `(* *)` comments are errors, so that no claim goes unchecked. Each toy
  has a SUMMARY, a `run()`, tests and a header link to a section of
  docs/notation.md; every section has a toy; the toys are numbered 001 to
  N without gaps, N >= 120; the README's index lists each once, in order,
  with its SUMMARY; main.rs registers each once.

What cannot be checked this way is listed below (UNCHECKED, ELIDED,
CLAIMS_UNCHECKED, CURSORS_UNCHECKED), each with its reason, and so is the
context a snippet needs (CONTEXT, CLAIM_CONTEXT, CLAIM_AS); an entry that
no longer matches anything in the documentation is an error too, so the
lists stay current.
`-v` prints every check.

The script builds a small crate under target/doc-check that runs the
compiler (rsml-compiler) on every snippet in one go.
"""

import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATE = os.path.join(ROOT, "target", "doc-check")
FILES = ["README.md", "docs/notation.md", "docs/editors.md", "rsml-fmt/README.md",
         "examples/toy/README.md"]

# ----- what is not checked, and why -----------------------------------------

# Table rows that are not compared, by file and OCaml cell (as written,
# without the escapes of `|`).
UNCHECKED = {
    ("docs/notation.md", "`xs` / `.iter()` / `.count()` on separate lines"):
        "about line breaks, which a table cell cannot show; tests/notation.rs runs it",
}

# Table rows whose Rust leaves code out with `..`, which then stands for any
# tokens (elsewhere `..` is Rust's).
ITEMS = "the items before the function shown are left out"
LOOP = "the loop's body is left out"
ELIDED = {
    ("docs/notation.md", "`rsml! { fn add(a: i64, b: i64) -> i64 { .. } fn f() { add 1 } }`"): ITEMS,
    ("docs/notation.md", "`rsml! { fn now() -> i64 { .. } fn f() { now () } }`"): ITEMS,
    ("docs/notation.md", "`rsml! { fn adder(n: i64) -> impl Fn(i64) -> i64 { .. } fn f() { adder 1 2 } }`"): ITEMS,
    ("docs/notation.md", "`rsml! { impl P { fn new(x: i64, y: i64) -> P { .. } fn o() -> P { Self::new 0 0 } } }`"):
        "all but the call is left out",
    ("docs/notation.md", "`let rec even (n : u32) : bool = .. odd (n - 1) and odd (n : u32) : bool = .. in even 10`"):
        "the bodies are left out; tests/notation.rs runs the whole example",
    ("docs/notation.md", "`for i in 0..n do .. done`"): LOOP,
    ("docs/notation.md", "`for i = 0 to n do .. done`"): LOOP,
    ("docs/notation.md", "`for i = n downto 0 do .. done`"): LOOP,
    ("docs/notation.md", "`for (k, v) in m do .. done`, `for k, v in m do .. done`"): LOOP,
    ("docs/notation.md", "`while c do .. done`"): LOOP,
    ("docs/notation.md", "`while true do .. done`"): LOOP,
}

# Context an OCaml cell needs, prepended to each of its snippets: names that
# the row's text says are functions of a known shape.
CONTEXT = {
    ("README.md", "`add 1` (`add` takes 2)"): "external add : _ -> _ -> _ ;;",
    ("README.md", "`add (expensive ())`"): "external add : _ -> _ -> _ ;;",
}

# Inline claims that are not compared, by file and the claim's two sides.
CLAIMS_UNCHECKED = {
    ("README.md", "open std::cmp", "use std::cmp::*;"):
        "a phrase, not an expression: the tables check `open std::cmp ;; max a b`",
    ("README.md", "x", "&T"):
        "the type of a pattern's binding, which rustc infers",
    ("README.md", "rest", "&[T]"):
        "the type of a pattern's binding, which rustc infers",
    ("rsml-fmt/README.md", "0...1.", "0 ... 1."):
        "how Rust lexes `0...1.`, which the formatter must not print",
    ("rsml-fmt/README.md", "\"vertical\"", "fit_or_vertical"):
        "an option of the formatter",
}

# Context a claim needs, prepended to both sides, as for CONTEXT.
CLAIM_CONTEXT = {
    ("README.md", "adder 1 2", "adder(1)(2)"): "external adder : _ -> (_ -> _) ;;",
    ("docs/notation.md", "x |> add(1)", "add(1, x)"): "external add : _ -> _ -> _ ;;",
}

# Claims checked in a sentence of their own: the OCaml and the Rust, with
# `{a}` and `{b}` standing for the claim's two sides.
TYPE = ("(e : {a})", "{{ let typed: {b} = e; typed }}")
CLAIM_AS = {
    ("README.md", "i64 * String", "(i64, String)"): TYPE,
    ("docs/notation.md", "i64 * String", "(i64, String)"): TYPE,
    ("README.md", "Vec<'a>", "Vec<A>"):
        ("let rec id (x : {a}) : {a} = x in id", "{{ fn id<A>(x: {b}) -> {b} {{ x }} id }}"),
}

# Code with a cursor that is not compared with the completion cases: what
# the documentation says does not complete, and the cursor sign itself.
CURSORS_UNCHECKED = {
    ("docs/notation.md", "‸"): "the cursor sign itself",
    ("docs/notation.md", ".it‸"): "documented as having nothing to complete",
    ("docs/editors.md", "Point { ‸ } -> .."): "documented as not offering the fields",
}

# ----- the expansion helper -------------------------------------------------

HELPER_TOML = """\
[package]
name = "doc-check"
version = "0.0.0"
edition = "2024"

[dependencies]
rsml-compiler = { path = %s }
proc-macro2 = "1"

[workspace]
"""

# Reads records `mode \\x1f source` separated by \\x1e on stdin; writes, for
# each, `ok \\x1f token \\x1f token ..` or `error \\x1f message`, separated by
# \\x1e. A punctuation token is `P` + its character + `j` (joined to the
# next) or `a`, and a name or literal of that shape (`PI`) has \\x05 before
# it; a group is its delimiters around its tokens.
HELPER_MAIN = r"""
use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use std::io::{Read, Write};

fn toks(ts: TokenStream, out: &mut Vec<String>) {
    for t in ts {
        match t {
            TokenTree::Group(g) => {
                let (open, close) = match g.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::None => ("\u{2}", "\u{3}"),
                };
                out.push(open.to_string());
                toks(g.stream(), out);
                out.push(close.to_string());
            }
            TokenTree::Punct(p) => {
                let j = if p.spacing() == Spacing::Joint { "j" } else { "a" };
                out.push(format!("P{}{}", p.as_char(), j));
            }
            // A name or literal that would read as punctuation (`PI`)
            // is marked.
            t => {
                let s = t.to_string();
                if s.starts_with('P') && (s.len() == 2 || s.len() == 3) {
                    out.push(format!("\u{5}{}", s));
                } else {
                    out.push(s);
                }
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut out = Vec::new();
    for rec in input.split('\u{1e}') {
        if rec.is_empty() {
            continue;
        }
        let (mode, src) = rec.split_once('\u{1f}').unwrap();
        let ts: TokenStream = match src.parse() {
            Ok(ts) => ts,
            Err(e) => {
                out.push(format!("error\u{1f}does not lex: {}", e));
                continue;
            }
        };
        let ts = rsml_compiler::without_markers(|| match mode {
            "ml" => rsml_compiler::expand(ts),
            "items" => rsml_compiler::expand_items(ts),
            _ => ts,
        });
        let text = ts.to_string();
        if mode != "rust" && text.contains("compile_error") {
            out.push(format!("error\u{1f}{}", text));
            continue;
        }
        let mut v = Vec::new();
        toks(ts, &mut v);
        out.push(format!("ok\u{1f}{}", v.join("\u{1f}")));
    }
    std::io::stdout()
        .write_all(out.join("\u{1e}").as_bytes())
        .unwrap();
}
"""


def run_helper(requests):
    """Expands or tokenizes every (mode, source); returns (ok, tokens or message)."""
    os.makedirs(os.path.join(CRATE, "src"), exist_ok=True)
    with open(os.path.join(CRATE, "Cargo.toml"), "w") as f:
        f.write(HELPER_TOML % json.dumps(os.path.join(ROOT, "rsml-compiler")))
    with open(os.path.join(CRATE, "src", "main.rs"), "w") as f:
        f.write(HELPER_MAIN)
    build = subprocess.run(["cargo", "build", "--quiet", "--release"], cwd=CRATE,
                           capture_output=True, text=True)
    if build.returncode != 0:
        sys.exit("the helper crate does not build:\n" + build.stderr)
    exe = os.path.join(CRATE, "target", "release", "doc-check")
    data = "\x1e".join("%s\x1f%s" % r for r in requests)
    out = subprocess.run([exe], input=data, capture_output=True, text=True, check=True).stdout
    results = []
    for rec in out.split("\x1e") if requests else []:
        kind, _, rest = rec.partition("\x1f")
        results.append((kind == "ok", normalize(rest.split("\x1f")) if kind == "ok" else rest))
    assert len(results) == len(requests)
    return results


# Two punctuation characters that are one Rust operator when joined.
OPERATORS = {"::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "..", ".=", "<<", ">>",
             "+=", "-=", "*=", "/=", "%=", "^=", "&=", "|="}


def normalize(tokens):
    """Tokens compared: spacing matters only between two punctuation
    characters that would form an operator, and `t.0.1` (whose `0.1` Rust
    lexes as one literal) is `t . 0 . 1`."""
    out = []
    for i, t in enumerate(tokens):
        if t.startswith("P") and len(t) == 3:
            nxt = tokens[i + 1] if i + 1 < len(tokens) else ""
            if not (nxt.startswith("P") and len(nxt) == 3 and t[1] + nxt[1] in OPERATORS):
                t = t[:2]
        if out and out[-1] == "P." and re.fullmatch(r"\d+\.\d+", t):
            a, b = t.split(".")
            out.extend([a, "P.", b])
            continue
        out.append(t)
    return out


def show(tokens):
    """Tokens as text again, for messages."""
    out = []
    for t in tokens:
        if t.startswith("P") and len(t) in (2, 3):
            out.append(t[1] + ("" if t.endswith("j") else " "))
        elif t in ("\x02", "\x03"):
            continue
        else:
            out.append(t.lstrip("\x05") + " ")
    return "".join(out).strip()


# ----- reading the documentation --------------------------------------------

def unfenced_lines(path):
    """The lines of a Markdown file, with fenced code blocks blanked."""
    lines, fenced = [], False
    for line in open(os.path.join(ROOT, path)):
        line = line.rstrip("\n")
        if line.startswith("```"):
            fenced = not fenced
            lines.append("")
            continue
        lines.append("" if fenced else line)
    return lines


def split_row(line):
    """The cells of a table row, with `\\|` unescaped."""
    cells, cur, i = [], "", 1
    while i < len(line):
        c = line[i]
        if c == "\\" and i + 1 < len(line) and line[i + 1] == "|":
            cur += "|"
            i += 2
            continue
        if c == "|":
            cells.append(cur.strip())
            cur = ""
        else:
            cur += c
        i += 1
    return cells


def tables(path):
    """(header cells, [(line number, cells)]) for every table in the file."""
    lines = unfenced_lines(path)
    out, i = [], 0
    while i < len(lines):
        if (lines[i].startswith("|") and i + 1 < len(lines)
                and re.match(r"^\|[\s:|-]+\|$", lines[i + 1])):
            header = split_row(lines[i])
            rows = []
            i += 2
            while i < len(lines) and lines[i].startswith("|"):
                rows.append((i + 1, split_row(lines[i])))
                i += 1
            out.append((header, rows))
        else:
            i += 1
    return out


def code_spans(cell):
    return re.findall(r"`([^`]+)`", cell)


def parse_cell(cell):
    """The code spans of a cell that is code spans separated by `,` (and
    perhaps `, ..` or a parenthesized note at the end), else None."""
    rest = re.sub(r"\s*\((?:[^()`]|`[^`]*`)*\)$", "", cell)
    rest = re.sub(r",\s*\.\.$", "", rest)
    if not re.fullmatch(r"`[^`]+`(\s*,\s*`[^`]+`)*", rest):
        return None
    return code_spans(rest)


def prose(path):
    """The text of a file outside tables and code blocks, one paragraph per
    item, with the line each starts on."""
    paras, cur, start = [], [], 0
    for n, line in enumerate(unfenced_lines(path), 1):
        if not line.strip() or line.startswith("|") or line.startswith("#"):
            if cur:
                paras.append((start, " ".join(cur)))
            cur = []
            continue
        if not cur:
            start = n
        cur.append(line.strip())
    if cur:
        paras.append((start, " ".join(cur)))
    return paras


# ----- comparing tokens -----------------------------------------------------

def wildcards(tokens):
    """`tokens` with each `..` (not `...` or `..=`) replaced by None, the
    wildcard."""
    out, i = [], 0
    while i < len(tokens):
        if tokens[i] == "P.j" and i + 1 < len(tokens) and tokens[i + 1] == "P.":
            out.append(None)
            i += 2
        else:
            out.append(tokens[i])
            i += 1
    return out


def match_elided(pattern, tokens):
    """Whether `tokens` matches `pattern`, where None matches any tokens."""
    memo = {}

    def go(i, j):
        if (i, j) in memo:
            return memo[(i, j)]
        if i == len(pattern):
            r = j == len(tokens)
        elif pattern[i] is None:
            r = any(go(i + 1, k) for k in range(j, len(tokens) + 1))
        else:
            r = j < len(tokens) and pattern[i] == tokens[j] and go(i + 1, j + 1)
        memo[(i, j)] = r
        return r

    return go(0, 0)


def binders(tokens):
    """The names bound by closures (`|a, b: T|`), in order."""
    out, i = [], 0
    starts = {"move", "(", "[", "{", "P,", "P=", "P;", "\x02"}
    while i < len(tokens):
        if tokens[i] in ("P|", "P|a") and (i == 0 or tokens[i - 1] in starts):
            j, typed = i + 1, False
            while j < len(tokens) and tokens[j] not in ("P|", "P|a"):
                t = tokens[j]
                if t in ("P:", "P:a"):
                    typed = True
                elif t in ("P,", "P,a"):
                    typed = False
                elif not typed and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", t) and t != "_":
                    out.append(t)
                j += 1
            i = j + 1
        else:
            i += 1
    return out


def alpha(tokens):
    """`tokens` with closure parameters renamed by order of appearance, and
    without `move` (which only says how a closure captures)."""
    names = {}
    for b in binders(tokens):
        names.setdefault(b, "$%d" % len(names))
    return [names.get(t, t) for t in tokens if t != "move"]


# ----- the checks -----------------------------------------------------------

class Checks:
    def __init__(self, verbose):
        self.verbose = verbose
        self.requests = []
        self.items = []
        self.failed = 0
        self.counts = {}

    def request(self, mode, src):
        self.requests.append((mode, src))
        return len(self.requests) - 1

    def count(self, kind):
        self.counts[kind] = self.counts.get(kind, 0) + 1

    def fail(self, where, msg):
        self.failed += 1
        print("FAIL %s: %s" % (where, msg))

    def ok(self, where, msg):
        if self.verbose:
            print("ok   %s: %s" % (where, msg))


def snippet_request(checks, code, context):
    """The request expanding an OCaml snippet (an `rsml! { .. }` is items)."""
    src = (context + " " + code).strip() if context else code
    m = re.fullmatch(r"rsml!\s*\{(.*)\}", src, re.S)
    if m:
        return checks.request("items", m.group(1))
    return checks.request("ml", src)


def table_rows(checks, path, used):
    for header, rows in tables(path):
        oc = [i for i, h in enumerate(header) if "OCaml" in h]
        rc = [i for i, h in enumerate(header) if "Rust" in h]
        if not oc or not rc:
            continue
        for line, cells in rows:
            a, b = cells[oc[0]], cells[rc[0]]
            where = "%s:%d %s" % (path, line, a)
            key = (path, a)
            if key in UNCHECKED:
                used.add(("table", key))
                checks.count("table rows unchecked")
                checks.ok(where, "unchecked: " + UNCHECKED[key])
                continue
            codes = parse_cell(a)
            if b.startswith("the same"):
                expect = codes
            else:
                expect = parse_cell(b)
            if codes is None or expect is None or len(expect) not in (1, len(codes)):
                checks.fail(where, "the cells do not pair up: make them code spans, or add the row to UNCHECKED")
                continue
            if key in CONTEXT:
                used.add(("context", key))
            if key in ELIDED:
                used.add(("elided", key))
            context = CONTEXT.get(key, "")
            for k, code in enumerate(codes):
                rust = expect[k if len(expect) > 1 else 0]
                checks.items.append(("table", where, code, rust,
                                     snippet_request(checks, code, context),
                                     checks.request("rust", rust), key in ELIDED))


def error_rows(checks, path):
    """The tables of Rust habits: each written snippet is an rsml error,
    and the messages of a row contain every code span it says they
    suggest."""
    for header, rows in tables(path):
        if header[:2] != ["Written", "The error suggests"]:
            continue
        for line, cells in rows:
            where = "%s:%d %s" % (path, line, cells[0])
            codes = parse_cell(cells[0])
            if codes is None:
                checks.fail(where, "the written cell is not code spans")
                continue
            reqs = [snippet_request(checks, c, "") for c in codes]
            checks.items.append(("error", where, codes, code_spans(cells[1]), reqs, None))


CLAIM = re.compile(r"`([^`]+)` (?:is|is always|becomes) `([^`]+)`")


def claims(checks, path, used):
    for line, text in prose(path):
        for a, b in CLAIM.findall(text):
            where = "%s:%d `%s` is `%s`" % (path, line, a, b)
            key = (path, a, b)
            if key in CLAIMS_UNCHECKED:
                used.add(("claim", key))
                checks.count("claims unchecked")
                checks.ok(where, "unchecked: " + CLAIMS_UNCHECKED[key])
                continue
            if key in CLAIM_AS:
                used.add(("claim as", key))
                ml, rust = (t.format(a=a, b=b) for t in CLAIM_AS[key])
                checks.items.append(("claim as", where, ml, rust,
                                     snippet_request(checks, ml, ""),
                                     checks.request("rust", rust)))
                continue
            context = CLAIM_CONTEXT.get(key, "")
            if context:
                used.add(("claim context", key))
            checks.items.append(("claim", where, a, b,
                                 snippet_request(checks, a, context),
                                 checks.request("rust", b),
                                 snippet_request(checks, b, context)))


def error_messages(text):
    """The messages of the `compile_error!`s in an expansion's text."""
    def unescape(m):
        e = m.group(1)
        if e.startswith("u{"):
            return chr(int(e[2:-1], 16))
        return {"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(e, e)
    return [re.sub(r"\\(u\{[0-9a-fA-F]+\}|.)", unescape, m) for m in
            re.findall(r'compile_error ! \("((?:[^"\\]|\\.)*)"\)', text)]


def compare(checks, results):
    for item in checks.items:
        kind, where, code, rust, req, rust_req = item[:6]
        if kind == "error":
            msgs = []
            for c, r in zip(code, req):
                ok, got = results[r]
                if ok:
                    checks.fail(where, "`%s` is not an error: it expands to %s" % (c, show(got)))
                msgs.append("" if ok else got)
            missing = [s for s in rust if not any(s in m for m in msgs)]
            if missing:
                checks.fail(where, "no message suggests %s; they say:\n       %s"
                            % (", ".join("`%s`" % m for m in missing), "\n       ".join(msgs)))
            elif all(msgs):
                checks.count("error rows")
                checks.ok(where, "errors suggest %s" % ", ".join(rust))
            continue
        ok, got = results[req]
        if kind == "quote":
            # `got` is the message, or the tokens of an expansion.
            said = error_messages(got) if not ok else []
            if ok:
                checks.fail(where, "`%s` is not an error: it expands to %s" % (code, show(got)))
            elif not any(rust in m for m in said):
                checks.fail(where, "the message is not quoted exactly; it says:\n       %s"
                            % "\n       ".join(said or [got]))
            else:
                checks.count("toy errors quoted")
                checks.ok(where, "the message says \"%s\"" % rust)
            continue
        if not ok:
            checks.fail(where, "`%s` does not expand: %s" % (code, got))
            continue
        ok_r, want = results[rust_req]
        if not ok_r:
            checks.fail(where, "`%s` does not lex: %s" % (rust, want))
            continue
        if got == want:
            checks.count({"table": "table rows exact", "claim": "claims exact",
                          "toy claim": "toy claims exact",
                          "claim as": "claims exact, in a sentence of their own"}[kind])
            checks.ok(where, "exact")
            continue
        if kind == "table" and item[6] and match_elided(wildcards(want), got):
            checks.count("table rows with `..` (ELIDED)")
            checks.ok(where, "matches with `..` elisions")
            continue
        if kind in ("claim", "toy claim"):
            ok_o, other = results[item[6]]
            # OCaml on the right: the same function, up to parameter names.
            if ok_o and alpha(other) == alpha(got):
                checks.count("toy claims equivalent" if kind == "toy claim" else "claims equivalent")
                checks.ok(where, "same expansion as `%s`" % rust)
                continue
        checks.fail(where, "`%s` expands to\n       %s\n     not %s" % (code, show(got), show(want)))


# ----- completion -----------------------------------------------------------

def load_cases():
    """(name, labels, code) of every case of scripts/completion_cases.txt."""
    cases, cur = [], None
    for line in open(os.path.join(ROOT, "scripts", "completion_cases.txt")):
        line = line.rstrip("\n")
        if line.startswith("#") and cur is None:
            continue
        m = re.match(r"^== (.*?) -> (.*)$", line)
        if m:
            # `a<b` (offered in this order) is the labels `a` and `b`.
            cur = [m.group(1).strip(), re.split(r"[\s<]+", m.group(2).strip()), []]
            cases.append(cur)
        elif cur is not None and (line.strip() or cur[2]):
            cur[2].append(line)
    return [(n, labels, squash("\n".join(code).replace("$0", "‸"))) for n, labels, code in cases]


def squash(code):
    """Code with every run of whitespace one space, and none inside
    brackets (a space before `(` means something in rsml)."""
    code = re.sub(r"\s+", " ", code.strip())
    code = re.sub(r"([(\[{]) ", r"\1", code)
    return re.sub(r" ([)\]}])", r"\1", code)


def cursor_pattern(code):
    """A regex for the cases a documented cursor position stands for: `..`
    stands for anything, and the case may set things up first (`let pairs
    = [(1, 2)] in ..`, `let r = ref 0 in ..`)."""
    parts = [p.strip() for p in squash(code).split("..")]
    body = ".*".join(re.escape(p) for p in parts)
    return re.compile("(?:.* in |.* ;; )?" + body)


def cursor_spans(path):
    """(line, code) for every code span with a cursor outside the checked
    positions' tables of docs/editors.md. In a table, `a` / `b‸` is code on
    two lines."""
    out = []
    for n, line in enumerate(unfenced_lines(path), 1):
        if not line.startswith("|") or path == "docs/editors.md":
            continue
        for cell in split_row(line):
            if re.fullmatch(r"`[^`]+`( / `[^`]+`)+", cell):
                cell = "`%s`" % " ".join(code_spans(cell))
            for code in code_spans(cell):
                if "‸" in code:
                    out.append((n, code))
    for n, text in prose(path):
        out.extend((n, code) for code in code_spans(text) if "‸" in code)
    return sorted(out)


def editor_positions():
    """(line, code, labels) for every row of docs/editors.md's "Every
    checked position" (code None for a row that is not three cells)."""
    lines = unfenced_lines("docs/editors.md")
    start = next(i for i, l in enumerate(lines) if l.startswith("### Every checked position"))
    out = []
    for n, line in enumerate(lines[start:], start + 1):
        if not line.startswith("|") or line.startswith("| Position") or line.startswith("|--"):
            continue
        cells = split_row(line)
        if len(cells) != 3 or len(code_spans(cells[1])) != 1:
            out.append((n, None, None))
            continue
        out.append((n, code_spans(cells[1])[0], code_spans(cells[2])))
    return out


def completion(checks, used):
    cases = load_cases()
    for path in FILES:
        for line, code in cursor_spans(path):
            where = "%s:%d `%s`" % (path, line, code)
            key = (path, code)
            if key in CURSORS_UNCHECKED:
                used.add(("cursor", key))
                checks.count("cursors unchecked")
                checks.ok(where, "unchecked: " + CURSORS_UNCHECKED[key])
                continue
            pat = cursor_pattern(code)
            hits = [n for n, _, c in cases if pat.fullmatch(c)]
            if hits:
                checks.count("cursors with a case")
                checks.ok(where, "case %r" % hits[0])
            else:
                checks.fail(where, "no case in scripts/completion_cases.txt has this code")
    # The checked positions: one row per case, with its code and labels.
    listed = set()
    for line, code, labels in editor_positions():
        where = "docs/editors.md:%d" % line
        if code is None:
            checks.fail(where, "not three cells with one code span (an unescaped `|`?)")
            continue
        where += " `%s`" % code
        same = [(n, l) for n, l, c in cases if c == squash(code)]
        if not same:
            checks.fail(where, "no case in scripts/completion_cases.txt has this code")
            continue
        hit = [n for n, l in same if l == labels]
        if not hit:
            checks.fail(where, "the case offers %s" % " ".join(same[0][1]))
            continue
        listed.update(hit)
        checks.count("checked positions listed")
        checks.ok(where, "case %r" % hit[0])
    for name, labels, code in cases:
        if name not in listed:
            checks.fail("docs/editors.md", "the case %r (`%s`) is not in \"Every checked position\""
                        % (name, code))


def slug(heading):
    """The anchor GitHub gives a heading."""
    text = re.sub(r"[^\w\- ]", "", heading.strip().lower())
    return text.replace(" ", "-")


def anchors(path):
    out, seen = set(), {}
    for line in unfenced_lines(path):
        m = re.match(r"^#+ (.*)$", line)
        if m:
            base = slug(m.group(1))
            n = seen.get(base, 0)
            seen[base] = n + 1
            out.add(base if n == 0 else "%s-%d" % (base, n))
    return out


def links(checks):
    """Every relative link goes to a file that exists, and its `#anchor` to
    a heading there."""
    for path in FILES:
        for n, text in prose(path) + [(i, l) for i, l in enumerate(unfenced_lines(path), 1)
                                      if l.startswith("|")]:
            for target in re.findall(r"\]\(([^)\s]+)\)", text):
                if re.match(r"^[a-z]+:", target):
                    continue
                where = "%s:%d (%s)" % (path, n, target)
                file, _, anchor = target.partition("#")
                full = os.path.normpath(os.path.join(os.path.dirname(path), file)) if file else path
                if not os.path.exists(os.path.join(ROOT, full)):
                    checks.fail(where, "no such file")
                elif anchor and (not full.endswith(".md") or anchor not in anchors(full)):
                    checks.fail(where, "no such heading")
                else:
                    checks.count("links")
                    checks.ok(where, "found")


# ----- the toys -------------------------------------------------------------

TOYS = "examples/toy"
TOY_FILE = re.compile(r"(\d{3})_([a-z][a-z0-9_]*)\.rs")
NOTATION = "docs/notation.md"
NOTATION_LINK = "../../docs/notation.md#"


def toy_files():
    """The toy files (NNN_name.rs), in order, and the other .rs files there."""
    names = sorted(os.listdir(os.path.join(ROOT, TOYS)))
    rs = [n for n in names if n.endswith(".rs")]
    return [n for n in rs if TOY_FILE.fullmatch(n)], [n for n in rs if not TOY_FILE.fullmatch(n)]


def lex_comments(text):
    """The `//` comments of Rust source (strings and characters skipped),
    as (line, own_line, text after the slashes); its `/* */` comments, as
    (lines, text) with the line of each character; and the code of each
    line without its comments, as {line: code}."""
    comments, blocks, code = [], [], {}
    i, n, line, line_start = 0, len(text), 1, 0

    def add_code(a, b):
        for k, part in enumerate(text[a:b].split("\n")):
            code[line_of(a) + k] = code.get(line_of(a) + k, "") + part

    def line_of(pos):
        return text.count("\n", 0, pos) + 1

    start = 0
    while i < n:
        c = text[i]
        if text.startswith("//", i):
            add_code(start, i)
            j = text.find("\n", i)
            j = n if j < 0 else j
            own = text[text.rfind("\n", 0, i) + 1:i].strip() == ""
            body = re.sub(r"^//[/!]?", "", text[i:j])
            comments.append((line_of(i), own, body[1:] if body.startswith(" ") else body))
            i = start = j
        elif text.startswith("/*", i):
            add_code(start, i)
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            parts = [(line_of(i) + k, re.sub(r"^\s*\*?\s*", "", l).strip())
                     for k, l in enumerate(text[i + 2:j - 2].split("\n"))]
            parts = [(l, t) for l, t in parts if t]
            blocks.append(([l for l, t in parts for _ in t + " "] or [line_of(i)],
                           " ".join(t for _, t in parts)))
            i = start = j
        elif c in "rb" and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")) \
                and re.match(r"(?:b?r#*\"|b\")", text[i:]):
            m = re.match(r"(b?)(r?)(#*)\"", text[i:])
            if m.group(2):
                end = text.find('"' + m.group(3), i + len(m.group(0)))
                i = (n if end < 0 else end) + 1 + len(m.group(3))
            else:
                i = skip_string(text, i + len(m.group(0)))
        elif c == '"':
            i = skip_string(text, i + 1)
        elif c == "'":
            m = re.match(r"'(?:\\u\{[0-9a-fA-F]+\}|\\x[0-9a-fA-F]{2}|\\.|[^\\'\n])'", text[i:])
            i += len(m.group(0)) if m else 1
        else:
            i += 1
    add_code(start, n)
    return comments, blocks, code


def skip_string(text, i):
    """The position after the string whose contents start at `i`."""
    while i < len(text) and text[i] != '"':
        i += 2 if text[i] == "\\" else 1
    return i + 1


def comment_paragraphs(comments):
    """(lines, text): consecutive own-line comments joined with spaces (an
    empty comment line ends one), and each comment after code alone;
    `lines` gives the line of each character of `text`."""
    paras, cur, last = [], [], -1

    def flush():
        if cur:
            text = " ".join(t for _, t in cur)
            lines = [l for l, t in cur for _ in t + " "]
            paras.append((lines, text))
        cur.clear()

    for line, own, text in comments:
        if cur and (not own or line != last + 1 or not text.strip()):
            flush()
        if not own:
            paras.append(([line] * (len(text.strip()) + 1), text.strip()))
        elif text.strip():
            cur.append((line, text.strip()))
        last = line
    flush()
    return paras


# A quoted rsml error: "`code` is an error: "message"" (the message may
# contain backticks, never `"`).
ERROR_CLAIM = re.compile(r'`([^`]+)`\s+is\s+an\s+error:\s+"([^"]+)"')
# Phrasings of a claim that the checks would not see, so they are errors:
# a claim about an expansion is written "`a` is `b`".
NEAR_MISS = re.compile(
    r"^\s*(?:expands? to|turns? into|compiles? to|lowers? to|is lowered to|desugars? to|"
    r"translates? to|is translated to|is short for|is shorthand for|is sugar for|"
    r"is the same as|means the same as|is equivalent to|is just|is Rust's|stands for|is an?|"
    r"->|→|=>|⇒)\s*$")
CLAIM_VERB = re.compile(r"^\s*,?\s+(?:which\s+)?(?:is|is always|is also|is still|is then|"
                        r"would be|becomes|stays|reads as|is read as)\s+$")
GIVEN = re.compile(r"(?:^|\W)[Gg]iven $")


def toy_claims(checks, path, lines, text, comment):
    """The claims of one paragraph (of comment, or of code if not
    `comment`): "`a` is `b`" (with "Given `c`, " before it: `c ;;` before
    both sides), and quoted errors. `lines` gives the line of each
    character."""
    def where_at(pos):
        return "%s:%d" % (path, lines[min(pos, len(lines) - 1)])

    for m in ERROR_CLAIM.finditer(text):
        checks.items.append(("quote", "%s `%s`" % (where_at(m.start()), m.group(1)), m.group(1),
                             m.group(2), snippet_request(checks, m.group(1), ""), None))
    # Blanked to the same length, so that positions stay.
    text = ERROR_CLAIM.sub(lambda m: " " * len(m.group(0)), text)
    parts = text.split("`")
    starts = [0]
    for p in parts[:-1]:
        starts.append(starts[-1] + len(p) + 1)
    if len(parts) % 2 == 0:
        if comment:
            checks.fail(where_at(0), "an odd number of backticks in this comment")
        return
    # parts: text, code, text, code, .., text
    context = ""
    for k in range(1, len(parts) - 2, 2):
        a, between, b = parts[k], parts[k + 1], parts[k + 2]
        where = where_at(starts[k])
        if GIVEN.search(parts[k - 1]) and between.startswith(","):
            context = a + " ;;"
        if CLAIM_VERB.match(between):
            w = "%s `%s` is `%s`" % (where, a, b)
            if context and re.match(r"rsml!\s*\{", a):
                checks.fail(w, "a `Given` context cannot go before `rsml! { .. }`")
                continue
            checks.items.append(("toy claim", w, a, b,
                                 snippet_request(checks, a, context),
                                 checks.request("rust", b),
                                 snippet_request(checks, b, context)))
        elif NEAR_MISS.match(between):
            checks.fail(where, "`%s`%s`%s`: write a claim about an expansion as \"`a` is `b`\""
                        % (a, between, b))
    for k in range(1, len(parts) - 1, 2):
        if re.match(r"^\s+is\s+an\s+error\b", parts[k + 1]):
            checks.fail(where_at(starts[k]), "`%s` is an error: quote the message, as "
                        "`%s` is an error: \"..\"" % (parts[k], parts[k]))


def toy_links(checks, path, text):
    """Every relative link of a toy file; returns the notation.md anchors
    its header (the comments before the first line of code) links."""
    header = text[:len(text) - len(text.lstrip())]
    for l in text.split("\n"):
        if l.strip() and not l.strip().startswith("//"):
            break
        header += l + "\n"
    found = set()
    for target in re.findall(r"\]\(([^)\s]+)\)", text):
        if re.match(r"^[a-z]+:", target):
            continue
        where = "%s (%s)" % (path, target)
        file, _, anchor = target.partition("#")
        full = os.path.normpath(os.path.join(os.path.dirname(path), file))
        if not os.path.exists(os.path.join(ROOT, full)):
            checks.fail(where, "no such file")
        elif anchor and (not full.endswith(".md") or anchor not in anchors(full)):
            checks.fail(where, "no such heading")
        else:
            checks.count("toy links")
            checks.ok(where, "found")
            if "](%s" % target in header and target.startswith(NOTATION_LINK):
                found.add(anchor)
    if not found:
        checks.fail(path, "the header links no section of docs/notation.md (`%s..`)" % NOTATION_LINK)
    return found


def toy_file(checks, name):
    """The checks of one toy file; returns (summary, notation.md anchors
    of its header)."""
    path = "%s/%s" % (TOYS, name)
    text = open(os.path.join(ROOT, path)).read()
    comments, blocks, code = lex_comments(text)
    for lines, para in comment_paragraphs(comments) + blocks:
        toy_claims(checks, path, lines, para, True)
    for line in sorted(code):
        if "`" not in code[line]:
            continue
        if "(*" in re.sub(r'"(?:[^"\\]|\\.)*"', '""', code[line]):
            # A claim in it could span lines, unseen: claims go in `//`.
            checks.fail("%s:%d" % (path, line), "code in backticks in a `(* *)` comment")
            continue
        toy_claims(checks, path, [line] * (len(code[line]) + 1), code[line], True)
    summaries = re.findall(r'^pub const SUMMARY: &str =\s*"((?:[^"\\]|\\.)*)";$', text, re.M)
    if len(summaries) != 1:
        checks.fail(path, "not one `pub const SUMMARY: &str = \"..\";` line")
    if not re.search(r"^pub fn run\(\) \{$", text, re.M):
        checks.fail(path, "no `pub fn run() {`")
    if "#[test]" not in text:
        checks.fail(path, "no tests")
    return (summaries[0] if summaries else None), toy_links(checks, path, text)


def toy_index(checks, files, info):
    """examples/toy/README.md's index: one row per toy, in order, with the
    toy's summary and a section its header links."""
    path = TOYS + "/README.md"
    index = [rows for header, rows in tables(path) if header[:2] == ["#", "Toy"]]
    if len(index) != 1:
        checks.fail(path, "not one table with the columns `# | Toy | ..`")
        return
    listed = []
    for line, cells in index[0]:
        where = "%s:%d" % (path, line)
        if len(cells) != 4:
            checks.fail(where, "not four cells")
            continue
        num, toy, summary, section = cells
        m = re.fullmatch(r"\[[^\]]+\]\((\d{3}_[a-z0-9_]+\.rs)\)", toy)
        if not m:
            checks.fail(where, "the toy cell is not a link to its file")
            continue
        listed.append(m.group(1))
        if m.group(1) not in info:
            checks.fail(where, "%s is not a toy" % m.group(1))
            continue
        want, anchors_ = info[m.group(1)]
        if num != m.group(1)[:3]:
            checks.fail(where, "number %s for %s" % (num, m.group(1)))
        if want is not None and summary != want.replace('\\"', '"'):
            checks.fail(where, "the summary is not the toy's SUMMARY:\n       %s\n       %s"
                        % (summary, want))
        s = re.fullmatch(r"\[[^\]]+\]\(%s([^)]+)\)" % re.escape(NOTATION_LINK), section)
        if not s or s.group(1) not in anchors_:
            checks.fail(where, "the section is not a link to docs/notation.md that the toy's header has")
        checks.count("toys indexed")
    if listed != files:
        missing = [f for f in files if f not in listed]
        twice = sorted({f for f in listed if listed.count(f) > 1})
        checks.fail(path, "the index does not list every toy once, in order (missing %s, twice %s)"
                    % (missing, twice))


def toy_main(checks, files):
    """examples/toy/main.rs registers every toy once (as a module named
    after it, in any order: rustfmt sorts them), and its TOYS table has
    each, in order."""
    path = TOYS + "/main.rs"
    text = open(os.path.join(ROOT, path)).read()
    mods = re.findall(r'^#\[path = "([^"]+)"\]\nmod ([a-z0-9_]+);$', text, re.M)
    table = re.findall(r'\(\s*"(\d{3}_[a-z0-9_]+)",\s*([a-z0-9_]+)::SUMMARY,\s*([a-z0-9_]+)::run,?\s*\)',
                       text)
    want = [(f, TOY_FILE.fullmatch(f).group(2)) for f in files]
    if sorted(mods) != want:
        checks.fail(path, "the `#[path] mod` lines are not one per toy, named after it")
    if [(f + ".rs", m) for f, m, r in table] != want or any(m != r for _, m, r in table):
        checks.fail(path, "the TOYS table is not one entry per toy, in order")
    else:
        checks.count("toys registered")


def toys(checks, only=None):
    files, others = toy_files()
    if only is not None:
        for name in only:
            toy_file(checks, os.path.basename(name))
        return
    for name in others:
        if name != "main.rs":
            checks.fail("%s/%s" % (TOYS, name), "not a toy name (NNN_name.rs)")
    info = {name: toy_file(checks, name) for name in files}
    nums = [int(TOY_FILE.fullmatch(f).group(1)) for f in files]
    if nums != list(range(1, len(files) + 1)):
        checks.fail(TOYS, "the toy numbers are not 001 to %03d without gaps" % len(files))
    if len(files) < 120:
        checks.fail(TOYS, "%d toys, fewer than 120" % len(files))
    toy_index(checks, files, info)
    toy_main(checks, files)
    linked = set().union(*(a for _, a in info.values())) if info else set()
    for line in unfenced_lines(NOTATION):
        m = re.match(r"^## (.*)$", line)
        if m and slug(m.group(1)) not in linked:
            checks.fail(NOTATION, "no toy's header links the section %r" % m.group(1))
        elif m:
            checks.count("notation.md sections with a toy")


def main():
    if sys.argv[1:2] == ["--toys"]:
        # Checks the claims and links of the toy files given, and nothing
        # else (for a toy being written).
        checks = Checks(False)
        toys(checks, sys.argv[2:])
        compare(checks, run_helper(checks.requests))
        print("%d checks passed, %d failed"
              % (sum(checks.counts.values()), checks.failed))
        sys.exit(1 if checks.failed else 0)
    if sys.argv[1:2] == ["--expand"]:
        # Prints what each snippet expands to, as the tables should say it.
        checks = Checks(False)
        reqs = [snippet_request(checks, code, "") for code in sys.argv[2:]]
        for code, (ok, got) in zip(sys.argv[2:], run_helper(checks.requests)):
            print("%s\n    %s" % (code, show(got) if ok else "error: " + got))
        return
    args = [a for a in sys.argv[1:] if a != "-v"]
    checks = Checks("-v" in sys.argv[1:])
    used = set()
    for path in FILES:
        table_rows(checks, path, used)
        error_rows(checks, path)
        claims(checks, path, used)
    toys(checks)
    if args:
        checks.items = [i for i in checks.items if args[0] in i[1]]
    compare(checks, run_helper(checks.requests))
    if not args:
        completion(checks, used)
        links(checks)
        stale = [("table", k) for k in UNCHECKED] + [("context", k) for k in CONTEXT] + \
            [("elided", k) for k in ELIDED] + \
            [("claim", k) for k in CLAIMS_UNCHECKED] + [("cursor", k) for k in CURSORS_UNCHECKED] + \
            [("claim context", k) for k in CLAIM_CONTEXT] + [("claim as", k) for k in CLAIM_AS]
        for kind, key in stale:
            if (kind, key) not in used:
                checks.fail("check_docs.py", "the %s entry %r matches nothing" % (kind, key))
    print()
    for kind in sorted(checks.counts):
        print("%5d %s" % (checks.counts[kind], kind))
    checked = sum(v for k, v in checks.counts.items() if "unchecked" not in k)
    print("%d checks passed, %d failed" % (checked, checks.failed))
    sys.exit(1 if checks.failed else 0)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Checks that rust-analyzer completes inside `ml!` / `rsml!` at every case
in scripts/completion_cases.txt.

    python3 scripts/check_completion.py [-v] [NAME_FILTER]

Each case is code with `$0` at the cursor and the completion labels that
must be offered there; `a<b` also means that `a` is listed before `b`
(ranked higher, as a value of the expected type is). The script writes a
small crate under target/completion-check, runs `rust-analyzer` (or
$RUST_ANALYZER) on it over LSP, and asks for completions at each cursor.
`-v` prints the labels offered for every case, not only for failures.

No label may be a local the macros generate (`__rsml_..`), except in the
cases of GENERATED_OFFERED, each with its reason; a case listed there that
offers none fails too, so the list stays current.
"""

import json
import os
import re
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATE = os.path.join(ROOT, "target", "completion-check")
MAIN = os.path.join(CRATE, "src", "main.rs")
URI = "file://" + MAIN

# The cases where rust-analyzer offers a generated local, and why: it offers
# every local in scope at the cursor, macro hygiene or not (see
# docs/editors.md, "Generated names").
GENERATED_OFFERED = {
    "fun: `function` body":
        "a `function` case body is inside the closure `function` stands for, "
        "where its parameter (the value matched) is in scope",
    "fun: `let rec .. = function` case body":
        "the same, in the `fn` a `let rec` stands for",
    "arity: argument after a variable argument, three parameters":
        "the arguments before the cursor are bound to locals ahead of the "
        "closure, so that the expansion lines up for completion",
    "arity: last argument after a constant, long function name":
        "rust-analyzer lines up the closure of `f \"x\"` with the call "
        "`f(\"x\", marker)`; past a long name the marker falls in the "
        "closure's body, where its parameter is in scope",
}

PRELUDE = """\
#![allow(unused)]
use rsml::{ml, rsml};

pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    pub fn norm(&self) -> i64 {
        self.x.abs() + self.y.abs()
    }
}

pub enum Shape {
    Circle(f64),
    Rect(f64, f64),
}

pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn twice(f: impl Fn(i64) -> i64, x: i64) -> i64 {
    f(f(x))
}

pub mod geometry {
    pub fn area(s: &super::Shape) -> f64 {
        0.0
    }
    pub const ORIGIN: super::Point = super::Point { x: 0, y: 0 };
}

fn main() {}
"""

# The locals every `ml!` case can use.
CASE_FN = """\
fn case(
    xs: Vec<i64>,
    s: &str,
    word: String,
    p: Point,
    o: Option<String>,
    n: i64,
    shape: Shape,
    ord: std::cmp::Ordering,
) {
    let _ = ml! {
%s
    };
}
"""


def load_cases(path):
    """`== name -> label ..` starts a case; the lines after it are the code."""
    cases, cur = [], None
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("#") and cur is None:
            continue
        m = re.match(r"^== (.*?) -> (.*)$", line)
        if m:
            cur = {"name": m.group(1).strip(), "expect": m.group(2).split(), "code": []}
            cases.append(cur)
        elif cur is not None and (line.strip() or cur["code"]):
            cur["code"].append(line)
    for c in cases:
        c["code"] = "\n".join(c["code"]).strip("\n")
    return cases


def source(case):
    code = case["code"]
    body = code if code.startswith("rsml!") else CASE_FN % code
    return PRELUDE + "\n" + body + "\n"


def offered(label, want):
    """`len` is offered as `len`, `len()` or `len(…)(as Trait)`, `Vec` as
    `Vec<…>`, `cmp` as `cmp::`."""
    return re.match(re.escape(want) + r"($|[(\s{<!:])", label) is not None


class Lsp:
    def __init__(self, cmd):
        env = dict(os.environ, CARGO_NET_OFFLINE="true")
        self.p = subprocess.Popen([cmd], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, cwd=CRATE, env=env)
        self.next_id = 0
        self.version = 1

    def send(self, msg):
        body = json.dumps(msg).encode()
        self.p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        self.p.stdin.flush()

    def recv(self):
        length = 0
        while True:
            line = self.p.stdout.readline().decode()
            if line in ("\r\n", "\n", ""):
                break
            if line.lower().startswith("content-length:"):
                length = int(line.split(":")[1])
        return json.loads(self.p.stdout.read(length))

    def request(self, method, params):
        self.next_id += 1
        self.send({"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params})
        while True:
            m = self.recv()
            if m.get("id") == self.next_id and "method" not in m:
                return m
            if "method" in m and "id" in m:  # a server request: answer null
                self.send({"jsonrpc": "2.0", "id": m["id"], "result": None})

    def notify(self, method, params):
        self.send({"jsonrpc": "2.0", "method": method, "params": params})

    def complete(self, text):
        """The (label, sortText) of every item offered at `$0` in `text`."""
        i = text.index("$0")
        text = text.replace("$0", "", 1)
        line = text[:i].count("\n")
        col = i - (text[:i].rfind("\n") + 1)
        self.version += 1
        self.notify("textDocument/didChange", {
            "textDocument": {"uri": URI, "version": self.version},
            "contentChanges": [{"text": text}]})
        r = self.request("textDocument/completion", {
            "textDocument": {"uri": URI}, "position": {"line": line, "character": col}})
        res = r.get("result") or []
        items = res["items"] if isinstance(res, dict) else res
        return [(it["label"], it.get("sortText", it["label"])) for it in items]


def main():
    args = [a for a in sys.argv[1:] if a != "-v"]
    verbose = "-v" in sys.argv[1:]
    cases = load_cases(os.path.join(ROOT, "scripts", "completion_cases.txt"))
    if args:
        cases = [c for c in cases if args[0] in c["name"]]

    os.makedirs(os.path.dirname(MAIN), exist_ok=True)
    with open(os.path.join(CRATE, "Cargo.toml"), "w") as f:
        f.write('[package]\nname = "completion-check"\nversion = "0.0.0"\nedition = "2024"\n\n'
                '[dependencies]\nrsml-lang = { path = %s }\n\n[workspace]\n' % json.dumps(ROOT))
    with open(MAIN, "w") as f:
        f.write(PRELUDE)

    lsp = Lsp(os.environ.get("RUST_ANALYZER", "rust-analyzer"))
    lsp.request("initialize", {
        "processId": os.getpid(), "rootUri": "file://" + CRATE,
        "capabilities": {
            "textDocument": {"completion": {"completionItem": {"snippetSupport": True}}},
            "workspace": {"configuration": False}},
        "initializationOptions": {"procMacro": {"enable": True},
                                  "cargo": {"buildScripts": {"enable": True}}}})
    lsp.notify("initialized", {})
    lsp.notify("textDocument/didOpen", {"textDocument": {
        "uri": URI, "languageId": "rust", "version": 1, "text": PRELUDE}})

    # Wait for the proc-macro server: a method completion inside `ml!`.
    warm = source({"code": "xs.$0"})
    deadline = time.time() + 600
    while not any(offered(l, "len") for l, _ in lsp.complete(warm)):
        if time.time() > deadline:
            sys.exit("rust-analyzer never completed inside ml!")
        time.sleep(2)

    failed = 0
    generated = 0
    for c in cases:
        items = lsp.complete(source(c))
        labels = [l for l, _ in sorted(items, key=lambda it: it[1])]
        problems = []
        for chain in c["expect"]:
            # `a<b`: both offered, and `a` sorted first.
            ranks = []
            for want in chain.split("<"):
                hits = [rank for l, rank in items if offered(l, want)]
                if not hits:
                    problems.append(f"missing: {want}")
                ranks.append(min(hits) if hits else None)
            if None not in ranks and any(a >= b for a, b in zip(ranks, ranks[1:])):
                problems.append(f"not in the order {chain}")
        gen = [l for l in labels if l.startswith("__rsml")]
        if gen:
            generated += 1
        if gen and c["name"] not in GENERATED_OFFERED:
            problems.append(f"offers generated locals: {' '.join(gen)}")
        if not gen and c["name"] in GENERATED_OFFERED:
            problems.append("offers no generated local: remove it from GENERATED_OFFERED")
        ok = not problems
        failed += not ok
        print(f"{'ok  ' if ok else 'FAIL'} {c['name']}")
        if not ok or verbose:
            for p in problems:
                print(f"       {p}")
            print(f"       offered ({len(labels)}): {', '.join(labels[:15])}")
    lsp.p.kill()
    names = {c["name"] for c in cases}
    stale = [n for n in GENERATED_OFFERED if not args and n not in names]
    for n in stale:
        print(f"FAIL GENERATED_OFFERED lists {n!r}, which is not a case")
    print(f"\n{len(cases) - failed} of {len(cases)} cases complete as expected"
          f" ({generated} offer generated locals, as listed)")
    sys.exit(1 if failed or stale else 0)


if __name__ == "__main__":
    main()

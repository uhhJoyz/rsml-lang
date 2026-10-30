#!/usr/bin/env python3
"""Measures rust-analyzer's completion latency in big files written with
`rsml!`, against the same code as plain Rust.

    python3 scripts/check_latency.py [--sizes 500,1500,3300] [--max-ms N] [--opt]

For each size (in lines) the script writes a crate under
target/latency-check holding the same functions three ways:

    plain   plain Rust
    whole   one `rsml! { .. }` around the whole file
    split   one `rsml! { .. }` per 10 pairs of functions (about 110 lines)

and, for each, starts `rust-analyzer` (or $RUST_ANALYZER) over LSP, waits
until completion after `xs.` in the last function offers `len`, then types
`le` there six times (as an editor sends it: the whole new text) and times
the completion request after each edit. It prints the median per file.

A check fails if completion ever stops offering `len`, or, with
`--max-ms`, if a median is above N milliseconds. `--opt` builds the macros
optimized, as docs/editors.md suggests (`opt-level = 3` for rsml's crates in
the crate's dev profile): rust-analyzer builds them with `cargo check`,
unoptimized by default.
"""

OPT = """
[profile.dev.package.rsml-macros]
opt-level = 3
[profile.dev.package.rsml-compiler]
opt-level = 3
[profile.dev.package.proc-macro2]
opt-level = 3
"""

import json
import os
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATE = os.path.join(ROOT, "target", "latency-check")
MAIN = os.path.join(CRATE, "src", "main.rs")

PLAIN_PAIR = """\
fn f{i}(xs: Vec<i64>, k: i64) -> i64 {{
    let ys = xs.iter().map(|x| x * k + {i}).collect::<Vec<_>>();
    match ys.first() {{
        Some(y) if *y > 0 => *y + f{h}_helper(k),
        _ => ys.len() as i64,
    }}
}}

fn f{i}_helper(k: i64) -> i64 {{
    if k > 10 {{ k }} else {{ k * 2 }}
}}

"""

RSML_PAIR = """\
fn f{i}(xs: Vec<i64>, k: i64) -> i64 {{
    let ys = xs |> .iter() |> .map(fun x -> x * k + {i}) |> .collect::<Vec<_>>() in
    match ys.first() with
    | Some y when !y > 0 -> !y + f{h}_helper k
    | _ -> ys.len() as i64
}}

fn f{i}_helper(k: i64) -> i64 {{
    if k > 10 then k else k * 2
}}

"""

TARGET = """\
fn target(xs: Vec<i64>) -> usize {
    xs.len()
}
"""


def pairs(template, lo, hi):
    return "".join(template.format(i=i, h=max(i - 1, 0)) for i in range(lo, hi))


def source(kind, lines):
    n = max((lines - 6) // 11, 1)
    if kind == "plain":
        return "#![allow(dead_code)]\n" + pairs(PLAIN_PAIR, 0, n) + TARGET + "fn main() {}\n"
    if kind == "whole":
        return ("#![allow(dead_code)]\nrsml::rsml! {\n" + pairs(RSML_PAIR, 0, n) + TARGET
                + "}\nfn main() {}\n")
    # One block per 10 pairs. A helper in the block before is a function of
    # unknown arity there: `f9_helper k` is `f9_helper(k)` all the same.
    out = "#![allow(dead_code)]\n"
    for lo in range(0, n, 10):
        body = pairs(RSML_PAIR, lo, min(lo + 10, n))
        if lo + 10 >= n:
            body += TARGET
        out += "rsml::rsml! {\n" + body + "}\n"
    return out + "fn main() {}\n"


class Lsp:
    def __init__(self):
        cmd = os.environ.get("RUST_ANALYZER", "rust-analyzer")
        self.p = subprocess.Popen([cmd], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, cwd=CRATE,
                                  env=dict(os.environ, CARGO_NET_OFFLINE="true"))
        self.next_id = 0
        self.request("initialize", {
            "processId": os.getpid(), "rootUri": "file://" + CRATE,
            "capabilities": {"textDocument": {"completion": {
                "completionItem": {"snippetSupport": False}}}},
            "initializationOptions": {"procMacro": {"enable": True}, "checkOnSave": False,
                                      "cachePriming": {"enable": False}}})
        self.notify("initialized", {})

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

    def close(self):
        self.p.kill()


def labels(r):
    items = r.get("result") or []
    if isinstance(items, dict):
        items = items.get("items", [])
    return [i["label"] for i in items]


def measure(text):
    """Seconds until ready, and the completion times after each edit."""
    with open(MAIN, "w") as f:
        f.write(text)
    lsp = Lsp()
    uri = "file://" + MAIN
    lsp.notify("textDocument/didOpen", {"textDocument": {
        "uri": uri, "languageId": "rust", "version": 1, "text": text}})
    lines = text.split("\n")
    ln = max(i for i, l in enumerate(lines) if l.strip() == "xs.len()")
    col = lines[ln].index("xs.") + 3
    start = time.time()
    while True:
        r = lsp.request("textDocument/completion", {
            "textDocument": {"uri": uri}, "position": {"line": ln, "character": col}})
        if any(l.startswith("len") for l in labels(r)):
            break
        if time.time() - start > 300:
            lsp.close()
            return None, []
        time.sleep(1)
    ready = time.time() - start
    times = []
    for k in range(6):
        new = lines[:]
        new[ln] = lines[ln][:col] + "le" + " " * k + lines[ln][col:]
        lsp.notify("textDocument/didChange", {
            "textDocument": {"uri": uri, "version": k + 2},
            "contentChanges": [{"text": "\n".join(new)}]})
        t = time.time()
        r = lsp.request("textDocument/completion", {
            "textDocument": {"uri": uri}, "position": {"line": ln, "character": col + 2}})
        dt = time.time() - t
        times.append(dt if any(l.startswith("len") for l in labels(r)) else None)
    lsp.close()
    return ready, times


def main():
    args = sys.argv[1:]
    sizes = [500, 1500, 3300]
    max_ms = None
    if "--sizes" in args:
        sizes = [int(s) for s in args[args.index("--sizes") + 1].split(",")]
    if "--max-ms" in args:
        max_ms = float(args[args.index("--max-ms") + 1])
    opt = OPT if "--opt" in args else ""

    os.makedirs(os.path.dirname(MAIN), exist_ok=True)
    with open(os.path.join(CRATE, "Cargo.toml"), "w") as f:
        f.write('[package]\nname = "latency-check"\nversion = "0.0.0"\nedition = "2024"\n\n'
                '[dependencies]\nrsml-lang = { path = %s }\n\n[workspace]\n%s' % (json.dumps(ROOT), opt))
    failed = 0
    checks = 0
    print("%-6s %6s %6s %10s %12s" % ("file", "lines", "blocks", "ready", "median"))
    for size in sizes:
        for kind in ["plain", "whole", "split"]:
            text = source(kind, size)
            ready, times = measure(text)
            checks += 1
            ok = ready is not None and len(times) == 6 and all(t is not None for t in times)
            median = sorted(times)[len(times) // 2] * 1000 if ok else None
            if ok and max_ms is not None and median > max_ms:
                ok = False
            failed += not ok
            print("%-6s %6d %6d %9.1fs %10s   %s" % (
                kind, text.count("\n"), text.count("rsml::rsml!"), ready or 0,
                "%.0f ms" % median if median is not None else "-",
                "ok" if ok else "FAIL"))
            sys.stdout.flush()
    print("\n%d of %d checks passed" % (checks - failed, checks))
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()

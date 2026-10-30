#!/usr/bin/env python3
"""Checks how rust-analyzer colors the keywords and operators inside `ml!`
/ `rsml!`, for every case in scripts/highlight_cases.txt.

    python3 scripts/check_highlight.py [-v] [NAME_FILTER]

Each case is code and the semantic token type every token with a given
text must get (`then=keyword`). The script writes a small crate under
target/highlight-check with each case in a function of its own, runs
`rust-analyzer` (or $RUST_ANALYZER) on it over LSP and asks for the
semantic tokens of the file. A token with no semantic token at all (an
identifier-colored `then`) fails. `-v` prints every token of every case.
"""

import json
import os
import re
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATE = os.path.join(ROOT, "target", "highlight-check")
MAIN = os.path.join(CRATE, "src", "main.rs")
URI = "file://" + MAIN

PRELUDE = """\
#![allow(unused)]
use rsml::{ml, rsml};

pub struct Point {
    pub x: i64,
    pub y: i64,
}

fn main() {}
"""

CASE_FN = """\
fn case_%d(
    xs: Vec<i64>,
    n: i64,
    o: Option<i64>,
    p: Point,
    a: bool,
    b: bool,
    r: Result<i64, String>,
    f: fn(i64) -> i64,
    g: fn(i64) -> i64,
) {
    let _ = ml! {
%s
    };
}
"""


def load_cases(path):
    """`== name -> text=type ..` starts a case; the lines after it are the code."""
    cases, cur = [], None
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("#") and cur is None:
            continue
        m = re.match(r"^== (.*?) -> (.*)$", line)
        if m:
            expect = [e.rsplit("=", 1) for e in m.group(2).split()]
            cur = {"name": m.group(1).strip(), "expect": expect, "code": []}
            cases.append(cur)
        elif cur is not None and (line.strip() or cur["code"]):
            cur["code"].append(line)
    for c in cases:
        c["code"] = "\n".join(c["code"]).strip("\n")
    return cases


def lex(code):
    """(offset, text) of each token: names (`r#type` too), numbers, strings,
    and every other character on its own."""
    out, i = [], 0
    while i < len(code):
        m = (re.compile(r"\s+").match(code, i) or re.compile(r'"(\\.|[^"\\])*"').match(code, i)
             or re.compile(r"(r#)?[A-Za-z_][A-Za-z0-9_]*|[0-9][0-9_]*(\.[0-9][0-9_]*)?").match(code, i))
        if m:
            if not code[i].isspace():
                out.append((i, m.group(0)))
            i = m.end()
        else:
            out.append((i, code[i]))
            i += 1
    return out


class Lsp:
    def __init__(self, cmd):
        env = dict(os.environ, CARGO_NET_OFFLINE="true")
        self.p = subprocess.Popen([cmd], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, cwd=CRATE, env=env)
        self.next_id = 0

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


def main():
    args = [a for a in sys.argv[1:] if a != "-v"]
    verbose = "-v" in sys.argv[1:]
    cases = load_cases(os.path.join(ROOT, "scripts", "highlight_cases.txt"))
    if args:
        cases = [c for c in cases if args[0] in c["name"]]

    text = PRELUDE
    for i, c in enumerate(cases):
        body = c["code"] if c["code"].startswith("rsml!") else CASE_FN % (i, c["code"])
        text += "\n"
        c["start"] = len(text) + body.index(c["code"])
        text += body
    os.makedirs(os.path.dirname(MAIN), exist_ok=True)
    with open(os.path.join(CRATE, "Cargo.toml"), "w") as f:
        f.write('[package]\nname = "highlight-check"\nversion = "0.0.0"\nedition = "2024"\n\n'
                '[dependencies]\nrsml-lang = { path = %s }\n\n[workspace]\n' % json.dumps(ROOT))
    with open(MAIN, "w") as f:
        f.write(text)
    line_starts = [0] + [m.end() for m in re.finditer("\n", text)]

    lsp = Lsp(os.environ.get("RUST_ANALYZER", "rust-analyzer"))
    init = lsp.request("initialize", {
        "processId": os.getpid(), "rootUri": "file://" + CRATE,
        "capabilities": {"textDocument": {"semanticTokens": {
            "requests": {"full": True}, "tokenTypes": [], "tokenModifiers": [],
            "formats": ["relative"]}},
            "workspace": {"configuration": False}},
        "initializationOptions": {"procMacro": {"enable": True},
                                  "cargo": {"buildScripts": {"enable": True}}}})
    legend = init["result"]["capabilities"]["semanticTokensProvider"]["legend"]
    lsp.notify("initialized", {})
    lsp.notify("textDocument/didOpen", {"textDocument": {
        "uri": URI, "languageId": "rust", "version": 1, "text": text}})

    # Wait for the proc-macro server: tokens from macro expansions, the
    # same twice in a row.
    tokens, last = {}, None
    deadline = time.time() + 600
    while True:
        r = lsp.request("textDocument/semanticTokens/full", {"textDocument": {"uri": URI}})
        data = (r.get("result") or {}).get("data") or []
        tokens, line, col = {}, 0, 0
        for i in range(0, len(data), 5):
            dl, dc, ln, ty, mods = data[i:i + 5]
            line += dl
            col = col + dc if dl == 0 else dc
            names = [m for b, m in enumerate(legend["tokenModifiers"]) if mods >> b & 1]
            tokens[line_starts[line] + col] = (ln, legend["tokenTypes"][ty], names)
        expanded = sum("macro" in t[2] for t in tokens.values())
        if expanded and expanded == last:
            break
        if time.time() > deadline:
            sys.exit("rust-analyzer never expanded the macros")
        last = expanded
        time.sleep(3)
    lsp.p.kill()

    failed = 0
    for c in cases:
        bad = []
        lexed = [(c["start"] + k, t) for k, t in lex(c["code"])]
        for want_text, want in c["expect"]:
            seen = [(at, t) for at, t in lexed if t == want_text]
            if not seen:
                bad.append("`%s` is not in the code" % want_text)
            for at, t in seen:
                got = tokens.get(at)
                if got is None or got[1] != want:
                    bad.append("`%s` at %d: %s, expected %s"
                               % (t, at - c["start"], got[1] if got else "no token", want))
        failed += bool(bad)
        print("%s %s" % ("FAIL" if bad else "ok  ", c["name"]))
        for b in bad:
            print("       " + b)
        if verbose:
            for at, t in lexed:
                got = tokens.get(at)
                print("       %-10r %s" % (t, "%s %s" % (got[1], " ".join(got[2])) if got else "-"))
    print("\n%d of %d cases highlighted as expected" % (len(cases) - failed, len(cases)))
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()

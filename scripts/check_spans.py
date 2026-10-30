#!/usr/bin/env python3
"""Checks where rustc points its errors in `ml!` / `rsml!` code, for every
case in scripts/span_cases.txt.

    python3 scripts/check_spans.py [-v] [NAME_FILTER]

Each case is erroneous code and the text of the primary span its error
must have. The script writes a crate under target/span-check with every
case in a module of its own, builds it once with `--message-format=json`,
and checks that each case gets an error with that span, that no error
falls back to the whole macro call, and that no message (notes and help
included) shows a name the macros generate for rust-analyzer
(`__rsml_..`). `-v` prints every error's span.
"""

import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATE = os.path.join(ROOT, "target", "span-check")
LIB = os.path.join(CRATE, "src", "lib.rs")

PRELUDE = """\
#![allow(unused)]
use rsml::{ml, rsml};

pub struct Point {
    pub x: i64,
    pub y: i64,
}

pub fn add(a: i64, b: i64) -> i64 {
    a + b
}
"""


def load_cases(path):
    """`== name -> text` starts a case; the lines after it are the code."""
    cases, cur = [], None
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("#") and cur is None:
            continue
        m = re.match(r"^== (.*?) -> (.*)$", line)
        if m:
            cur = {"name": m.group(1).strip(), "expect": m.group(2).strip(), "code": []}
            cases.append(cur)
        elif cur is not None and (line.strip() or cur["code"]):
            cur["code"].append(line)
    for c in cases:
        c["code"] = "\n".join(c["code"]).strip("\n")
    return cases


def is_ours(span):
    return os.path.abspath(os.path.join(CRATE, span["file_name"])) == LIB


def main():
    args = [a for a in sys.argv[1:] if a != "-v"]
    verbose = "-v" in sys.argv[1:]
    cases = load_cases(os.path.join(ROOT, "scripts", "span_cases.txt"))
    if args:
        cases = [c for c in cases if args[0] in c["name"]]

    # Every case in a module; remember the byte range of each.
    text = PRELUDE
    for i, c in enumerate(cases):
        text += "\nmod case_%d {\n    use super::*;\n" % i
        c["start"] = len(text.encode())
        text += c["code"] + "\n"
        c["end"] = len(text.encode())
        text += "}\n"
    os.makedirs(os.path.dirname(LIB), exist_ok=True)
    with open(os.path.join(CRATE, "Cargo.toml"), "w") as f:
        f.write('[package]\nname = "span-check"\nversion = "0.0.0"\nedition = "2024"\n\n'
                '[lib]\npath = "src/lib.rs"\n\n'
                '[dependencies]\nrsml-lang = { path = %s }\n\n[workspace]\n' % json.dumps(ROOT))
    with open(LIB, "w") as f:
        f.write(text)
    data = text.encode()

    out = subprocess.run(["cargo", "build", "--quiet", "--message-format=json"],
                         cwd=CRATE, capture_output=True, text=True).stdout
    errors = []
    for line in out.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("reason") != "compiler-message" or m["message"]["level"] != "error":
            continue
        msg = m["message"]
        for s in msg["spans"]:
            if not s["is_primary"]:
                continue
            # Like rustc's own output, a span inside a macro of another
            # crate (`vec!`) is shown where that macro is called.
            while not is_ours(s) and s.get("expansion"):
                s = s["expansion"]["span"]
            ours = is_ours(s)
            errors.append({
                "message": msg["message"],
                "rendered": msg.get("rendered") or "",
                "start": s["byte_start"] if ours else None,
                "text": data[s["byte_start"]:s["byte_end"]].decode() if ours
                else "%s:%d" % (s["file_name"], s["line_start"]),
            })

    failed = 0
    for c in cases:
        mine = [e for e in errors if e["start"] is not None and c["start"] <= e["start"] < c["end"]]
        whole = [e for e in mine if re.match(r"(rsml|ml)\s*!", e["text"])]
        generated = [e for e in mine if "__rsml" in e["rendered"]]
        ok = any(e["text"] == c["expect"] for e in mine) and not whole and not generated
        failed += not ok
        print("%s %s" % ("ok  " if ok else "FAIL", c["name"]))
        if not ok or verbose:
            if not mine:
                print("     no error")
            for e in mine:
                print("     %r: %s" % (e["text"], e["message"]))
            for e in generated:
                print("     a generated name in:\n" + e["rendered"])
            if not ok:
                print("     expected %r" % c["expect"])
    stray = [e for e in errors if e["start"] is None or not any(
        c["start"] <= e["start"] < c["end"] for c in cases)]
    for e in stray:
        print("FAIL error outside the cases: %r: %s" % (e["text"], e["message"]))
    failed += len(stray)
    print("\n%d of %d cases point where expected" % (len(cases) - failed, len(cases)))
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Checks that every `ml_*` function in examples/zero_cost.rs compiles to the
same machine code as its hand-written `rs_*` twin.

    cargo rustc --release --example zero_cost -- --emit=asm
    python3 scripts/compare_asm.py target/release/examples/zero_cost-*.s

LLVM merges functions with identical bodies; such pairs show up as aliases
(`_rs_pipe = _ml_pipe`), which counts as identical.
"""

import difflib
import re
import sys


def parse(path):
    funcs, aliases, cur = {}, {}, None
    for line in open(path):
        line = line.rstrip("\n")
        m = re.match(r"^_?((?:ml|rs)_\w+) = _?(\w+)", line)
        if m:
            aliases[m.group(1)] = m.group(2)
            continue
        m = re.match(r"^_?((?:ml|rs)_\w+):", line)
        if m:
            cur = m.group(1)
            funcs[cur] = []
            continue
        if cur is None:
            continue
        s = line.strip()
        if s.startswith(".cfi_endproc"):
            cur = None
            continue
        if not s or s.startswith((".cfi", ";", "#", ".loh", ".p2align")):
            continue
        s = re.sub(r"\.?L\w+", "L", s)  # local labels
        s = re.sub(r"h[0-9a-f]{16}", "H", s)  # symbol hashes
        funcs[cur].append(s)
    return funcs, aliases


def main():
    funcs, aliases = parse(sys.argv[1])
    names = sorted({n[3:] for n in list(funcs) + list(aliases) if n.startswith(("ml_", "rs_"))})
    ok = True
    for name in names:
        ml, rs = "ml_" + name, "rs_" + name
        if aliases.get(rs) == ml or aliases.get(ml) == rs:
            print(f"{name:10} identical (merged by LLVM)")
            continue
        a, b = funcs.get(ml), funcs.get(rs)
        same = a is not None and a == b
        ok &= same
        print(f"{name:10} {'identical' if same else 'DIFFERENT'} ({len(a or [])} instructions)")
        if not same:
            for d in difflib.unified_diff(b or [], a or [], rs, ml, lineterm="", n=1):
                print("   ", d)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()

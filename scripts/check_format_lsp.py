#!/usr/bin/env python3
"""Checks that rust-analyzer formats a file through `rsml-fmt --rustfmt`,
set as its `rustfmt.overrideCommand`.

    python3 scripts/check_format_lsp.py [-v] [--nvim]

The script builds rsml-fmt, writes a small crate (edition 2021) under
target/format-check whose main.rs has messy Rust, `ml!` blocks and an
`rsml!` block, starts `rust-analyzer` (or $RUST_ANALYZER) over LSP with

    "rustfmt": { "overrideCommand": ["<target/debug/rsml-fmt>", "--rustfmt"] }

and sends `textDocument/formatting`. The edits it gets back must turn the
file into EXPECTED, which also shows that the crate's edition was found
(`use x::{a10, a9}` is 2021's order; 2024 sorts `a9` first). A file that
does not parse must get no edits. `-v` also runs the command through a
wrapper that prints what rust-analyzer passes it (arguments, directory).
`--nvim` also formats the file in a headless Neovim (0.11+, no plugins)
whose built-in LSP client starts rust-analyzer with the same setting in
`settings`, as docs/editors.md shows, and checks the buffer.
"""

import json
import os
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATE = os.path.join(ROOT, "target", "format-check")
MAIN = os.path.join(CRATE, "src", "main.rs")
BROKEN = os.path.join(CRATE, "src", "broken.rs")
BIN = os.path.join(ROOT, "target", "debug", "rsml-fmt")

SAMPLE = """\
use rsml::{ml, rsml};
use x::{a9, a10};
fn   main( ) { let xs=vec![1,2,3];
  let n = ml!{ xs |>   .iter() |> .map(fun x ->x*2)|> .sum::<i64>() };
    assert_eq!(n,12);
    println!("{}",ml!{ if n>10 then begin println!("big"); n end else 0 }) }
fn double(x: i64) -> i64 { ml! { x*2 } }

rsml! {
pub struct   Point{ x:i64,y :i64 }
// Adds.
pub fn   add(a:i64,b:i64)->i64 { a+b }
pub fn   twice(p : Point) -> i64 { add p.x p.y |> fun s -> s*2 }
}
"""

EXPECTED = """\
use rsml::{ml, rsml};
use x::{a10, a9};
fn main() {
    let xs = vec![1, 2, 3];
    let n = ml! { xs |> .iter() |> .map(fun x -> x * 2) |> .sum::<i64>() };
    assert_eq!(n, 12);
    println!(
        "{}",
        ml! {
        if n > 10 then begin
          println!("big");
          n
        end
        else
          0
        }
    )
}
fn double(x: i64) -> i64 { ml! {
    x * 2
}}

rsml! {
pub struct Point {
    x: i64,
    y: i64,
}
// Adds.
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}
pub fn twice(p: Point) -> i64 {
    add p.x p.y |> fun s -> s * 2
}
}
"""

BROKEN_TEXT = "fn main( { let x = ml! { 1 |> f };\n"

NVIM_INIT = """\
vim.cmd("filetype on")
vim.lsp.config("rust-analyzer", {
  cmd = { RA },
  filetypes = { "rust" },
  root_markers = { "Cargo.toml" },
  settings = {
    ["rust-analyzer"] = {
      rustfmt = { overrideCommand = { BIN, "--rustfmt" } },
    },
  },
})
vim.lsp.enable("rust-analyzer")
vim.cmd("edit " .. FILE)
if not vim.wait(120000, function() return #vim.lsp.get_clients({ bufnr = 0 }) > 0 end, 200) then
  io.stdout:write("no rust-analyzer client")
  os.exit(1)
end
vim.wait(3000, function() return false end, 200)
vim.lsp.buf.format({ async = false, timeout_ms = 60000 })
io.stdout:write(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), "\\n") .. "\\n")
os.exit(0)
"""

WRAPPER = """\
#!/usr/bin/env python3
import os, subprocess, sys
data = sys.stdin.read()
sys.stderr.write("argv=%r cwd=%r stdin=%d bytes\\n" % (sys.argv[1:], os.getcwd(), len(data)))
open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "wrapper.log"), "w").write(
    "argv=%r\\ncwd=%r\\nstdin=%d bytes\\n" % (sys.argv[1:], os.getcwd(), len(data)))
r = subprocess.run([BIN_PATH] + sys.argv[1:], input=data, capture_output=True, text=True)
sys.stdout.write(r.stdout); sys.stderr.write(r.stderr); sys.exit(r.returncode)
"""


class Lsp:
    def __init__(self, cmd, command):
        env = dict(os.environ, CARGO_NET_OFFLINE="true")
        self.p = subprocess.Popen([cmd], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, cwd=CRATE, env=env)
        self.next_id = 0
        self.request("initialize", {
            "processId": os.getpid(), "rootUri": "file://" + CRATE, "capabilities": {},
            "initializationOptions": {"rustfmt": {"overrideCommand": command},
                                      "procMacro": {"enable": True}}})
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

    def format(self, path, text):
        uri = "file://" + path
        self.notify("textDocument/didOpen", {"textDocument": {
            "uri": uri, "languageId": "rust", "version": 1, "text": text}})
        deadline = time.time() + 300
        while True:
            r = self.request("textDocument/formatting", {
                "textDocument": {"uri": uri}, "options": {"tabSize": 4, "insertSpaces": True}})
            # rust-analyzer answers once the workspace is loaded.
            if "error" not in r or time.time() > deadline or "rustfmt" in json.dumps(r["error"]):
                return r
            time.sleep(2)

    def close(self):
        self.p.kill()


def apply(text, edits):
    """Applies LSP text edits (the sample is ASCII, so UTF-16 columns are
    byte columns)."""
    starts = [0]
    for line in text.split("\n"):
        starts.append(starts[-1] + len(line) + 1)

    def offset(pos):
        return starts[pos["line"]] + pos["character"]

    for e in sorted(edits, key=lambda e: offset(e["range"]["start"]), reverse=True):
        a, b = offset(e["range"]["start"]), offset(e["range"]["end"])
        text = text[:a] + e["newText"] + text[b:]
    return text


def main():
    verbose = "-v" in sys.argv[1:]
    subprocess.run(["cargo", "build", "-q", "-p", "rsml-fmt"], cwd=ROOT, check=True)
    os.makedirs(os.path.dirname(MAIN), exist_ok=True)
    with open(os.path.join(CRATE, "Cargo.toml"), "w") as f:
        f.write('[package]\nname = "format-check"\nversion = "0.0.0"\nedition = "2021"\n\n'
                '[dependencies]\nrsml-lang = { path = %s }\n\n[workspace]\n' % json.dumps(ROOT))
    with open(MAIN, "w") as f:
        f.write(SAMPLE)
    with open(BROKEN, "w") as f:
        f.write(BROKEN_TEXT)
    ra = os.environ.get("RUST_ANALYZER", "rust-analyzer")
    failed = 0

    # The command alone, as an editor would set it.
    direct = subprocess.run([BIN, "--rustfmt"], input=SAMPLE, capture_output=True, text=True,
                            cwd=os.path.dirname(MAIN))
    ok = direct.returncode == 0 and direct.stdout == EXPECTED
    failed += not ok
    print("%s rsml-fmt --rustfmt on stdin" % ("ok  " if ok else "FAIL"))
    if not ok:
        print(direct.stderr + direct.stdout)

    lsp = Lsp(ra, [BIN, "--rustfmt"])
    r = lsp.format(MAIN, SAMPLE)
    got = apply(SAMPLE, r.get("result") or []) if "result" in r else None
    ok = got == EXPECTED
    failed += not ok
    print("%s textDocument/formatting through rust-analyzer" % ("ok  " if ok else "FAIL"))
    if not ok:
        print(json.dumps(r)[:2000] if got is None else got)

    r = lsp.format(BROKEN, BROKEN_TEXT)
    ok = "error" in r or not r.get("result")
    failed += not ok
    print("%s a file that does not parse gets no edits" % ("ok  " if ok else "FAIL"))
    if not ok:
        print(json.dumps(r)[:2000])
    lsp.close()

    checks = 3
    if "--nvim" in sys.argv[1:]:
        checks += 1
        init = os.path.join(CRATE, "nvim_init.lua")
        with open(MAIN, "w") as f:
            f.write(SAMPLE)
        with open(init, "w") as f:
            f.write(NVIM_INIT.replace("RA", json.dumps(ra)).replace("BIN", json.dumps(BIN))
                    .replace("FILE", json.dumps(MAIN)))
        r = subprocess.run(["nvim", "--headless", "-u", "NONE", "-l", init],
                           capture_output=True, text=True, timeout=600)
        ok = r.stdout == EXPECTED
        failed += not ok
        print("%s vim.lsp.buf.format() in Neovim" % ("ok  " if ok else "FAIL"))
        if not ok:
            print(r.stdout + r.stderr)

    if verbose:
        wrapper = os.path.join(CRATE, "wrapper.py")
        with open(wrapper, "w") as f:
            f.write(WRAPPER.replace("BIN_PATH", repr(BIN)))
        os.chmod(wrapper, 0o755)
        lsp = Lsp(ra, [wrapper, "--rustfmt"])
        lsp.format(MAIN, SAMPLE)
        lsp.close()
        print("rust-analyzer ran the command with:")
        print(open(os.path.join(CRATE, "wrapper.log")).read())

    print("\n%d of %d checks passed" % (checks - failed, checks))
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()

-- Checks editors/nvim/queries/rust/highlights.scm, which colors OCaml
-- `(* .. *)` comments inside macros as comments, in a headless Neovim:
--
--     nvim --headless -u NONE -l scripts/check_nvim_query.lua
--
-- It needs tree-sitter's Rust parser. The usual places are searched (the
-- data directory's `site` and `ts-parsers`, nvim-treesitter's, the
-- runtimepath); RSML_NVIM_PARSERS can name another directory holding
-- `parser/rust.so`. For each case, the capture at a position of a sample
-- file must (or must not) be `@comment` from the query.

local root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h:h")
vim.opt.runtimepath:append(root .. "/editors/nvim")
local data = vim.fn.stdpath("data")
for _, dir in ipairs({
  os.getenv("RSML_NVIM_PARSERS") or "",
  data .. "/site",
  data .. "/ts-parsers",
  data .. "/lazy/nvim-treesitter",
  data .. "/site/pack/core/opt/nvim-treesitter",
  data .. "/site/pack/core/opt/nvim-treesitter/runtime",
}) do
  if dir ~= "" and vim.fn.isdirectory(dir) == 1 then
    vim.opt.runtimepath:append(dir)
  end
end

local sample = [[
use rsml::{ml, rsml};

fn f(x: &i64, y: i64) -> i64 {
    let p = (*x) * y;
    ml! {
        (* a comment *) y + ( * ) 1 2 + ( *x) + (* line one
           line two *) y + ( ** ) 2.0 3.0 + (**) 1.0 2.0 + (* *) 1
        + (* outer (* inner *) still *) 2
    }
}

rsml! {
    fn g(n: i64) -> i64 {
        (* in an rsml! body *) n * 2
    }
}
]]

-- (text to find, offset in it, expected: a comment or not)
local cases = {
  { "(* a comment *)", 3, true },
  { "(* a comment *)", 0, true },
  { "line two", 0, true },
  { "(* *)", 0, true },
  { "outer", 0, true },
  { "still", 0, true },
  { "in an rsml! body", 0, true },
  { "( * )", 2, false },
  { "( *x)", 3, false },
  { "( ** )", 2, false },
  { "(**)", 1, false },
  { "(*x) * y", 2, false },
  { "y + ( * )", 0, false },
  { "n * 2", 0, false },
}

local ok, err = pcall(vim.treesitter.language.add, "rust")
if not ok or not vim.treesitter.language.add("rust") then
  io.stdout:write("FAIL no tree-sitter parser for Rust: " .. tostring(err) .. "\n")
  os.exit(1)
end

local file = vim.fn.tempname() .. ".rs"
vim.fn.writefile(vim.split(sample, "\n"), file)
vim.cmd("edit " .. vim.fn.fnameescape(file))
vim.treesitter.start(0, "rust")
vim.treesitter.get_parser(0, "rust"):parse(true)
local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
local ours = root .. "/editors/nvim/queries/rust/highlights.scm"

local function find(text)
  for row, line in ipairs(lines) do
    local col = line:find(text, 1, true)
    if col then
      return row - 1, col - 1
    end
  end
end

local failed = 0
for _, c in ipairs(cases) do
  local text, off, want = c[1], c[2], c[3]
  local row, col = find(text)
  local got, prio = false, nil
  if row then
    for _, cap in ipairs(vim.treesitter.get_captures_at_pos(0, row, col + off)) do
      if cap.capture == "comment" and cap.metadata and tonumber(cap.metadata.priority) == 130 then
        got, prio = true, cap.metadata.priority
      end
    end
  end
  local pass = row ~= nil and got == want
  if not pass then
    failed = failed + 1
  end
  io.stdout:write(string.format("%s %-22s %s\n", pass and "ok  " or "FAIL",
    string.format("%q", text) .. "+" .. off, want and "comment" or "not a comment"))
end
local files = vim.treesitter.query.get_files("rust", "highlights")
local loaded = vim.tbl_contains(files, ours)
if not loaded then
  failed = failed + 1
  io.stdout:write("FAIL the query was not loaded from " .. ours .. "\n")
end
io.stdout:write(string.format("\n%d of %d cases as expected (%d highlight query files for rust)\n",
  #cases - math.max(failed - (loaded and 0 or 1), 0), #cases, #files))
os.exit(failed == 0 and 0 or 1)

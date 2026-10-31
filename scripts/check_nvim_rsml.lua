-- Checks editors/nvim/lua/rsml (hidden `ml! {` / `}`, marked lines, the
-- statusline indicator) in a headless Neovim:
--
--     nvim --headless -u NONE -l scripts/check_nvim_rsml.lua
--
-- It needs tree-sitter's Rust parser, found as scripts/check_nvim_query.lua
-- finds it (RSML_NVIM_PARSERS can name a directory holding
-- `parser/rust.so`).

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
use rsml::ml;

fn demo(a: i128, b: u64) -> usize { ml! {
    .saturating_add (a as usize) (b as usize)
}}

fn own_line(xs: &[i64]) -> i64 {
    let k = 3;
    ml! {
    xs |> .iter() |> .sum()
    }
}

fn args() {
    assert_eq!(ml! { 1 + 2 }, 3);
}

rsml! {
fn f(x: i64) -> i64 {
    x |> g
}
}
]]

local ok, err = pcall(vim.treesitter.language.add, "rust")
if not ok or not vim.treesitter.language.add("rust") then
  io.stdout:write("FAIL no tree-sitter parser for Rust: " .. tostring(err) .. "\n")
  os.exit(1)
end

local file = vim.fn.tempname() .. ".rs"
vim.fn.writefile(vim.split(sample, "\n"), file)
vim.cmd("edit " .. vim.fn.fnameescape(file))
vim.bo.filetype = "rust"
local rsml = require("rsml")
rsml.setup()
local buf = vim.api.nvim_get_current_buf()
local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
local ns = vim.api.nvim_get_namespaces()["rsml"]

local function find(text, from)
  for row = from or 1, #lines do
    local col = lines[row]:find(text, 1, true)
    if col then
      return row - 1, col - 1
    end
  end
end

-- The text each line shows: concealed ranges left out.
local function shown(row)
  local text = lines[row + 1]
  local hidden = {}
  for _, m in ipairs(vim.api.nvim_buf_get_extmarks(buf, ns, 0, -1, { details = true })) do
    local r, c, d = m[2], m[3], m[4]
    if d.conceal then
      for k = r, d.end_row do
        if k == row then
          local from = k == r and c or 0
          local to = k == d.end_row and d.end_col or #lines[k + 1]
          table.insert(hidden, { from, to })
        end
      end
    end
  end
  local out = {}
  for i = 1, #text do
    local keep = true
    for _, h in ipairs(hidden) do
      if i - 1 >= h[1] and i - 1 < h[2] then
        keep = false
      end
    end
    if keep then
      table.insert(out, text:sub(i, i))
    end
  end
  return table.concat(out)
end

local function marked(row)
  for _, m in ipairs(vim.api.nvim_buf_get_extmarks(buf, ns, { row, 0 }, { row, -1 }, { details = true })) do
    if m[4].line_hl_group == "RsmlBlock" then
      return true
    end
  end
  return false
end

local failed, total = 0, 0
local function check(what, got, want)
  total = total + 1
  local pass = got == want
  if not pass then
    failed = failed + 1
  end
  io.stdout:write(
    string.format("%s %-44s %s\n", pass and "ok  " or "FAIL", what, pass and "" or vim.inspect(got))
  )
end

local demo = find("fn demo")
check("hugged: `ml! {` hidden on the fn's line", shown(demo), "fn demo(a: i128, b: u64) -> usize {")
check("hugged: that line marked", marked(demo), true)
local close = find("}}")
check("hugged: `}}` shows one `}`", shown(close), "}")
check("hugged: the closing line marked", marked(close), true)
check("hugged: body line untouched", shown(demo + 1), lines[demo + 2])

local own = find("ml! {", demo + 4)
check("own line: `ml! {` hidden", shown(own), "    ")
check("own line: marked", marked(own), true)
check("own line: `}` hidden", shown(own + 2), "    ")

local args = find("assert_eq!")
check("one line, in a macro's arguments", shown(args), "    assert_eq!(1 + 2, 3);")
check("one line: not a marked line", marked(args), false)

local items = find("rsml! {")
check("rsml!: hidden", shown(items), "")

-- The indicator, inside a block's brackets only.
local _, body_col = find(".saturating_add")
check("in rsml code", rsml.in_rsml(buf, demo + 1, body_col), true)
check("in the signature", rsml.in_rsml(buf, demo, 3), false)
local _, one = find("1 + 2")
check("in `ml! { 1 + 2 }`", rsml.in_rsml(buf, args, one), true)
check("in `assert_eq!`'s `3`", rsml.in_rsml(buf, args, #lines[args + 1] - 3), false)
vim.api.nvim_win_set_cursor(0, { demo + 2, body_col })
check("statusline()", rsml.statusline(), "- RSML -")
check("'statusline' has the item", vim.go.statusline:find("rsml", 1, true) ~= nil, true)
local function status()
  return vim.api.nvim_eval_statusline(vim.go.statusline, { maxwidth = 200 }).str
end
check("the statusline shows it", status():sub(1, 9), "- RSML - ")
vim.api.nvim_win_set_cursor(0, { demo + 1, 3 })
check("not outside rsml code", status():find("RSML", 1, true), nil)
vim.api.nvim_win_set_cursor(0, { demo + 2, body_col })
check("'conceallevel' in the window", vim.wo.conceallevel, 2)

-- Edits move the marks; `:RsmlToggle` shows the macros again.
vim.api.nvim_buf_set_lines(buf, 0, 0, false, { "// a new first line" })
vim.api.nvim_exec_autocmds("TextChanged", { buffer = buf })
lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
check("after an edit", shown(demo + 1), "fn demo(a: i128, b: u64) -> usize {")
vim.cmd("RsmlToggle")
check("toggled off", shown(demo + 1), lines[demo + 2])
check("toggled off: 'conceallevel'", vim.wo.conceallevel, 0)

io.stdout:write(string.format("\n%d of %d cases as expected\n", total - failed, total))
os.exit(failed == 0 and 0 or 1)

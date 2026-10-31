-- rsml in Neovim: the `ml! {` and `}` around rsml code hidden, the lines
-- they were on marked with a background, and "- RSML -" in the statusline
-- while the cursor is in rsml code. Needs tree-sitter's Rust parser.
--
--     require("rsml").setup()                     -- the defaults below
--     require("rsml").setup({ statusline = false }) -- your own statusline:
--         -- lualine: sections = { lualine_a = { "mode", require("rsml").statusline } }
--
-- `:RsmlToggle` shows or hides the macros. The cursor's line shows them as
-- written, as 'concealcursor' is empty by default.

local M = {}

M.config = {
  -- Hide `ml! {`, `rsml! {` and their `}` (with 'conceallevel' 2 in Rust
  -- windows).
  conceal = true,
  -- What the statusline shows in rsml code.
  indicator = "- RSML -",
  -- Put the indicator at the front of 'statusline'.
  statusline = true,
}

local ns = vim.api.nvim_create_namespace("rsml")
local macros = { ml = true, rsml = true }
-- Per buffer: the rsml code's ranges, { start_row, start_col, end_row,
-- end_col } (0-based, end exclusive), the inside of each block's brackets.
local ranges = {}
local attached = {}
local shown = false

local query
local function calls_query()
  query = query
    or vim.treesitter.query.parse(
      "rust",
      [[
      (macro_invocation macro: (_) @name (token_tree) @body)
      (token_tree (identifier) @name . "!" . (token_tree) @body)
      ]]
    )
  return query
end

--- The `ml!` and `rsml!` calls of a buffer, as { name = node, body = node }:
--- calls in Rust code, and in the arguments of other macros
--- (`assert_eq!(ml! { .. }, 3)`), which tree-sitter keeps as tokens.
function M.blocks(buf)
  local ok, parser = pcall(vim.treesitter.get_parser, buf, "rust")
  if not ok or not parser then
    return {}
  end
  local tree = parser:parse()[1]
  if not tree then
    return {}
  end
  local out = {}
  for _, match in calls_query():iter_matches(tree:root(), buf, 0, -1) do
    local call = {}
    for id, nodes in pairs(match) do
      -- One node per capture (Neovim 0.11+ gives lists).
      local node = type(nodes) == "table" and nodes[#nodes] or nodes
      call[query.captures[id]] = node
    end
    local text = vim.treesitter.get_node_text(call.name, buf)
    if macros[text:match("([%w_]+)%s*$") or ""] then
      table.insert(out, call)
    end
  end
  return out
end

local function line(buf, row)
  return vim.api.nvim_buf_get_lines(buf, row, row + 1, false)[1] or ""
end

--- Recomputes the hidden macros and the marked lines of `buf`.
function M.refresh(buf)
  buf = (buf == nil or buf == 0) and vim.api.nvim_get_current_buf() or buf
  vim.api.nvim_buf_clear_namespace(buf, ns, 0, -1)
  local found = {}
  for _, call in ipairs(M.blocks(buf)) do
    local sr, sc = call.name:range()
    local br, bc, er, ec = call.body:range()
    table.insert(found, { br, bc + 1, er, ec - 1 })
    if M.config.conceal then
      local one_line = br == er
      -- `ml! {`, with the spaces before it after code (`fn f() { ml! {`)
      -- and, on one line, the space after it.
      local first = line(buf, sr)
      local from = sc
      if first:sub(1, sc):find("%S") then
        while from > 0 and first:sub(from, from) == " " do
          from = from - 1
        end
      end
      local open_end = bc + 1
      if one_line and line(buf, br):sub(bc + 2, bc + 2) == " " then
        open_end = open_end + 1
      end
      vim.api.nvim_buf_set_extmark(buf, ns, sr, from, {
        end_row = br,
        end_col = open_end,
        conceal = "",
      })
      -- `}`, and on one line the space before it.
      local close = ec - 1
      if one_line and close > open_end and line(buf, er):sub(close, close) == " " then
        close = close - 1
      end
      vim.api.nvim_buf_set_extmark(buf, ns, er, close, {
        end_row = er,
        end_col = ec,
        conceal = "",
      })
      if one_line then
        vim.api.nvim_buf_set_extmark(buf, ns, br, open_end, {
          end_row = er,
          end_col = close,
          hl_group = "RsmlBlock",
        })
      else
        for _, row in ipairs({ sr, er }) do
          vim.api.nvim_buf_set_extmark(buf, ns, row, 0, { line_hl_group = "RsmlBlock" })
        end
      end
    end
  end
  ranges[buf] = found
end

--- Whether the cursor (or `row`, `col`, 0-based) is in rsml code.
function M.in_rsml(buf, row, col)
  buf = (buf == nil or buf == 0) and vim.api.nvim_get_current_buf() or buf
  if row == nil then
    local cursor = vim.api.nvim_win_get_cursor(0)
    row, col = cursor[1] - 1, cursor[2]
  end
  for _, r in ipairs(ranges[buf] or {}) do
    local after_start = row > r[1] or (row == r[1] and col >= r[2])
    local before_end = row < r[3] or (row == r[3] and col < r[4])
    if after_start and before_end then
      return true
    end
  end
  return false
end

--- The indicator in rsml code, else "": for a statusline.
function M.statusline()
  return M.in_rsml() and M.config.indicator or ""
end

--- The statusline item `setup` adds: the indicator, colored, and a space.
function M.statusline_item()
  if not M.in_rsml() then
    return ""
  end
  return "%#RsmlMode#" .. M.config.indicator .. "%* "
end

local function conceal_windows(buf)
  for _, win in ipairs(vim.fn.win_findbuf(buf)) do
    vim.api.nvim_set_option_value(
      "conceallevel",
      M.config.conceal and 2 or 0,
      { scope = "local", win = win }
    )
  end
end

--- Starts hiding the macros of `buf` (a Rust buffer) and following edits.
function M.attach(buf)
  buf = (buf == nil or buf == 0) and vim.api.nvim_get_current_buf() or buf
  conceal_windows(buf)
  M.refresh(buf)
  if attached[buf] then
    return
  end
  attached[buf] = true
  local group = vim.api.nvim_create_augroup("rsml_buf_" .. buf, { clear = true })
  vim.api.nvim_create_autocmd({ "TextChanged", "TextChangedI", "InsertLeave" }, {
    group = group,
    buffer = buf,
    callback = function()
      M.refresh(buf)
    end,
  })
  vim.api.nvim_create_autocmd("BufWinEnter", {
    group = group,
    buffer = buf,
    callback = function()
      conceal_windows(buf)
    end,
  })
  vim.api.nvim_create_autocmd({ "CursorMoved", "CursorMovedI" }, {
    group = group,
    buffer = buf,
    callback = function()
      local now = M.in_rsml(buf)
      if now ~= shown then
        shown = now
        vim.cmd.redrawstatus()
      end
    end,
  })
  vim.api.nvim_create_autocmd("BufWipeout", {
    group = group,
    buffer = buf,
    callback = function()
      attached[buf], ranges[buf] = nil, nil
      vim.api.nvim_del_augroup_by_id(group)
    end,
  })
end

--- Shows or hides the macros in every Rust buffer.
function M.toggle()
  M.config.conceal = not M.config.conceal
  for buf in pairs(attached) do
    conceal_windows(buf)
    M.refresh(buf)
  end
end

function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", M.config, opts or {})
  vim.api.nvim_set_hl(0, "RsmlBlock", { link = "CursorLine", default = true })
  vim.api.nvim_set_hl(0, "RsmlMode", { link = "ModeMsg", default = true })
  local group = vim.api.nvim_create_augroup("rsml", { clear = true })
  vim.api.nvim_create_autocmd("FileType", {
    group = group,
    pattern = "rust",
    callback = function(args)
      M.attach(args.buf)
    end,
  })
  for _, buf in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_loaded(buf) and vim.bo[buf].filetype == "rust" then
      M.attach(buf)
    end
  end
  vim.api.nvim_create_user_command("RsmlToggle", M.toggle, { force = true })
  if M.config.statusline and not vim.go.statusline:find("rsml", 1, true) then
    local rest = vim.go.statusline
    if rest == "" then
      rest = "%<%f %h%w%m%r%=%-14.(%l,%c%V%) %P"
    end
    vim.go.statusline = "%{%v:lua.require'rsml'.statusline_item()%}" .. rest
  end
end

return M

" The checks of scripts/check_vim.sh, which runs this file in a headless
" Vim (or Neovim) without plugins, with target/debug at the front of the
" PATH, after writing the files of target/vim-check:
"
"     messy.rs     a messy file: Rust, `ml!` blocks, one that does not parse
"     expected.rs  what `rsml-fmt --rustfmt` makes of it on the command line
"     broken.rs    a file that is not valid Rust
"
" The results go to target/vim-check/result-NAME.txt (g:rsml_check_name):
" one line per check, `ok` or `FAIL`, then a summary.

let s:root = fnamemodify(expand('<sfile>:p'), ':h:h')
let s:dir = s:root . '/target/vim-check'
let s:out = []
let s:failed = 0
let s:count = 0

function! s:Check(ok, what, ...) abort
  let s:count += 1
  call add(s:out, (a:ok ? 'ok   ' : 'FAIL ') . a:what)
  if !a:ok
    let s:failed += 1
    for detail in a:000
      for line in type(detail) == v:t_list ? detail : [detail]
        call add(s:out, '       ' . line)
      endfor
    endfor
  endif
endfunction

" A copy of `src` in the check directory, opened in a fresh buffer.
function! s:Open(src, name) abort
  let file = s:dir . '/src/' . a:name
  call writefile(readfile(s:dir . '/' . a:src), file)
  silent! %bwipeout!
  execute 'silent edit ' . fnameescape(file)
  return file
endfunction

" Ends an undo step, as Vim does between commands typed by a user.
function! s:SyncUndo() abort
  let &undolevels = &undolevels
endfunction

function! s:Diff(got, want) abort
  for i in range(max([len(a:got), len(a:want)]))
    let g = get(a:got, i, '<none>')
    let w = get(a:want, i, '<none>')
    if g !=# w
      return printf('line %d: got %s, want %s', i + 1, string(g), string(w))
    endif
  endfor
  return 'same'
endfunction

let s:messy = readfile(s:dir . '/messy.rs')
let s:expected = readfile(s:dir . '/expected.rs')
let s:broken = readfile(s:dir . '/broken.rs')
call s:Check(s:messy !=# s:expected, 'the sample needs formatting')

" ----- the filter and 'formatprg' -------------------------------------------

call s:Open('messy.rs', 'filter.rs')
silent %!rsml-fmt --rustfmt --stdin-filepath %:p 2>/dev/null
call s:Check(getline(1, '$') ==# s:expected,
      \ ':%!rsml-fmt --rustfmt --stdin-filepath %:p 2>/dev/null formats like the command line',
      \ s:Diff(getline(1, '$'), s:expected))

call s:Open('messy.rs', 'formatprg.rs')
setlocal formatprg=rsml-fmt\ --rustfmt\ --stdin-filepath\ %:p\ 2>/dev/null
silent normal! gggqG
call s:Check(getline(1, '$') ==# s:expected,
      \ "'formatprg' and gggqG format like the command line",
      \ s:Diff(getline(1, '$'), s:expected))

" `gq` on part of the file: a function is formatted as a file of its own,
" a statement (not valid Rust alone) is replaced with nothing.
call s:Open('messy.rs', 'part.rs')
setlocal formatprg=rsml-fmt\ --rustfmt\ --stdin-filepath\ %:p\ 2>/dev/null
let s:start = search('^fn   shout')
let s:end = search('^}', 'W')
let s:want = systemlist('rsml-fmt --rustfmt --stdin-filepath ' . shellescape(expand('%:p'))
      \ . ' 2>/dev/null', getline(s:start, s:end))
execute 'silent normal! ' . s:start . 'Ggq' . s:end . 'G'
call s:Check(s:start > 0 && v:shell_error == 0
      \ && getline(s:start, s:start + len(s:want) - 1) ==# s:want
      \ && getline(1, s:start - 1) ==# s:messy[: s:start - 2],
      \ 'gq on a function formats it as a file of its own',
      \ s:Diff(getline(s:start, s:start + len(s:want) - 1), s:want))
let s:line = search('^    let   total')
let s:before = line('$')
execute 'silent normal! ' . s:line . 'Ggq' . s:line . 'G'
call s:Check(s:line > 0 && line('$') == s:before - 1,
      \ 'gq on a statement replaces it with nothing', 'lines: ' . line('$') . ', was ' . s:before)

call s:Open('broken.rs', 'broken-filter.rs')
silent %!rsml-fmt --rustfmt --stdin-filepath %:p 2>/dev/null
let s:emptied = getline(1, '$') ==# ['']
silent undo
call s:Check(s:emptied && getline(1, '$') ==# s:broken,
      \ 'the filter on a file that is not Rust leaves the buffer empty, and u restores it')

" ----- format on save, as docs/editors.md has it --------------------------

let s:docs = readfile(s:root . '/docs/editors.md')
let s:code = []
let s:in = 0
for s:l in s:docs
  if s:in && s:l ==# '```'
    if !empty(filter(copy(s:code), 'v:val =~# "^function! RsmlFmt("'))
      break
    endif
    let s:in = 0
    let s:code = []
  elseif s:in
    call add(s:code, s:l)
  elseif s:l ==# '```vim'
    let s:in = 1
  endif
endfor
call s:Check(!empty(s:code), 'docs/editors.md has the format-on-save code')
call writefile(s:code, s:dir . '/rsml_fmt.vim')
execute 'source ' . fnameescape(s:dir . '/rsml_fmt.vim')

" A messy file, with the cursor on code below what changes and a change
" of the user's in the last line.
let s:file = s:Open('messy.rs', 'save.rs')
call s:SyncUndo()
let s:mark = search('cursor goes here')
call cursor(s:mark, 12)
let s:text = getline('.')
normal! zt
let s:row = winline()
call setline('$', '// the end, edited')
call s:SyncUndo()
let s:seq = undotree().seq_last
write
let s:want = s:expected[:-2] + ['// the end, edited']
call s:Check(getline(1, '$') ==# s:want && readfile(s:file) ==# s:want,
      \ 'saving formats the buffer and writes it formatted', s:Diff(readfile(s:file), s:want))
call s:Check(getline('.') ==# s:text && col('.') == 12 && line('.') != s:mark
      \ && winline() == s:row,
      \ 'the cursor stays on its text (which moved), at its row of the window',
      \ 'at ' . line('.') . ':' . col('.') . ', row ' . winline() . ' (was ' . s:row . '): ' . getline('.'))
call s:Check(undotree().seq_last == s:seq, 'saving adds no undo step',
      \ 'seq_last ' . undotree().seq_last . ', was ' . s:seq)
call s:Check(empty(filter(getline(1, '$'), 'v:val =~# "does not parse"')),
      \ "rsml-fmt's warning is not in the buffer")
call s:Check(execute('messages') =~# 'does not parse', "rsml-fmt's warning is in :messages")
silent undo
call s:Check(getline(1, '$') ==# s:messy, 'u undoes the change and its formatting together',
      \ s:Diff(getline(1, '$'), s:messy))
silent redo
silent write

" Saving a formatted file changes nothing.
let s:seq = undotree().seq_last
let s:lines = getline(1, '$')
call cursor(3, 5)
write
call s:Check(getline(1, '$') ==# s:lines && undotree().seq_last == s:seq && !&modified
      \ && line('.') == 3 && col('.') == 5,
      \ 'saving a formatted file changes nothing, not even undo')

" A file that does not parse is saved as it is.
let s:file = s:Open('broken.rs', 'broken-save.rs')
call setline(1, getline(1) . ' ')
write
call s:Check(getline(1, '$') ==# readfile(s:file) && readfile(s:file)[1:] ==# s:broken[1:],
      \ 'a file that is not Rust is saved as it is')
call s:Check(execute('messages') =~# 'rustfmt failed', "rsml-fmt's error is in :messages")

" :RsmlFmt is an undo step of its own.
call s:Open('messy.rs', 'command.rs')
call s:SyncUndo()
RsmlFmt
call s:Check(getline(1, '$') ==# s:expected && &modified, ':RsmlFmt formats the buffer')
silent undo
call s:Check(getline(1, '$') ==# s:messy, 'u after :RsmlFmt undoes the formatting')

" ----- highlighting ---------------------------------------------------------

" `out` lines are outside the macros; the probes below give positions in
" the others.
let s:sample = [
      \ ['out', 'use rsml::{ml, rsml};'],
      \ ['out', ''],
      \ ['out', "fn f(x: &i64, y: i64, c: bool, s: &'static str) -> i64 {"],
      \ ['out', '    let fun = 1; let with = 2; let done = 3; let to = 4; let end = fun + with + done + to;'],
      \ ['out', '    let p = (*x) * y; // a Rust comment with then; do'],
      \ ['mixed', '    let r = ml! {'],
      \ ['in', '        (* a comment *) if c then y + ( * ) 1 2 + ( *x) + (*x) + (* line one'],
      \ ['in', '           line two *) y + ( ** ) 2 3 + (**) 1 2 + (* *) 1'],
      \ ['in', '        + (* outer (* inner *) still *) 2 + (* f(x) "*)" *) 3'],
      \ ['in', '        + (match Some 1 with Some k when k > 0 -> k | _ -> 0)'],
      \ ['in', '        + (let rec go n = n in go 1) + (fun v -> v) 1 + (function _ -> 1) 2'],
      \ ['in', '        + (for i = 0 to 3 do () done; for i = 3 downto 0 do () done; 0)'],
      \ ['in', '        + (begin 1 end) + (if not c then 1 else 0) + (1 land 3) + (1 lsl 2)'],
      \ ['in', '        + (let final = 1 in final) + (if c.then(fun () -> 1).is_some() then 1 else 0)'],
      \ ['in', '        + (let r#end = 1 in r#end) + (assert c; 0) + (String::from "}").len() as i64'],
      \ ['in', "        + ('}' as i64) + { Point { x = 1; y = 2 } with x = 0 }.x"],
      \ ['mixed', '    };'],
      \ ['mixed', '    let q = ml!(if c then 1 else 0) + ml![if c then 1 else 0] + end;'],
      \ ['out', "    let ch = 'a'; let t = \"with } fun\"; // after: with done"],
      \ ['out', '    let u = [1, 2].iter().map(|v| v + to).sum::<i64>();'],
      \ ['out', '    end + p + r + q + (ch as i64) + (t.len() as i64) + (s.len() as i64) + u'],
      \ ['out', '}'],
      \ ['out', ''],
      \ ['mixed', 'rsml! {'],
      \ ['in', '    fn g(n: i64) -> i64 {'],
      \ ['in', '        (* in an rsml! body *) match n with 0 -> 1 | _ -> n * 2'],
      \ ['in', '    }'],
      \ ['mixed', '}'],
      \ ['out', ''],
      \ ['out', 'fn h() -> i64 { let done = 1; let to = 2; let fun = done; do_it(done, to, fun) }'],
      \ ]

" [text, offset in it, syntax item with the file, without it]
let s:probes = [
      \ ['(* a comment', 0, 'rsmlComment', ''],
      \ ['(* a comment', 3, 'rsmlComment', ''],
      \ ['*) if c', 1, 'rsmlComment', ''],
      \ ['if c then y', 0, 'rustConditional', 'rustConditional'],
      \ ['then y', 0, 'rsmlConditional', ''],
      \ ['( * )', 2, 'rustOperator', 'rustOperator'],
      \ ['( *x)', 2, 'rustSigil', 'rustSigil'],
      \ ['(*x) +', 1, 'rustSigil', 'rustSigil'],
      \ ['(*x) +', 3, 'rsmlBracket', ''],
      \ ['(* line one', 3, 'rsmlComment', ''],
      \ ['line two', 0, 'rsmlComment', ''],
      \ ['line two *) y', 12, '', ''],
      \ ['( ** )', 2, 'rustSigil', 'rustSigil'],
      \ ['(**)', 1, 'rustSigil', 'rustSigil'],
      \ ['(* *)', 0, 'rsmlComment', ''],
      \ ['(* *) 1', 6, 'rustDecNumber', 'rustDecNumber'],
      \ ['outer', 0, 'rsmlComment', ''],
      \ ['inner', 0, 'rsmlComment', ''],
      \ ['still', 0, 'rsmlComment', ''],
      \ ['(* f(x)', 5, 'rsmlComment', ''],
      \ ['"*)" *) 3', 2, 'rsmlComment', 'rustString'],
      \ ['"*)" *) 3', 8, 'rustDecNumber', 'rustDecNumber'],
      \ ['with Some', 0, 'rsmlConditional', ''],
      \ ['when k', 0, 'rsmlConditional', ''],
      \ ['rec go', 0, 'rsmlKeyword', ''],
      \ ['fun v', 0, 'rsmlKeyword', ''],
      \ ['function _', 0, 'rsmlConditional', ''],
      \ ['to 3', 0, 'rsmlRepeat', ''],
      \ ['do ()', 0, 'rsmlRepeat', 'rustReservedKeyword'],
      \ ['done;', 0, 'rsmlRepeat', ''],
      \ ['downto', 0, 'rsmlRepeat', ''],
      \ ['begin 1', 0, 'rsmlKeyword', ''],
      \ ['end)', 0, 'rsmlKeyword', ''],
      \ ['not c', 0, 'rsmlKeyword', ''],
      \ ['land', 0, 'rsmlOperator', ''],
      \ ['lsl', 0, 'rsmlOperator', ''],
      \ ['final = 1', 0, '', 'rustReservedKeyword'],
      \ ['then(fun', 0, 'rustFuncCall', 'rustFuncCall'],
      \ ['r#end', 2, 'rustRawIdent', 'rustRawIdent'],
      \ ['assert c', 0, 'rsmlKeyword', ''],
      \ ['"}"', 1, 'rustString', 'rustString'],
      \ ['"}").len() as i64', 11, 'rustOperator', 'rustOperator'],
      \ ["'}'", 1, 'rustCharacter', 'rustCharacter'],
      \ ['} with x', 2, 'rsmlConditional', ''],
      \ ['in final', 0, 'rustRepeat', 'rustRepeat'],
      \ ['ml!(if c then', 9, 'rsmlConditional', ''],
      \ ['ml!(if c then', 0, 'rustMacro', 'rustMacro'],
      \ ['ml![if c then', 9, 'rsmlConditional', ''],
      \ ['+ end;', 2, '', ''],
      \ ['in an rsml', 0, 'rsmlComment', 'rustRepeat'],
      \ ['let r = ml! {', 0, 'rustKeyword', 'rustKeyword'],
      \ ['let r = ml! {', 8, 'rustMacro', 'rustMacro'],
      \ ['let r = ml! {', 12, 'rsmlBracket', ''],
      \ ['    };', 4, 'rsmlBracket', ''],
      \ ['    };', 5, '', ''],
      \ ['rsml! {', 0, 'rustMacro', 'rustMacro'],
      \ ['rsml! {', 6, 'rsmlBracket', ''],
      \ ['match n with', 8, 'rsmlConditional', ''],
      \ ['let fun', 4, '', ''],
      \ ['let with', 4, '', ''],
      \ ['let done', 4, '', ''],
      \ ['let to', 4, '', ''],
      \ ['let end', 4, '', ''],
      \ ['do_it(done, to, fun)', 6, '', ''],
      \ ['do_it(done, to, fun)', 16, '', ''],
      \ ['then; do', 0, 'rustCommentLine', 'rustCommentLine'],
      \ ["'a'", 0, 'rustCharacter', 'rustCharacter'],
      \ ['"with } fun"', 1, 'rustString', 'rustString'],
      \ ['// after: with', 10, 'rustCommentLine', 'rustCommentLine'],
      \ ["'static", 1, 'rustLifetime', 'rustLifetime'],
      \ ['|v| v + to', 8, '', ''],
      \ ]

let s:sfile = s:dir . '/src/sample.rs'
call writefile(map(copy(s:sample), 'v:val[1]'), s:sfile)
let s:after = s:root . '/editors/vim/after'

" The syntax item at every position, and at each probe.
function! s:Items() abort
  let all = []
  for l in range(1, line('$'))
    call add(all, map(range(1, max([col([l, '$']) - 1, 1])), 'synIDattr(synID(l, v:val, 1), "name")'))
  endfor
  let probes = []
  for [text, off; rest] in s:probes
    let found = ''
    for l in range(1, line('$'))
      let c = stridx(getline(l), text)
      if c >= 0
        let found = synIDattr(synID(l, c + 1 + off, 1), 'name')
        break
      endif
      if l == line('$')
        let found = '<text not found>'
      endif
    endfor
    call add(probes, found)
  endfor
  return [all, probes]
endfunction

function! s:Highlight(with_file) abort
  silent! %bwipeout!
  let &runtimepath = s:rtp . (a:with_file ? ',' . s:after : '')
  let v:errmsg = ''
  execute 'silent edit ' . fnameescape(s:sfile)
  return [&syntax, v:errmsg] + s:Items()
endfunction

" Vim's own runtime files only, not those of the user's configuration.
let s:rtp = $VIMRUNTIME
syntax on
let [s:syn_with, s:err_with, s:with, s:probes_with] = s:Highlight(1)
let [s:syn_without, s:err_without, s:without, s:probes_without] = s:Highlight(0)
call s:Check(s:syn_with ==# 'rust' && s:syn_without ==# 'rust' && s:err_with ==# '',
      \ "the sample is highlighted with Vim's Rust syntax, with the file and without",
      \ s:syn_with . ' / ' . s:syn_without . ' / ' . s:err_with)
let s:bad = []
for s:i in range(len(s:probes))
  let [s:text, s:off, s:want_with, s:want_without] = s:probes[s:i]
  if s:probes_with[s:i] !=# s:want_with || s:probes_without[s:i] !=# s:want_without
    call add(s:bad, printf('%s+%d: %s / %s, want %s / %s', s:text, s:off,
          \ s:probes_with[s:i], s:probes_without[s:i], s:want_with, s:want_without))
  endif
endfor
call s:Check(empty(s:bad), printf('%d positions inside the macros, with the file and without', len(s:probes)), s:bad)
let s:bad = []
let s:n = 0
for s:l in range(len(s:sample))
  if s:sample[s:l][0] ==# 'out'
    let s:n += len(s:with[s:l])
    if s:with[s:l] !=# s:without[s:l]
      call add(s:bad, printf('line %d: %s', s:l + 1, s:sample[s:l][1]))
    endif
  endif
endfor
call s:Check(empty(s:bad) && s:n > 400,
      \ printf('every one of %d positions outside the macros is highlighted as without the file', s:n), s:bad)
let s:bad = []
for [s:group, s:link] in [['rsmlComment', 'Comment'], ['rsmlConditional', 'Conditional'],
      \ ['rsmlRepeat', 'Repeat'], ['rsmlKeyword', 'Keyword'], ['rsmlOperator', 'Operator']]
  if execute('highlight ' . s:group) !~# 'links to ' . s:link . '\>'
    call add(s:bad, s:group . ': ' . trim(execute('highlight ' . s:group)))
  endif
endfor
call s:Check(empty(s:bad), 'the groups link to Comment, Conditional, Repeat, Keyword and Operator', s:bad)

" A whole file in one `rsml!`: highlighted from its start, so the end of a
" long one is still inside the block.
let s:long = ['rsml! {']
for s:k in range(1500)
  call add(s:long, printf('    fn f%d(n: i64) -> i64 { n + %d }', s:k, s:k))
endfor
let s:long += ['    fn last(c: bool) -> i64 { if c then 1 else 0 }', '}']
let s:lfile = s:dir . '/src/long.rs'
call writefile(s:long, s:lfile)
let &runtimepath = s:rtp . ',' . s:after
silent! %bwipeout!
execute 'silent edit ' . fnameescape(s:lfile)
let s:at = len(s:long) - 1
let s:item = synIDattr(synID(s:at, stridx(getline(s:at), 'then') + 1, 1), 'name')
call s:Check(s:item ==# 'rsmlConditional', 'the last lines of a long whole-file rsml! are highlighted', s:item)

call add(s:out, '')
call add(s:out, printf('%d of %d checks passed', s:count - s:failed, s:count))
call writefile(s:out, s:dir . '/result-' . get(g:, 'rsml_check_name', 'vim') . '.txt')
qall!

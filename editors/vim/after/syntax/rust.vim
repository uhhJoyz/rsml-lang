" Vim syntax additions for rsml (see docs/editors.md in its repository):
" the OCaml inside `ml! { .. }` and `rsml! { .. }`, on top of Vim's own
" Rust syntax, which keeps coloring everything else (strings, characters,
" lifetimes, `//` and `/* */` comments, numbers, Rust keywords).
"
" Inside the body of an `ml!` or `rsml!` call (with `{}`, `()` or `[]`),
" and only there:
"
" * `(* .. *)` comments, nested ones too, are Comment. `( * )` and
"   `( ** )` (operator sections), `( *x)` and `(*x)` (dereferences) are not
"   comments: a comment starts with `(*` and ends with the `*)` that closes
"   its parentheses, as the macros read it (here with at most two levels of
"   parentheses inside it, and not spaced as `( * c * )`, which the macros
"   take for a comment too).
" * The OCaml keywords that Rust does not have are colored: `then`, `when`,
"   `with` and `function` as Conditional, `do`, `done`, `to`, `downto` and `til` as
"   Repeat, `fun`, `rec`, `and`, `begin`, `end`, `open`, `external`, `not`
"   and `assert` as Keyword, and `land`, `lor`, `lxor`, `lsl`, `lsr`, `asr`
"   and `or` as Operator. Not after `.`, `::` or `r#` (`c.then(f)`,
"   `r#end`), nor before `!` (`assert!`). `let`, `in`, `if`, `else`,
"   `match`, `for`, `while`, `ref`, `as` and `mod` keep their Rust colors.
" * The words Vim's Rust syntax colors as reserved, as errors, are not:
"   `do` is OCaml's, and the others (`final`, `priv`, `become`, ..) are
"   names in rsml.
"
" In an `rsml!` block, this covers the Rust items too (a field named `to`
" would be colored as a keyword). A file that is one `rsml!` block is
" highlighted from its start, so that this holds at any line.
"
" Install: put this file in ~/.vim/after/syntax/ (Neovim:
" ~/.config/nvim/after/syntax/), or add the directory above it to the end
" of 'runtimepath':
"
"     set runtimepath+=/path/to/rsml/editors/vim/after

if get(b:, 'current_syntax', '') !=# 'rust'
  finish
endif

" The bodies, and the brackets nested in them (so that the body ends at its
" own closing bracket). Rust's syntax applies inside, but for its `{}`
" region (replaced by the nested ones) and its reserved words. The brackets
" themselves are `matchgroup`s, not colored, so that nothing else starts
" at them.
syn cluster rsmlCode contains=rsmlCodeBraces,rsmlCodeParens,rsmlCodeBrackets,rsmlCodeNestedBraces,rsmlCodeNestedParens,rsmlCodeNestedBrackets
syn region rsmlCodeBraces matchgroup=rsmlBracket start=/\%(\<\%(rsml\|ml\)!\s*\)\@<={/ end=/}/ fold transparent contains=TOP,rustFoldBraces,rustReservedKeyword
syn region rsmlCodeParens matchgroup=rsmlBracket start=/\%(\<\%(rsml\|ml\)!\s*\)\@<=(/ end=/)/ fold transparent contains=TOP,rustFoldBraces,rustReservedKeyword
syn region rsmlCodeBrackets matchgroup=rsmlBracket start=/\%(\<\%(rsml\|ml\)!\s*\)\@<=\[/ end=/]/ fold transparent contains=TOP,rustFoldBraces,rustReservedKeyword
syn region rsmlCodeNestedBraces matchgroup=rsmlBracket start=/{/ end=/}/ fold transparent contained containedin=@rsmlCode contains=TOP,rustFoldBraces,rustReservedKeyword
syn region rsmlCodeNestedParens matchgroup=rsmlBracket start=/(/ end=/)/ transparent contained containedin=@rsmlCode contains=TOP,rustFoldBraces,rustReservedKeyword
syn region rsmlCodeNestedBrackets matchgroup=rsmlBracket start=/\[/ end=/]/ transparent contained containedin=@rsmlCode contains=TOP,rustFoldBraces,rustReservedKeyword

" OCaml keywords. Matches, not keywords, so that they can be told apart
" from the same words as Rust names after `.`, `::` and `r#`.
syn match rsmlConditional /\%(\.\|::\|r#\)\@<!\<\%(then\|when\|with\|function\)\>!\@!/ contained containedin=@rsmlCode
syn match rsmlRepeat /\%(\.\|::\|r#\)\@<!\<\%(do\|done\|to\|downto\|til\)\>!\@!/ contained containedin=@rsmlCode
syn match rsmlKeyword /\%(\.\|::\|r#\)\@<!\<\%(fun\|rec\|and\|begin\|end\|open\|external\|not\|assert\)\>!\@!/ contained containedin=@rsmlCode
syn match rsmlOperator /\%(\.\|::\|r#\)\@<!\<\%(land\|lor\|lxor\|lsl\|lsr\|asr\|or\)\>!\@!/ contained containedin=@rsmlCode

" `(* .. *)`: `(*` not followed by `*)` (that is `( ** )`), and only where
" the group it opens ends with `*)`, at most two levels of parentheses
" further in. Defined after the nested parentheses, so that it wins where
" both start. Strings and parentheses inside it are skipped, as Rust's lexer
" skips them, and so are comments nested in it.
syn region rsmlComment matchgroup=rsmlComment start=/(\*\%(\*)\)\@!\%(\%(\_[^()]\|(\%(\_[^()]\|(\_[^()]*)\)*)\)*\*)\)\@=/ end=/\*)/ contained containedin=@rsmlCode contains=rsmlCommentParens,rsmlCommentString,rustTodo,@Spell
syn region rsmlCommentParens matchgroup=rsmlComment start=/(/ end=/)/ contained transparent contains=rsmlCommentParens,rsmlCommentString,rustTodo,@Spell
syn region rsmlCommentString start=/"/ skip=/\\\\\|\\"/ end=/"/ contained transparent contains=NONE

hi def link rsmlConditional Conditional
hi def link rsmlRepeat Repeat
hi def link rsmlKeyword Keyword
hi def link rsmlOperator Operator
hi def link rsmlComment Comment

" A whole file in one `rsml!` block: Rust's syntax looks back only a few
" hundred lines to find where it is.
if search('^\%(::\)\=\%(rsml::\)\=rsml!\s*[{([]', 'nw')
  syn sync fromstart
endif

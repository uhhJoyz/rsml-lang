; extends

; rsml: OCaml comments, `(* .. *)`, inside macro calls (`ml! { .. }`,
; `rsml! { .. }`). To Rust they are a parenthesized token tree that starts
; with `(*` and ends with `*)`. `( * )` and `( ** )` (operator sections) and
; `( *x)` (a dereference) are not comments. The priority is above that of
; rust-analyzer's semantic tokens (125), which color Rust keywords inside
; such a comment.
((token_tree
  .
  "("
  .
  "*"
  "*"
  .
  ")"
  .) @comment @spell
  (#not-lua-match? @comment "^%(%s*%*%*%s*%)$")
  (#set! priority 130))

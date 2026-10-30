//! Procedural macros behind [`rsml-lang`](https://docs.rs/rsml-lang). Use the
//! `rsml-lang` crate, which re-exports them together with the runtime they
//! expand to.
//! The translation itself lives in `rsml-compiler`.

/// Writes an expression in OCaml syntax. See the `rsml` crate documentation.
#[proc_macro]
pub fn ml(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    rsml_compiler::expand(input.into()).into()
}

/// Rust items whose function bodies are written in OCaml syntax. See the
/// `rsml` crate documentation.
#[proc_macro]
pub fn rsml(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    rsml_compiler::expand_items(input.into()).into()
}

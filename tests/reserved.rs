//! Raw identifiers, and the words Rust reserves without syntax rsml uses
//! (`gen`, `try`, `yield`, ..): names in OCaml code, emitted raw. This file
//! is compiled as edition 2024 (this crate) and as edition 2021
//! (tests/edition2021): the macros cannot see their caller's edition.

#![deny(warnings)]

use rsml::{ml, rsml};

struct Rng(u32);

impl Rng {
    // `gen` is a keyword from 2024 on, so a definition names it raw.
    fn r#gen(&mut self) -> u32 {
        self.0 += 1;
        self.0
    }
}

struct Token {
    r#type: i32,
}

fn r#match(x: i32) -> i32 {
    x + 1
}

rsml! {
    fn next_twice(rng: &mut Rng) -> u32 {
        rng.gen() + rng.gen()
    }

    fn successor(r#gen: u32) -> u32 {
        gen + 1
    }
}

#[test]
fn reserved_words_are_names() {
    assert_eq!(
        ml! {
            let gen = 3 in
            gen + 1
        },
        4
    );
    assert_eq!(
        ml! {
            let try = 2 in
            let yield = 3 in
            try * yield
        },
        6
    );
    assert_eq!(
        ml! {
            let box = 1 in
            let macro = 2 in
            let typeof = 3 in
            box + macro + typeof
        },
        6
    );
    let mut rng = Rng(0);
    assert_eq!(ml! { rng.gen() + rng.gen() }, 3);
    assert_eq!(next_twice(&mut rng), 3 + 4);
    assert_eq!(successor(41), 42);
    let r = ml! {
        let f ~gen = gen * 2 in
        f ~gen:5
    };
    assert_eq!(r, 10);
}

#[test]
fn raw_identifiers_are_names() {
    assert_eq!(
        ml! {
            let r#type = 5 in
            r#type * 2
        },
        10
    );
    // `r#match` is a function named `match`, not the keyword.
    assert_eq!(ml! { r#match 1 }, 2);
    let t = Token { r#type: 7 };
    assert_eq!(ml! { t.r#type + Token { r#type = 1 }.r#type }, 8);
    assert_eq!(ml! { match Some 3 with Some r#in -> r#in | None -> 0 }, 3);
    // `r#x` and `x` are one name.
    assert_eq!(
        ml! {
            let r#x = ref 0 in
            incr x;
            !r#x
        },
        1
    );
}

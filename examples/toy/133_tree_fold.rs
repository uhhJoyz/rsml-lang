// Toy 133: a binary tree and a fold over it.
//
// What it is: a binary search tree of numbers, built by inserting them
// one by one, and one generic `fold` from which the sum, the size, the
// depth and the sorted list of the numbers all follow.
//
// When to use it: whenever a recursive data type is taken apart: `match`
// on its constructors (toy 068) with guards (toy 074), and a
// closure-typed parameter (toy 113) for what to do at each node.
//
// How to write it: the type is Rust, in the `rsml!`; the function bodies
// are rsml code. `fold`'s parameter `node : &dyn Fn(A, i64, A) -> A` has
// a known arity, so `node l v r` is a plain call. Gotcha:
// `Box::new (insert ( *l) x)` needs the parentheses around `*l`, since
// after a function `*` multiplies (toy 091).
//
// Reference: [Patterns](../../docs/notation.md#patterns)

use rsml::rsml;

pub const SUMMARY: &str = "A search tree built with `fold`, and a fold over the tree";

rsml! {
/// A binary tree of numbers.
#[derive(Debug)]
pub enum Tree {
    Leaf,
    Node(Box<Tree>, i64, Box<Tree>),
}

/// `t` with `x` added: smaller numbers go left.
pub fn insert(t: Tree, x: i64) -> Tree {
    // `insert *l x` would be `insert * l(x)`; `( *l)` takes the
    // subtree out of its box (`(*l)` would open a comment).
    match t with
    | Tree::Leaf -> Tree::Node (Box::new Tree::Leaf, x, Box::new Tree::Leaf)
    | Tree::Node (l, v, r) when x < v -> Tree::Node (Box::new (insert ( *l) x), v, r)
    | Tree::Node (l, v, r) -> Tree::Node (l, v, Box::new (insert ( *r) x))
}

/// A tree of the numbers, inserted in order.
pub fn of_list(xs: &[i64]) -> Tree {
    // `insert` is passed by name: `fold` calls it with the tree so far
    // and each number.
    xs |> .iter() |> .copied() |> .fold(Tree::Leaf, insert)
}

/// Folds the tree: `leaf` for an empty tree, and `node` to combine
/// the left result, the number and the right result.
pub fn fold<A: Clone>(t: &Tree, leaf: A, node: &dyn Fn(A, i64, A) -> A) -> A {
    // `l` has type `&Box<Tree>`, which Rust lends as a `&Tree`; `!v`
    // copies the number out of its reference (toy 092).
    match t with
    | Tree::Leaf -> leaf
    | Tree::Node (l, v, r) -> node (fold l leaf.clone() node) !v (fold r leaf node)
}

pub fn sum(t: &Tree) -> i64 {
    fold t 0 &(fun l v r -> l + v + r)
}

pub fn size(t: &Tree) -> i64 {
    fold t 0 &(fun l _ r -> l + 1 + r)
}

pub fn depth(t: &Tree) -> i64 {
    fold t 0 &(fun l _ r -> 1 + std::cmp::max l r)
}

/// The numbers in order: a fold whose results are lists, joined with
/// `@` (toy 063).
pub fn in_order(t: &Tree) -> Vec<i64> {
    fold t [] &(fun l v r -> l @ [v] @ r)
}
}

pub fn run() {
    let t = of_list(&[5, 3, 8, 1, 4, 9]);
    println!("of_list [5; 3; 8; 1; 4; 9], in order: {:?}", in_order(&t));
    println!("sum {}, size {}, depth {}", sum(&t), size(&t), depth(&t));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds() {
        let t = of_list(&[5, 3, 8, 1, 4, 9]);
        assert_eq!(in_order(&t), [1, 3, 4, 5, 8, 9]);
        assert_eq!((sum(&t), size(&t), depth(&t)), (30, 6, 3));
    }

    #[test]
    fn an_empty_tree() {
        let t = of_list(&[]);
        assert!(in_order(&t).is_empty());
        assert_eq!((sum(&t), size(&t), depth(&t)), (0, 0, 0));
    }
}

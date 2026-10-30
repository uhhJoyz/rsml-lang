//! Helpers that `ml!` expansions call. Each one is a small `#[inline]`
//! function equivalent to what you would write by hand.

use std::collections::VecDeque;

/// `head :: tail` when `tail` is not a list literal.
#[inline]
pub fn cons<T, L: Cons<T>>(head: T, tail: L) -> L::Output {
    tail.cons(head)
}

/// Types a value can be consed onto. `Vec` is a contiguous buffer, so this
/// is O(n); build lists with iterators, literals or `VecDeque` when consing
/// in a loop.
pub trait Cons<T> {
    type Output;
    fn cons(self, head: T) -> Self::Output;
}

impl<T> Cons<T> for Vec<T> {
    type Output = Vec<T>;
    #[inline]
    fn cons(mut self, head: T) -> Vec<T> {
        self.insert(0, head);
        self
    }
}

impl<T: Clone> Cons<T> for &[T] {
    type Output = Vec<T>;
    #[inline]
    fn cons(self, head: T) -> Vec<T> {
        let mut v = Vec::with_capacity(self.len() + 1);
        v.push(head);
        v.extend_from_slice(self);
        v
    }
}

impl<T: Clone> Cons<T> for &Vec<T> {
    type Output = Vec<T>;
    #[inline]
    fn cons(self, head: T) -> Vec<T> {
        self.as_slice().cons(head)
    }
}

impl<T> Cons<T> for VecDeque<T> {
    type Output = VecDeque<T>;
    #[inline]
    fn cons(mut self, head: T) -> VecDeque<T> {
        self.push_front(head);
        self
    }
}

/// `a @ b`
#[inline]
pub fn append<T, A: IntoIterator<Item = T>, B: IntoIterator<Item = T>>(a: A, b: B) -> Vec<T> {
    let mut v: Vec<T> = a.into_iter().collect();
    v.extend(b);
    v
}

/// `a ^ b`. Reuses `a`'s buffer when it is an owned `String`.
#[inline]
pub fn concat<A: Concat<B>, B>(a: A, b: B) -> String {
    a.concat(b)
}

pub trait Concat<B> {
    fn concat(self, b: B) -> String;
}

impl<B: AsRef<str>> Concat<B> for String {
    #[inline]
    fn concat(mut self, b: B) -> String {
        self.push_str(b.as_ref());
        self
    }
}

impl<B: AsRef<str>> Concat<B> for &str {
    #[inline]
    fn concat(self, b: B) -> String {
        let b = b.as_ref();
        let mut s = String::with_capacity(self.len() + b.len());
        s.push_str(self);
        s.push_str(b);
        s
    }
}

impl<B: AsRef<str>> Concat<B> for &String {
    #[inline]
    fn concat(self, b: B) -> String {
        self.as_str().concat(b)
    }
}

impl<B: AsRef<str>> Concat<B> for &mut String {
    #[inline]
    fn concat(self, b: B) -> String {
        self.as_str().concat(b)
    }
}

/// Scrutinee of a `match` with list patterns (`[]`, `x :: xs`).
#[inline]
pub fn as_slice<S: AsSlice + ?Sized>(s: &S) -> &[S::Elem] {
    s.as_slice_()
}

pub trait AsSlice {
    type Elem;
    fn as_slice_(&self) -> &[Self::Elem];
}

impl<T> AsSlice for [T] {
    type Elem = T;
    #[inline]
    fn as_slice_(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> AsSlice for [T; N] {
    type Elem = T;
    #[inline]
    fn as_slice_(&self) -> &[T] {
        self
    }
}

impl<T> AsSlice for Vec<T> {
    type Elem = T;
    #[inline]
    fn as_slice_(&self) -> &[T] {
        self
    }
}

impl<T> AsSlice for Box<[T]> {
    type Elem = T;
    #[inline]
    fn as_slice_(&self) -> &[T] {
        self
    }
}

impl<S: AsSlice + ?Sized> AsSlice for &S {
    type Elem = S::Elem;
    #[inline]
    fn as_slice_(&self) -> &[S::Elem] {
        (**self).as_slice_()
    }
}

impl<S: AsSlice + ?Sized> AsSlice for &mut S {
    type Elem = S::Elem;
    #[inline]
    fn as_slice_(&self) -> &[S::Elem] {
        (**self).as_slice_()
    }
}

/// Scrutinee of a `match` with string literal patterns.
#[inline]
pub fn as_str<S: AsRef<str> + ?Sized>(s: &S) -> &str {
    s.as_ref()
}

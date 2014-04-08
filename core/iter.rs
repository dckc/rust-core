// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use option::{Option, Some, None};

pub trait Iterator<A> {
    fn next(&mut self) -> Option<A>;

    /// Return a lower bound and upper bound on the remaining length of the iterator.
    ///
    /// The common use case for the estimate is pre-allocating space to store the results.
    #[inline(always)]
    fn size_hint(&self) -> (uint, Option<uint>) { (0, None) }

    #[inline]
    fn fold<B>(&mut self, init: B, f: |B, A| -> B) -> B {
        let mut accum = init;
        loop {
            match self.next() {
                Some(x) => { accum = f(accum, x); }
                None    => { break; }
            }
        }
        accum
    }

    #[inline]
    fn all(&mut self, f: |A| -> bool) -> bool {
        for x in *self { if !f(x) { return false; } }
        true
    }

    #[inline]
    fn any(&mut self, f: |A| -> bool) -> bool {
        for x in *self { if f(x) { return true; } }
        false
    }
}

pub trait DoubleEndedIterator<A>: Iterator<A> {
    fn next_back(&mut self) -> Option<A>;

    #[inline(always)]
    fn invert(self) -> Invert<Self> {
        Invert { iter: self }
    }
}

#[deriving(Clone)]
pub struct Invert<T> {
    iter: T
}

impl<A, T: DoubleEndedIterator<A>> Iterator<A> for Invert<T> {
    #[inline(always)]
    fn next(&mut self) -> Option<A> { self.iter.next_back() }

    #[inline(always)]
    fn size_hint(&self) -> (uint, Option<uint>) { self.iter.size_hint() }
}

impl<A, T: DoubleEndedIterator<A>> DoubleEndedIterator<A> for Invert<T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<A> { self.iter.next() }
}

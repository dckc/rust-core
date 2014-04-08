// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A priority queue implemented with a binary heap

use container::Container;
use vec::Vec;
use cmp::Ord;
use option::{Option, Some, None};
use mem::swap;
use slice;

/// A priority queue implemented with a binary heap
pub struct PriorityQueue<T> {
    data: Vec<T>
}

impl<T> Container for PriorityQueue<T> {
    #[inline(always)]
    fn len(&self) -> uint {
        self.data.len()
    }
}

impl<T: Ord> PriorityQueue<T> {
    #[inline(always)]
    pub fn capacity(&self) -> uint {
        self.data.capacity()
    }

    pub fn reserve(&mut self, n: uint) {
        self.data.reserve(n)
    }

    pub fn top<'a>(&'a self) -> Option<&'a T> {
        if self.len() == 0 {
            None
        } else {
            Some(&self.data.as_slice()[0])
        }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
        let new_len = self.len() - 1;
        self.siftup(0, new_len);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            let mut item = self.data.pop().get();
            if self.len() != 0 {
                swap(&mut item, &mut self.data.as_mut_slice()[0]);
                self.siftdown(0);
            }
            Some(item)
        }
    }

    pub fn to_vec(self) -> Vec<T> {
        self.data
    }

    pub fn to_sorted_vec(self) -> Vec<T> {
        let mut q = self;
        let mut end = q.len();
        while end > 1 {
            end -= 1;
            slice::swap(q.data.as_mut_slice(), 0, end);
            q.siftdown_range(0, end)
        }
        q.to_vec()
    }

    #[inline(always)]
    pub fn new() -> PriorityQueue<T> {
        PriorityQueue { data: Vec::new() }
    }

    #[inline(always)]
    pub fn with_capacity(capacity: uint) -> PriorityQueue<T> {
        PriorityQueue { data: Vec::with_capacity(capacity) }
    }

    pub fn from_vec(xs: Vec<T>) -> PriorityQueue<T> {
        let mut q = PriorityQueue { data: xs };
        let mut n = q.len() / 2;
        while n > 0 {
            n -= 1;
            q.siftdown(n)
        }
        q
    }

    fn siftup(&mut self, start: uint, mut pos: uint) {
        while pos > start {
            let parent = (pos - 1) >> 1;
            if self.data.as_slice()[pos] > self.data.as_slice()[parent] {
                slice::swap(self.data.as_mut_slice(), parent, pos);
                pos = parent;
                continue
            }
            break
        }
    }

    fn siftdown_range(&mut self, mut pos: uint, end: uint) {
        let start = pos;

        let mut child = 2 * pos + 1;
        while child < end {
            let right = child + 1;
            if right < end && !(self.data.as_slice()[child] > self.data.as_slice()[right]) {
                child = right;
            }
            slice::swap(self.data.as_mut_slice(), child, pos);
            pos = child;
            child = 2 * pos + 1;
        }

        self.siftup(start, pos);
    }

    fn siftdown(&mut self, pos: uint) {
        let len = self.len();
        self.siftdown_range(pos, len);
    }
}

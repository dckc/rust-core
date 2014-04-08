// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A double-ended queue implemented as a circular buffer

// A circular buffer written with fully safe code would use `Vec<Option<T>>`. Instead, this module
// uses `unsafe` to directly use `Vec<T>` and leave a portion of the vector uninitialized. This
// reduces the size of the queue by 2x for types without a compact `Option<T>` representation like
// non-nullable pointers.

use container::Container;
use mem::move_val_init;
use ptr::read_ptr;
use ops::Drop;
use vec::Vec;
use slice::{unchecked_get, unchecked_mut_get, unchecked_swap};
use fail::{abort, assert};
use option::{Option, Some, None};

pub struct Deque<T> {
    nelts: uint,
    lo: uint,
    elts: Vec<T>
}

fn raw_index(lo: uint, len: uint, index: uint) -> uint {
    if lo >= len - index {
        lo + index - len
    } else {
        lo + index
    }
}

impl<T> Container for Deque<T> {
    #[inline(always)]
    fn len(&self) -> uint {
        self.nelts
    }
}

impl<T> Deque<T> {
    pub fn new() -> Deque<T> {
        Deque{ nelts: 0, lo: 0, elts: Vec::new() }
    }

    pub fn with_capacity(capacity: uint) -> Deque<T> {
        Deque{ nelts: 0, lo: 0, elts: Vec::with_capacity(capacity) }
    }

    #[inline(always)]
    pub fn capacity(&self) -> uint {
        self.elts.capacity()
    }

    pub fn reserve(&mut self, capacity: uint) {
        self.elts.reserve(capacity)
    }

    pub fn get<'a>(&'a self, index: uint) -> Option<&'a T> {
        if index < self.nelts {
            let idx = self.raw_index(index);
            unsafe {
                Some(unchecked_get(self.elts.as_slice(), idx))
            }
        } else {
            None
        }
    }

    pub fn get_mut<'a>(&'a mut self, index: uint) -> Option<&'a mut T> {
        if index < self.nelts {
            let idx = self.raw_index(index);
            unsafe {
                Some(unchecked_mut_get(self.elts.as_mut_slice(), idx))
            }
        } else {
            None
        }
    }

    pub fn swap(&mut self, i: uint, j: uint) {
        if i >= self.nelts || j >= self.nelts {
            abort()
        }
        let a = self.raw_index(i);
        let b = self.raw_index(j);
        unsafe { unchecked_swap(self.elts.as_mut_slice(), a, b) }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.nelts > 0 {
            unsafe {
                let result = read_ptr(unchecked_get(self.elts.as_slice(), self.lo));
                self.lo = (self.lo + 1) % self.elts.capacity();
                self.nelts -= 1;
                Some(result)
            }
        } else {
            None
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.nelts > 0 {
            unsafe {
                self.nelts -= 1;
                let hi = self.raw_index(self.nelts);
                Some(read_ptr(unchecked_get(self.elts.as_slice(), hi)))
            }
        } else {
            None
        }
    }

    pub fn push_front(&mut self, item: T) {
        if self.nelts == self.elts.capacity() {
            grow(self.nelts, &mut self.lo, &mut self.elts);
        }
        if self.lo == 0 {
            self.lo = self.elts.capacity() - 1;
        } else {
            self.lo -= 1;
        }
        unsafe {
            move_val_init(unchecked_mut_get(self.elts.as_mut_slice(), self.lo), item);
        }
        self.nelts += 1;
    }

    pub fn push_back(&mut self, item: T) {
        if self.nelts == self.elts.capacity() {
            grow(self.nelts, &mut self.lo, &mut self.elts);
        }
        let hi = self.raw_index(self.nelts);
        unsafe {
            move_val_init(unchecked_mut_get(self.elts.as_mut_slice(), hi), item);
        }
        self.nelts += 1;
    }

    #[inline(always)]
    fn raw_index(&self, index: uint) -> uint {
        raw_index(self.lo, self.capacity(), index)
    }
}

#[unsafe_destructor]
impl<T> Drop for Deque<T> {
    fn drop(&mut self) {
        // Make sure the Vec destructor isn't going to ruin our day
        assert(self.elts.len() == 0);

        let mut i = 0;
        let len = self.len();
        while i < len {
            let idx = self.raw_index(i);
            unsafe {
                read_ptr(unchecked_get(self.elts.as_slice(), idx));
            }
            i += 1;
        }
    }
}

fn grow<T>(nelts: uint, loptr: &mut uint, elts: &mut Vec<T>) {
    assert(nelts == elts.capacity());
    let lo = *loptr;
    let mut newlen = nelts * 2;
    if newlen == 0 { newlen = 4 }
    elts.reserve(newlen);

    // Move the shortest half into the newly reserved area.

    assert(newlen - nelts / 2 >= nelts);

    if lo <= (nelts - lo) {
        // Before: [o o o|o o o o o]
        // After:  [. . .|o o o o o o o o|. . . . .]
        let mut i = 0;
        while i < lo {
            unsafe { unchecked_swap(elts.as_mut_slice(), i, nelts + i); }
            i += 1;
        }
    } else {
        // Before: [o o o o o|o o o]
        // After:  [o o o o o|. . . . . . . .|o o o]
        let mut i = lo;
        while i < nelts {
            unsafe { unchecked_swap(elts.as_mut_slice(), i, newlen - nelts + i); }
            i += 1;
        }
        *loptr += newlen - nelts;
    }
}

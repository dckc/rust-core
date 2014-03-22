// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use container::Container;
use mem::{forget, move_val_init, size_of, transmute};
use fail::out_of_memory;
use heap::{free, alloc, realloc};
use ops::Drop;
use slice::{Items, Slice, iter, unchecked_get, unchecked_mut_get};
use ptr::{offset, read_ptr};
use uint::mul_with_overflow;
use option::{Option, Some, None};
use iter::{Iterator, DoubleEndedIterator};
use cmp::expect;
use clone::Clone;

pub struct Vec<T> {
    priv len: uint,
    priv cap: uint,
    priv ptr: *mut T
}

impl<T> Vec<T> {
    #[inline(always)]
    pub fn new() -> Vec<T> {
        Vec { len: 0, cap: 0, ptr: 0 as *mut T }
    }

    pub fn with_capacity(capacity: uint) -> Vec<T> {
        if capacity == 0 {
            Vec::new()
        } else {
            let (size, overflow) = mul_with_overflow(capacity, size_of::<T>());
            if overflow {
                out_of_memory();
            }
            let ptr = unsafe { alloc(size) };
            Vec { len: 0, cap: capacity, ptr: ptr as *mut T }
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> uint {
        self.cap
    }

    pub fn reserve(&mut self, capacity: uint) {
        if capacity >= self.len {
            let (size, overflow) = mul_with_overflow(capacity, size_of::<T>());
            if overflow {
                out_of_memory();
            }
            self.cap = capacity;
            unsafe {
                self.ptr = realloc(self.ptr as *mut u8, size) as *mut T;
            }
        }
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        if self.len == 0 {
            unsafe { free(self.ptr as *mut u8) };
            self.cap = 0;
            self.ptr = 0 as *mut T;
        } else {
            unsafe {
                // Overflow check is unnecessary as the vector is already at least this large.
                self.ptr = realloc(self.ptr as *mut u8, self.len * size_of::<T>()) as *mut T;
            }
            self.cap = self.len;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(read_ptr(unchecked_get(self.as_slice(), self.len())))
            }
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        if unlikely!(self.len == self.cap) {
            if self.cap == 0 { self.cap += 2 }
            let old_size = self.cap * size_of::<T>();
            self.cap = self.cap * 2;
            let size = old_size * 2;
            if old_size > size { out_of_memory() }
            unsafe {
                self.ptr = realloc(self.ptr as *mut u8, size) as *mut T;
            }
        }

        unsafe {
            let end = offset(self.ptr as *T, self.len as int) as *mut T;
            move_val_init(&mut *end, value);
            self.len += 1;
        }
    }

    pub fn truncate(&mut self, len: uint) {
        unsafe {
            let mut i = len;
            // drop any extra elements
            while i < self.len {
                read_ptr(unchecked_get(self.as_slice(), i));
                i += 1;
            }
        }
        self.len = len;
    }

    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [T] {
        let slice = Slice { data: self.ptr as *T, len: self.len };
        unsafe { transmute(slice) }
    }

    #[inline]
    pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [T] {
        let slice = Slice { data: self.ptr as *T, len: self.len };
        unsafe { transmute(slice) }
    }

    pub fn move_iter(self) -> MoveItems<T> {
        unsafe {
            let iter = transmute(iter(self.as_slice()));
            let ptr = self.ptr as *mut u8;
            forget(self);
            MoveItems { allocation: ptr, iter: iter }
        }
    }

    pub unsafe fn set_len(&mut self, len: uint) {
        self.len = len;
    }
}

impl<T: Clone> Vec<T> {
    pub fn from_elem(length: uint, value: T) -> Vec<T> {
        unsafe {
            let mut xs = Vec::with_capacity(length);
            while xs.len < length {
                move_val_init(unchecked_mut_get(xs.as_mut_slice(), xs.len), value.clone());
                xs.len += 1;
            }
            xs
        }
    }

    pub fn from_fn(length: uint, op: |uint| -> T) -> Vec<T> {
        unsafe {
            let mut xs = Vec::with_capacity(length);
            while xs.len < length {
                move_val_init(unchecked_mut_get(xs.as_mut_slice(), xs.len), op(xs.len));
                xs.len += 1;
            }
            xs
        }
    }
}

impl<T> Container for Vec<T> {
    #[inline(always)]
    fn len(&self) -> uint {
        self.len
    }
}

#[unsafe_destructor]
impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            for x in iter(self.as_mut_slice()) {
                read_ptr(x);
            }
            free(self.ptr as *mut u8)
        }
    }
}

pub struct MoveItems<T> {
    priv allocation: *mut u8, // the block of memory allocated for the vector
    priv iter: Items<'static, T>
}

impl<T> Iterator<T> for MoveItems<T> {
    fn next(&mut self) -> Option<T> {
        unsafe {
            self.iter.next().map(|x| read_ptr(x))
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator<T> for MoveItems<T> {
    fn next_back(&mut self) -> Option<T> {
        unsafe {
            self.iter.next_back().map(|x| read_ptr(x))
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for MoveItems<T> {
    fn drop(&mut self) {
        // destroy the remaining elements
        for _x in *self {}
        unsafe {
            free(self.allocation)
        }
    }
}

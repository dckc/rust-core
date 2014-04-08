// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Task-local reference counted smart pointers with weak pointer support

use mem::transmute;
use ops::Drop;
use cmp::{Eq, Ord};
use clone::{Clone, DeepClone};
use heap::free;
use ptr::read_ptr;
use option::{Option, Some, None};
use kinds::marker::NoSend;

struct RcBox<T> {
    value: T,
    strong: uint,
    weak: uint,
    no_send: NoSend
}

#[unsafe_no_drop_flag]
pub struct Strong<T> {
    ptr: *mut RcBox<T>
}

impl<T> Strong<T> {
    pub fn new(value: T) -> Strong<T> {
        unsafe {
            // The `Strong` pointers share a single `weak` reference count.  This prevents the
            // premature deallocation of the box when the last weak pointer is freed.
            Strong { ptr: transmute(~RcBox { value: value, strong: 1, weak: 1, no_send: NoSend }) }
        }
    }

    #[inline(always)]
    pub fn borrow<'a>(&'a self) -> &'a T {
        unsafe { &(*self.ptr).value }
    }

    pub fn downgrade(&self) -> Weak<T> {
        unsafe {
            (*self.ptr).weak += 1;
            Weak { ptr: self.ptr }
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for Strong<T> {
    fn drop(&mut self) {
        unsafe {
            if self.ptr != 0 as *mut RcBox<T> {
                (*self.ptr).strong -= 1;
                if (*self.ptr).strong == 0 {
                    read_ptr(self.borrow()); // destroy the contained object
                    (*self.ptr).weak -= 1;
                    if (*self.ptr).weak == 0 {
                        free(self.ptr as *mut u8)
                    }
                }
            }
        }
    }
}

impl<T> Clone for Strong<T> {
    #[inline]
    fn clone(&self) -> Strong<T> {
        unsafe {
            (*self.ptr).strong += 1;
            Strong { ptr: self.ptr }
        }
    }
}

impl<T: DeepClone> DeepClone for Strong<T> {
    #[inline]
    fn deep_clone(&self) -> Strong<T> {
        Strong::new(self.borrow().deep_clone())
    }
}

impl<T: Eq> Eq for Strong<T> {
    #[inline(always)]
    fn eq(&self, other: &Strong<T>) -> bool { *self.borrow() == *other.borrow() }

    #[inline(always)]
    fn ne(&self, other: &Strong<T>) -> bool { *self.borrow() != *other.borrow() }
}

impl<T: Ord> Ord for Strong<T> {
    #[inline(always)]
    fn lt(&self, other: &Strong<T>) -> bool { *self.borrow() < *other.borrow() }

    #[inline(always)]
    fn le(&self, other: &Strong<T>) -> bool { *self.borrow() <= *other.borrow() }

    #[inline(always)]
    fn gt(&self, other: &Strong<T>) -> bool { *self.borrow() > *other.borrow() }

    #[inline(always)]
    fn ge(&self, other: &Strong<T>) -> bool { *self.borrow() >= *other.borrow() }
}

#[unsafe_no_drop_flag]
pub struct Weak<T> {
    ptr: *mut RcBox<T>
}

impl<T> Weak<T> {
    pub fn upgrade(&self) -> Option<Strong<T>> {
        unsafe {
            if (*self.ptr).strong == 0 {
                None
            } else {
                (*self.ptr).strong += 1;
                Some(Strong { ptr: self.ptr })
            }
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        unsafe {
            if self.ptr != 0 as *mut RcBox<T> {
                (*self.ptr).weak -= 1;
                if (*self.ptr).weak == 0 {
                    free(self.ptr as *mut u8)
                }
            }
        }
    }
}

impl<T> Clone for Weak<T> {
    #[inline]
    fn clone(&self) -> Weak<T> {
        unsafe {
            (*self.ptr).weak += 1;
            Weak { ptr: self.ptr }
        }
    }
}

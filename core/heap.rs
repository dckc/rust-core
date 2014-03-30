// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use fail::out_of_memory;
use mem::Allocator;
use kinds::marker::ContravariantLifetime;

mod detail {
    extern {
        pub fn free(ptr: *mut u8);
        pub fn realloc(ptr: *mut u8, size: uint) -> *mut u8;
    }
}

extern {
    fn calloc(nmemb: uint, size: uint) -> *mut u8;
    fn malloc(size: uint) -> *mut u8;
}

#[inline(always)]
#[lang = "exchange_free"]
pub unsafe fn free(ptr: *mut u8) {
    detail::free(ptr)
}

#[inline]
#[lang = "exchange_malloc"]
pub unsafe fn alloc(size: uint) -> *mut u8 {
    if size == 0 {
        0 as *mut u8
    } else {
        let ptr = malloc(size);
        if ptr == 0 as *mut u8 {
            out_of_memory()
        }
        ptr
    }
}

#[inline]
pub unsafe fn zero_alloc(size: uint) -> *mut u8 {
    if size == 0 {
        0 as *mut u8
    } else {
        let ptr = calloc(1, size);
        if ptr == 0 as *mut u8 {
            out_of_memory()
        }
        ptr
    }
}

#[inline]
pub unsafe fn realloc(ptr: *mut u8, size: uint) -> *mut u8 {
    if size == 0 {
        free(ptr);
        0 as *mut u8
    } else {
        let ptr = detail::realloc(ptr, size);
        if ptr == 0 as *mut u8 {
            out_of_memory()
        }
        ptr
    }
}

pub struct Heap<'a> {
    lifetime: ContravariantLifetime<'a>
}

pub static mut Heap: Heap<'static> = Heap { lifetime: ContravariantLifetime::<'static> };

impl Allocator for Heap<'static> {
    unsafe fn alloc(&mut self, size: uint) -> (*mut u8, uint) {
        (alloc(size), size)
    }

    unsafe fn zero_alloc(&mut self, size: uint) -> (*mut u8, uint) {
        (zero_alloc(size), size)
    }

    unsafe fn realloc(&mut self, ptr: *mut u8, size: uint) -> (*mut u8, uint) {
        (realloc(ptr, size), size)
    }

    unsafe fn free(&mut self, ptr: *mut u8) {
        free(ptr)
    }
}

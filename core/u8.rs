// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern "rust-intrinsic" {
    fn u8_add_with_overflow(x: u8, y: u8) -> (u8, bool);
    fn u8_sub_with_overflow(x: u8, y: u8) -> (u8, bool);
    fn u8_mul_with_overflow(x: u8, y: u8) -> (u8, bool);
}

#[inline(always)]
pub fn add_with_overflow(x: u8, y: u8) -> (u8, bool) {
    unsafe { u8_add_with_overflow(x, y) }
}

#[inline(always)]
pub fn sub_with_overflow(x: u8, y: u8) -> (u8, bool) {
    unsafe { u8_sub_with_overflow(x, y) }
}

#[inline(always)]
pub fn mul_with_overflow(x: u8, y: u8) -> (u8, bool) {
    unsafe { u8_mul_with_overflow(x, y) }
}

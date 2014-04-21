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
    fn bswap32(x: u32) -> u32;
    pub fn ctpop32(x: u32) -> u32;
    pub fn ctlz32(x: u32) -> u32;
    pub fn cttz32(x: u32) -> u32;
    fn i32_add_with_overflow(x: i32, y: i32) -> (i32, bool);
    fn i32_sub_with_overflow(x: i32, y: i32) -> (i32, bool);
    fn i32_mul_with_overflow(x: i32, y: i32) -> (i32, bool);
}

#[inline(always)]
pub fn add_with_overflow(x: i32, y: i32) -> (i32, bool) {
    unsafe { i32_add_with_overflow(x, y) }
}

#[inline(always)]
pub fn sub_with_overflow(x: i32, y: i32) -> (i32, bool) {
    unsafe { i32_sub_with_overflow(x, y) }
}

#[inline(always)]
pub fn mul_with_overflow(x: i32, y: i32) -> (i32, bool) {
    unsafe { i32_mul_with_overflow(x, y) }
}

pub fn bswap(x: u32) -> u32 {
    unsafe { bswap32(x) }
}

#[cfg(target_endian = "big")]
pub fn to_be(x: i32) -> i32 {
    x
}

#[cfg(target_endian = "little")]
pub fn to_be(x: u32) -> u32 {
    bswap(x)
}

#[cfg(target_endian = "big")]
pub fn to_le(x: i32) -> i32 {
    bswap(x)
}

#[cfg(target_endian = "little")]
pub fn to_le(x: i32) -> i32 {
    x
}

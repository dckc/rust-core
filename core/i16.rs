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
    fn bswap16(x: u16) -> u16;
    pub fn ctpop16(x: u16) -> u16;
    pub fn ctlz16(x: u16) -> u16;
    pub fn cttz16(x: u16) -> u16;
    fn i16_add_with_overflow(x: i16, y: i16) -> (i16, bool);
    fn i16_sub_with_overflow(x: i16, y: i16) -> (i16, bool);
    fn i16_mul_with_overflow(x: i16, y: i16) -> (i16, bool);
}

#[inline(always)]
pub fn add_with_overflow(x: i16, y: i16) -> (i16, bool) {
    unsafe { i16_add_with_overflow(x, y) }
}

#[inline(always)]
pub fn sub_with_overflow(x: i16, y: i16) -> (i16, bool) {
    unsafe { i16_sub_with_overflow(x, y) }
}

#[inline(always)]
pub fn mul_with_overflow(x: i16, y: i16) -> (i16, bool) {
    unsafe { i16_mul_with_overflow(x, y) }
}

pub fn bswap(x: u16) -> u16 {
    unsafe { bswap16(x) }
}

#[cfg(target_endian = "big")]
pub fn to_be(x: i16) -> i16 {
    x
}

#[cfg(target_endian = "little")]
pub fn to_be(x: u16) -> u16 {
    bswap(x)
}

#[cfg(target_endian = "big")]
pub fn to_le(x: i16) -> i16 {
    bswap(x)
}

#[cfg(target_endian = "little")]
pub fn to_le(x: i16) -> i16 {
    x
}

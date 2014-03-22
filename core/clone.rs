// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub trait Clone {
    fn clone(&self) -> Self;

    #[inline(always)]
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

impl<T: Clone> Clone for ~T {
    fn clone(&self) -> ~T { ~(**self).clone() }

    fn clone_from(&mut self, source: &~T) {
        **self = (**source).clone()
    }
}

impl<'a, T> Clone for &'a T {
    #[inline(always)]
    fn clone(&self) -> &'a T { *self }
}

impl<'a, T> Clone for &'a [T] {
    #[inline(always)]
    fn clone(&self) -> &'a [T] { *self }
}

impl<'a> Clone for &'a str {
    #[inline(always)]
    fn clone(&self) -> &'a str { *self }
}

impl<T> Clone for *T {
    #[inline(always)]
    fn clone(&self) -> *T { *self }
}

impl<T> Clone for *mut T {
    #[inline(always)]
    fn clone(&self) -> *mut T { *self }
}

macro_rules! clone_impl(
    ($t:ty) => {
        impl Clone for $t {
            #[inline(always)]
            fn clone(&self) -> $t { *self }
        }
    }
)

clone_impl!(int)
clone_impl!(i8)
clone_impl!(i16)
clone_impl!(i32)
clone_impl!(i64)

clone_impl!(uint)
clone_impl!(u8)
clone_impl!(u16)
clone_impl!(u32)
clone_impl!(u64)

clone_impl!(f32)
clone_impl!(f64)

clone_impl!(())
clone_impl!(bool)
clone_impl!(char)

macro_rules! extern_fn_clone(
    ($($A:ident),*) => (
        impl<$($A,)* ReturnType> Clone for extern "Rust" fn($($A),*) -> ReturnType {
            #[inline]
            fn clone(&self) -> extern "Rust" fn($($A),*) -> ReturnType { *self }
        }
    )
)

extern_fn_clone!()
extern_fn_clone!(A)
extern_fn_clone!(A, B)
extern_fn_clone!(A, B, C)
extern_fn_clone!(A, B, C, D)
extern_fn_clone!(A, B, C, D, E)
extern_fn_clone!(A, B, C, D, E, F)
extern_fn_clone!(A, B, C, D, E, F, G)
extern_fn_clone!(A, B, C, D, E, F, G, H)

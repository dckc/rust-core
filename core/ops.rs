// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[lang="drop"]
pub trait Drop {
    fn drop(&mut self);
}

#[lang="add"]
pub trait Add<RHS, Result> {
    fn add(&self, rhs: &RHS) -> Result;
}

#[lang="sub"]
pub trait Sub<RHS, Result> {
    fn sub(&self, rhs: &RHS) -> Result;
}

#[lang="mul"]
pub trait Mul<RHS, Result> {
    fn mul(&self, rhs: &RHS) -> Result;
}

#[lang="div"]
pub trait Div<RHS, Result> {
    fn div(&self, rhs: &RHS) -> Result;
}

#[lang="rem"]
pub trait Rem<RHS, Result> {
    fn rem(&self, rhs: &RHS) -> Result;
}

#[lang="neg"]
pub trait Neg<Result> {
    fn neg(&self) -> Result;
}

macro_rules! num_impl(
    ($T:ty) => {
        impl Add<$T,$T> for $T {
            #[inline]
            fn add(&self, other: &$T) -> $T { *self + *other }
        }

        impl Sub<$T,$T> for $T {
            #[inline]
            fn sub(&self, other: &$T) -> $T { *self - *other }
        }

        impl Mul<$T,$T> for $T {
            #[inline]
            fn mul(&self, other: &$T) -> $T { *self * *other }
        }

        impl Div<$T,$T> for $T {
            #[inline]
            fn div(&self, other: &$T) -> $T { *self / *other }
        }

        impl Rem<$T,$T> for $T {
            #[inline]
            fn rem(&self, other: &$T) -> $T { *self % *other }
        }

        impl Neg<$T> for $T {
            #[inline]
            fn neg(&self) -> $T { -*self }
        }
    }
)

num_impl!(int)
num_impl!(i8)
num_impl!(i16)
num_impl!(i32)
num_impl!(i64)

num_impl!(uint)
num_impl!(u8)
num_impl!(u16)
num_impl!(u32)
num_impl!(u64)

#[lang="not"]
pub trait Not<Result> {
    fn not(&self) -> Result;
}

#[lang="bitand"]
pub trait BitAnd<RHS, Result> {
    fn bitand(&self, rhs: &RHS) -> Result;
}

#[lang="bitor"]
pub trait BitOr<RHS, Result> {
    fn bitor(&self, rhs: &RHS) -> Result;
}

#[lang="bitxor"]
pub trait BitXor<RHS, Result> {
    fn bitxor(&self, rhs: &RHS) -> Result;
}

#[lang="shl"]
pub trait Shl<RHS, Result> {
    fn shl(&self, rhs: &RHS) -> Result;
}

#[lang="shr"]
pub trait Shr<RHS, Result> {
    fn shr(&self, rhs: &RHS) -> Result;
}

macro_rules! bitwise_impl(
        ($T:ty) => {
        impl Not<$T> for $T {
                #[inline]
                fn not(&self) -> $T { !*self }
        }

        impl BitAnd<$T,$T> for $T {
                #[inline]
                fn bitand(&self, other: &$T) -> $T { *self & *other }
        }

        impl BitOr<$T,$T> for $T {
                #[inline]
                fn bitor(&self, other: &$T) -> $T { *self | *other }
        }

        impl BitXor<$T,$T> for $T {
                #[inline]
                fn bitxor(&self, other: &$T) -> $T { *self ^ *other }
        }

        impl Shl<$T,$T> for $T {
                #[inline]
                fn shl(&self, other: &$T) -> $T { *self << *other }
        }

        impl Shr<$T,$T> for $T {
                #[inline]
                fn shr(&self, other: &$T) -> $T { *self >> *other }
        }
    }
)

bitwise_impl!(int)
bitwise_impl!(i8)
bitwise_impl!(i16)
bitwise_impl!(i32)
bitwise_impl!(i64)

bitwise_impl!(uint)
bitwise_impl!(u8)
bitwise_impl!(u16)
bitwise_impl!(u32)
bitwise_impl!(u64)

#[lang="index"]
pub trait Index<Index, Result> {
    fn index(&self, index: &Index) -> Result;
}

#[lang="deref"]
pub trait Deref<Result> {
    fn deref<'a>(&'a self) -> &'a Result;
}

#[lang="deref_mut"]
pub trait DerefMut<Result>: Deref<Result> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Result;
}

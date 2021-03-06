// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Built-in traits

/// Types able to be transferred across thread boundaries.
#[lang="send"]
pub trait Send {}

/// Types that are either immutable or have inherited mutability.
#[lang="freeze"]
pub trait Freeze {}

/// **P**lain **o**ld **d**ata types without move semantics.
#[lang="pod"]
pub trait Pod {}

/// Marker types are special types that are used with unsafe code to
/// inform the compiler of special constraints. Marker types should
/// only be needed when you are creating an abstraction that is
/// implemented using unsafe code. In that case, you may want to embed
/// some of the marker types below into your type.
pub mod marker {

    /// A marker type whose type parameter `T` is considered to be
    /// covariant with respect to the type itself. This is (typically)
    /// used to indicate that an instance of the type `T` is being stored
    /// into memory and read from, even though that may not be apparent.
    ///
    /// For more information about variance, refer to this Wikipedia
    /// article <http://en.wikipedia.org/wiki/Variance_%28computer_science%29>.
    ///
    /// *Note:* It is very unusual to have to add a covariant constraint.
    /// If you are not sure, you probably want to use `InvariantType`.
    ///
    /// # Example
    ///
    /// Given a struct `S` that includes a type parameter `T`
    /// but does not actually *reference* that type parameter:
    ///
    /// ```
    /// struct S<T> { x: *() }
    /// fn get<T>(s: &S<T>) -> T {
    /// unsafe {
    /// let x: *T = cast::transmute(s.x);
    /// *x
    /// }
    /// }
    /// ```
    ///
    /// The type system would currently infer that the value of
    /// the type parameter `T` is irrelevant, and hence a `S<int>` is
    /// a subtype of `S<~[int]>` (or, for that matter, `S<U>` for
    /// for any `U`). But this is incorrect because `get()` converts the
    /// `*()` into a `*T` and reads from it. Therefore, we should include the
    /// a marker field `CovariantType<T>` to inform the type checker that
    /// `S<T>` is a subtype of `S<U>` if `T` is a a subtype of `U`
    /// (for example, `S<&'static int>` is a subtype of `S<&'a int>`
    /// for some lifetime `'a`, but not the other way around).
    #[lang="covariant_type"]
    #[deriving(Eq,Clone)]
    pub struct CovariantType<T>;

    /// A marker type whose type parameter `T` is considered to be
    /// contravariant with respect to the type itself. This is (typically)
    /// used to indicate that an instance of the type `T` will be consumed
    /// (but not read from), even though that may not be apparent.
    ///
    /// For more information about variance, refer to this Wikipedia
    /// article <http://en.wikipedia.org/wiki/Variance_%28computer_science%29>.
    ///
    /// *Note:* It is very unusual to have to add a contravariant constraint.
    /// If you are not sure, you probably want to use `InvariantType`.
    ///
    /// # Example
    ///
    /// Given a struct `S` that includes a type parameter `T`
    /// but does not actually *reference* that type parameter:
    ///
    /// ```
    /// struct S<T> { x: *() }
    /// fn get<T>(s: &S<T>, v: T) {
    /// unsafe {
    /// let x: fn(T) = cast::transmute(s.x);
    /// x(v)
    /// }
    /// }
    /// ```
    ///
    /// The type system would currently infer that the value of
    /// the type parameter `T` is irrelevant, and hence a `S<int>` is
    /// a subtype of `S<~[int]>` (or, for that matter, `S<U>` for
    /// for any `U`). But this is incorrect because `get()` converts the
    /// `*()` into a `fn(T)` and then passes a value of type `T` to it.
    ///
    /// Supplying a `ContravariantType` marker would correct the
    /// problem, because it would mark `S` so that `S<T>` is only a
    /// subtype of `S<U>` if `U` is a subtype of `T`; given that the
    /// function requires arguments of type `T`, it must also accept
    /// arguments of type `U`, hence such a conversion is safe.
    #[lang="contravariant_type"]
    #[deriving(Eq,Clone)]
    pub struct ContravariantType<T>;

    /// A marker type whose type parameter `T` is considered to be
    /// invariant with respect to the type itself. This is (typically)
    /// used to indicate that instances of the type `T` may be read or
    /// written, even though that may not be apparent.
    ///
    /// For more information about variance, refer to this Wikipedia
    /// article <http://en.wikipedia.org/wiki/Variance_%28computer_science%29>.
    ///
    /// # Example
    ///
    /// The Cell type is an example which uses unsafe code to achieve
    /// "interior" mutability:
    ///
    /// ```
    /// struct Cell<T> { priv value: T }
    /// ```
    ///
    /// The type system would infer that `value` is only read here and
    /// never written, but in fact `Cell` uses unsafe code to achieve
    /// interior mutability.
    #[lang="invariant_type"]
    #[deriving(Eq,Clone)]
    pub struct InvariantType<T>;

    /// As `CovariantType`, but for lifetime parameters. Using
    /// `CovariantLifetime<'a>` indicates that it is ok to substitute
    /// a *longer* lifetime for `'a` than the one you originally
    /// started with (e.g., you could convert any lifetime `'foo` to
    /// `'static`). You almost certainly want `ContravariantLifetime`
    /// instead, or possibly `InvariantLifetime`. The only case where
    /// it would be appropriate is that you have a (type-casted, and
    /// hence hidden from the type system) function pointer with a
    /// signature like `fn(&'a T)` (and no other uses of `'a`). In
    /// this case, it is ok to substitute a larger lifetime for `'a`
    /// (e.g., `fn(&'static T)`), because the function is only
    /// becoming more selective in terms of what it accepts as
    /// argument.
    ///
    /// For more information about variance, refer to this Wikipedia
    /// article <http://en.wikipedia.org/wiki/Variance_%28computer_science%29>.
    #[lang="covariant_lifetime"]
    #[deriving(Eq,Clone)]
    pub struct CovariantLifetime<'a>;

    /// As `ContravariantType`, but for lifetime parameters. Using
    /// `ContravariantLifetime<'a>` indicates that it is ok to
    /// substitute a *shorter* lifetime for `'a` than the one you
    /// originally started with (e.g., you could convert `'static` to
    /// any lifetime `'foo`). This is appropriate for cases where you
    /// have an unsafe pointer that is actually a pointer into some
    /// memory with lifetime `'a`, and thus you want to limit the
    /// lifetime of your data structure to `'a`. An example of where
    /// this is used is the iterator for vectors.
    ///
    /// For more information about variance, refer to this Wikipedia
    /// article <http://en.wikipedia.org/wiki/Variance_%28computer_science%29>.
    #[lang="contravariant_lifetime"]
    #[deriving(Eq,Clone)]
    pub struct ContravariantLifetime<'a>;

    /// As `InvariantType`, but for lifetime parameters. Using
    /// `InvariantLifetime<'a>` indicates that it is not ok to
    /// substitute any other lifetime for `'a` besides its original
    /// value. This is appropriate for cases where you have an unsafe
    /// pointer that is actually a pointer into memory with lifetime `'a`,
    /// and this pointer is itself stored in an inherently mutable
    /// location (such as a `Cell`).
    #[lang="invariant_lifetime"]
    #[deriving(Eq,Clone)]
    pub struct InvariantLifetime<'a>;

    /// A type which is considered "not freezable", meaning that
    /// its contents could change even if stored in an immutable
    /// context or it is the referent of an `&T` pointer. This is
    /// typically embedded in other types, such as `Cell`.
    #[lang="no_freeze_bound"]
    #[deriving(Eq,Clone)]
    pub struct NoFreeze;

    /// A type which is considered "not sendable", meaning that it cannot
    /// be safely sent between tasks, even if it is owned. This is
    /// typically embedded in other types, such as `Gc`, to ensure that
    /// their instances remain thread-local.
    #[lang="no_send_bound"]
    #[deriving(Eq,Clone)]
    pub struct NoSend;

    /// A type which is considered "not POD", meaning that it is not
    /// implicitly copyable. This is typically embedded in other types to
    /// ensure that they are never copied, even if they lack a destructor.
    #[lang="no_pod_bound"]
    #[deriving(Eq,Clone)]
    pub struct NoPod;

    /// A type which is considered managed by the GC. This is typically
    /// embedded in other types.
    #[lang="managed_bound"]
    #[deriving(Eq,Clone)]
    pub struct Managed;
}

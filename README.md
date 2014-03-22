A lightweight standard library for Rust with freestanding support. It provides
a baseline level of functionality without any external dependencies, and an
extended set of features in a traditional hosted environment.

# Configuration

* `--cfg libc` to enable features depending on a C standard library implementation
* `--cfg debug` to enable debugging features (assertions)

# Building

Currently, building to bytecode with `--emit-llvm` and then compiling/linking
with `clang` is required because the Rust compiler cannot build code without
the requirement for segmented stack support from the runtime. There is also no
way to avoid position independent code and linking against the runtime without
making use of `clang`.

```
rustc --cfg libc core/lib.rs --out-dir . -O -Z no-landing-pads
rustc --emit=bc example.rs -O -Z no-landing-pads -L .
clang -O2 -flto -o example example.bc
```

# Macros

Importing macros from crates is currently feature gated. If you wish to load
macros in addition to normal linkage, use the following attributes:
```rust
#[feature(phase)];
#[phase(syntax, link)]
extern crate core;
```

# Freestanding usage

For freestanding use, simply omit the `libc` configuration switch.

LLVM will emit calls to `memcpy`, `memmove` and `memset`. Until [the
fix](https://github.com/mozilla/rust/pull/9945) for symbol visibility lands,
these must be provided by the auxilliary `support.rs` module.

The `support.rs` module must be compiled with `rustc --lib --emit-llvm -passes
inline` and then linked against the bytecode for the main module with `clang
-flto -O2 main.bc support.bc`.

The `inline` pass *must* be run separately to due to
[issue #10116](https://github.com/mozilla/rust/issues/10116) or LLVM will
generate infinitely recursive functions.

# Stack safety

Ideally, stack safety is provided with one or more guard pages and compiler
support for inserting checks based on awareness of the guard pages. GCC has
this as [-fcheck-stack](http://gcc.gnu.org/onlinedocs/gccint/Stack-Checking.html)
but LLVM is missing the feature.

At the moment, `core` only has OS-provided guard pages without the necessary
checks on frames larger than the guard size.

Rust's standard library provides stack safety via LLVM's segmented stack
support, but this has a negative performance and code size impact. It's also
unavailable without the Rust runtime.

# Allocators

Allocators are not yet included due to language deficiencies. See
[issue #31](https://github.com/thestinger/rust-core/issues/31) for details.

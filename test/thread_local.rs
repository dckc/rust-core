// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[no_std];
#[feature(macro_rules, phase)];

#[phase(syntax, link)]
extern crate core;

use core::thread::spawn;
use core::fail::abort;

thread_local!(foo, int, 5)

#[start]
fn main(_: int, _: **u8) -> int {
    if foo::get() != 5 { abort() }
    foo::set(10);
    if foo::get() != 10 { abort() }
    spawn(proc() {
        if foo::get() != 5 { abort() }
        foo::set(20);
        if foo::get() != 20 { abort() }
    });
    if foo::get() != 10 { abort() }
    0
}

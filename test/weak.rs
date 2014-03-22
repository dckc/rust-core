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

extern crate core;

use core::option::{Option, Some, None};
use core::weak::{Strong, Weak};
use core::ignore;
use core::fail::abort;
use core::cell::RefCell;
use core::clone::Clone;

fn test_live() {
    let x = Strong::new(5);
    let y = x.downgrade();
    if !y.upgrade().is_some() { abort() }
}

fn test_dead() {
    let x = Strong::new(5);
    let y = x.downgrade();
    ignore(x);
    if y.upgrade().is_some() { abort() }
}

fn weak_self_cyclic() {
    struct Cycle {
        x: RefCell<Option<Weak<Cycle>>>
    }

    let a = Strong::new(Cycle { x: RefCell::new(None) });
    let b = a.clone().downgrade();
    *a.borrow().x.borrow_mut().get() = Some(b);
}

#[start]
fn main(_: int, _: **u8) -> int {
    test_live();
    test_dead();
    weak_self_cyclic();
    0
}

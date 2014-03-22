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

use core::thread::Pool;
use core::io::stderr;
use core::time::{Time, sleep};

#[start]
fn main(_: int, _: **u8) -> int {
    let pool = Pool::new(4);
    let mut i = 0;
    while i < 16 {
        pool.submit(proc() {
            stderr().write(bytes!("sleeping\n"));
            sleep(Time::from_seconds(1));
            stderr().write(bytes!("waking\n"));
        });
        i += 1
    }
    0
}

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

use core::clone::Clone;
use core::thread::{spawn, spawn_detached};
use core::fail::abort;
use core::concurrent::Queue;

fn foo() -> int {
    10
}

fn bar() -> int {
    5
}

fn baz() {}

#[start]
fn main(_: int, _: **u8) -> int {
    let a = spawn(foo);
    let b = spawn(bar);
    let _c = spawn(baz);

    if *a.join() != 10 || *b.join() != 5 {
        abort()
    }

    let queue = Queue::<int>::new();

    let active = 10;

    let send_queue = queue.clone();
    let consumer = spawn(proc() {
        let mut received = 0;
        let mut active = active;
        loop {
            if send_queue.pop() == -1 {
                active -= 1;
            } else {
                received += 1;
            }
            if active == 0 {
                break;
            }
        }
        received
    });

    let mut i = 0;
    while i < active {
        let send_queue = queue.clone();
        spawn_detached(proc() {
            let mut i = 0;
            while i < 1000 {
                send_queue.push(i);
                i += 1;
            }
            send_queue.push(-1);
        });
        i += 1;
    }

    if *consumer.join() != 10000 {
        abort()
    }

    // A harmless leak of the procs and queue used in the detached producers may occur, since
    // there's no call to `pthread_exit` here.

    0
}

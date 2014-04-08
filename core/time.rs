// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use fail::{EINTR, abort, assert};
use mem::uninit;
use c_types::{c_int, time_t, clockid_t, timespec};
use cmp::{Eq, Ord};

static CLOCK_REALTIME: clockid_t = 0;
static CLOCK_MONOTONIC: clockid_t = 1;
static TIMER_ABSTIME: c_int = 1;

pub struct Time {
    time: timespec
}

impl Time {
    pub fn from_seconds(seconds: time_t) -> Time {
        Time { time: timespec { tv_sec: seconds, tv_nsec: 0 } }
    }

    pub fn from_timespec(time: timespec) -> Time {
        Time { time: time }
    }

    pub fn to_timespec(&self) -> timespec {
        self.time
    }
}

impl Eq for Time {
    fn eq(&self, other: &Time) -> bool {
        self.time.tv_sec == other.time.tv_sec && self.time.tv_nsec == other.time.tv_nsec
    }
}

impl Ord for Time {
    fn lt(&self, other: &Time) -> bool {
        self.time.tv_sec < other.time.tv_sec || self.time.tv_nsec < other.time.tv_nsec
    }
}

extern {
    fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> c_int;
    fn clock_nanosleep(clock_id: clockid_t, flags: c_int, rqtp: *timespec,
                       rmtp: *mut timespec) -> c_int;
}

fn get_time(clock: clockid_t) -> Time {
    unsafe {
        let mut time = uninit();
        if clock_gettime(clock, &mut time) != 0 {
            abort()
        }
        Time { time: time }
    }
}

pub fn real() -> Time {
    get_time(CLOCK_REALTIME)
}

pub fn monotonic() -> Time {
    get_time(CLOCK_MONOTONIC)
}

fn sleep_absolute(clock_id: clockid_t, abstime: Time) {
    loop {
        let ret = unsafe {
            clock_nanosleep(clock_id, TIMER_ABSTIME, &abstime.time, 0 as *mut timespec)
        };
        if ret != EINTR {
            assert(ret == 0);
            break
        }
    }
}

pub fn sleep_until(abstime: Time) {
    sleep_absolute(CLOCK_REALTIME, abstime)
}

pub fn monotonic_sleep_until(abstime: Time) {
    sleep_absolute(CLOCK_MONOTONIC, abstime)
}

pub fn sleep(mut reltime: Time) {
    loop {
        unsafe {
            let mut remain = uninit();
            let ret = clock_nanosleep(CLOCK_MONOTONIC, 0, &reltime.time, &mut remain);
            if ret != EINTR {
                assert(ret == 0);
                break
            }
            reltime.time = remain;
        }
    }
}

// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use container::Container;
use c_types::{c_int, pthread_t, pthread_attr_t, pthread_mutex_t, pthread_mutexattr_t};
use c_types::{pthread_cond_t, pthread_condattr_t, clockid_t, timespec};
use time::Time;
use fail::{EBUSY, ETIMEDOUT, abort, assert};
use ops::Drop;
use kinds::Send;
use mem::{forget, uninit, transmute};
use concurrent::Queue;
use vec::Vec;
use option::{Option, Some, None};
use clone::Clone;

#[deriving(Eq, Clone)]
pub enum TimeoutStatus {
    NoTimeout,
    Timeout
}

extern {
    fn pthread_create(thread: *mut pthread_t, attr: *pthread_attr_t,
                      start_routine: extern "C" fn(*mut u8) -> *mut u8,
                      arg: *mut u8) -> c_int;
    fn pthread_join(thread: pthread_t, retval: *mut *mut u8) -> c_int;

    fn sched_yield() -> c_int;

    fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int;
    fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> c_int;
    fn pthread_attr_setdetachstate(attr: *mut pthread_attr_t, detachstate: c_int) -> c_int;

    #[cfg(debug)]
    fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int;
    #[cfg(debug)]
    fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> c_int;
    #[cfg(debug)]
    fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, ty: c_int) -> c_int;

    fn pthread_mutex_init(mutex: *mut pthread_mutex_t, attr: *pthread_mutexattr_t) -> c_int;
    fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int;
    fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int;
    fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int;
    fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int;

    fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> c_int;
    fn pthread_condattr_destroy(attr: *mut pthread_condattr_t) -> c_int;
    fn pthread_condattr_setclock(attr: *mut pthread_condattr_t, clock_id: clockid_t) -> c_int;

    fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *pthread_condattr_t) -> c_int;
    fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int;
    fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int;
    fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int;
    fn pthread_cond_timedwait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t,
                              abstime: *timespec) -> c_int;
    fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> c_int;
}

static CLOCK_MONOTONIC: clockid_t = 1;
#[cfg(target_os = "android")]
#[cfg(target_os = "freebsd")]
#[cfg(target_os = "linux")]
static PTHREAD_CREATE_DETACHED: c_int = 1;
#[cfg(target_os = "macos")]
static PTHREAD_CREATE_DETACHED: c_int = 2;
#[cfg(debug)]
static PTHREAD_MUTEX_ERRORCHECK: c_int = 2;

/// An owned thread type, joined in the destructor.
pub struct Thread<A> {
    thread: pthread_t
}

extern "C" fn shim(ptr: *mut u8) -> *mut u8 {
    let start_routine = unsafe { *transmute::<*mut u8, ~proc() -> *mut u8>(ptr) };
    start_routine()
}

/// Spawn an owned, joined thread. Joining the thread will block until it completes execution, and
/// this is done automatically by the destructor if the thread isn't manually joined.
pub fn spawn<A: Send>(start_routine: proc() -> A) -> Thread<A> {
    unsafe {
        // FIXME: this wrapper should be unnecessary, shim should be a generic function instead
        // https://github.com/mozilla/rust/issues/10353
        let wrapper: proc() -> ~A = proc() ~start_routine();
        let ptr: *mut u8 = transmute(~wrapper);
        let mut thread = uninit();
        if pthread_create(&mut thread, 0 as *pthread_attr_t, shim, ptr) != 0 {
            abort()
        }
        Thread { thread: thread }
    }
}

extern "C" fn detached_shim(ptr: *mut u8) -> *mut u8 {
    let start_routine = unsafe { *transmute::<*mut u8, ~proc()>(ptr) };
    start_routine();
    0 as *mut u8
}

/// Spawn an unowned, detached thread. If the `main` function returns, the program will exit
/// immediately even if there are unfinished detached threads.
pub fn spawn_detached(start_routine: proc()) {
    unsafe {
        let ptr: *mut u8 = transmute(~start_routine);
        let mut attr = uninit();
        if pthread_attr_init(&mut attr) != 0 {
            abort()
        }
        pthread_attr_setdetachstate(&mut attr, PTHREAD_CREATE_DETACHED);
        let mut thread = uninit();
        if pthread_create(&mut thread, &attr, detached_shim, ptr) != 0 {
            abort()
        }
        assert(pthread_attr_destroy(&mut attr) == 0);
    }
}

impl<A: Send> Thread<A> {
    /// Manually join the thread, retrieving the result of the `proc`.
    pub fn join(self) -> ~A {
        unsafe {
            let mut result = uninit();
            assert(pthread_join(self.thread, &mut result) == 0);
            forget(self);
            transmute(result)
        }
    }
}

#[unsafe_destructor]
impl<A: Send> Drop for Thread<A> {
    fn drop(&mut self) {
        unsafe {
            let mut result = uninit();
            assert(pthread_join(self.thread, &mut result) == 0);
            let _: ~A = transmute(result);
        }
    }
}

/// Yield control from the current thread
pub fn deschedule() {
    unsafe {
        assert(sched_yield() == 0)
    }
}

pub struct Mutex {
    mutex: pthread_mutex_t
}

impl Mutex {
    #[cfg(not(debug))]
    pub fn new() -> Mutex {
        unsafe {
            let mut mutex = uninit();
            if pthread_mutex_init(&mut mutex, 0 as *pthread_mutexattr_t) != 0 {
                abort()
            }
            Mutex { mutex: mutex }
        }
    }

    #[cfg(debug)]
    pub fn new() -> Mutex {
        unsafe {
            let mut attr = uninit();
            if pthread_mutexattr_init(&mut attr) != 0 {
                abort()
            }
            assert(pthread_mutexattr_settype(&mut attr, PTHREAD_MUTEX_ERRORCHECK) == 0);
            let mut mutex = uninit();
            if pthread_mutex_init(&mut mutex, &attr) != 0 {
                abort()
            }
            assert(pthread_mutexattr_destroy(&mut attr) == 0);
            Mutex { mutex: mutex }
        }
    }

    /// Grab ownership of the mutex.
    pub unsafe fn lock(&mut self) {
        assert(pthread_mutex_lock(&mut self.mutex) == 0)
    }

    /// Grab ownership of the mutex, returning a `LockGuard` value releasing ownership of the mutex
    /// in the destructor.
    pub unsafe fn lock_guard<'a>(&'a mut self) -> LockGuard<'a> {
        self.lock();
        LockGuard { mutex: self }
    }

    /// Try to grab ownership of a lock, and return `true` if successful
    pub unsafe fn trylock(&mut self) -> bool {
        let rc = pthread_mutex_trylock(&mut self.mutex);
        if rc == EBUSY {
            false
        } else {
            assert(rc == 0);
            true
        }
    }

    /// Release ownership of the mutex.
    pub unsafe fn unlock(&mut self) {
        assert(pthread_mutex_unlock(&mut self.mutex) == 0)
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        unsafe {
            assert(pthread_mutex_destroy(&mut self.mutex) == 0)
        }
    }
}

pub struct Cond {
    cond: pthread_cond_t
}

impl Cond {
    pub fn new() -> Cond {
        unsafe {
            let mut attr = uninit();
            if pthread_condattr_init(&mut attr) != 0 {
                abort()
            }
            assert(pthread_condattr_setclock(&mut attr, CLOCK_MONOTONIC) == 0);
            let mut cond = uninit();
            if pthread_cond_init(&mut cond, &attr) != 0 {
                abort()
            }
            assert(pthread_condattr_destroy(&mut attr) == 0);
            Cond { cond: cond }
        }
    }

    /// Unblock at least one thread blocked on the condition variable.
    pub unsafe fn signal(&mut self) {
        assert(pthread_cond_signal(&mut self.cond) == 0)
    }

    /// Unblock all the threads blocked on the condition variable.
    pub unsafe fn broadcast(&mut self) {
        assert(pthread_cond_broadcast(&mut self.cond) == 0)
    }

    /// Block on the condition variable, releasing ownership of the mutex until notified. Upon
    /// returning, the mutex will be owned again. Note that spurious wakeups may occur.
    pub unsafe fn wait(&mut self, mutex: &mut Mutex) {
        assert(pthread_cond_wait(&mut self.cond, &mut mutex.mutex) == 0)
    }

    /// Block on the condition variable, releasing ownership of the mutex until notified or the
    /// timeout expires. Upon returning, the mutex will be owned again. Note that spurious wakeups
    /// may occur. Return `Timeout` if a timeout occurs, otherwise `NoTimeout`.
    pub unsafe fn wait_until(&mut self, mutex: &mut Mutex, abstime: Time) -> TimeoutStatus {
        let ret = pthread_cond_timedwait(&mut self.cond, &mut mutex.mutex, &abstime.to_timespec());
        if ret == ETIMEDOUT {
            Timeout
        } else {
            assert(ret == 0);
            NoTimeout
        }
    }

    /// Block on the condition variable, releasing ownership of the mutex until notified or the
    /// timeout expires. Upon returning, the mutex will be owned by the `LockGuard` again. Note that
    /// spurious wakeups may occur. Return `Timeout` if a timeout occurs, otherwise `NoTimeout`.
    pub unsafe fn wait_until_guard(&mut self, guard: &mut LockGuard,
                                   abstime: Time) -> TimeoutStatus {
        self.wait_until(guard.mutex, abstime)
    }

    /// Block on the condition variable, releasing ownership of the mutex until notified. Upon
    /// returning, the mutex will be owned by the `LockGuard` again. Note that spurious wakeups may
    /// occur.
    pub unsafe fn wait_guard(&mut self, guard: &mut LockGuard) {
        self.wait(guard.mutex)
    }
}

impl Drop for Cond {
    fn drop(&mut self) {
        unsafe {
            assert(pthread_cond_destroy(&mut self.cond) == 0);
        }
    }
}

/// A scoped lock taking ownership of a mutex
pub struct LockGuard<'a> {
    mutex: &'a mut Mutex
}

#[unsafe_destructor]
impl<'a> Drop for LockGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            self.mutex.unlock()
        }
    }
}

/// A pool of worker threads
pub struct Pool {
    queue: Queue<Option<proc():Send>>,
    pool: Vec<Thread<()>>
}

impl Pool {
    /// Create a thread pool with `n_threads` threads.
    pub fn new(n_threads: uint) -> Pool {
        let queue: Queue<Option<proc():Send>> = Queue::new();
        let mut pool = Vec::with_capacity(n_threads);
        let mut i = 0;
        while i < n_threads {
            let send_queue = queue.clone();
            pool.push(spawn(proc() {
                let queue = send_queue;
                loop {
                    match queue.pop() {
                        Some(function) => function(),
                        None => break
                    }
                }
            }));
            i += 1;
        }
        Pool { queue: queue, pool: pool }
    }

    /// Submit a task to the thread pool. They are run in FIFO order to completion.
    pub fn submit(&self, task: proc():Send) {
        self.queue.push(Some(task))
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        let mut i = 0;
        while i < self.pool.len() {
            self.queue.push(None);
            i += 1;
        }
    }
}

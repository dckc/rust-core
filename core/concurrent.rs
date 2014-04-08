// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Concurrent data structures
//!
//! This module defines mutable concurrent data structures built on top of
//! atomic reference counting (`core::arc`).

use container::Container;
use clone::Clone;
use arc::Arc;
use deque::Deque;
use priority_queue::PriorityQueue;
use mem::transmute;
use thread::{Mutex, Cond, Timeout};
use cmp::{Eq, Ord};
use option::{Some, None, Option};
use hash::{Hash, HashMap};
use vec::Vec;
use kinds::Send;
use kinds::marker::NoFreeze;
use time::{Time, monotonic};

trait GenericQueue<T>: Container {
    fn generic_push(&mut self, item: T);
    fn generic_pop(&mut self) -> Option<T>;
}

impl<T> GenericQueue<T> for Deque<T> {
    fn generic_push(&mut self, item: T) { self.push_back(item) }
    fn generic_pop(&mut self) -> Option<T> { self.pop_front() }
}

impl<T: Ord> GenericQueue<T> for PriorityQueue<T> {
    fn generic_push(&mut self, item: T) { self.push(item) }
    fn generic_pop(&mut self) -> Option<T> { self.pop() }
}

struct QueueBox<T> {
    queue: T,
    mutex: Mutex,
    not_empty: Cond,
    no_freeze: NoFreeze
}

struct QueuePtr<T> {
    ptr: Arc<QueueBox<T>>
}

impl<A: Send, T: GenericQueue<A>> QueuePtr<T> {
    fn new(queue: T) -> QueuePtr<T> {
        unsafe {
            let b = QueueBox { queue: queue, mutex: Mutex::new(), not_empty: Cond::new(),
                               no_freeze: NoFreeze };
            QueuePtr { ptr: Arc::new_unchecked(b) }
        }
    }

    fn pop(&self) -> A {
        unsafe {
            let ptr: &mut QueueBox<T> = transmute(self.ptr.borrow());
            let mut guard = ptr.mutex.lock_guard();
            while ptr.queue.is_empty() {
                ptr.not_empty.wait_guard(&mut guard)
            }
            ptr.queue.generic_pop().get()
        }
    }

    fn try_pop(&self) -> Option<A> {
        unsafe {
            let ptr: &mut QueueBox<T> = transmute(self.ptr.borrow());
            let _guard = ptr.mutex.lock_guard();
            ptr.queue.generic_pop()
        }
    }

    fn pop_timeout(&self, reltime: Time) -> Option<A> {
        unsafe {
            let mut abstime = monotonic().to_timespec();
            abstime.tv_sec += reltime.to_timespec().tv_sec;
            abstime.tv_nsec += reltime.to_timespec().tv_nsec;

            let ptr: &mut QueueBox<T> = transmute(self.ptr.borrow());
            let mut guard = ptr.mutex.lock_guard();
            while ptr.queue.is_empty() {
                if ptr.not_empty.wait_until_guard(&mut guard, Time::from_timespec(abstime)) == Timeout {
                    return None
                }
            }
            Some(ptr.queue.generic_pop().get())
        }
    }

    fn push(&self, item: A) {
        unsafe {
            let ptr: &mut QueueBox<T> = transmute(self.ptr.borrow());
            let _guard = ptr.mutex.lock_guard();
            ptr.queue.generic_push(item);
            ptr.not_empty.signal()
        }
    }
}

impl<T> Clone for QueuePtr<T> {
    fn clone(&self) -> QueuePtr<T> {
        QueuePtr { ptr: self.ptr.clone() }
    }
}

/// An unbounded, blocking concurrent queue
pub struct Queue<T> {
    ptr: QueuePtr<Deque<T>>
}

impl<T: Send> Queue<T> {
    /// Return a new `Queue` instance.
    pub fn new() -> Queue<T> {
        Queue { ptr: QueuePtr::new(Deque::new()) }
    }

    /// Pop a value from the front of the queue, blocking until the queue is not empty.
    pub fn pop(&self) -> T {
        self.ptr.pop()
    }

    /// Pop a value from the front of the queue, or return None if the queue is empty.
    pub fn try_pop(&self) -> Option<T> {
        self.ptr.try_pop()
    }

    /// Pop a value from the front of the queue, blocking until the queue is not empty or the
    /// timeout expires.
    pub fn pop_timeout(&self, reltime: Time) -> Option<T> {
        self.ptr.pop_timeout(reltime)
    }

    /// Push a value to the back of the queue.
    pub fn push(&self, item: T) {
        self.ptr.push(item)
    }
}

impl<T> Clone for Queue<T> {
    /// Return a shallow copy of the queue
    fn clone(&self) -> Queue<T> {
        Queue { ptr: self.ptr.clone() }
    }
}

/// An unbounded, blocking concurrent priority queue
pub struct BlockingPriorityQueue<T> {
    ptr: QueuePtr<PriorityQueue<T>>
}

impl<T: Ord + Send> BlockingPriorityQueue<T> {
    /// Return a new `BlockingPriorityQueue` instance.
    pub fn new() -> BlockingPriorityQueue<T> {
        BlockingPriorityQueue { ptr: QueuePtr::new(PriorityQueue::new()) }
    }

    /// Pop the largest value from the queue, blocking until the queue is not empty.
    pub fn pop(&self) -> T {
        self.ptr.pop()
    }

    /// Pop the largest value from the queue, or return None if the queue is empty.
    pub fn try_pop(&self) -> Option<T> {
        self.ptr.try_pop()
    }

    /// Pop the largest value from the queue, blocking until the queue is not empty or the timeout
    /// expires.
    pub fn pop_timeout(&self, reltime: Time) -> Option<T> {
        self.ptr.pop_timeout(reltime)
    }

    /// Push a value into the queue.
    pub fn push(&self, item: T) {
        self.ptr.push(item)
    }
}

impl<T> Clone for BlockingPriorityQueue<T> {
    /// Return a shallow copy of the queue
    fn clone(&self) -> BlockingPriorityQueue<T> {
        BlockingPriorityQueue { ptr: self.ptr.clone() }
    }
}

struct BoundedQueueBox<T> {
    deque: T,
    mutex: Mutex,
    not_empty: Cond,
    not_full: Cond,
    maximum: uint,
    no_freeze: NoFreeze
}

struct BoundedQueuePtr<T> {
    ptr: Arc<BoundedQueueBox<T>>
}

impl<A: Send, T: GenericQueue<A>> BoundedQueuePtr<T> {
    fn new(maximum: uint, queue: T) -> BoundedQueuePtr<T> {
        unsafe {
            let b = BoundedQueueBox { deque: queue, mutex: Mutex::new(), not_empty: Cond::new(),
                                      not_full: Cond::new(), maximum: maximum,
                                      no_freeze: NoFreeze };
            BoundedQueuePtr { ptr: Arc::new_unchecked(b) }
        }
    }

    fn pop(&self) -> A {
        unsafe {
            let ptr: &mut BoundedQueueBox<T> = transmute(self.ptr.borrow());
            let mut guard = ptr.mutex.lock_guard();
            while ptr.deque.is_empty() {
                ptr.not_empty.wait_guard(&mut guard)
            }
            let item = ptr.deque.generic_pop().get();
            ptr.not_full.signal();
            item
        }
    }

    fn try_pop(&self) -> Option<A> {
        unsafe {
            let ptr: &mut BoundedQueueBox<T> = transmute(self.ptr.borrow());
            let _guard = ptr.mutex.lock_guard();
            match ptr.deque.generic_pop() {
                Some(x) => {
                    ptr.not_full.signal();
                    Some(x)
                }
                None => None
            }
        }
    }

    fn pop_timeout(&self, reltime: Time) -> Option<A> {
        unsafe {
            let mut abstime = monotonic().to_timespec();
            abstime.tv_sec += reltime.to_timespec().tv_sec;
            abstime.tv_nsec += reltime.to_timespec().tv_nsec;

            let ptr: &mut BoundedQueueBox<T> = transmute(self.ptr.borrow());
            let mut guard = ptr.mutex.lock_guard();
            while ptr.deque.is_empty() {
                if ptr.not_empty.wait_until_guard(&mut guard, Time::from_timespec(abstime)) == Timeout {
                    return None
                }
            }
            let item = ptr.deque.generic_pop().get();
            ptr.not_full.signal();
            Some(item)
        }
    }

    fn push(&self, item: A) {
        unsafe {
            let ptr: &mut BoundedQueueBox<T> = transmute(self.ptr.borrow());
            let mut guard = ptr.mutex.lock_guard();
            while ptr.deque.len() == ptr.maximum {
                ptr.not_full.wait_guard(&mut guard)
            }
            ptr.deque.generic_push(item);
            ptr.not_empty.signal()
        }
    }

    fn try_push(&self, item: A) -> Option<A> {
        unsafe {
            let ptr: &mut BoundedQueueBox<T> = transmute(self.ptr.borrow());
            let _guard = ptr.mutex.lock_guard();
            if ptr.deque.len() == ptr.maximum {
                Some(item)
            } else {
                ptr.deque.generic_push(item);
                ptr.not_empty.signal();
                None
            }
        }
    }

    fn push_timeout(&self, item: A, reltime: Time) -> Option<A> {
        unsafe {
            let mut abstime = monotonic().to_timespec();
            abstime.tv_sec += reltime.to_timespec().tv_sec;
            abstime.tv_nsec += reltime.to_timespec().tv_nsec;

            let ptr: &mut BoundedQueueBox<T> = transmute(self.ptr.borrow());
            let mut guard = ptr.mutex.lock_guard();
            while ptr.deque.len() == ptr.maximum {
                if ptr.not_full.wait_until_guard(&mut guard, Time::from_timespec(abstime)) == Timeout {
                    return Some(item)
                }
            }
            ptr.deque.generic_push(item);
            ptr.not_empty.signal();
            None
        }
    }
}

impl<T> Clone for BoundedQueuePtr<T> {
    fn clone(&self) -> BoundedQueuePtr<T> {
        BoundedQueuePtr { ptr: self.ptr.clone() }
    }
}

/// A bounded, blocking concurrent queue
pub struct BoundedQueue<T> {
    ptr: BoundedQueuePtr<Deque<T>>
}

impl<T: Send> BoundedQueue<T> {
    /// Return a new `BoundedQueue` instance, holding at most `maximum` elements.
    pub fn new(maximum: uint) -> BoundedQueue<T> {
        BoundedQueue { ptr: BoundedQueuePtr::new(maximum, Deque::new()) }
    }

    /// Pop a value from the front of the queue, blocking until the queue is not empty.
    pub fn pop(&self) -> T {
        self.ptr.pop()
    }

    /// Pop a value from the front of the queue, or return `None` if the queue is empty.
    pub fn try_pop(&self) -> Option<T> {
        self.ptr.try_pop()
    }

    /// Pop a value from the front of the queue, blocking until the queue is not empty or the
    /// timeout expires.
    pub fn pop_timeout(&self, reltime: Time) -> Option<T> {
        self.ptr.pop_timeout(reltime)
    }

    /// Push a value to the back of the queue, blocking until the queue is not full.
    pub fn push(&self, item: T) {
        self.ptr.push(item)
    }

    /// Push a value to the back of the queue, or return `Some(item)` if the queue is full.
    pub fn try_push(&self, item: T) -> Option<T> {
        self.ptr.try_push(item)
    }

    /// Push a value to the back of the queue, blocking until the queue is not full or the timeout
    /// expires. If the timeout expires, return `Some(item)`.
    pub fn push_timeout(&self, item: T, reltime: Time) -> Option<T> {
        self.ptr.push_timeout(item, reltime)
    }
}

impl<T> Clone for BoundedQueue<T> {
    /// Return a shallow copy of the queue
    fn clone(&self) -> BoundedQueue<T> {
        BoundedQueue { ptr: self.ptr.clone() }
    }
}

/// A bounded, blocking concurrent priority queue
pub struct BoundedPriorityQueue<T> {
    ptr: BoundedQueuePtr<PriorityQueue<T>>
}

impl<T: Ord + Send> BoundedPriorityQueue<T> {
    /// Return a new `BoundedPriorityQueue` instance, holding at most `maximum` elements.
    pub fn new(maximum: uint) -> BoundedPriorityQueue<T> {
        BoundedPriorityQueue { ptr: BoundedQueuePtr::new(maximum, PriorityQueue::new()) }
    }

    /// Pop the largest value from the queue, blocking until the queue is not empty
    pub fn pop(&self) -> T {
        self.ptr.pop()
    }

    /// Pop the largest value from the queue, or return `None` if the queue is empty.
    pub fn try_pop(&self) -> Option<T> {
        self.ptr.try_pop()
    }

    /// Pop the largest value from the queue, blocking until the queue is not empty or the timeout
    /// expires.
    pub fn pop_timeout(&self, reltime: Time) -> Option<T> {
        self.ptr.pop_timeout(reltime)
    }

    /// Push a value into the queue, blocking until the queue is not full.
    pub fn push(&self, item: T) {
        self.ptr.push(item)
    }

    /// Push a value into the queue, or return `Some(item)` if the queue is full.
    pub fn try_push(&self, item: T) -> Option<T> {
        self.ptr.try_push(item)
    }

    /// Push a value into the queue, blocking until the queue is not full or the timeout expires. If
    /// the timeout expires, return `Some(item)`.
    pub fn push_timeout(&self, item: T, reltime: Time) -> Option<T> {
        self.ptr.push_timeout(item, reltime)
    }
}

impl<T> Clone for BoundedPriorityQueue<T> {
    /// Return a shallow copy of the queue
    fn clone(&self) -> BoundedPriorityQueue<T> {
        BoundedPriorityQueue { ptr: self.ptr.clone() }
    }
}

struct LockedHashMap<K, V> {
    map: HashMap<K, V>,
    mutex: Mutex,
    no_freeze: NoFreeze
}

impl<K: Hash + Eq, V> LockedHashMap<K, V> {
    fn with_capacity_and_keys(k0: u64, k1: u64, capacity: uint) -> LockedHashMap<K, V> {
        LockedHashMap {
            map: HashMap::with_capacity_and_keys(k0, k1, capacity),
            mutex: Mutex::new(),
            no_freeze: NoFreeze
        }
    }

    fn swap(&mut self, k: K, v: V) -> Option<V> {
        unsafe {
            let _guard = self.mutex.lock_guard();
            self.map.swap(k, v)
        }
    }

    fn pop(&mut self, k: &K) -> Option<V> {
        unsafe {
            let _guard = self.mutex.lock_guard();
            self.map.pop(k)
        }
    }
}

impl<K: Hash + Eq, V: Clone> LockedHashMap<K, V> {
    fn find(&mut self, k: &K) -> Option<V> {
        unsafe {
            let _guard = self.mutex.lock_guard();
            self.map.find(k).map(|v| v.clone())
        }
    }
}

/// A concurrent hash table based a single lock per instance
pub struct ConcurrentHashMap<K, V> {
    ptr: Arc<LockedHashMap<K, V>>
}

impl<K: Hash + Eq + Send, V: Send> ConcurrentHashMap<K, V> {
    /// Create a new `ConcurrentHashMap` with the specified 128-bit hash key (`k0` and `k1`) and
    /// initial `capacity`.
    pub fn with_capacity_and_keys(k0: u64, k1: u64, capacity: uint) -> ConcurrentHashMap<K, V> {
        let b = LockedHashMap::with_capacity_and_keys(k0, k1, capacity);
        unsafe {
            ConcurrentHashMap { ptr: Arc::new_unchecked(b) }
        }
    }

    /// Insert a key-value pair into the hash table. Return the old value corresponding to the key.
    pub fn swap(&self, k: K, v: V) -> Option<V> {
        unsafe {
            let ptr: &mut LockedHashMap<K, V> = transmute(self.ptr.borrow());
            ptr.swap(k, v)
        }
    }

    /// Remove a key-value pair from the map. Return the value corresponding to the key.
    pub fn pop(&self, k: &K) -> Option<V> {
        unsafe {
            let ptr: &mut LockedHashMap<K, V> = transmute(self.ptr.borrow());
            ptr.pop(k)
        }
    }
}

impl<K: Hash + Eq, V: Clone> ConcurrentHashMap<K, V> {
    /// Return the value corresponding to the key via `clone`.
    ///
    /// A reference cannot be returned directly, because a lock has to be obtained and released by
    /// the function.
    pub fn find(&self, k: &K) -> Option<V> {
        unsafe {
            let ptr: &mut LockedHashMap<K, V> = transmute(self.ptr.borrow());
            ptr.find(k)
        }
    }
}

impl<K, V> Clone for ConcurrentHashMap<K, V> {
    /// Return a shallow copy of the map
    fn clone(&self) -> ConcurrentHashMap<K, V> {
        ConcurrentHashMap { ptr: self.ptr.clone() }
    }
}

struct ShardMapBox<K, V> {
    maps: Vec<LockedHashMap<K, V>>,
    k0: u64,
    k1: u64,
    no_freeze: NoFreeze
}

impl<K: Hash + Eq, V> ShardMapBox<K, V> {
    fn get_shard(&self, k: &K) -> uint {
        k.hash(self.k0, self.k1) as uint % self.maps.len()
    }
}

/// A concurrent hash table distributing keys across shards, with locking on a per-shard basis
pub struct ShardMap<K, V> {
    ptr: Arc<ShardMapBox<K, V>>
}

impl<K: Hash + Eq + Send, V: Send> ShardMap<K, V> {
    /// Create a new `ShardMap` with `shards` internal hash tables, the specified 128-bit hash key
    /// (`k0` and `k1`) and an initial `capacity`.
    pub fn with_capacity_and_keys(shards: uint, k0: u64, k1: u64, capacity: uint) -> ShardMap<K, V> {
        let mut xs = Vec::with_capacity(shards);
        let mut i = 0;
        while i < shards {
            xs.push(LockedHashMap::with_capacity_and_keys(k0, k1, capacity));
            i += 1;
        }
        let inner = ShardMapBox { maps: xs, k0: k0, k1: k1, no_freeze: NoFreeze };
        unsafe {
            ShardMap { ptr: Arc::new_unchecked(inner) }
        }
    }

    /// Insert a key-value pair into the hash table. Return the old value corresponding to the key.
    pub fn swap(&self, k: K, v: V) -> Option<V> {
        unsafe {
            let ptr: &mut ShardMapBox<K, V> = transmute(self.ptr.borrow());
            let shard = ptr.get_shard(&k);
            ptr.maps.as_mut_slice()[shard].swap(k, v)
        }
    }

    /// Remove a key-value pair from the map. Return the value corresponding to the key.
    pub fn pop(&self, k: &K) -> Option<V> {
        unsafe {
            let ptr: &mut ShardMapBox<K, V> = transmute(self.ptr.borrow());
            let shard = ptr.get_shard(k);
            ptr.maps.as_mut_slice()[shard].pop(k)
        }
    }
}

impl<K: Hash + Eq, V: Clone> ShardMap<K, V> {
    /// Return the value corresponding to the key via `clone`.
    ///
    /// A reference cannot be returned directly, because a lock has to be obtained and released by
    /// the function.
    pub fn find(&self, k: &K) -> Option<V> {
        unsafe {
            let ptr: &mut ShardMapBox<K, V> = transmute(self.ptr.borrow());
            let shard = ptr.get_shard(k);
            ptr.maps.as_mut_slice()[shard].find(k)
        }
    }
}

impl<K, V> Clone for ShardMap<K, V> {
    /// Return a shallow copy of the map
    fn clone(&self) -> ShardMap<K, V> {
        ShardMap { ptr: self.ptr.clone() }
    }
}

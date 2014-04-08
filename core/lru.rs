// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A cache holding a limited number of key-value pairs. When the capacity of
//! the cache is exceeded, the least-recently-used (where "used" means a
//! look-up or putting the pair into the cache) pair is automatically removed.
//!
//! # Example
//!
//! ```rust
//! let mut cache: LruCache<int, int> = LruCache::new(2);
//! cache.put(1, 10);
//! cache.put(2, 20);
//! cache.put(3, 30);
//! assert!(cache.get(&1).is_none());
//! assert_eq!(*cache.get(&2).get(), 20);
//! assert_eq!(*cache.get(&3).get(), 30);
//!
//! cache.put(2, 22);
//! assert_eq!(*cache.get(&2).get(), 22);
//!
//! cache.put(6, 60);
//! assert!(cache.get(&3).is_none());
//!
//! cache.change_capacity(1);
//! assert!(cache.get(&2).is_none());
//! ```

use container::Container;
use hash::{HashMap, HashBytes};
use mem::transmute;
use option::{Some, None, Option};
use cmp::Eq;
use ops::Drop;

struct KeyRef<K> { k: *K }

struct LruEntry<K, V> {
    key: Option<K>,
    value: Option<V>,
    next: *mut LruEntry<K, V>,
    prev: *mut LruEntry<K, V>,
}

/// An LRU Cache.
pub struct LruCache<K, V> {
    map: HashMap<KeyRef<K>, ~LruEntry<K, V>>,
    max_size: uint,
    head: *mut LruEntry<K, V>,
    tail: *mut LruEntry<K, V>,
}

impl<K: HashBytes> HashBytes for KeyRef<K> {
    fn hash_bytes(&self, f: |&[u8]|) {
        unsafe { (*self.k).hash_bytes(f) }
    }
}

impl<K: Eq> Eq for KeyRef<K> {
    fn eq(&self, other: &KeyRef<K>) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

impl<K, V> LruEntry<K, V> {
    fn new() -> LruEntry<K, V> {
        LruEntry {
            key: None,
            value: None,
            next: 0 as *mut LruEntry<K, V>,
            prev: 0 as *mut LruEntry<K, V>
        }
    }

    fn with_key_value(k: K, v: V) -> LruEntry<K, V> {
        LruEntry {
            key: Some(k),
            value: Some(v),
            next: 0 as *mut LruEntry<K, V>,
            prev: 0 as *mut LruEntry<K, V>
        }
    }
}

impl<K: HashBytes + Eq, V> LruCache<K, V> {
    /// Create an LRU Cache holding at most `capacity` items.
    pub fn new(k0: u64, k1: u64, capacity: uint) -> LruCache<K, V> {
        let cache = LruCache {
            map: HashMap::with_capacity_and_keys(k0, k1, capacity),
            max_size: capacity,
            head: unsafe { transmute(~LruEntry::<K, V>::new()) },
            tail: unsafe { transmute(~LruEntry::<K, V>::new()) },
        };
        unsafe {
            (*cache.head).next = cache.tail;
            (*cache.tail).prev = cache.head;
        }
        return cache;
    }

    /// Put a key-value pair into cache.
    pub fn put(&mut self, k: K, v: V) {
        let mut key_existed = false;
        let (node_ptr, node_opt) = match self.map.find_mut(&KeyRef{k: &k}) {
            Some(node) => {
                key_existed = true;
                node.value = Some(v);
                let node_ptr: *mut LruEntry<K, V> = &mut **node;
                (node_ptr, None)
            }
            None => {
                let mut node = ~LruEntry::with_key_value(k, v);
                let node_ptr: *mut LruEntry<K, V> = &mut *node;
                (node_ptr, Some(node))
            }
        };
        if key_existed {
            self.detach(node_ptr);
            self.attach(node_ptr);
        } else {
            let keyref = unsafe { (*node_ptr).key.as_ref().get() };
            self.map.swap(KeyRef{k: keyref}, node_opt.get());
            self.attach(node_ptr);
            if self.len() > self.capacity() {
                self.remove_lru();
            }
        }
    }

    /// Return a value corresponding to the key in the cache.
    pub fn get<'a>(&'a mut self, k: &K) -> Option<&'a V> {
        let (value, node_ptr_opt) = match self.map.find_mut(&KeyRef{k: k}) {
            None => (None, None),
            Some(node) => {
                let node_ptr: *mut LruEntry<K, V> = &mut **node;
                unsafe {
                    match (*node_ptr).value {
                        None => (None, None),
                        Some(ref value) => (Some(value), Some(node_ptr))
                    }
                }
            }
        };
        match node_ptr_opt {
            None => (),
            Some(node_ptr) => {
                self.detach(node_ptr);
                self.attach(node_ptr);
            }
        }
        return value;
    }

    /// Remove and return a value corresponding to the key from the cache.
    pub fn pop(&mut self, k: &K) -> Option<V> {
        match self.map.pop(&KeyRef{k: k}) {
            None => None,
            Some(lru_entry) => lru_entry.value
        }
    }

    /// Return the maximum number of key-value pairs the cache can hold.
    pub fn capacity(&self) -> uint {
        self.max_size
    }

    /// Change the number of key-value pairs the cache can hold. Remove
    /// least-recently-used key-value pairs if necessary.
    pub fn change_capacity(&mut self, capacity: uint) {
        let mut i = capacity;
        while i < self.len() {
            self.remove_lru();
            i += 1;
        }
        self.max_size = capacity;
    }

    #[inline]
    fn remove_lru(&mut self) {
        if self.len() > 0 {
            let lru = unsafe { (*self.tail).prev };
            self.detach(lru);
            unsafe {
                match (*lru).key {
                    None => (),
                    Some(ref k) => { self.map.pop(&KeyRef{k: k}); }
                }
            }
        }
    }

    #[inline]
    fn detach(&mut self, node: *mut LruEntry<K, V>) {
        unsafe {
            (*(*node).prev).next = (*node).next;
            (*(*node).next).prev = (*node).prev;
        }
    }

    #[inline]
    fn attach(&mut self, node: *mut LruEntry<K, V>) {
        unsafe {
            (*node).next = (*self.head).next;
            (*node).prev = self.head;
            (*self.head).next = node;
            (*(*node).next).prev = node;
        }
    }
}

impl<K: HashBytes + Eq, V> Container for LruCache<K, V> {
    /// Return the number of key-value pairs in the cache.
    fn len(&self) -> uint {
        self.map.len()
    }
}

#[unsafe_destructor]
impl<K, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        unsafe {
            let _: ~LruEntry<K, V> = transmute(self.head);
            let _: ~LruEntry<K, V> = transmute(self.tail);
        }
    }
}

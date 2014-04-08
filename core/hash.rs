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
use iter::Iterator;
use slice::iter;
use option::{None, Option, Some};
use fail::abort;
use cmp::{Eq, max};
use vec::Vec;
use mem::{replace, size_of};

pub trait Hash {
    fn hash(&self, k0: u64, k1: u64) -> u64;
}

pub trait HashBytes {
    fn hash_bytes(&self, f: |&[u8]|);
}

impl<A: HashBytes> Hash for A {
    #[inline]
    fn hash(&self, k0: u64, k1: u64) -> u64 {
        let mut s = State::new(k0, k1);
        self.hash_bytes(|bytes| s.write(bytes));
        s.result()
    }
}

impl HashBytes for () {
    #[inline(always)]
    fn hash_bytes(&self, _: |&[u8]|) {}
}

impl HashBytes for bool {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        f([*self as u8])
    }
}

impl HashBytes for u8 {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        f([*self])
    }
}

impl HashBytes for u16 {
    #[inline]
    fn hash_bytes(&self, f: |&[u8]|) {
        f([*self as u8, (*self >> 8) as u8])
    }
}

impl HashBytes for u32 {
    #[inline]
    fn hash_bytes(&self, f: |&[u8]|) {
        f([*self as u8,
           (*self >> 8) as u8,
           (*self >> 16) as u8,
           (*self >> 24) as u8])
    }
}

impl HashBytes for u64 {
    #[inline]
    fn hash_bytes(&self, f: |&[u8]|) {
        f([*self as u8,
           (*self >> 8) as u8,
           (*self >> 16) as u8,
           (*self >> 24) as u8,
           (*self >> 32) as u8,
           (*self >> 40) as u8,
           (*self >> 48) as u8,
           (*self >> 56) as u8])
    }
}

impl HashBytes for i8 {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        (*self as u8).hash_bytes(f)
    }
}

impl HashBytes for i16 {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        (*self as u16).hash_bytes(f)
    }
}

impl HashBytes for i32 {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        (*self as u32).hash_bytes(f)
    }
}

impl HashBytes for i64 {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        (*self as u64).hash_bytes(f)
    }
}

impl HashBytes for char {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        (*self as u32).hash_bytes(f)
    }
}

#[cfg(target_word_size = "32")]
impl HashBytes for uint {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        (*self as u32).hash_bytes(f)
    }
}

#[cfg(target_word_size = "64")]
impl HashBytes for uint {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        (*self as u64).hash_bytes(f)
    }
}

impl HashBytes for int {
    #[inline(always)]
    fn hash_bytes(&self, f: |&[u8]|) {
        (*self as uint).hash_bytes(f)
    }
}

impl<'a, A: HashBytes> HashBytes for &'a [A] {
    #[inline]
    fn hash_bytes(&self, f: |&[u8]|) {
        self.len().hash_bytes(|x| f(x));
        for e in iter(*self) {
            e.hash_bytes(|x| f(x))
        }
    }
}

pub struct State {
    k0: u64,
    k1: u64,
    length: uint, // how many bytes we've processed
    v0: u64,      // hash state
    v1: u64,
    v2: u64,
    v3: u64,
    tail: [u8, ..8], // unprocessed bytes
    ntail: uint      // how many bytes in tail are valid
}

macro_rules! u8to64_le (
    ($buf:expr, $i:expr) =>
    ($buf[0+$i] as u64 |
     $buf[1+$i] as u64 << 8 |
     $buf[2+$i] as u64 << 16 |
     $buf[3+$i] as u64 << 24 |
     $buf[4+$i] as u64 << 32 |
     $buf[5+$i] as u64 << 40 |
     $buf[6+$i] as u64 << 48 |
     $buf[7+$i] as u64 << 56)
)

macro_rules! rotl (
    ($x:expr, $b:expr) =>
    (($x << $b) | ($x >> (64 - $b)))
)

macro_rules! compress (
    ($v0:expr, $v1:expr, $v2:expr, $v3:expr) =>
    ({
        $v0 += $v1; $v1 = rotl!($v1, 13); $v1 ^= $v0;
        $v0 = rotl!($v0, 32);
        $v2 += $v3; $v3 = rotl!($v3, 16); $v3 ^= $v2;
        $v0 += $v3; $v3 = rotl!($v3, 21); $v3 ^= $v0;
        $v2 += $v1; $v1 = rotl!($v1, 17); $v1 ^= $v2;
        $v2 = rotl!($v2, 32);
    })
)

impl State {
    #[inline]
    pub fn new(key0: u64, key1: u64) -> State {
        let mut state = State {
            k0: key0,
            k1: key1,
            length: 0,
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            tail: [ 0, 0, 0, 0, 0, 0, 0, 0 ],
            ntail: 0,
        };
        state.reset();
        state
    }

    #[inline]
    pub fn result(&mut self) -> u64 {
        let mut v0 = self.v0;
        let mut v1 = self.v1;
        let mut v2 = self.v2;
        let mut v3 = self.v3;

        let mut b : u64 = (self.length as u64 & 0xff) << 56;

        if self.ntail > 0 { b |= self.tail[0] as u64 <<  0; }
        if self.ntail > 1 { b |= self.tail[1] as u64 <<  8; }
        if self.ntail > 2 { b |= self.tail[2] as u64 << 16; }
        if self.ntail > 3 { b |= self.tail[3] as u64 << 24; }
        if self.ntail > 4 { b |= self.tail[4] as u64 << 32; }
        if self.ntail > 5 { b |= self.tail[5] as u64 << 40; }
        if self.ntail > 6 { b |= self.tail[6] as u64 << 48; }

        v3 ^= b;
        compress!(v0, v1, v2, v3);
        compress!(v0, v1, v2, v3);
        v0 ^= b;

        v2 ^= 0xff;
        compress!(v0, v1, v2, v3);
        compress!(v0, v1, v2, v3);
        compress!(v0, v1, v2, v3);
        compress!(v0, v1, v2, v3);

        (v0 ^ v1 ^ v2 ^ v3)
    }

    #[inline]
    pub fn write(&mut self, msg: &[u8]) {
        let length = msg.len();
        self.length += length;

        let mut needed = 0u;

        if self.ntail != 0 {
            needed = 8 - self.ntail;

            if length < needed {
                let mut t = 0;
                while t < length {
                    self.tail[self.ntail+t] = msg[t];
                    t += 1;
                }
                self.ntail += length;
                return;
            }

            let mut t = 0;
            while t < needed {
                self.tail[self.ntail+t] = msg[t];
                t += 1;
            }

            let m = u8to64_le!(self.tail, 0);

            self.v3 ^= m;
            compress!(self.v0, self.v1, self.v2, self.v3);
            compress!(self.v0, self.v1, self.v2, self.v3);
            self.v0 ^= m;

            self.ntail = 0;
        }

        // Buffered tail is now flushed, process new input.
        let len = length - needed;
        let end = len & (!0x7);
        let left = len & 0x7;

        let mut i = needed;
        while i < end {
            let mi = u8to64_le!(msg, i);

            self.v3 ^= mi;
            compress!(self.v0, self.v1, self.v2, self.v3);
            compress!(self.v0, self.v1, self.v2, self.v3);
            self.v0 ^= mi;

            i += 8;
        }

        let mut t = 0u;
        while t < left {
            self.tail[t] = msg[i+t];
            t += 1
        }
        self.ntail = left;
    }


    #[inline]
    pub fn reset(&mut self) {
        self.length = 0;
        self.v0 = self.k0 ^ 0x736f6d6570736575;
        self.v1 = self.k1 ^ 0x646f72616e646f6d;
        self.v2 = self.k0 ^ 0x6c7967656e657261;
        self.v3 = self.k1 ^ 0x7465646279746573;
        self.ntail = 0;
    }
}

static INITIAL_CAPACITY: uint = 32u; // 2^5

struct Bucket<K,V> {
    hash: uint,
    key: K,
    value: V,
}

pub struct HashMap<K,V> {
    k0: u64,
    k1: u64,
    resize_at: uint,
    size: uint,
    buckets: Vec<Option<Bucket<K, V>>>
}

enum SearchResult {
    FoundEntry(uint), FoundHole(uint), TableFull
}

#[inline(always)]
fn resize_at(capacity: uint) -> uint {
    (capacity * 3) / 4
}

impl<K:Hash + Eq,V> HashMap<K, V> {
    #[inline(always)]
    fn to_bucket(&self, h: uint) -> uint {
        h % self.buckets.len()
    }

    #[inline(always)]
    fn next_bucket(&self, idx: uint, len_buckets: uint) -> uint {
        (idx + 1) % len_buckets
    }

    #[inline]
    fn bucket_sequence(&self, hash: uint, op: |uint| -> bool) -> bool {
        let start_idx = self.to_bucket(hash);
        let len_buckets = self.buckets.len();
        let mut idx = start_idx;
        loop {
            if !op(idx) { return false; }
            idx = self.next_bucket(idx, len_buckets);
            if idx == start_idx {
                return true;
            }
        }
    }

    #[inline]
    fn bucket_for_key(&self, k: &K) -> SearchResult {
        let hash = k.hash(self.k0, self.k1) as uint;
        self.bucket_for_key_with_hash(hash, k)
    }

    #[inline]
    fn bucket_for_key_with_hash(&self,
                                hash: uint,
                                k: &K)
                             -> SearchResult {
        let mut ret = TableFull;
        self.bucket_sequence(hash, |i| {
            match self.buckets.as_slice()[i] {
                Some(ref bkt) if bkt.hash == hash && *k == bkt.key => {
                    ret = FoundEntry(i); false
                },
                None => { ret = FoundHole(i); false }
                _ => true,
            }
        });
        ret
    }

    /// Expand the capacity of the array to the next power of two
    /// and re-insert each of the existing buckets.
    #[inline]
    fn expand(&mut self) {
        let new_capacity = self.buckets.len() * 2;
        self.resize(new_capacity);
    }

    /// Expands the capacity of the array and re-insert each of the
    /// existing buckets.
    fn resize(&mut self, new_capacity: uint) {
        self.resize_at = resize_at(new_capacity);

        let mut xs = Vec::with_capacity(new_capacity);
        let mut i = 0;
        while i < new_capacity {
            xs.push(None);
            i += 1;
        }

        let mut old_buckets = replace(&mut self.buckets, xs);

        self.size = 0;
        while !old_buckets.is_empty() {
            self.insert_opt_bucket(old_buckets.pop().get());
        }
    }

    fn insert_opt_bucket(&mut self, bucket: Option<Bucket<K, V>>) {
        match bucket {
            Some(Bucket{hash: hash, key: key, value: value}) => {
                self.insert_internal(hash, key, value);
            }
            None => {}
        }
    }

    #[inline]
    fn value_for_bucket<'a>(&'a self, idx: uint) -> &'a V {
        match self.buckets.as_slice()[idx] {
            Some(ref bkt) => &bkt.value,
            None => abort()
        }
    }

    #[inline]
    fn mut_value_for_bucket<'a>(&'a mut self, idx: uint) -> &'a mut V {
        match self.buckets.as_mut_slice()[idx] {
            Some(ref mut bkt) => &mut bkt.value,
            None => abort()
        }
    }

    /// Inserts the key value pair into the buckets.
    /// Assumes that there will be a bucket.
    /// True if there was no previous entry with that key
    fn insert_internal(&mut self, hash: uint, k: K, v: V) -> Option<V> {
        match self.bucket_for_key_with_hash(hash, &k) {
            TableFull => abort(),
            FoundHole(idx) => {
                self.buckets.as_mut_slice()[idx] = Some(Bucket{hash: hash, key: k,
                                                        value: v});
                self.size += 1;
                None
            }
            FoundEntry(idx) => {
                match self.buckets.as_mut_slice()[idx] {
                    None => abort(),
                    Some(ref mut b) => {
                        b.hash = hash;
                        b.key = k;
                        Some(replace(&mut b.value, v))
                    }
                }
            }
        }
    }

    fn pop_internal(&mut self, hash: uint, k: &K) -> Option<V> {
        // Removing from an open-addressed hashtable
        // is, well, painful.  The problem is that
        // the entry may lie on the probe path for other
        // entries, so removing it would make you think that
        // those probe paths are empty.
        //
        // To address this we basically have to keep walking,
        // re-inserting entries we find until we reach an empty
        // bucket.  We know we will eventually reach one because
        // we insert one ourselves at the beginning (the removed
        // entry).
        //
        // I found this explanation elucidating:
        // http://www.maths.lse.ac.uk/Courses/MA407/del-hash.pdf
        let mut idx = match self.bucket_for_key_with_hash(hash, k) {
            TableFull | FoundHole(_) => return None,
            FoundEntry(idx) => idx
        };

        let len_buckets = self.buckets.len();
        let bucket = self.buckets.as_mut_slice()[idx].take();
        let value = bucket.map(|bucket| bucket.value);

        /* re-inserting buckets may cause changes in size, so remember
        what our new size is ahead of time before we start insertions */
        let size = self.size - 1;
        idx = self.next_bucket(idx, len_buckets);
        while self.buckets.as_slice()[idx].is_some() {
            let bucket = self.buckets.as_mut_slice()[idx].take();
            self.insert_opt_bucket(bucket);
            idx = self.next_bucket(idx, len_buckets);
        }
        self.size = size;

        value
    }
}

impl<K:Hash + Eq,V> Container for HashMap<K, V> {
    /// Return the number of elements in the map
    fn len(&self) -> uint { self.size }
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    /// Return a reference to the value corresponding to the key
    pub fn find<'a>(&'a self, k: &K) -> Option<&'a V> {
        match self.bucket_for_key(k) {
            FoundEntry(idx) => Some(self.value_for_bucket(idx)),
            TableFull | FoundHole(_) => None,
        }
    }

    /// Return a mutable reference to the value corresponding to the key
    pub fn find_mut<'a>(&'a mut self, k: &K) -> Option<&'a mut V> {
        let idx = match self.bucket_for_key(k) {
            FoundEntry(idx) => idx,
            TableFull | FoundHole(_) => return None
        };
        Some(self.mut_value_for_bucket(idx))
    }

    /// Insert a key-value pair from the map. If the key already had a value
    /// present in the map, that value is returned. Otherwise None is returned.
    pub fn swap(&mut self, k: K, v: V) -> Option<V> {
        // this could be faster.

        if self.size >= self.resize_at {
            // n.b.: We could also do this after searching, so
            // that we do not resize if this call to insert is
            // simply going to update a key in place.  My sense
            // though is that it's worse to have to search through
            // buckets to find the right spot twice than to just
            // resize in this corner case.
            self.expand();
        }

        let hash = k.hash(self.k0, self.k1) as uint;
        self.insert_internal(hash, k, v)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    pub fn pop(&mut self, k: &K) -> Option<V> {
        let hash = k.hash(self.k0, self.k1) as uint;
        self.pop_internal(hash, k)
    }

    pub fn with_capacity_and_keys(k0: u64, k1: u64, capacity: uint) -> HashMap<K, V> {
        let capacity = max(INITIAL_CAPACITY, capacity);
        let mut xs = Vec::with_capacity(capacity);
        let mut i = 0;
        while i < capacity {
            xs.push(None);
            i += 1;
        }

        HashMap {
            k0: k0, k1: k1,
            resize_at: resize_at(capacity),
            size: 0,
            buckets: xs
        }
    }

    /// Reserve space for at least `n` elements in the hash table.
    pub fn reserve_at_least(&mut self, n: uint) {
        if n > self.buckets.len() {
            let buckets = n * 4 / 3 + 1;
            self.resize(next_power_of_two(buckets));
        }
    }

    /// Modify and return the value corresponding to the key in the map, or
    /// insert and return a new value if it doesn't exist.
    pub fn mangle<'a,
                  A>(
                  &'a mut self,
                  k: K,
                  a: A,
                  not_found: |&K, A| -> V,
                  found: |&K, &mut V, A|)
                  -> &'a mut V {
        if self.size >= self.resize_at {
            // n.b.: We could also do this after searching, so
            // that we do not resize if this call to insert is
            // simply going to update a key in place.  My sense
            // though is that it's worse to have to search through
            // buckets to find the right spot twice than to just
            // resize in this corner case.
            self.expand();
        }

        let hash = k.hash(self.k0, self.k1) as uint;
        let idx = match self.bucket_for_key_with_hash(hash, &k) {
            TableFull => abort(),
            FoundEntry(idx) => { found(&k, self.mut_value_for_bucket(idx), a); idx }
            FoundHole(idx) => {
                let v = not_found(&k, a);
                self.buckets.as_mut_slice()[idx] = Some(Bucket{hash: hash, key: k, value: v});
                self.size += 1;
                idx
            }
        };

        self.mut_value_for_bucket(idx)
    }

    /// Return the value corresponding to the key in the map, or insert
    /// and return the value if it doesn't exist.
    pub fn find_or_insert<'a>(&'a mut self, k: K, v: V) -> &'a mut V {
        self.mangle(k, v, |_k, a| a, |_k,_v,_a| ())
    }

    /// Return the value corresponding to the key in the map, or create,
    /// insert, and return a new value if it doesn't exist.
    pub fn find_or_insert_with<'a>(&'a mut self, k: K, f: |&K| -> V)
                               -> &'a mut V {
        self.mangle(k, (), |k,_a| f(k), |_k,_v,_a| ())
    }

    /// Insert a key-value pair into the map if the key is not already present.
    /// Otherwise, modify the existing value for the key.
    /// Returns the new or modified value for the key.
    pub fn insert_or_update_with<'a>(
                                 &'a mut self,
                                 k: K,
                                 v: V,
                                 f: |&K, &mut V|)
                                 -> &'a mut V {
        self.mangle(k, v, |_k,a| a, |k,v,_a| f(k,v))
    }
}

fn next_power_of_two(n: uint) -> uint {
    let halfbits: uint = size_of::<uint>() * 4u;
    let mut tmp: uint = n - 1u;
    let mut shift: uint = 1u;
    while shift <= halfbits { tmp |= tmp >> shift; shift <<= 1u; }
    tmp + 1u
}

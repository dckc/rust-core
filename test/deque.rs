#[no_std];

extern crate core;

use core::container::Container;
use core::deque::Deque;
use core::fail::abort;

fn test_new() {
    let xs = Deque::<int>::new();
    if xs.len() != 0 { abort() }
    if xs.capacity() != 0 { abort() }
}

fn test_with_capacity() {
    let xs = Deque::<int>::with_capacity(50);
    if xs.len() != 0 { abort() }
    if xs.capacity() != 50 { abort() }
}

fn test_with_capacity_zero() {
    let xs = Deque::<int>::with_capacity(0);
    if xs.len() != 0 { abort() }
    if xs.capacity() != 0 { abort() }
}

fn test_reserve() {
    let mut xs = Deque::<int>::new();
    xs.reserve(10);
    if xs.len() != 0 { abort() }
    if xs.capacity() != 10 { abort() }
}

fn test_destructor() {
    let mut xs = Deque::with_capacity(10);
    xs.push_back(~10);
    xs.push_back(~2);
    xs.push_back(~5);
    xs.push_back(~11);
}

fn test_swap() {
    let mut xs = Deque::new();
    xs.push_back(3);
    xs.push_front(2);
    xs.push_back(4);
    xs.push_front(1);
    xs.push_back(5);
    if *xs.get(0).get() != 1 { abort() }
    if *xs.get(1).get() != 2 { abort() }
    if *xs.get(2).get() != 3 { abort() }
    if *xs.get(3).get() != 4 { abort() }
    if *xs.get(4).get() != 5 { abort() }
    xs.swap(2, 4);
    if *xs.get(0).get() != 1 { abort() }
    if *xs.get(1).get() != 2 { abort() }
    if *xs.get(2).get() != 5 { abort() }
    if *xs.get(3).get() != 4 { abort() }
    if *xs.get(4).get() != 3 { abort() }
    xs.swap(3, 2);
    if *xs.get(0).get() != 1 { abort() }
    if *xs.get(1).get() != 2 { abort() }
    if *xs.get(2).get() != 4 { abort() }
    if *xs.get(3).get() != 5 { abort() }
    if *xs.get(4).get() != 3 { abort() }
}

#[start]
fn main(_: int, _: **u8) -> int {
    test_new();
    test_with_capacity();
    test_with_capacity_zero();
    test_reserve();
    test_destructor();
    test_swap();
    0
}

#[no_std];

extern crate core;

use core::container::Container;
use core::fail::abort;
use core::priority_queue::PriorityQueue;

fn test_new() {
    let xs = PriorityQueue::<int>::new();
    if xs.len() != 0 { abort() }
    if xs.capacity() != 0 { abort() }
}

fn test_with_capacity() {
    let xs = PriorityQueue::<int>::with_capacity(50);
    if xs.len() != 0 { abort() }
    if xs.capacity() != 50 { abort() }
}

fn test_with_capacity_zero() {
    let xs = PriorityQueue::<int>::with_capacity(0);
    if xs.len() != 0 { abort() }
    if xs.capacity() != 0 { abort() }
}

fn test_reserve() {
    let mut xs = PriorityQueue::<int>::new();
    xs.reserve(10);
    if xs.len() != 0 { abort() }
    if xs.capacity() != 10 { abort() }
}

fn test_destructor() {
    let mut xs = PriorityQueue::with_capacity(10);
    xs.push(~10);
    xs.push(~2);
    xs.push(~5);
    xs.push(~11);
}

fn test_push_top_pop() {
    let mut heap = PriorityQueue::new();

    heap.push(2);
    if heap.len() != 1 { abort() }
    if *heap.top().get() != 2 { abort() }

    heap.push(4);
    if heap.len() != 2 { abort() }
    if *heap.top().get() != 4 { abort() }

    heap.push(9);
    if heap.len() != 3 { abort() }
    if *heap.top().get() != 9 { abort() }

    heap.push(11);
    if heap.len() != 4 { abort() }
    if *heap.top().get() != 11 { abort() }

    heap.push(5);
    if heap.len() != 5 { abort() }
    if *heap.top().get() != 11 { abort() }

    heap.push(27);
    if heap.len() != 6 { abort() }
    if *heap.top().get() != 27 { abort() }

    heap.push(3);
    if heap.len() != 7 { abort() }
    if *heap.top().get() != 27 { abort() }

    heap.push(103);
    if heap.len() != 8 { abort() }
    if *heap.top().get() != 103 { abort() }

    if heap.pop().get() != 103 { abort() }
    if heap.pop().get() != 27 { abort() }
    if heap.pop().get() != 11 { abort() }
    if heap.pop().get() != 9 { abort() }
    if heap.pop().get() != 5 { abort() }
    if heap.pop().get() != 4 { abort() }
    if heap.pop().get() != 3 { abort() }
}

#[start]
fn main(_: int, _: **u8) -> int {
    test_new();
    test_with_capacity();
    test_with_capacity_zero();
    test_reserve();
    test_destructor();
    test_push_top_pop();
    0
}

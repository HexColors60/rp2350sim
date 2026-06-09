#![allow(dead_code)]
//! Ring buffer for trace data.

/// Ring buffer.
#[derive(Debug)]
pub struct RingBuffer<T> {
    data: Vec<T>,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![T::default(); capacity],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.data[self.head] = item;
        self.head = (self.head + 1) % self.data.len();
        if self.len < self.data.len() {
            self.len += 1;
        } else {
            self.tail = (self.tail + 1) % self.data.len();
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            let item = self.data[self.tail].clone();
            self.tail = (self.tail + 1) % self.data.len();
            self.len -= 1;
            Some(item)
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
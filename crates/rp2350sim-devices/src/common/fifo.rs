//! FIFO implementation.

use std::collections::VecDeque;

/// Generic FIFO.
#[derive(Debug)]
pub struct Fifo<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T> Fifo<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) -> bool {
        if self.data.len() < self.capacity {
            self.data.push_back(item);
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.front()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.data.len() >= self.capacity
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}
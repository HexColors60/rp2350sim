//! Event queue implementation.

use super::{Event, EventId};
use std::collections::BinaryHeap;

/// Wrapper for events to implement reverse priority ordering.
#[derive(Debug, Clone)]
struct OrderedEvent {
    event: Event,
}

impl PartialEq for OrderedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.tick == other.event.tick && self.event.priority == other.event.priority
    }
}

impl Eq for OrderedEvent {}

impl PartialOrd for OrderedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sort by tick first (earlier = higher priority)
        // Then by priority (lower value = higher priority)
        match self.event.tick.cmp(&other.event.tick) {
            std::cmp::Ordering::Equal => other.event.priority.0.cmp(&self.event.priority.0),
            other => other.reverse(),
        }
    }
}

/// Event queue for scheduling simulation events.
#[derive(Debug, Default)]
pub struct EventQueue {
    events: BinaryHeap<OrderedEvent>,
    next_id: u64,
    current_tick: u64,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: BinaryHeap::new(),
            next_id: 0,
            current_tick: 0,
        }
    }

    /// Schedule a new event.
    pub fn schedule(&mut self, mut event: Event) -> EventId {
        event.id = EventId::new(self.next_id);
        self.next_id += 1;
        self.events.push(OrderedEvent { event });
        EventId::new(self.next_id - 1)
    }

    /// Schedule an event at a specific tick.
    pub fn schedule_at(&mut self, tick: u64, event: super::EventKind) -> EventId {
        let event = Event::new(EventId::new(0), event, tick);
        self.schedule(event)
    }

    /// Schedule an event after a delay.
    pub fn schedule_after(&mut self, delay: u64, event: super::EventKind) -> EventId {
        let tick = self.current_tick + delay;
        self.schedule_at(tick, event)
    }

    /// Get the next event without removing it.
    pub fn peek(&self) -> Option<&Event> {
        self.events.peek().map(|e| &e.event)
    }

    /// Pop the next event.
    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop().map(|e| e.event)
    }

    /// Pop the next event if it's due at or before the given tick.
    pub fn pop_if_due(&mut self, tick: u64) -> Option<Event> {
        if let Some(event) = self.peek() {
            if event.tick <= tick {
                return self.pop();
            }
        }
        None
    }

    /// Get the tick of the next event.
    pub fn next_tick(&self) -> Option<u64> {
        self.peek().map(|e| e.tick)
    }

    /// Get the current tick.
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Advance to a specific tick.
    pub fn advance_to(&mut self, tick: u64) {
        self.current_tick = tick;
    }

    /// Advance by a number of ticks.
    pub fn advance_by(&mut self, ticks: u64) {
        self.current_tick += ticks;
    }

    /// Get the number of pending events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Cancel an event by ID.
    pub fn cancel(&mut self, id: EventId) -> bool {
        let before = self.events.len();
        self.events.retain(|e| e.event.id != id);
        self.events.len() != before
    }

    /// Process all due events up to the current tick.
    pub fn process_due<F>(&mut self, mut handler: F)
    where
        F: FnMut(&Event),
    {
        while let Some(event) = self.pop_if_due(self.current_tick) {
            handler(&event);

            // Reschedule recurring events
            if event.recurring {
                if let Some(interval) = event.interval {
                    let mut new_event = event.clone();
                    new_event.tick = self.current_tick + interval;
                    self.schedule(new_event);
                }
            }
        }
    }
}
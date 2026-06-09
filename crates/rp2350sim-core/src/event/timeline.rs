//! Timeline for event visualization and navigation.

use super::Event;
use std::collections::VecDeque;

/// Maximum events stored in the timeline.
const MAX_TIMELINE_EVENTS: usize = 100_000;

/// Timeline event record.
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub tick: u64,
    pub event: Event,
}

/// Timeline for recording and visualizing events.
#[derive(Debug, Default)]
pub struct Timeline {
    entries: VecDeque<TimelineEntry>,
    max_entries: usize,
    start_tick: u64,
    end_tick: u64,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(1024),
            max_entries: MAX_TIMELINE_EVENTS,
            start_tick: 0,
            end_tick: 0,
        }
    }

    /// Record an event in the timeline.
    pub fn record(&mut self, event: Event) {
        if self.entries.is_empty() {
            self.start_tick = event.tick;
        }
        self.end_tick = event.tick;

        // Remove oldest entries if at capacity
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
            if let Some(first) = self.entries.front() {
                self.start_tick = first.tick;
            }
        }

        self.entries.push_back(TimelineEntry {
            tick: event.tick,
            event,
        });
    }

    /// Get entries in a tick range.
    pub fn get_range(&self, start: u64, end: u64) -> impl Iterator<Item = &TimelineEntry> {
        self.entries
            .iter()
            .filter(move |e| e.tick >= start && e.tick <= end)
    }

    /// Get all entries.
    pub fn entries(&self) -> &VecDeque<TimelineEntry> {
        &self.entries
    }

    /// Get the start tick.
    pub fn start_tick(&self) -> u64 {
        self.start_tick
    }

    /// Get the end tick.
    pub fn end_tick(&self) -> u64 {
        self.end_tick
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the timeline is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear the timeline.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.start_tick = 0;
        self.end_tick = 0;
    }

    /// Set the maximum number of entries.
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }
}
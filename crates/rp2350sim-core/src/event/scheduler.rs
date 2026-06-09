//! Simulation scheduler.

use super::{Event, EventKind, EventQueue};
use crate::Ticks;

/// Scheduler callback trait.
pub trait ScheduleCallback: Send + Sync {
    fn call(&self, event: &Event);
}

impl<F: Fn(&Event) + Send + Sync> ScheduleCallback for F {
    fn call(&self, event: &Event) {
        self(event)
    }
}

/// Simulation scheduler.
pub struct Scheduler {
    queue: EventQueue,
    callbacks: Vec<Box<dyn ScheduleCallback>>,
    paused: bool,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            queue: EventQueue::new(),
            callbacks: Vec::new(),
            paused: false,
        }
    }

    /// Register a callback for events.
    pub fn register_callback<F>(&mut self, callback: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    /// Schedule an event.
    pub fn schedule(&mut self, event: Event) {
        self.queue.schedule(event);
    }

    /// Schedule an event at a specific tick.
    pub fn schedule_at(&mut self, tick: u64, kind: EventKind) {
        self.queue.schedule_at(tick, kind);
    }

    /// Schedule an event after a delay.
    pub fn schedule_after(&mut self, delay: u64, kind: EventKind) {
        self.queue.schedule_after(delay, kind);
    }

    /// Run the simulation for a number of ticks.
    pub fn run_for(&mut self, ticks: u64) {
        if self.paused {
            return;
        }

        let target_tick = self.queue.current_tick() + ticks;
        self.queue.advance_to(target_tick);

        self.process_events();
    }

    /// Run until a specific tick.
    pub fn run_until(&mut self, tick: u64) {
        if self.paused {
            return;
        }

        self.queue.advance_to(tick);
        self.process_events();
    }

    /// Process all due events.
    fn process_events(&mut self) {
        let callbacks: Vec<_> = self.callbacks.iter().collect();

        self.queue.process_due(|event| {
            for callback in &callbacks {
                callback.call(event);
            }
        });
    }

    /// Step one event.
    pub fn step(&mut self) -> Option<Event> {
        if self.paused {
            return None;
        }

        let event = self.queue.pop();
        if let Some(ref e) = event {
            self.queue.advance_to(e.tick);

            for callback in &self.callbacks {
                callback.call(e);
            }
        }
        event
    }

    /// Get the current tick.
    pub fn current_tick(&self) -> u64 {
        self.queue.current_tick()
    }

    /// Get the current tick as Ticks.
    pub fn ticks(&self) -> Ticks {
        Ticks::new(self.queue.current_tick())
    }

    /// Pause the scheduler.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the scheduler.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Check if the scheduler is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Reset the scheduler.
    pub fn reset(&mut self) {
        self.queue.clear();
        self.paused = false;
    }

    /// Get the number of pending events.
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }
}
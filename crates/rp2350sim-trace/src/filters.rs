#![allow(dead_code)]
//! Trace filters.

use rp2350sim_core::traits::TraceEvent;

/// Trace filter trait.
pub trait TraceFilter: Send + Sync {
    fn should_trace(&self, event: &TraceEvent) -> bool;
}

/// Accept all filter.
pub struct AcceptAllFilter;

impl TraceFilter for AcceptAllFilter {
    fn should_trace(&self, _event: &TraceEvent) -> bool {
        true
    }
}
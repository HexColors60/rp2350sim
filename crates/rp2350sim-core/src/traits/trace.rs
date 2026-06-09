//! Trace trait.

use crate::Result;

/// Trace event.
#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub tick: u64,
    pub kind: TraceEventKind,
}

/// Trace event kinds.
#[derive(Debug, Clone)]
pub enum TraceEventKind {
    CpuInstruction {
        core: u8,
        pc: u32,
        opcode: u32,
    },
    MemoryRead {
        addr: u32,
        value: u64,
        width: u8,
    },
    MemoryWrite {
        addr: u32,
        value: u64,
        width: u8,
    },
    PeripheralRead {
        device: u16,
        offset: u32,
        value: u32,
    },
    PeripheralWrite {
        device: u16,
        offset: u32,
        value: u32,
    },
    Interrupt {
        irq: u8,
        active: bool,
    },
    GpioChange {
        pin: u8,
        value: bool,
    },
}

/// Trace sink trait.
pub trait TraceSink: Send + Sync {
    fn write(&mut self, event: TraceEvent) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

/// Trace filter trait.
pub trait TraceFilter: Send + Sync {
    fn should_trace(&self, event: &TraceEvent) -> bool;
}
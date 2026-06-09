//! Breakpoint management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Breakpoint kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BreakpointKind {
    /// PC breakpoint
    Pc,
    /// Symbol breakpoint
    Symbol,
    /// Memory read breakpoint
    MemoryRead,
    /// Memory write breakpoint
    MemoryWrite,
    /// MMIO access breakpoint
    Mmio,
    /// IRQ breakpoint
    Irq,
    /// GPIO edge breakpoint
    GpioEdge,
    /// PIO instruction breakpoint
    PioInstruction,
}

/// Breakpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Breakpoint ID
    pub id: u64,
    /// Breakpoint kind
    pub kind: BreakpointKind,
    /// Address (for PC/memory breakpoints)
    pub address: Option<u32>,
    /// Symbol name (for symbol breakpoints)
    pub symbol: Option<String>,
    /// Size (for memory breakpoints)
    pub size: Option<u32>,
    /// IRQ number (for IRQ breakpoints)
    pub irq: Option<u8>,
    /// GPIO pin (for GPIO breakpoints)
    pub gpio: Option<u8>,
    /// Whether the breakpoint is enabled
    pub enabled: bool,
    /// Hit count
    pub hit_count: u64,
    /// Ignore count
    pub ignore_count: u64,
    /// Condition expression
    pub condition: Option<String>,
}

impl Breakpoint {
    pub fn pc(address: u32) -> Self {
        Self {
            id: 0,
            kind: BreakpointKind::Pc,
            address: Some(address),
            symbol: None,
            size: None,
            irq: None,
            gpio: None,
            enabled: true,
            hit_count: 0,
            ignore_count: 0,
            condition: None,
        }
    }

    pub fn symbol(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            kind: BreakpointKind::Symbol,
            address: None,
            symbol: Some(name.into()),
            size: None,
            irq: None,
            gpio: None,
            enabled: true,
            hit_count: 0,
            ignore_count: 0,
            condition: None,
        }
    }

    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    pub fn with_ignore(mut self, count: u64) -> Self {
        self.ignore_count = count;
        self
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Breakpoint manager.
#[derive(Debug, Default)]
pub struct BreakpointManager {
    breakpoints: HashMap<u64, Breakpoint>,
    next_id: u64,
    pc_breakpoints: HashMap<u32, u64>,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, mut bp: Breakpoint) -> u64 {
        bp.id = self.next_id;
        self.next_id += 1;
        let id = bp.id;

        if bp.kind == BreakpointKind::Pc {
            if let Some(addr) = bp.address {
                self.pc_breakpoints.insert(addr, id);
            }
        }

        self.breakpoints.insert(id, bp);
        id
    }

    pub fn remove(&mut self, id: u64) -> Option<Breakpoint> {
        let bp = self.breakpoints.remove(&id)?;
        if bp.kind == BreakpointKind::Pc {
            if let Some(addr) = bp.address {
                self.pc_breakpoints.remove(&addr);
            }
        }
        Some(bp)
    }

    pub fn get(&self, id: u64) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Breakpoint> {
        self.breakpoints.get_mut(&id)
    }

    pub fn check_pc(&mut self, pc: u32) -> Option<&Breakpoint> {
        let id = *self.pc_breakpoints.get(&pc)?;
        let bp = self.breakpoints.get_mut(&id)?;
        if !bp.enabled {
            return None;
        }
        bp.hit_count += 1;
        if bp.hit_count <= bp.ignore_count {
            return None;
        }
        Some(self.breakpoints.get(&id).unwrap())
    }

    pub fn breakpoints(&self) -> impl Iterator<Item = &Breakpoint> {
        self.breakpoints.values()
    }

    pub fn clear(&mut self) {
        self.breakpoints.clear();
        self.pc_breakpoints.clear();
        self.next_id = 0;
    }
}
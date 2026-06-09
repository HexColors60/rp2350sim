//! Core type definitions.

use serde::{Deserialize, Serialize};

/// CPU architecture type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CpuArch {
    /// ARM Cortex-M33
    Arm,
    /// Hazard3 RISC-V
    Hazard3,
}

impl Default for CpuArch {
    fn default() -> Self {
        Self::Arm
    }
}

/// Execution mode for the simulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Fast functional mode - focus on correctness
    Fast,
    /// Debug mode - single-step, breakpoints, full visibility
    Debug,
    /// Peripheral timing mode - simulated bus/peripheral delays
    PeripheralTiming,
    /// Instruction trace mode - log every instruction
    Trace,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Fast
    }
}

/// Boot source configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BootSource {
    /// Boot from flash/XIP
    Flash,
    /// Boot from SRAM
    Sram,
    /// Boot from USB (drag-and-drop)
    Usb,
}

impl Default for BootSource {
    fn default() -> Self {
        Self::Flash
    }
}

/// Simulator run state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunState {
    /// Simulation is halted
    #[default]
    Halted,
    /// Simulation is running
    Running,
    /// Simulation is paused (e.g., at breakpoint)
    Paused,
    /// Simulation hit a breakpoint
    Breakpoint,
    /// Simulation encountered an error
    Error,
    /// Simulation completed
    Complete,
}

/// Access permissions for memory regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permissions {
    pub const fn none() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
        }
    }

    pub const fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
        }
    }

    pub const fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            execute: false,
        }
    }

    pub const fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
        }
    }

    pub const fn read_execute() -> Self {
        Self {
            read: true,
            write: false,
            execute: true,
        }
    }

    pub const fn all() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }
}

/// Memory access width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessWidth {
    Byte = 1,
    HalfWord = 2,
    Word = 4,
    DoubleWord = 8,
}

impl AccessWidth {
    pub const fn bytes(&self) -> usize {
        *self as usize
    }
}

/// Memory access type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessType {
    Read,
    Write,
    Execute,
}

/// Result of a CPU step operation.
#[derive(Debug, Clone, Default)]
pub struct CpuStepResult {
    /// Number of cycles executed
    pub cycles: u64,
    /// Whether a breakpoint was hit
    pub breakpoint_hit: bool,
    /// Whether an exception occurred
    pub exception: Option<ExceptionInfo>,
    /// Whether the CPU is now halted
    pub halted: bool,
}

/// Exception information.
#[derive(Debug, Clone)]
pub struct ExceptionInfo {
    /// Exception type
    pub kind: ExceptionKind,
    /// Exception code
    pub code: u32,
    /// PC where exception occurred
    pub pc: u32,
}

/// Exception types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionKind {
    /// Reset
    Reset,
    /// Non-maskable interrupt
    Nmi,
    /// Hard fault
    HardFault,
    /// Memory management fault
    MemManage,
    /// Bus fault
    BusFault,
    /// Usage fault
    UsageFault,
    /// Supervisor call
    SVCall,
    /// Debug monitor
    DebugMonitor,
    /// Pending supervisor call
    PendSV,
    /// System tick
    SysTick,
    /// External interrupt
    Interrupt,
    /// Unknown exception
    Unknown,
}

/// Simulator configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    /// CPU architecture to use
    pub cpu_arch: CpuArch,
    /// Number of CPU cores
    pub core_count: usize,
    /// Clock frequency in Hz
    pub clock_hz: u64,
    /// Execution mode
    pub execution_mode: ExecutionMode,
    /// Boot source
    pub boot_source: BootSource,
    /// Enable tracing
    pub trace_enable: bool,
    /// Enable USB simulation
    pub usb_enable: bool,
    /// Enable WLAN stub
    pub wlan_stub: bool,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            cpu_arch: CpuArch::Arm,
            core_count: 2,
            clock_hz: 150_000_000,
            execution_mode: ExecutionMode::Fast,
            boot_source: BootSource::Flash,
            trace_enable: false,
            usb_enable: true,
            wlan_stub: true,
        }
    }
}
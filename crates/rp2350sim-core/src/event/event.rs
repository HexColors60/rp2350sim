//! Event types for the simulation scheduler.

use crate::ids::{DeviceId, IrqId, PioSmId};
use serde::{Deserialize, Serialize};

/// Unique event identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl EventId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Event priority (lower = higher priority).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventPriority(pub u8);

impl EventPriority {
    pub const HIGHEST: Self = Self(0);
    pub const HIGH: Self = Self(64);
    pub const NORMAL: Self = Self(128);
    pub const LOW: Self = Self(192);
    pub const LOWEST: Self = Self(255);
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Event kinds in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    /// Timer tick event
    TimerTick {
        timer_id: u8,
    },
    /// Interrupt request
    Irq {
        irq: IrqId,
        level: bool,
    },
    /// GPIO state change
    GpioChange {
        pin: u8,
        old_value: bool,
        new_value: bool,
    },
    /// UART data available
    UartRx {
        uart_id: u8,
        byte: u8,
    },
    /// UART transmit complete
    UartTxComplete {
        uart_id: u8,
    },
    /// SPI transfer complete
    SpiTransferComplete {
        spi_id: u8,
    },
    /// I2C transaction
    I2cTransaction {
        i2c_id: u8,
        address: u8,
        read: bool,
    },
    /// ADC sample ready
    AdcSampleReady {
        channel: u8,
        value: u16,
    },
    /// PWM wrap
    PwmWrap {
        slice: u8,
    },
    /// PIO state machine event
    PioSmEvent {
        sm: PioSmId,
        event: PioEvent,
    },
    /// USB event
    UsbEvent {
        endpoint: u8,
        event: UsbEventKind,
    },
    /// Watchdog timeout
    WatchdogTimeout,
    /// Device-specific event
    Device {
        device: DeviceId,
        event_id: u32,
        data: Vec<u8>,
    },
    /// Checkpoint event
    Checkpoint {
        name: String,
    },
    /// Custom event
    Custom {
        name: String,
        data: Vec<u8>,
    },
}

/// PIO event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PioEvent {
    /// FIFO not empty
    FifoNotEmpty,
    /// FIFO full
    FifoFull,
    /// Instruction executed
    InstructionExecuted,
    /// Stall
    Stall,
    /// IRQ set
    IrqSet(u8),
}

/// USB event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbEventKind {
    /// Reset received
    Reset,
    /// Setup packet received
    Setup,
    /// Data received
    DataReceived,
    /// Data sent
    DataSent,
    /// Suspend
    Suspend,
    /// Resume
    Resume,
}

/// A scheduled event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event identifier
    pub id: EventId,
    /// Event kind
    pub kind: EventKind,
    /// Scheduled tick
    pub tick: u64,
    /// Priority
    pub priority: EventPriority,
    /// Whether this event is recurring
    pub recurring: bool,
    /// Recurrence interval (in ticks), if recurring
    pub interval: Option<u64>,
}

impl Event {
    pub fn new(id: EventId, kind: EventKind, tick: u64) -> Self {
        Self {
            id,
            kind,
            tick,
            priority: EventPriority::NORMAL,
            recurring: false,
            interval: None,
        }
    }

    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn recurring(mut self, interval: u64) -> Self {
        self.recurring = true;
        self.interval = Some(interval);
        self
    }
}
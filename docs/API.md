# RP2350 Simulator API Documentation

This document describes the main APIs for using the RP2350 simulator.

## Table of Contents

1. [Core Types](#core-types)
2. [SoC Interface](#soc-interface)
3. [CPU Interface](#cpu-interface)
4. [Memory Interface](#memory-interface)
5. [Peripheral Interface](#peripheral-interface)
6. [Debugger Interface](#debugger-interface)
7. [Tracing Interface](#tracing-interface)

---

## Core Types

### DeviceId

Unique identifier for each device/peripheral.

```rust
pub struct DeviceId(u16);

impl DeviceId {
    pub const GPIO: Self = Self(0);
    pub const UART: Self = Self(1);
    pub const SPI: Self = Self(2);
    pub const I2C: Self = Self(3);
    pub const PWM: Self = Self(4);
    pub const ADC: Self = Self(5);
    pub const USB: Self = Self(6);
    pub const PIO: Self = Self(7);
    pub const TIMER: Self = Self(8);
    pub const WATCHDOG: Self = Self(9);
    pub const CLOCKS: Self = Self(10);
    pub const RESETS: Self = Self(11);
    pub const WLAN: Self = Self(12);
    pub const DMA: Self = Self(13);
    pub const XIP: Self = Self(14);
    pub const I2S: Self = Self(15);
    pub const RTC: Self = Self(16);
    pub const PLL: Self = Self(17);
    pub const SHA256: Self = Self(18);
    pub const TRNG: Self = Self(19);
}
```

### CoreId

Identifier for CPU cores.

```rust
pub struct CoreId(u8);

impl CoreId {
    pub const CORE0: Self = Self(0);
    pub const CORE1: Self = Self(1);
}
```

### Result

Standard result type for simulator operations.

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

### AccessType

Memory access type.

```rust
pub enum AccessType {
    Read,
    Write,
    Execute,
}
```

### AccessWidth

Memory access width.

```rust
pub enum AccessWidth {
    Byte = 0,
    Halfword = 1,
    Word = 2,
}
```

---

## SoC Interface

### Creating a SoC Instance

```rust
use rp2350sim_soc::Soc;
use rp2350sim_core::CoreId;

// Create default SoC
let mut soc = Soc::new();

// Create with custom configuration
let config = SocConfig {
    cpu_type: CpuType::Arm,
    sram_size: 0x80000,
    flash_size: 0x400000,
    ..Default::default()
};
let mut soc = Soc::with_config(config);
```

### Loading Firmware

```rust
// Load ELF file
soc.load_elf("firmware.elf")?;

// Load binary at address
soc.load_binary("firmware.bin", 0x10000000)?;

// Load HEX file
soc.load_hex("firmware.hex")?;
```

### Running Simulation

```rust
// Step single instruction
soc.step_instruction(CoreId::CORE0);

// Step multiple cycles
soc.step(1000);

// Run until breakpoint
soc.run_until_breakpoint()?;

// Run indefinitely (with callback)
soc.run(|state| {
    // Check for exit condition
    if state.cycles > 1_000_000 {
        false  // Stop
    } else {
        true   // Continue
    }
});
```

### Reset

```rust
// Full reset
soc.reset();

// Reset specific core
soc.reset_core(CoreId::CORE0);
```

---

## CPU Interface

### Register Access

```rust
// Get program counter
let pc = soc.cpu_get_pc(CoreId::CORE0);

// Set program counter
soc.cpu_set_pc(CoreId::CORE0, 0x10000000);

// Get general purpose register (ARM)
let r0 = soc.cpu_get_reg(CoreId::CORE0, 0);

// Get general purpose register (RISC-V)
let x1 = soc.cpu_get_reg(CoreId::CORE0, 1);

// Get special registers
let sp = soc.cpu_get_sp(CoreId::CORE0);
let lr = soc.cpu_get_lr(CoreId::CORE0);
```

### CPU State

```rust
// Check if CPU is halted
let halted = soc.cpu_is_halted(CoreId::CORE0);

// Get CPU mode (ARM)
let mode = soc.cpu_get_mode(CoreId::CORE0);

// Get current architecture
let arch = soc.cpu_get_arch(CoreId::CORE0);  // Arm or RiscV
```

---

## Memory Interface

### Memory Read/Write

```rust
// Read byte
let byte = soc.memory_read_byte(0x20000000);

// Read halfword
let half = soc.memory_read_halfword(0x20000000);

// Read word
let word = soc.memory_read_word(0x20000000);

// Write byte
soc.memory_write_byte(0x20000000, 0x42);

// Write halfword
soc.memory_write_halfword(0x20000000, 0x1234);

// Write word
soc.memory_write_word(0x20000000, 0xDEADBEEF);
```

### Memory Regions

```rust
// Get memory region info
let region = soc.memory_get_region(0x20000000);
println!("Region: {:?}, size: {}", region.kind, region.size);

// Check if address is valid
let valid = soc.memory_is_valid(0x20000000);
```

---

## Peripheral Interface

### GPIO

```rust
// Set pin direction
soc.gpio_set_dir(0, true);  // Output
soc.gpio_set_dir(1, false); // Input

// Set pin value
soc.gpio_set(0, true);
soc.gpio_set(0, false);

// Get pin value
let value = soc.gpio_get(0);

// Set pin function
soc.gpio_set_function(0, 1);  // Alternate function 1

// Set pull resistors
soc.gpio_set_pullup(0, true);
soc.gpio_set_pulldown(0, false);
```

### UART

```rust
// Set baud rate
soc.uart_set_baud(0, 115200);

// Write byte
soc.uart_write(0, b'A');

// Write string
soc.uart_write_string(0, "Hello, World!");

// Read byte (non-blocking)
if let Some(byte) = soc.uart_read(0) {
    println!("Received: {}", byte as char);
}

// Check TX/RX status
let tx_full = soc.uart_tx_full(0);
let rx_empty = soc.uart_rx_empty(0);
```

### SPI

```rust
// Configure SPI
soc.spi_set_clock(0, 1_000_000);  // 1 MHz
soc.spi_set_mode(0, 0, 0);  // CPOL=0, CPHA=0

// Transfer byte
let rx_byte = soc.spi_transfer(0, tx_byte);

// Transfer buffer
let rx_buf = soc.spi_transfer_buffer(0, &tx_buf);
```

### I2C

```rust
// Configure I2C
soc.i2c_set_clock(0, 100_000);  // 100 kHz

// Write to device
soc.i2c_write(0, 0x50, &[0x00, 0x01, 0x02])?;

// Read from device
let data = soc.i2c_read(0, 0x50, 4)?;
```

### Timer

```rust
// Get timer value
let time = soc.timer_get();

// Set alarm
soc.timer_set_alarm(0, time + 1_000_000);  // 1 second

// Check alarm
let triggered = soc.timer_alarm_triggered(0);
```

### DMA

```rust
// Configure DMA channel
soc.dma_set_read_addr(0, 0x20000000);
soc.dma_set_write_addr(0, 0x20001000);
soc.dma_set_count(0, 256);
soc.dma_set_data_size(0, 2);  // 32-bit

// Start transfer
soc.dma_start(0);

// Check status
let busy = soc.dma_is_busy(0);
```

---

## Debugger Interface

### Breakpoints

```rust
// Set breakpoint
let bp_id = soc.debugger_add_breakpoint(0x10000100)?;

// Remove breakpoint
soc.debugger_remove_breakpoint(bp_id)?;

// List breakpoints
let breakpoints = soc.debugger_list_breakpoints();
```

### Watchpoints

```rust
// Add read watchpoint
soc.debugger_add_read_watchpoint(0x20000000)?;

// Add write watchpoint
soc.debugger_add_write_watchpoint(0x20000004)?;

// Check if watchpoint triggered
let triggered = soc.debugger_watchpoint_triggered();
```

### Single Stepping

```rust
// Step single instruction
soc.debugger_step(CoreId::CORE0);

// Step over (skip calls)
soc.debugger_step_over(CoreId::CORE0)?;

// Step out (return from function)
soc.debugger_step_out(CoreId::CORE0)?;
```

### Disassembly

```rust
// Disassemble at address
let disasm = soc.debugger_disasm(0x10000100, 10);  // 10 instructions
for inst in disasm {
    println!("{:08X}: {}", inst.address, inst.text);
}
```

---

## Tracing Interface

### Enable Tracing

```rust
// Enable instruction tracing
soc.trace_enable(TraceKind::Instruction);

// Enable memory access tracing
soc.trace_enable(TraceKind::Memory);

// Enable GPIO tracing
soc.trace_enable(TraceKind::Gpio);

// Enable MMIO tracing
soc.trace_enable(TraceKind::Mmio);
```

### Trace Events

```rust
// Get trace events
let events = soc.trace_get_events();
for event in events {
    match event {
        TraceEvent::Instruction { core, pc, opcode } => {
            println!("Core {} @ {:08X}: {:04X}", core, pc, opcode);
        }
        TraceEvent::Memory { addr, value, is_write, width } => {
            println!("Memory {:08X} = {:08X} ({})", addr, value, 
                if is_write { "W" } else { "R" });
        }
        TraceEvent::Gpio { pin, value } => {
            println!("GPIO {} = {}", pin, value);
        }
        _ => {}
    }
}

// Clear trace events
soc.trace_clear();
```

### Trace Filters

```rust
// Filter by address range
soc.trace_set_filter(TraceFilter::AddressRange(0x10000000, 0x10010000));

// Filter by peripheral
soc.trace_set_filter(TraceFilter::Peripheral(DeviceId::UART));

// Clear filter
soc.trace_clear_filter();
```

---

## Save/Load State

```rust
// Save state to file
soc.save_state("state.bin")?;

// Load state from file
soc.load_state("state.bin")?;

// Get state as bytes
let state_bytes = soc.get_state_bytes()?;

// Restore state from bytes
soc.restore_state_bytes(&state_bytes)?;
```

---

## Error Handling

```rust
use rp2350sim_core::Error;

match soc.load_elf("firmware.elf") {
    Ok(()) => println!("Loaded successfully"),
    Err(Error::FileNotFound(path)) => {
        eprintln!("File not found: {}", path);
    }
    Err(Error::InvalidFormat(msg)) => {
        eprintln!("Invalid format: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```

---

## Thread Safety

The simulator is designed for single-threaded use. For multi-threaded scenarios:

```rust
use std::sync::{Arc, Mutex};

let soc = Arc::new(Mutex::new(Soc::new()));

// In thread 1
{
    let mut s = soc.lock().unwrap();
    s.step(1000);
}

// In thread 2
{
    let s = soc.lock().unwrap();
    if let Some(byte) = s.uart_read(0) {
        println!("UART: {}", byte as char);
    }
}
```
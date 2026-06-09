# RP2350 Simulator Architecture

This document describes the internal architecture of the RP2350 simulator.

## Overview

The simulator is organized as a collection of Rust crates, each responsible for a specific aspect of the simulation.

```
┌─────────────────────────────────────────────────────────────────┐
│                        rp2350sim-app                             │
│  (Main application, CLI, GUI integration)                       │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        rp2350sim-soc                             │
│  (SoC composition, device wiring, top-level API)                │
└─────────────────────────────────────────────────────────────────┘
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐     ┌─────────────────┐     ┌───────────────────┐
│ rp2350sim-bus │     │ rp2350sim-cpu-* │     │ rp2350sim-devices │
│ (Bus, Memory  │     │ (ARM, RISC-V)   │     │ (Peripherals)     │
│  Map)         │     │                 │     │                   │
└───────────────┘     └─────────────────┘     └───────────────────┘
        │                       │                       │
        ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                        rp2350sim-core                            │
│  (Core types, traits, utilities)                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Core Crates

### rp2350sim-core

Provides fundamental types and traits used throughout the simulator.

**Key Types:**
- `DeviceId` - Unique peripheral identifier
- `CoreId` - CPU core identifier
- `Result<T>` - Standard result type
- `Error` - Error enumeration

**Key Traits:**
- `Device` - Peripheral device interface
- `Bus` - Memory bus interface
- `Cpu` - CPU interface

### rp2350sim-bus

Implements the memory bus and address decoding.

**Components:**
- `Bus` - Main bus implementation
- `MemoryMap` - Address to device mapping
- `Region` - Memory region descriptor

**Memory Map:**
```
0x00000000 - 0x0FFFFFFF  Reserved
0x10000000 - 0x13FFFFFF  XIP ROM (Flash)
0x15000000 - 0x1500FFFF  XIP RAM
0x20000000 - 0x200FFFFF  SRAM (4 banks)
0x40000000 - 0x4FFFFFFF  Peripherals
0x50000000 - 0x5FFFFFFF  APB Peripherals
0xD0000000 - 0xDFFFFFFF  CoreSight Debug
0xE0000000 - 0xE00FFFFF  PPB (ARM)
```

### rp2350sim-mem

Implements memory components.

**Components:**
- `Ram` - Random access memory
- `Rom` - Read-only memory
- `Flash` - Non-volatile memory
- `Loader` - ELF/HEX file loading

## CPU Crates

### rp2350sim-cpu-common

Defines the common CPU interface.

**Key Traits:**
```rust
pub trait Cpu {
    fn step(&mut self) -> Result<()>;
    fn reset(&mut self);
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, pc: u32);
    fn get_reg(&self, reg: u8) -> u32;
    fn set_reg(&mut self, reg: u8, value: u32);
    fn is_halted(&self) -> bool;
}
```

### rp2350sim-cpu-arm

Implements the ARM Cortex-M33 CPU.

**Features:**
- Thumb-2 instruction set
- FPU (single precision)
- DSP instructions
- M-profile extensions
- NVIC (Nested Vectored Interrupt Controller)
- SysTick timer

**Pipeline:**
1. Fetch (16-bit or 32-bit Thumb instruction)
2. Decode (determine instruction type)
3. Execute (perform operation)
4. Memory (load/store)
5. Writeback (update registers)

### rp2350sim-cpu-hazard3

Implements the Hazard3 RISC-V CPU.

**Features:**
- RV32IMAC base + extensions
- Machine mode only
- PLIC (Platform Level Interrupt Controller)
- CSR (Control and Status Registers)

**Supported Extensions:**
- M: Integer multiplication/division
- A: Atomic operations
- C: Compressed instructions

## Device Crates

### rp2350sim-devices

Implements all peripheral devices.

**Device Structure:**
```rust
pub trait Device {
    fn id(&self) -> DeviceId;
    fn read(&mut self, addr: u32) -> Result<u32>;
    fn write(&mut self, addr: u32, value: u32) -> Result<()>;
    fn reset(&mut self);
}
```

**Peripheral Categories:**

| Category | Peripherals |
|----------|-------------|
| Communication | UART, SPI, I2C, I2S |
| Timing | Timer, RTC, PLL |
| Analog | ADC, PWM |
| Security | SHA-256, TRNG |
| System | GPIO, DMA, XIP, PIO, USB, Watchdog |

### rp2350sim-irq

Implements interrupt handling.

**Components:**
- `IrqController` - Interrupt controller interface
- `Nvic` - ARM Nested Vectored Interrupt Controller
- `Plic` - RISC-V Platform Level Interrupt Controller

### rp2350sim-clocks

Implements the clock system.

**Components:**
- `Clocks` - Clock configuration
- `Pll` - PLL implementation
- `ClockGate` - Clock gating

## Debug Crates

### rp2350sim-debug

Implements debugging support.

**Components:**
- `Debugger` - Main debugger interface
- `Breakpoints` - Breakpoint management
- `Watchpoints` - Watchpoint management
- `Disassembler` - Instruction disassembly

### rp2350sim-gdb

Implements GDB remote protocol.

**Supported Commands:**
- `g` - Read registers
- `G` - Write registers
- `m` - Read memory
- `M` - Write memory
- `c` - Continue
- `s` - Step
- `Z` - Insert breakpoint
- `z` - Remove breakpoint

### rp2350sim-trace

Implements execution tracing.

**Trace Types:**
- `Instruction` - Instruction execution
- `Memory` - Memory access
- `Gpio` - GPIO events
- `Mmio` - MMIO access
- `Irq` - Interrupt events

## UI Crates

### rp2350sim-ui

Implements the user interface.

**Components:**
- `Ui` - Main UI structure
- `PeripheralPanelManager` - Peripheral panels
- `MemoryView` - Memory viewer
- `DisasmView` - Disassembly view

### rp2350sim-gpu

Implements GPU-accelerated rendering.

**Components:**
- `BoardRenderer` - Board visualization
- `WaveformRenderer` - Waveform display
- `HeatmapRenderer` - Memory heatmap

## Data Flow

### Instruction Execution

```
┌──────────────┐
│ Fetch        │ ← Bus → Memory
│ Instruction  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Decode       │
│ Instruction  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Execute      │
│ Operation    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Memory       │ ← Bus → Memory/Device
│ Access       │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Writeback    │
│ Result       │
└──────────────┘
```

### Peripheral Access

```
CPU
 │
 ▼
Bus.read(addr) / Bus.write(addr, value)
 │
 ▼
MemoryMap.lookup(addr)
 │
 ▼
Device.read(offset) / Device.write(offset, value)
 │
 ▼
Peripheral-specific handling
```

### Interrupt Handling

```
Peripheral
 │
 ▼
IrqController.request(irq_num)
 │
 ▼
CPU.check_interrupts()
 │
 ▼
CPU.handle_interrupt()
 │
 ▼
CPU.jump_to_handler()
```

## Performance Considerations

### Memory Access Optimization

- Memory regions are looked up via a sorted array
- Frequently accessed regions (SRAM) are checked first
- Device access uses a hash map for O(1) lookup

### Instruction Execution

- Direct threaded dispatch for instruction decoding
- Cached decoded instructions for hot paths
- Minimal branching in hot loops

### Peripheral Simulation

- Lazy evaluation of peripheral state
- Event-driven updates where possible
- Batch processing of DMA transfers

## Testing Strategy

### Unit Tests

Each crate has unit tests for:
- Core functionality
- Edge cases
- Error handling

### Integration Tests

Located in `tests/` directory:
- CPU instruction tests
- Peripheral behavior tests
- Full system tests

### Differential Testing

`rp2350sim-difftest` provides:
- Comparison with reference implementation
- State divergence detection
- Automated regression testing

## Extending the Simulator

### Adding a New Peripheral

1. Create new module in `rp2350sim-devices/src/`
2. Implement `Device` trait
3. Add to `DeviceId` enum
4. Register in `rp2350sim-soc`
5. Add UI panel if needed

### Adding a New CPU Backend

1. Create new crate `rp2350sim-cpu-<name>`
2. Implement `Cpu` trait
3. Add to `CpuType` enum
4. Integrate with `rp2350sim-soc`

### Adding a New UI Panel

1. Create new file in `rp2350sim-ui/src/panels/`
2. Implement panel struct
3. Add to `PeripheralTab` enum
4. Add state to `PeripheralState`
5. Register in `PeripheralPanelManager`
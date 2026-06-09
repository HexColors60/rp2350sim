# RP2350 Pico 2 W Full-System Simulator

A full-system Raspberry Pi Pico 2 W / RP2350 simulator in Rust with ARM + Hazard3 support,
peripheral emulation, GPU-accelerated visualization, and integrated debugger UI.

## Features

### CPU Emulation
- **Dual CPU Backend Support**
  - ARM Cortex-M33 dual-core execution with FPU and DSP
  - Hazard3 RISC-V dual-core execution with RV32IMAC support
  - Instruction-level accuracy
  - GDB debugging support for both architectures

### Peripheral Emulation

#### Communication Interfaces
| Peripheral | Description |
|------------|-------------|
| **UART** | 2x UART with FIFO, configurable baud rate, parity |
| **SPI** | 2x SPI master/slave with configurable clock |
| **I2C** | 2x I2C master/slave with clock stretching |
| **I2S** | Audio interface with TX/RX FIFO, multiple formats |

#### Timing and Clocks
| Peripheral | Description |
|------------|-------------|
| **Timer** | 64-bit timer with alarms |
| **RTC** | Real-time clock with date/time and alarm |
| **PLL** | SYS and USB PLL clock generators |
| **Clocks** | Configurable clock system |

#### Analog and PWM
| Peripheral | Description |
|------------|-------------|
| **ADC** | 4-channel 12-bit ADC |
| **PWM** | 24 channels with configurable duty cycle |

#### Security and Crypto
| Peripheral | Description |
|------------|-------------|
| **SHA-256** | Hardware hash accelerator |
| **TRNG** | True random number generator |

#### System
| Peripheral | Description |
|------------|-------------|
| **GPIO** | 48 pins with mux and pad control |
| **DMA** | 12-channel DMA controller |
| **XIP** | Execute-in-place with cache |
| **PIO** | 2x PIO with 4 state machines each |
| **USB** | USB 1.1 device controller |
| **Watchdog** | Watchdog timer |

### Development Tools
- **Integrated Debugger**
  - Breakpoints and watchpoints
  - Memory inspection and modification
  - Register view for both CPU types
  - GDB remote debugging protocol support

- **Tracing System**
  - Instruction tracing
  - Memory access tracing
  - GPIO event tracing
  - MMIO access logging

- **Checkpoint/Restore**
  - Save and restore complete system state
  - Snapshot-based debugging

### Visualization
- **GPU-Accelerated Rendering**
  - Real-time peripheral state display
  - Board visualization with pin states
  - Memory heatmap

- **Peripheral Panels**
  - GPIO pin state viewer
  - UART/SPI/I2C transaction monitor
  - DMA channel status
  - XIP cache statistics
  - I2S audio visualization
  - RTC time display
  - PLL configuration
  - SHA-256 hash output
  - TRNG random data

## Project Structure

```
rp2350sim/
├── crates/
│   ├── rp2350sim-app/          # Main application entry point
│   ├── rp2350sim-core/         # Core types, traits, and utilities
│   ├── rp2350sim-bus/          # Bus implementation and memory mapping
│   ├── rp2350sim-mem/          # Memory system (RAM, Flash, ROM)
│   ├── rp2350sim-cpu-common/   # CPU interface and common code
│   ├── rp2350sim-cpu-arm/      # ARM Cortex-M33 backend
│   ├── rp2350sim-cpu-hazard3/  # Hazard3 RISC-V backend
│   ├── rp2350sim-clocks/       # Clock and PLL system
│   ├── rp2350sim-irq/          # Interrupt controller (NVIC/PLIC)
│   ├── rp2350sim-devices/      # Peripheral implementations
│   ├── rp2350sim-soc/          # SoC composition and wiring
│   ├── rp2350sim-debug/        # Debugger and disassembler
│   ├── rp2350sim-trace/        # Tracing system
│   ├── rp2350sim-save/         # Save state management
│   ├── rp2350sim-difftest/     # Differential testing framework
│   ├── rp2350sim-gpu/          # GPU rendering (wgpu)
│   ├── rp2350sim-ecs/          # Entity Component System
│   ├── rp2350sim-ui/           # egui-based UI panels
│   ├── rp2350sim-virtual-devices/ # Virtual device implementations
│   ├── rp2350sim-scripting/    # Lua scripting support
│   ├── rp2350sim-project/      # Project management
│   └── rp2350sim-gdb/          # GDB stub implementation
├── configs/                    # Configuration files
└── tests/                      # Integration tests
```

## Building

### Prerequisites
- Rust 1.70 or later
- Platform-specific dependencies for wgpu

### Build Commands

```bash
# Debug build
cargo build

# Release build (recommended for performance)
cargo build --release

# Build with all features
cargo build --all-features
```

## Usage

### GUI Mode

```bash
# Launch GUI simulator
cargo run -- gui

# Load firmware at startup
cargo run -- gui --firmware path/to/firmware.elf
```

### Headless Mode

```bash
# Run firmware in headless mode
cargo run -- run firmware.elf

# Run with GDB server on port 3333
cargo run -- run --gdb 3333 firmware.elf
```

### Information Commands

```bash
# Show firmware info
cargo run -- info firmware.elf

# Disassemble firmware
cargo run -- disasm firmware.elf
```

### GDB Debugging

```bash
# Start simulator with GDB server
cargo run -- run --gdb 3333 firmware.elf

# In another terminal, connect with GDB
arm-none-eabi-gdb firmware.elf
(gdb) target remote localhost:3333
```

## Configuration

The simulator uses TOML configuration files. Example:

```toml
[cpu]
type = "arm"  # or "riscv"
cores = 2
frequency = 150_000_000  # 150 MHz

[memory]
sram_size = 0x80000  # 512 KB
flash_size = 0x400000  # 4 MB

[peripherals]
uart0_enabled = true
uart1_enabled = true
```

## API Usage

### Basic Simulation

```rust
use rp2350sim_soc::Soc;
use rp2350sim_core::CoreId;

// Create SoC instance
let mut soc = Soc::new();

// Load firmware
soc.load_elf("firmware.elf")?;

// Run simulation
loop {
    soc.step(1000);  // Execute 1000 cycles
    
    // Check UART output
    if let Some(byte) = soc.uart_read(0) {
        print!("{}", byte as char);
    }
}
```

### Peripheral Access

```rust
// Set GPIO pin
soc.gpio_set(0, true);

// Read GPIO pin
let value = soc.gpio_get(0);

// Write to UART
soc.uart_write(0, b'H');
soc.uart_write(0, b'i');

// Read from UART
if let Some(byte) = soc.uart_read(0) {
    println!("Received: {}", byte);
}
```

## Memory Map

| Region | Start | End | Size |
|--------|-------|-----|------|
| SRAM Bank 0 | 0x20000000 | 0x2003FFFF | 256 KB |
| SRAM Bank 1 | 0x20040000 | 0x2007FFFF | 256 KB |
| SRAM Bank 2 | 0x20080000 | 0x200BFFFF | 256 KB |
| SRAM Bank 3 | 0x200C0000 | 0x200FFFFF | 256 KB |
| XIP ROM | 0x10000000 | 0x103FFFFF | 4 MB |
| XIP RAM | 0x15000000 | 0x1500FFFF | 64 KB |
| Peripheral | 0x40000000 | 0x4FFFFFFF | - |

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_uart

# Run with verbose output
cargo test -- --nocapture
```

## Contributing

Contributions are welcome! Please read the contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Check formatting: `cargo fmt --check`
6. Run clippy: `cargo clippy`
7. Submit a pull request

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

## References

- [RP2350 Datasheet](https://www.raspberrypi.com/documentation/microcontrollers/rp2350.html)
- [ARM Cortex-M33 Technical Reference Manual](https://developer.arm.com/documentation/100230/latest/)
- [Hazard3 RISC-V Core](https://github.com/Wren6991/Hazard3)
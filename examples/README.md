# RP2350 Simulator Example Programs

This directory contains example firmware programs for the RP2350 simulator.

## Directory Structure

```
examples/
├── arm/           # ARM Cortex-M33 examples
│   ├── blink.rs   # LED blink example (Rust source)
│   └── uart_echo.rs # UART echo example (Rust source)
└── riscv/         # Hazard3 RISC-V examples
    └── blink.rs   # LED blink example (Rust source)
```

## Building Examples

### Prerequisites

Install the required Rust targets:

```bash
# ARM Cortex-M33 target
rustup target add thumbv8m.main-none-eabi

# RISC-V target
rustup target add riscv32imac-unknown-none-elf
```

### Building ARM Examples

```bash
cd rp2350sim
cargo build --target thumbv8m.main-none-eabi --example blink
```

### Building RISC-V Examples

```bash
cd rp2350sim
cargo build --target riscv32imac-unknown-none-elf --example blink_rv32
```

## Running Examples

### Using the CLI

```bash
# Run ARM program
rp2350sim run --arch arm examples/arm/blink.hex

# Run RISC-V program
rp2350sim run --arch riscv examples/riscv/blink.hex

# Run with verbose output (shows registers)
rp2350sim run -v examples/arm/blink.hex

# Run with trace (shows PC at each step)
rp2350sim run -t examples/arm/blink.hex
```

### Using the GUI

```bash
rp2350sim gui
```

Then use **File > Load Firmware** to load a hex or bin file.

## Example Descriptions

### blink.rs

A simple LED blink program that:
1. Configures GPIO 25 as output
2. Toggles the LED in a loop
3. Uses a delay loop for timing

### uart_echo.rs

A UART echo program that:
1. Initializes UART0 at 115200 baud
2. Echoes back any received characters
3. Prints a welcome message

## Memory Map

| Region | Start Address | Size | Description |
|--------|--------------|------|-------------|
| Flash | 0x10000000 | 16 MB | External flash (XIP) |
| SRAM | 0x20000000 | 512 KB | Internal SRAM |
| Boot ROM | 0x00000000 | 64 KB | Mask ROM |
| Peripherals | 0x40000000 | - | Peripheral registers |
| SIO | 0xD0000000 | - | Single-cycle I/O |

## GPIO Pin Mapping (Pico 2 W)

| Pin | GPIO | Function |
|-----|------|----------|
| LED | 25 | Onboard LED |
| TX0 | 0 | UART0 TX |
| RX0 | 1 | UART0 RX |
| SDA0 | 4 | I2C0 SDA |
| SCL0 | 5 | I2C0 SCL |

## Creating Custom Examples

1. Create a new `.rs` file in the appropriate directory
2. Use `#![no_std]` and `#![no_main]`
3. Define a `#[entry]` function
4. Implement a `#[panic_handler]`
5. Build with the appropriate target

### Template

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m::asm;

#[entry]
fn main() -> ! {
    // Your code here
    
    loop {
        asm::nop();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::nop();
    }
}
```

## Debugging

### Using GDB

```bash
# Start the simulator with GDB server
rp2350sim run --gdb 3333 examples/arm/blink.elf

# In another terminal
arm-none-eabi-gdb examples/arm/blink.elf
(gdb) target remote localhost:3333
(gdb) continue
```

### Using the GUI Debugger

1. Launch the GUI: `rp2350sim gui`
2. Load firmware: **File > Load Firmware**
3. Set breakpoints by clicking in the disassembly view
4. Use **Run > Step** or press F6 to step through code
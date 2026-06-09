# RP2350 Pico 2 W Simulator - User Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [CLI Commands](#cli-commands)
5. [GUI Reference](#gui-reference)
6. [Peripheral Panels](#peripheral-panels)
7. [Debugging](#debugging)
8. [Configuration](#configuration)
9. [Advanced Topics](#advanced-topics)

---

## Getting Started

The RP2350 Pico 2 W Simulator is a full-system simulator for the Raspberry Pi Pico 2 development board. It emulates both the ARM Cortex-M33 and Hazard3 RISC-V cores, along with all major peripherals.

### Features

- Dual CPU architecture support (ARM + RISC-V)
- Cycle-accurate peripheral emulation
- GPU-accelerated visualization
- Integrated debugger with GDB support
- Checkpoint/restore functionality
- VCD waveform export

---

## Installation

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd rp2350sim

# Build release version
cargo build --release

# Run tests
cargo test
```

### Required Rust Targets

For building firmware examples:

```bash
rustup target add thumbv8m.main-none-eabi
rustup target add riscv32imac-unknown-none-elf
```

---

## Quick Start

### Running the GUI

```bash
cargo run --release -- gui
```

Or using the batch script:

```bash
rungui.bat
```

### Running a Program

```bash
# Run a firmware file
cargo run --release -- run firmware.hex

# With verbose output
cargo run --release -- run -v firmware.hex

# With PC tracing
cargo run --release -- run -t firmware.hex
```

### Loading Firmware in GUI

1. Launch the GUI
2. Select **File > Load Firmware**
3. Choose a `.hex`, `.bin`, or `.elf` file
4. Click **Run** or press F5

---

## CLI Commands

### gui

Launch the graphical user interface.

```bash
rp2350sim gui [OPTIONS]

Options:
  -c, --config <FILE>  Configuration file to load
  -f, --firmware <FILE>  Firmware to load on startup
```

### run

Run firmware in headless mode.

```bash
rp2350sim run [OPTIONS] <FILE>

Arguments:
  <FILE>  Firmware file (.hex, .bin, .elf)

Options:
  -v, --verbose     Show register state
  -t, --trace       Show PC at each cycle
  -a, --arch <ARCH> CPU architecture (arm, riscv)
  -c, --cycles <N>  Maximum cycles to run
  -i, --instructions <N>  Maximum instructions to execute
  --gdb <PORT>     Start GDB server on port
```

### info

Display information about a firmware file.

```bash
rp2350sim info <FILE>

# Shows:
# - File format
# - Entry point
# - Memory regions
# - Section information
# - Symbol table (if available)
```

### disasm

Disassemble a firmware file.

```bash
rp2350sim disasm [OPTIONS] <FILE>

Options:
  -a, --arch <ARCH>  Architecture (arm, riscv)
  -s, --start <ADDR> Start address
  -e, --end <ADDR>   End address
  -n, --count <N>    Number of instructions
```

### test

Run firmware as a test.

```bash
rp2350sim test <FILE>

# Exits with success if program:
# - Completes without errors
# - Reaches expected end state
```

---

## GUI Reference

### Menu Bar

#### File Menu
- **Load Firmware** (Ctrl+O): Load a firmware file
- **Load Configuration**: Load a configuration file
- **Save Configuration**: Save current settings
- **Exit**: Quit the application

#### Run Menu
- **Run** (F5): Start/resume execution
- **Pause** (F6): Pause execution
- **Step** (F7): Execute one instruction
- **Reset** (F8): Reset the simulation
- **Speed**: Set simulation speed

#### View Menu
- **CPU Registers**: Toggle CPU register panel
- **Memory View**: Toggle memory view panel
- **Disassembly**: Toggle disassembly panel
- **Console**: Toggle console panel
- **Waveform**: Toggle waveform panel
- **Board View**: Toggle board visualization
- **Peripheral Panels**: Toggle peripheral panels

### Toolbar

| Button | Shortcut | Function |
|--------|----------|----------|
| ▶ | F5 | Run |
| ⏸ | F6 | Pause |
| ⏭ | F7 | Step |
| ⏹ | F8 | Reset |

### Status Bar

Shows:
- Current PC value
- Cycle count
- Instructions executed
- Current speed

---

## Peripheral Panels

### GPIO Panel

Shows the state of all 48 GPIO pins:
- Pin number and name
- Direction (input/output)
- Current value (high/low)
- Pull configuration
- Function select

### UART Panel

Displays UART activity:
- TX/RX FIFO contents
- Baud rate
- Configuration
- Transfer log

### SPI Panel

Shows SPI bus activity:
- MOSI/MISO data
- Clock configuration
- Chip select state
- Transaction log

### I2C Panel

Displays I2C bus activity:
- Address and data
- Read/Write direction
- ACK/NACK status
- Transaction log

### Timer Panel

Shows timer state:
- Current counter value
- Alarm settings
- Interrupt status

### DMA Panel

Displays DMA channel status:
- Channel enable/disable
- Transfer count
- Source/destination addresses
- Busy status

---

## Debugging

### Breakpoints

1. Open the **Disassembly** panel
2. Click on an instruction address
3. A red dot indicates the breakpoint
4. Click again to remove

### Memory Inspection

1. Open the **Memory View** panel
2. Enter an address in the address bar
3. View/edit memory contents

### Register Inspection

The **CPU Registers** panel shows:
- General purpose registers (R0-R15)
- Status flags (N, Z, C, V)
- Current mode
- Stack pointers

### GDB Integration

Start the simulator with GDB server:

```bash
rp2350sim run --gdb 3333 firmware.elf
```

Connect with GDB:

```bash
arm-none-eabi-gdb firmware.elf
(gdb) target remote localhost:3333
(gdb) break main
(gdb) continue
```

Supported GDB commands:
- continue, step, next
- break, delete
- info registers
- x/Nx address (memory examine)
- set variable

---

## Configuration

### Configuration Files

Configuration files are TOML format:

```toml
# default.toml
[cpu]
architecture = "arm"
clock_speed = 150000000  # 150 MHz

[memory]
sram_size = 524288       # 512 KB
flash_size = 16777216    # 16 MB

[simulation]
max_cycles = 0           # 0 = unlimited
trace_enabled = false

[display]
theme = "dark"
fps_limit = 60
```

### Loading Configuration

```bash
rp2350sim gui -c configs/debug.toml
```

### Board Profiles

Board profiles define pin mappings and default peripherals:

```bash
rp2350sim gui -c configs/boards/pico2w.toml
```

---

## Advanced Topics

### Checkpointing

Save and restore simulation state:

```bash
# In GUI: File > Save Checkpoint
# Or via console:
> save checkpoint.json

# Restore:
> load checkpoint.json
```

### VCD Export

Export waveforms for analysis:

```bash
rp2350sim run --vcd trace.vcd firmware.hex
```

Open in GTKWave or other waveform viewers.

### Scripting

Control the simulator via Rhai scripts:

```rhai
// script.rhai
load_firmware("blink.hex");
run_until(10000);
print_registers();
```

```bash
rp2350sim script script.rhai
```

### Dual Core

The RP2350 has dual cores. By default, both are enabled:

```bash
# Run on both cores
rp2350sim run firmware.elf

# Run on specific core
rp2350sim run --core 0 firmware.elf
```

### Performance

For faster execution:
- Use release builds: `cargo build --release`
- Disable tracing: `trace_enabled = false`
- Use fast mode: `-c configs/fast.toml`

---

## Troubleshooting

### Program doesn't run

1. Check architecture matches (ARM vs RISC-V)
2. Verify entry point address
3. Check memory map for correct loading

### GUI performance issues

1. Reduce FPS limit in configuration
2. Disable unnecessary panels
3. Use fast mode configuration

### GDB connection fails

1. Verify port is not in use
2. Check firewall settings
3. Ensure same architecture

---

## Support

For issues and feature requests, please use the project's issue tracker.
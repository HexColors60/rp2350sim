# Changelog

All notable changes to the RP2350 Simulator will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Example firmware programs for ARM and RISC-V
- USER_GUIDE.md with comprehensive usage documentation
- GitHub Actions CI/CD workflows
- Timer interrupt example

### Changed
- Improved test coverage for rp2350sim-mem (44 new tests)

## [0.1.0] - 2026-03-23

### Added
- Initial release of RP2350 Pico 2 W Full-System Simulator
- Dual CPU backend support (ARM Cortex-M33 + Hazard3 RISC-V)
- Full peripheral emulation:
  - Communication: UART, SPI, I2C, I2S
  - Timing: Timer, RTC, PLL, Clocks
  - Analog: ADC, PWM
  - Security: SHA-256, TRNG
  - System: GPIO, DMA, XIP, PIO, USB, Watchdog
  - Additional: NVIC, SysTick, PLIC, PowerManager, Sysinfo, OTP, MPU, CoreSight, HSTX, Interp, Bootram, BusCtrl
- GPU-accelerated visualization
- Integrated debugger with GDB support
- Checkpoint/restore functionality
- VCD waveform export
- 335 unit and integration tests
- 21 workspace crates

### CPU Emulation
- ARM Cortex-M33 dual-core execution with FPU and DSP
- Hazard3 RISC-V dual-core execution with RV32IMAC support
- Instruction-level accuracy
- GDB debugging support for both architectures

### Peripherals
- GPIO: 48 pins with mux and pad control
- UART: 2x UART with FIFO, configurable baud rate
- SPI: 2x SPI master/slave
- I2C: 2x I2C master/slave
- Timer: 64-bit timer with alarms
- DMA: 12-channel DMA controller
- XIP: Execute-in-place with cache
- PIO: 2x PIO with 4 state machines each
- And many more...

### Development Tools
- Breakpoints and watchpoints
- Memory inspection and modification
- Instruction tracing
- Memory access tracing
- GPIO event tracing
- Save/restore complete system state

### Visualization
- Real-time peripheral state display
- Board visualization with pin states
- Memory heatmap
- Peripheral panels for all major peripherals

[Unreleased]: https://github.com/user/rp2350sim/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/user/rp2350sim/releases/tag/v0.1.0
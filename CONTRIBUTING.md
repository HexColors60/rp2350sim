# Contributing to RP2350 Simulator

Thank you for your interest in contributing to the RP2350 Pico 2 W Full-System Simulator!

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git

### Getting Started

```bash
# Clone the repository
git clone https://github.com/user/rp2350sim.git
cd rp2350sim

# Build the project
cargo build

# Run tests
cargo test

# Run the GUI
cargo run -- gui
```

### Rust Targets

For building firmware examples:

```bash
rustup target add thumbv8m.main-none-eabi
rustup target add riscv32imac-unknown-none-elf
```

## Project Structure

```
rp2350sim/
├── crates/
│   ├── rp2350sim-app/          # Main application
│   ├── rp2350sim-core/         # Core types and traits
│   ├── rp2350sim-bus/          # Bus implementation
│   ├── rp2350sim-mem/          # Memory system
│   ├── rp2350sim-cpu-arm/      # ARM backend
│   ├── rp2350sim-cpu-hazard3/  # RISC-V backend
│   ├── rp2350sim-devices/      # Peripheral implementations
│   ├── rp2350sim-soc/          # SoC composition
│   ├── rp2350sim-debug/        # Debugger
│   ├── rp2350sim-gpu/          # GPU rendering
│   ├── rp2350sim-ui/           # UI panels
│   └── ...                     # Other crates
├── tests/                       # Integration tests
├── examples/                    # Example firmware
├── configs/                     # Configuration files
└── docs/                        # Documentation
```

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the bug report template
3. Include:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Environment details

### Adding Features

1. Open an issue to discuss the feature
2. Fork the repository
3. Create a feature branch
4. Implement the feature
5. Add tests
6. Update documentation
7. Submit a pull request

### Adding Peripherals

1. Create a new module in `rp2350sim-devices/src/`
2. Implement the `Device` trait
3. Add register definitions
4. Implement read/write handlers
5. Add to `lib.rs` exports
6. Add tests
7. Add GUI panel in `rp2350sim-ui/src/panels/`
8. Update documentation

### Adding Instructions

For ARM Cortex-M33:
1. Add decode logic in `rp2350sim-cpu-arm/src/thumb/decode.rs`
2. Add execution in `rp2350sim-cpu-arm/src/thumb/execute.rs`
3. Add tests

For RISC-V:
1. Add decode logic in `rp2350sim-cpu-hazard3/src/rv32/decode.rs`
2. Add execution in `rp2350sim-cpu-hazard3/src/rv32/execute.rs`
3. Add tests

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add documentation comments to public APIs

## Testing

- All new code should have tests
- Run `cargo test` to ensure all tests pass
- Aim for high test coverage

### Test Categories

- **Unit tests**: In module files with `#[cfg(test)]`
- **Integration tests**: In `tests/` directory
- **Firmware tests**: In `tests/firmware_tests.rs`

## Documentation

- Update README.md for user-facing changes
- Update docs/ for API changes
- Add inline documentation for public APIs
- Update CHANGELOG.md

## Pull Request Process

1. Ensure all tests pass
2. Update documentation
3. Update CHANGELOG.md
4. Request review from maintainers
5. Address review comments

## License

By contributing, you agree that your contributions will be licensed under the project's license.

## Questions?

Open an issue for questions or discussions about the project.
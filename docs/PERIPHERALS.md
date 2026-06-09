# RP2350 Peripheral Documentation

This document provides detailed information about the emulated peripherals in the RP2350 simulator.

## Table of Contents

1. [Communication Peripherals](#communication-peripherals)
   - [UART](#uart)
   - [SPI](#spi)
   - [I2C](#i2c)
   - [I2S](#i2s)
2. [Timing Peripherals](#timing-peripherals)
   - [Timer](#timer)
   - [RTC](#rtc)
   - [PLL](#pll)
3. [Analog Peripherals](#analog-peripherals)
   - [ADC](#adc)
   - [PWM](#pwm)
4. [Security Peripherals](#security-peripherals)
   - [SHA-256](#sha-256)
   - [TRNG](#trng)
5. [System Peripherals](#system-peripherals)
   - [GPIO](#gpio)
   - [DMA](#dma)
   - [XIP](#xip)
   - [PIO](#pio)
   - [USB](#usb)
   - [Watchdog](#watchdog)

---

## Communication Peripherals

### UART

The Universal Asynchronous Receiver/Transmitter (UART) provides serial communication.

#### Features
- 2 independent UART instances (UART0, UART1)
- Configurable baud rate (up to 1 Mbps)
- 32-byte TX and RX FIFOs
- Programmable data bits (5-8), parity, and stop bits
- Hardware flow control (CTS/RTS)
- Interrupt support

#### Base Addresses
- UART0: `0x40034000`
- UART1: `0x40038000`

#### Example Usage

```rust
// Initialize UART
soc.uart_set_baud(0, 115200);

// Write data
soc.uart_write(0, b'H');
soc.uart_write(0, b'e');
soc.uart_write(0, b'l');
soc.uart_write(0, b'l');
soc.uart_write(0, b'o');

// Read data
while let Some(byte) = soc.uart_read(0) {
    println!("Received: {}", byte as char);
}
```

### SPI

The Serial Peripheral Interface (SPI) provides synchronous serial communication.

#### Features
- 2 independent SPI instances (SPI0, SPI1)
- Master and slave modes
- Configurable clock polarity (CPOL) and phase (CPHA)
- Up to 62.5 MHz clock
- 8-entry TX and RX FIFOs
- DMA support

#### Base Addresses
- SPI0: `0x4003C000`
- SPI1: `0x40040000`

### I2C

The Inter-Integrated Circuit (I2C) bus provides two-wire serial communication.

#### Features
- 2 independent I2C instances (I2C0, I2C1)
- Master and slave modes
- Standard (100 kHz) and Fast (400 kHz) modes
- 7-bit and 10-bit addressing
- Clock stretching support
- DMA support

#### Base Addresses
- I2C0: `0x40044000`
- I2C1: `0x40048000`

### I2S

The Inter-IC Sound (I2S) interface provides digital audio communication.

#### Features
- 2 independent I2S instances (I2S0, I2S1)
- Master and slave modes
- Configurable sample rate (up to 192 kHz)
- Multiple data widths (8, 16, 24, 32-bit)
- I2S, Left-Justified, Right-Justified formats
- 32-entry TX and RX FIFOs
- DMA support

#### Base Addresses
- I2S0: `0x50008000`
- I2S1: `0x5000C000`

#### Configuration

```rust
// Configure I2S
i2s.set_sample_rate(48000);
i2s.data_width = DataWidth::Bits16;
i2s.audio_format = AudioFormat::I2S;
i2s.channels = 2;

// Write audio samples
i2s.write_tx_fifo(left_sample | (right_sample << 16));

// Read audio samples
if let Some(sample) = i2s.read_rx_fifo() {
    let left = (sample & 0xFFFF) as i16;
    let right = ((sample >> 16) & 0xFFFF) as i16;
}
```

---

## Timing Peripherals

### Timer

The system timer provides precise timing and alarm functionality.

#### Features
- 64-bit free-running counter
- 4 alarm channels
- Microsecond resolution
- Interrupt support

#### Base Address
- Timer: `0x400B4000`

### RTC

The Real-Time Clock provides date and time tracking.

#### Features
- Full date/time tracking (seconds to years)
- Day of week calculation
- Leap year support
- Alarm with flexible matching
- Interrupt support

#### Base Address
- RTC: `0x400BC000`

#### Time Structure

```rust
let time = RtcTime {
    second: 30,
    minute: 45,
    hour: 14,
    day_of_week: 3,  // Wednesday
    day: 15,
    month: 6,
    year: 2024,
};
rtc.set_time(time);
```

### PLL

The Phase-Locked Loop generates high-frequency clocks.

#### Features
- 2 PLL instances (SYS_PLL, USB_PLL)
- Configurable reference divider (1-63)
- Configurable feedback divider (16-320)
- Dual post-dividers
- Lock detection
- Bypass mode

#### Base Addresses
- PLL_SYS: `0x50028000`
- PLL_USB: `0x5002C000`

#### Frequency Calculation

```
VCO_freq = (ref_freq / refdiv) × fbdiv
Output_freq = VCO_freq / (postdiv1 × postdiv2)
```

#### Typical Configurations

| PLL | Input | REFDIV | FBDIV | POSTDIV1 | POSTDIV2 | Output |
|-----|-------|--------|-------|----------|----------|--------|
| SYS | 12 MHz | 1 | 133 | 6 | 2 | 133 MHz |
| USB | 12 MHz | 1 | 40 | 5 | 2 | 48 MHz |

---

## Analog Peripherals

### ADC

The Analog-to-Digital Converter measures analog signals.

#### Features
- 4 input channels
- 12-bit resolution
- Up to 500 kSps
- Temperature sensor
- FIFO with DMA support

#### Base Address
- ADC: `0x4004C000`

### PWM

The Pulse-Width Modulation generates analog-like signals.

#### Features
- 24 PWM channels (12 slices × 2 channels)
- 16-bit resolution
- Configurable frequency and duty cycle
- Phase-correct mode
- Interrupt support

#### Base Address
- PWM: `0x40050000`

---

## Security Peripherals

### SHA-256

The SHA-256 accelerator computes cryptographic hashes.

#### Features
- Full SHA-256 algorithm implementation
- Double-SHA mode (SHA256(SHA256(data)))
- Big and little endian support
- 256-bit hash output
- Streaming input via register writes

#### Base Address
- SHA-256: `0x50011000`

#### Usage

```rust
// Reset and enable
sha256.reset_hash();

// Write data (32-bit words)
sha256.write_data(0x48656C6C);  // "Hell"
sha256.write_data(0x6F000000);  // "o"

// Start computation
sha256.finalize();

// Read hash
let hash = sha256.get_hash();
for i in 0..8 {
    println!("H{}: {:08X}", i, hash[i]);
}
```

### TRNG

The True Random Number Generator produces random numbers.

#### Features
- Hardware random number generation
- 16-entry FIFO
- Configurable sample count
- Reseed capability
- Interrupt support

#### Base Address
- TRNG: `0x50012000`

#### Usage

```rust
// Enable TRNG
trng.enabled = true;

// Read random data
let random = trng.read_data();
println!("Random: {:08X}", random);
```

---

## System Peripherals

### GPIO

The General-Purpose Input/Output controls pins.

#### Features
- 48 GPIO pins
- Configurable direction (input/output)
- Pull-up and pull-down resistors
- 8 alternate functions per pin
- Interrupt on level/edge

#### Base Address
- GPIO: `0x400D0000`

#### Pin Functions

| Function | Description |
|----------|-------------|
| 0 | GPIO |
| 1-7 | Alternate functions (UART, SPI, I2C, etc.) |

### DMA

The Direct Memory Access controller moves data without CPU intervention.

#### Features
- 12 independent channels
- Multiple transfer sizes (8, 16, 32-bit)
- Address increment modes
- Channel chaining
- Interrupt support

#### Base Address
- DMA: `0x50000000`

### XIP

The Execute-In-Place controller allows code execution from external Flash.

#### Features
- 16 KB cache
- Configurable cache policies
- Direct Flash access
- Cache statistics

#### Base Address
- XIP: `0x50000000`

### PIO

The Programmable I/O allows custom peripheral implementations.

#### Features
- 2 PIO instances
- 4 state machines per instance
- 32 instruction memory per instance
- 8 TX and RX FIFOs per state machine
- Independent clock dividers

#### Base Addresses
- PIO0: `0x50200000`
- PIO1: `0x50300000`

### USB

The USB controller provides USB device functionality.

#### Features
- USB 1.1 device controller
- Multiple endpoints
- Control, bulk, interrupt, and isochronous transfers
- DMA support

#### Base Address
- USB: `0x50100000`

### Watchdog

The Watchdog Timer provides system reset capability.

#### Features
- Configurable timeout
- Pause on debug
- Force reset capability

#### Base Address
- Watchdog: `0x40058000`

---

## Interrupt Mapping

Each peripheral has dedicated interrupt lines. See the RP2350 datasheet for the complete interrupt vector table.

## DMA Trigger Sources

DMA channels can be triggered by various peripheral events:
- UART TX empty / RX full
- SPI TX empty / RX full
- I2C transaction complete
- ADC conversion complete
- Timer alarm
- PIO FIFO status

## Clock Sources

Peripherals can be clocked from various sources:
- CLK_SYS (system clock)
- CLK_PERI (peripheral clock)
- CLK_USB (USB clock)
- CLK_ADC (ADC clock)
- CLK_RTC (RTC clock)
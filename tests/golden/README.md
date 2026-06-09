# Golden Test Files

This directory contains test fixtures for integration tests.

## Structure

- `firmware/` - Test firmware binaries
- `traces/` - Expected trace logs
- `expected/` - Expected output files

## Adding Golden Tests

1. Place firmware files in `firmware/`
2. Place expected traces in `traces/`
3. Place expected outputs in `expected/`

## Test Firmware

The following test programs should be provided:

- `blink.elf` - Simple GPIO blink program
- `uart_echo.elf` - UART echo program
- `spi_loopback.elf` - SPI loopback test
- `i2c_scan.elf` - I2C bus scanner
- `pwm_fade.elf` - PWM LED fade program
- `adc_read.elf` - ADC reading program
- `pio_ws2812.elf` - PIO WS2812 LED driver
- `dual_core.elf` - Dual-core mailbox demo
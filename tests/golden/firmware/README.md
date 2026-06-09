# Golden Firmware Files

Place test firmware binaries here.

## Required Files

- `blink.elf` - GPIO blink test
- `uart_echo.elf` - UART echo test
- `spi_loopback.elf` - SPI loopback test
- `i2c_scan.elf` - I2C scan test
- `pwm_fade.elf` - PWM fade test
- `adc_read.elf` - ADC read test
- `pio_ws2812.elf` - PIO WS2812 test
- `dual_core.elf` - Dual-core test

## Building Firmware

Firmware should be built using the RP2350 SDK:

```bash
cd firmware_source
mkdir build && cd build
cmake ..
make
```

Copy the resulting .elf files to this directory.
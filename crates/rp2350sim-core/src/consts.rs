//! RP2350 constants and hardware parameters.

/// RP2350 clock frequency in Hz (150 MHz)
pub const CLOCK_FREQ_HZ: u64 = 150_000_000;

/// SRAM size in bytes (520 KB)
pub const SRAM_SIZE: usize = 520 * 1024;

/// Flash size in bytes (default 16 MB external flash)
pub const FLASH_SIZE: usize = 16 * 1024 * 1024;

/// Boot ROM size in bytes
pub const BOOTROM_SIZE: usize = 32 * 1024;

/// Number of GPIO pins
pub const GPIO_COUNT: usize = 48;

/// Number of external GPIO pins on Pico 2 W
pub const EXTERNAL_GPIO_COUNT: usize = 30;

/// Number of UART peripherals
pub const UART_COUNT: usize = 2;

/// Number of SPI peripherals
pub const SPI_COUNT: usize = 2;

/// Number of I2C peripherals
pub const I2C_COUNT: usize = 2;

/// Number of PWM channels
pub const PWM_CHANNEL_COUNT: usize = 24;

/// Number of ADC channels
pub const ADC_CHANNEL_COUNT: usize = 4;

/// Number of PIO instances
pub const PIO_COUNT: usize = 2;

/// Number of state machines per PIO
pub const PIO_SM_COUNT: usize = 6;

/// Total PIO state machines
pub const PIO_SM_TOTAL: usize = PIO_COUNT * PIO_SM_COUNT;

/// Number of CPU cores
pub const CPU_CORE_COUNT: usize = 2;

// Memory map base addresses

/// Boot ROM base address
pub const BOOTROM_BASE: u32 = 0x0000_0000;

/// XIP/Flash base address
pub const XIP_BASE: u32 = 0x1000_0000;

/// SRAM base address
pub const SRAM_BASE: u32 = 0x2000_0000;

/// Peripheral base address
pub const PERIPH_BASE: u32 = 0x4000_0000;

/// APB peripheral base
pub const APB_BASE: u32 = 0x5000_0000;

/// IOPORT base address (fast GPIO access)
pub const IOPORT_BASE: u32 = 0xD000_0000;

// Peripheral specific base addresses

/// GPIO base address
pub const GPIO_BASE: u32 = 0x4001_4000;

/// UART0 base address
pub const UART0_BASE: u32 = 0x4003_4000;

/// UART1 base address
pub const UART1_BASE: u32 = 0x4003_8000;

/// SPI0 base address
pub const SPI0_BASE: u32 = 0x4003_C000;

/// SPI1 base address
pub const SPI1_BASE: u32 = 0x4004_0000;

/// I2C0 base address
pub const I2C0_BASE: u32 = 0x4004_4000;

/// I2C1 base address
pub const I2C1_BASE: u32 = 0x4004_8000;

/// PWM base address
pub const PWM_BASE: u32 = 0x4005_0000;

/// ADC base address
pub const ADC_BASE: u32 = 0x4005_4000;

/// PIO0 base address
pub const PIO0_BASE: u32 = 0x5020_0000;

/// PIO1 base address
pub const PIO1_BASE: u32 = 0x5030_0000;

/// Timer base address
pub const TIMER_BASE: u32 = 0x4005_8000;

/// USB controller base address
pub const USB_BASE: u32 = 0x5011_0000;

/// Clocks base address
pub const CLOCKS_BASE: u32 = 0x4001_0000;

/// Resets base address
pub const RESETS_BASE: u32 = 0x4002_0000;

/// Watchdog base address
pub const WATCHDOG_BASE: u32 = 0x4005_C000;

// Interrupt Request (IRQ) numbers for RP2350
// See RP2350 datasheet section 2.3.2 for full list

/// Timer alarm 0
pub const IRQ_TIMER_0: u8 = 0;
/// Timer alarm 1
pub const IRQ_TIMER_1: u8 = 1;
/// Timer alarm 2
pub const IRQ_TIMER_2: u8 = 2;
/// Timer alarm 3
pub const IRQ_TIMER_3: u8 = 3;
/// PWM wrap (summed across all PWM slices)
pub const IRQ_PWM_WRAP: u8 = 4;
/// USB
pub const IRQ_USB: u8 = 5;
/// PIO0 IRQ0
pub const IRQ_PIO0_0: u8 = 6;
/// PIO0 IRQ1
pub const IRQ_PIO0_1: u8 = 7;
/// PIO1 IRQ0
pub const IRQ_PIO1_0: u8 = 8;
/// PIO1 IRQ1
pub const IRQ_PIO1_1: u8 = 9;
/// DMA_IRQ_0
pub const IRQ_DMA_0: u8 = 10;
/// DMA_IRQ_1
pub const IRQ_DMA_1: u8 = 11;
/// IO_BANK0 (GPIO)
pub const IRQ_IO_BANK0: u8 = 13;
/// IO_BANK0 (GPIO) - additional
pub const IRQ_IO_BANK0_NS: u8 = 14;
/// SysTick
pub const IRQ_SYSTICK: u8 = 15;
/// I2C0
pub const IRQ_I2C0: u8 = 19;
/// I2C1
pub const IRQ_I2C1: u8 = 18;
/// SPI0
pub const IRQ_SPI0: u8 = 16;
/// SPI1
pub const IRQ_SPI1: u8 = 17;
/// UART0
pub const IRQ_UART0: u8 = 20;
/// UART1
pub const IRQ_UART1: u8 = 21;
/// ADC FIFO
pub const IRQ_ADC_FIFO: u8 = 22;
/// CoreSight
pub const IRQ_CORESIGHT: u8 = 24;
/// RTC
pub const IRQ_RTC: u8 = 25;
/// SHA-256
pub const IRQ_SHA256: u8 = 26;
/// I2S0
pub const IRQ_I2S0: u8 = 27;
/// I2S1
pub const IRQ_I2S1: u8 = 28;
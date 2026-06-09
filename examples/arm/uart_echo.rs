//! UART Echo Example
//!
//! This example demonstrates UART communication on the RP2350.
//! It echoes back any character received on UART0.
//!
//! ## UART0 Pin Mapping (Pico 2)
//! - TX: GPIO 0 (Physical pin 1)
//! - RX: GPIO 1 (Physical pin 2)
//!
//! ## Running in Simulator
//!
//! ```bash
//! rp2350sim run examples/arm/uart_echo.hex
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m::asm;

/// UART0 base address
const UART0_BASE: u32 = 0x40034000;

/// UART registers
const UART_DR: *mut u32 = (UART0_BASE + 0x000) as *mut u32;
const UART_FR: *const u32 = (UART0_BASE + 0x018) as *const u32;
const UART_CR: *mut u32 = (UART0_BASE + 0x030) as *mut u32;
const UART_IBRD: *mut u32 = (UART0_BASE + 0x024) as *mut u32;
const UART_FBRD: *mut u32 = (UART0_BASE + 0x028) as *mut u32;
const UART_LCR_H: *mut u32 = (UART0_BASE + 0x02C) as *mut u32;

/// UART Flag Register bits
const FR_TXFF: u32 = 1 << 5;  // Transmit FIFO full
const FR_RXFE: u32 = 1 << 4;  // Receive FIFO empty

/// IO_BANK0 base
const IO_BANK0_BASE: u32 = 0x40028000;
const GPIO0_CTRL: *mut u32 = (IO_BANK0_BASE + 0x004) as *mut u32;
const GPIO1_CTRL: *mut u32 = (IO_BANK0_BASE + 0x008) as *mut u32;

/// Function select for UART
const GPIO_FUNC_UART: u32 = 2;

#[entry]
fn main() -> ! {
    // Initialize UART0
    unsafe {
        // Set GPIO0 and GPIO1 to UART function
        GPIO0_CTRL.write_volatile(GPIO_FUNC_UART);
        GPIO1_CTRL.write_volatile(GPIO_FUNC_UART);
        
        // Disable UART
        UART_CR.write_volatile(0);
        
        // Set baud rate (assuming 125 MHz clock, 115200 baud)
        // IBRD = 125000000 / (16 * 115200) = 67.81 -> 67
        // FBRD = 0.81 * 64 + 0.5 = 52
        UART_IBRD.write_volatile(67);
        UART_FBRD.write_volatile(52);
        
        // Set line control: 8 bits, no parity, 1 stop bit, enable FIFO
        UART_LCR_H.write_volatile(0x70);
        
        // Enable UART, TX, RX
        UART_CR.write_volatile(0x301);
    }
    
    // Print welcome message
    print_str("UART Echo Example\r\n");
    print_str("Type characters to echo...\r\n");
    
    // Echo loop
    loop {
        // Check if data available
        let fr = unsafe { UART_FR.read_volatile() };
        if (fr & FR_RXFE) == 0 {
            // Read character
            let c = unsafe { UART_DR.read_volatile() } as u8;
            
            // Echo back
            uart_putc(c);
            
            // Also echo newline with carriage return
            if c == b'\r' {
                uart_putc(b'\n');
            }
        }
    }
}

/// Send a character over UART
fn uart_putc(c: u8) {
    // Wait for TX FIFO not full
    while unsafe { UART_FR.read_volatile() } & FR_TXFF != 0 {}
    
    // Write character
    unsafe {
        UART_DR.write_volatile(c as u32);
    }
}

/// Print a string
fn print_str(s: &str) {
    for c in s.bytes() {
        uart_putc(c);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::nop();
    }
}
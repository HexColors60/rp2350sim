//! ARM Cortex-M33 LED Blink Example
//!
//! This example demonstrates a simple LED blink program for the RP2350.
//! It toggles GPIO pin 25 (the onboard LED on Pico 2) in a loop.
//!
//! ## Building
//!
//! ```bash
//! cargo build --target thumbv8m.main-none-eabi --example blink
//! ```
//!
//! ## Running in Simulator
//!
//! ```bash
//! rp2350sim run --arch arm examples/arm/blink.bin
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m::asm;

/// RP2350 GPIO base address
const SIO_BASE: u32 = 0xD0000000;

/// SIO registers
const GPIO_OUT: *mut u32 = (SIO_BASE + 0x010) as *mut u32;
const GPIO_OUT_SET: *mut u32 = (SIO_BASE + 0x014) as *mut u32;
const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + 0x018) as *mut u32;
const GPIO_OE_SET: *mut u32 = (SIO_BASE + 0x024) as *mut u32;

/// IO_BANK0 base address
const IO_BANK0_BASE: u32 = 0x40028000;

/// GPIO25 function select (set to SIO)
const GPIO25_CTRL: *mut u32 = (IO_BANK0_BASE + 0x0CC) as *mut u32;

/// LED pin number
const LED_PIN: u32 = 25;

/// Function select for GPIO (SIO = 5)
const GPIO_FUNC_SIO: u32 = 5;

/// Delay loop iterations (adjust for desired blink rate)
const DELAY_ITERATIONS: u32 = 500_000;

#[entry]
fn main() -> ! {
    // Initialize GPIO 25 as output
    unsafe {
        // Set GPIO25 function to SIO (GPIO function)
        GPIO25_CTRL.write_volatile(GPIO_FUNC_SIO);
        
        // Set GPIO 25 as output
        GPIO_OE_SET.write_volatile(1 << LED_PIN);
    }
    
    // Main blink loop
    loop {
        // Turn LED on
        unsafe {
            GPIO_OUT_SET.write_volatile(1 << LED_PIN);
        }
        
        // Delay
        delay(DELAY_ITERATIONS);
        
        // Turn LED off
        unsafe {
            GPIO_OUT_CLR.write_volatile(1 << LED_PIN);
        }
        
        // Delay
        delay(DELAY_ITERATIONS);
    }
}

/// Simple delay loop
#[inline(never)]
fn delay(iterations: u32) {
    let mut i = iterations;
    while i > 0 {
        i -= 1;
        asm::nop();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::nop();
    }
}
//! Timer Interrupt Example
//!
//! This example demonstrates timer-based interrupt handling on the RP2350.
//! It uses the Timer peripheral to generate periodic interrupts.
//!
//! ## Running in Simulator
//!
//! ```bash
//! rp2350sim run examples/arm/timer_interrupt.hex
//! ```

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m::asm;

/// Timer base address
const TIMER_BASE: u32 = 0x40054000;

/// Timer registers
const TIMEHR: *const u32 = (TIMER_BASE + 0x00) as *const u32;
const TIMELR: *const u32 = (TIMER_BASE + 0x04) as *const u32;
const ALARM0: *mut u32 = (TIMER_BASE + 0x10) as *mut u32;
const ARMED: *mut u32 = (TIMER_BASE + 0x20) as *mut u32;
const TIMERAWH: *const u32 = (TIMER_BASE + 0x28) as *const u32;
const TIMERAWL: *const u32 = (TIMER_BASE + 0x2C) as *const u32;
const INTR: *mut u32 = (TIMER_BASE + 0x30) as *mut u32;
const INTE: *mut u32 = (TIMER_BASE + 0x34) as *mut u32;

/// NVIC base address
const NVIC_BASE: u32 = 0xE000E100;
const NVIC_ISER0: *mut u32 = NVIC_BASE as *mut u32;

/// Timer interrupt number
const TIMER0_IRQ: u32 = 0;  // Alarm 0 uses IRQ 0

/// Alarm interval in microseconds
const ALARM_INTERVAL: u32 = 1_000_000;  // 1 second

/// Counter for interrupts
static mut INTERRUPT_COUNT: u32 = 0;

#[entry]
fn main() -> ! {
    // Enable timer interrupt in NVIC
    unsafe {
        NVIC_ISER0.write_volatile(1 << TIMER0_IRQ);
    }
    
    // Enable alarm 0 interrupt in timer
    unsafe {
        INTE.write_volatile(0x01);  // Enable alarm 0
    }
    
    // Set initial alarm
    set_alarm(ALARM_INTERVAL);
    
    // Main loop - just wait for interrupts
    loop {
        asm::wfi();
    }
}

/// Set alarm to trigger after specified microseconds
fn set_alarm(delay_us: u32) {
    unsafe {
        // Read current time
        let low = TIMELR.read_volatile();
        let high = TIMERAWH.read_volatile();
        let current = ((high as u64) << 32) | (low as u64);
        
        // Set alarm
        let alarm_time = current + delay_us as u64;
        ALARM0.write_volatile(alarm_time as u32);
        
        // Clear armed flag
        ARMED.write_volatile(0x01);
    }
}

/// Timer interrupt handler
#[no_mangle]
pub unsafe extern "C" fn TIMER_0_Handler() {
    // Increment counter
    INTERRUPT_COUNT += 1;
    
    // Clear interrupt
    INTR.write_volatile(0x01);
    
    // Set next alarm
    set_alarm(ALARM_INTERVAL);
    
    // Toggle LED (GPIO 25) as visual feedback
    toggle_led();
}

/// Toggle the onboard LED
fn toggle_led() {
    const SIO_BASE: u32 = 0xD0000000;
    const GPIO_OUT_XOR: *mut u32 = (SIO_BASE + 0x01C) as *mut u32;
    
    unsafe {
        GPIO_OUT_XOR.write_volatile(1 << 25);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        asm::nop();
    }
}
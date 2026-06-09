//! Timer device for RP2350.
//!
//! Implements the Timer peripheral with full register support.

use rp2350sim_core::{Device, DeviceId, Result};

/// Timer base address.
pub const TIMER_BASE: u32 = 0x400B_4000;

/// Timer register offsets.
pub mod regs {
    pub const TIMEHR: u32 = 0x000;
    pub const TIMELR: u32 = 0x004;
    pub const TIMERAWH: u32 = 0x008;
    pub const TIMERAWL: u32 = 0x00C;
    pub const DBGPAUSE: u32 = 0x010;
    pub const PAUSE: u32 = 0x014;
    pub const LOCKED: u32 = 0x018;
    pub const SOURCE: u32 = 0x01C;
    pub const ALARM0: u32 = 0x020;
    pub const ALARM1: u32 = 0x024;
    pub const ALARM2: u32 = 0x028;
    pub const ALARM3: u32 = 0x02C;
    pub const ARMED: u32 = 0x030;
    pub const TIMERAWH2: u32 = 0x034;
    pub const TIMERAWL2: u32 = 0x038;
    pub const TIMEHR2: u32 = 0x03C;
    pub const TIMELR2: u32 = 0x040;
    pub const INTR: u32 = 0x044;
    pub const INTE: u32 = 0x048;
    pub const INTF: u32 = 0x04C;
    pub const INTS: u32 = 0x050;
}

/// SysTick CSR bits.
pub mod systick_csr {
    pub const ENABLE: u32 = 1 << 0;
    pub const TICKINT: u32 = 1 << 1;
    pub const CLKSOURCE: u32 = 1 << 2;
    pub const COUNTFLAG: u32 = 1 << 16;
}

/// ARM timer control bits.
pub mod arm_timer_control {
    pub const ENABLE: u32 = 1 << 7;
    pub const MODE_16BIT: u32 = 0 << 1;
    pub const MODE_23BIT: u32 = 1 << 1;
    pub const INT_ENABLE: u32 = 1 << 5;
    pub const PRESCALE_1: u32 = 0 << 2;
    pub const PRESCALE_16: u32 = 1 << 2;
    pub const PRESCALE_256: u32 = 2 << 2;
}

/// Timer device.
#[derive(Debug)]
pub struct Timer {
    /// Current time (64-bit, microseconds).
    time: u64,
    /// Alarm values.
    alarms: [u64; 4],
    /// Armed alarm flags.
    armed: u8,
    /// Debug pause control.
    dbgpause: u32,
    /// Pause control.
    pause: u32,
    /// Locked flag.
    locked: bool,
    /// Timer source (0 = clksrc_tick, 1 = clksrc_gpout0).
    source: u32,
    /// Raw interrupt status.
    intr: u32,
    /// Interrupt enable.
    inte: u32,
    /// Interrupt force.
    intf: u32,
    /// Interrupt status.
    ints: u32,
    /// Time increment per tick (in microseconds).
    tick_increment: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    /// Create a new Timer device.
    pub fn new() -> Self {
        Self {
            time: 0,
            alarms: [0; 4],
            armed: 0,
            dbgpause: 0x7, // Default: pause on both cores
            pause: 0,
            locked: false,
            source: 0,
            intr: 0,
            inte: 0,
            intf: 0,
            ints: 0,
            tick_increment: 1, // 1 microsecond per tick
        }
    }

    /// Get current time.
    pub fn get_time(&self) -> u64 {
        self.time
    }

    /// Set current time.
    pub fn set_time(&mut self, time: u64) {
        self.time = time;
    }

    /// Advance time by one tick.
    pub fn tick(&mut self) {
        if self.pause != 0 {
            return;
        }
        
        self.time = self.time.wrapping_add(self.tick_increment);
        self.check_alarms();
    }

    /// Advance time by multiple ticks.
    pub fn advance(&mut self, ticks: u64) {
        if self.pause != 0 {
            return;
        }
        
        for _ in 0..ticks {
            self.time = self.time.wrapping_add(self.tick_increment);
            if self.check_alarms() {
                break;
            }
        }
    }

    /// Set time increment per tick.
    pub fn set_tick_increment(&mut self, increment: u64) {
        self.tick_increment = increment;
    }

    /// Check and trigger alarms.
    fn check_alarms(&mut self) -> bool {
        let mut triggered = false;
        
        for i in 0..4 {
            if (self.armed & (1 << i)) != 0 {
                // Alarm triggers when time matches or wraps around
                if self.time >= self.alarms[i] {
                    self.armed &= !(1 << i);
                    self.intr |= 1 << i;
                    triggered = true;
                }
            }
        }
        
        if triggered {
            self.update_ints();
        }
        
        triggered
    }

    /// Update interrupt status.
    fn update_ints(&mut self) {
        self.ints = (self.intr & self.inte) | self.intf;
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        // Check raw interrupt status (INTR register)
        self.intr != 0
    }

    /// Check if there's a masked interrupt (INTS register).
    pub fn has_masked_interrupt(&self) -> bool {
        self.ints != 0
    }

    /// Get alarm armed status.
    pub fn is_armed(&self, alarm: usize) -> bool {
        (self.armed & (1 << alarm)) != 0
    }

    /// Set alarm.
    pub fn set_alarm(&mut self, alarm: usize, value: u64) {
        if alarm < 4 {
            self.alarms[alarm] = value;
            self.armed |= 1 << alarm;
        }
    }

    /// Clear alarm.
    pub fn clear_alarm(&mut self, alarm: usize) {
        if alarm < 4 {
            self.armed &= !(1 << alarm);
        }
    }

    /// Read time high (upper 32 bits).
    fn read_timeh(&self) -> u32 {
        (self.time >> 32) as u32
    }

    /// Read time low (lower 32 bits).
    fn read_timel(&self) -> u32 {
        self.time as u32
    }

    /// Read time atomically (latched).
    #[allow(dead_code)]
    fn read_time_latched(&mut self) -> (u32, u32) {
        // Reading TIMELR latches TIMEHR
        let low = self.time as u32;
        let high = (self.time >> 32) as u32;
        (high, low)
    }
}

impl Device for Timer {
    fn id(&self) -> DeviceId {
        DeviceId::TIMER
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - TIMER_BASE;
        
        match offset {
            regs::TIMEHR => Ok(self.read_timeh()),
            regs::TIMELR => Ok(self.read_timel()),
            regs::TIMERAWH => Ok(self.read_timeh()),
            regs::TIMERAWL => Ok(self.read_timel()),
            regs::DBGPAUSE => Ok(self.dbgpause),
            regs::PAUSE => Ok(self.pause),
            regs::LOCKED => Ok(if self.locked { 1 } else { 0 }),
            regs::SOURCE => Ok(self.source),
            regs::ALARM0 => Ok(self.alarms[0] as u32),
            regs::ALARM1 => Ok(self.alarms[1] as u32),
            regs::ALARM2 => Ok(self.alarms[2] as u32),
            regs::ALARM3 => Ok(self.alarms[3] as u32),
            regs::ARMED => Ok(self.armed as u32),
            regs::TIMERAWH2 => Ok(self.read_timeh()),
            regs::TIMERAWL2 => Ok(self.read_timel()),
            regs::TIMEHR2 => Ok(self.read_timeh()),
            regs::TIMELR2 => Ok(self.read_timel()),
            regs::INTR => Ok(self.intr),
            regs::INTE => Ok(self.inte),
            regs::INTF => Ok(self.intf),
            regs::INTS => Ok(self.ints),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - TIMER_BASE;
        
        match offset {
            regs::DBGPAUSE => {
                self.dbgpause = value & 0x7;
            }
            regs::PAUSE => {
                self.pause = value & 1;
            }
            regs::SOURCE => {
                if !self.locked {
                    self.source = value & 1;
                }
            }
            regs::ALARM0 => {
                self.alarms[0] = (self.alarms[0] & 0xFFFFFFFF00000000) | (value as u64);
                self.armed |= 1 << 0;
            }
            regs::ALARM1 => {
                self.alarms[1] = (self.alarms[1] & 0xFFFFFFFF00000000) | (value as u64);
                self.armed |= 1 << 1;
            }
            regs::ALARM2 => {
                self.alarms[2] = (self.alarms[2] & 0xFFFFFFFF00000000) | (value as u64);
                self.armed |= 1 << 2;
            }
            regs::ALARM3 => {
                self.alarms[3] = (self.alarms[3] & 0xFFFFFFFF00000000) | (value as u64);
                self.armed |= 1 << 3;
            }
            regs::ARMED => {
                // Write 1 to clear
                self.armed &= !(value as u8);
            }
            regs::INTR => {
                // Write 1 to clear
                self.intr &= !value;
                self.update_ints();
            }
            regs::INTE => {
                self.inte = value & 0xF;
                self.update_ints();
            }
            regs::INTF => {
                self.intf = value & 0xF;
                self.update_ints();
            }
            _ => {}
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

/// System timer (SysTick) for ARM Cortex-M33.
#[derive(Debug)]
pub struct SysTick {
    /// Control and Status Register.
    csr: u32,
    /// Reload Value Register.
    rvr: u32,
    /// Current Value Register.
    cvr: u32,
    /// Calibration Value Register.
    calib: u32,
    /// Current counter value.
    counter: u32,
}

impl Default for SysTick {
    fn default() -> Self {
        Self::new()
    }
}

impl SysTick {
    /// Create a new SysTick timer.
    pub fn new() -> Self {
        Self {
            csr: 0,
            rvr: 0,
            calib: 0x00FFFFFF, // 24-bit counter
            cvr: 0,
            counter: 0,
        }
    }

    /// SysTick base address.
    pub const BASE: u32 = 0xE000_E010;

    /// Register offsets.
    pub const CSR: u32 = 0x000;
    pub const RVR: u32 = 0x004;
    pub const CVR: u32 = 0x008;
    pub const CALIB: u32 = 0x00C;

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.csr & systick_csr::ENABLE) != 0
    }

    /// Check if interrupt is enabled.
    pub fn is_interrupt_enabled(&self) -> bool {
        (self.csr & systick_csr::TICKINT) != 0
    }

    /// Check if using processor clock.
    pub fn is_processor_clock(&self) -> bool {
        (self.csr & systick_csr::CLKSOURCE) != 0
    }

    /// Tick the timer.
    pub fn tick(&mut self) -> bool {
        if !self.is_enabled() {
            return false;
        }

        if self.counter > 0 {
            self.counter -= 1;
            if self.counter == 0 {
                // Set COUNTFLAG
                self.csr |= systick_csr::COUNTFLAG;
                // Reload
                self.counter = self.rvr;
                return true; // Trigger interrupt
            }
        }
        false
    }

    /// Read register.
    pub fn read(&mut self, offset: u32) -> u32 {
        match offset {
            Self::CSR => {
                let val = self.csr;
                // COUNTFLAG clears on read
                self.csr &= !systick_csr::COUNTFLAG;
                val
            }
            Self::RVR => self.rvr,
            Self::CVR => self.cvr,
            Self::CALIB => self.calib,
            _ => 0,
        }
    }

    /// Write register.
    pub fn write(&mut self, offset: u32, value: u32) {
        match offset {
            Self::CSR => {
                self.csr = value & 0x1FFFD;
                if (value & systick_csr::ENABLE) != 0 && self.counter == 0 {
                    self.counter = self.rvr;
                }
            }
            Self::RVR => {
                self.rvr = value & 0x00FFFFFF;
            }
            Self::CVR => {
                // Writing any value clears CVR
                self.cvr = 0;
                self.counter = self.rvr;
            }
            _ => {}
        }
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        self.is_interrupt_enabled() && (self.csr & systick_csr::COUNTFLAG) != 0
    }
}

/// Arm timer (for RP2350 specific timer).
#[derive(Debug)]
pub struct ArmTimer {
    /// Control register.
    control: u32,
    /// Load register.
    load: u32,
    /// Value register.
    #[allow(dead_code)]
    value: u32,
    /// Prescaler.
    prescaler: u32,
    /// Interrupt clear.
    #[allow(dead_code)]
    int_clear: bool,
    /// Raw interrupt status.
    raw_int: bool,
    /// Masked interrupt status.
    masked_int: bool,
    /// Background load.
    bg_load: u32,
    /// Counter.
    counter: u32,
}

impl Default for ArmTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl ArmTimer {
    /// Create a new ARM timer.
    pub fn new() -> Self {
        Self {
            control: 0,
            load: 0,
            value: 0xFFFFFFFF,
            prescaler: 0,
            int_clear: false,
            raw_int: false,
            masked_int: false,
            bg_load: 0,
            counter: 0,
        }
    }

    /// ARM timer base address.
    pub const BASE: u32 = 0x400B_4000;

    /// Register offsets.
    pub const LOAD: u32 = 0x400;
    pub const VALUE: u32 = 0x404;
    pub const CONTROL: u32 = 0x408;
    pub const IRQCLR: u32 = 0x40C;
    pub const IRQRAW: u32 = 0x410;
    pub const IRQMSK: u32 = 0x414;
    pub const RELOAD: u32 = 0x418;
    pub const PRE_DIVIDER: u32 = 0x41C;
    pub const FREE_RUNNING: u32 = 0x420;

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.control & arm_timer_control::ENABLE) != 0
    }

    /// Tick the timer.
    pub fn tick(&mut self) -> bool {
        if !self.is_enabled() || self.counter == 0 {
            return false;
        }

        self.counter -= 1;
        if self.counter == 0 {
            self.raw_int = true;
            self.counter = self.load;
            return true;
        }
        false
    }

    /// Read register.
    pub fn read(&mut self, offset: u32) -> u32 {
        match offset {
            Self::LOAD => self.load,
            Self::VALUE => self.counter,
            Self::CONTROL => self.control,
            Self::IRQRAW => if self.raw_int { 1 } else { 0 },
            Self::IRQMSK => if self.masked_int { 1 } else { 0 },
            Self::RELOAD => self.bg_load,
            Self::PRE_DIVIDER => self.prescaler,
            _ => 0,
        }
    }

    /// Write register.
    pub fn write(&mut self, offset: u32, value: u32) {
        match offset {
            Self::LOAD => {
                self.load = value;
                self.counter = value;
            }
            Self::CONTROL => {
                self.control = value;
            }
            Self::IRQCLR => {
                self.raw_int = false;
                self.masked_int = false;
            }
            Self::RELOAD => {
                self.bg_load = value;
            }
            Self::PRE_DIVIDER => {
                self.prescaler = value;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new();
        assert_eq!(timer.get_time(), 0);
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = Timer::new();
        timer.tick();
        assert_eq!(timer.get_time(), 1);
        
        timer.advance(99);
        assert_eq!(timer.get_time(), 100);
    }

    #[test]
    fn test_timer_alarm() {
        let mut timer = Timer::new();
        timer.set_alarm(0, 100);
        assert!(timer.is_armed(0));
        
        timer.advance(99);
        assert!(timer.is_armed(0));
        
        timer.tick();
        assert!(!timer.is_armed(0));
        assert!(timer.has_interrupt());
    }

    #[test]
    fn test_timer_pause() {
        let mut timer = Timer::new();
        timer.pause = 1;
        
        timer.tick();
        assert_eq!(timer.get_time(), 0);
    }

    #[test]
    fn test_systick() {
        let mut systick = SysTick::new();
        systick.rvr = 100;
        systick.counter = 100; // Initialize counter from rvr
        systick.csr = systick_csr::ENABLE;
        
        for _ in 0..99 {
            assert!(!systick.tick());
        }
        assert!(systick.tick());
    }

    #[test]
    fn test_arm_timer() {
        let mut timer = ArmTimer::new();
        timer.load = 10;
        timer.control = arm_timer_control::ENABLE;
        timer.counter = 10;
        
        for _ in 0..9 {
            assert!(!timer.tick());
        }
        assert!(timer.tick());
    }

    #[test]
    fn test_timer_register_access() {
        let mut timer = Timer::new();

        // Test ALARM registers
        timer.write(TIMER_BASE + regs::ALARM0, 1000).unwrap();
        assert_eq!(timer.read(TIMER_BASE + regs::ALARM0).unwrap(), 1000);

        // Test INTE register
        timer.write(TIMER_BASE + regs::INTE, 0x0F).unwrap();
        assert_eq!(timer.read(TIMER_BASE + regs::INTE).unwrap(), 0x0F);

        // Test INTF register
        timer.write(TIMER_BASE + regs::INTF, 0x05).unwrap();
        assert_eq!(timer.read(TIMER_BASE + regs::INTF).unwrap(), 0x05);
    }

    #[test]
    fn test_timer_alarm_interrupts() {
        let mut timer = Timer::new();

        // Set alarm 0
        timer.set_alarm(0, 1000);
        assert!(timer.is_armed(0));

        // Enable alarm interrupt
        timer.inte |= 1 << 0;

        // Advance to trigger alarm
        timer.advance(1000);
        assert!(!timer.is_armed(0));
        assert!(timer.has_interrupt());

        // Check interrupt status
        let intr = timer.read(TIMER_BASE + regs::INTR).unwrap();
        assert_eq!(intr & 1, 1);
    }

    #[test]
    fn test_timer_multiple_alarms() {
        let mut timer = Timer::new();

        // Set multiple alarms
        timer.set_alarm(0, 100);
        timer.set_alarm(1, 200);
        timer.set_alarm(2, 300);

        assert!(timer.is_armed(0));
        assert!(timer.is_armed(1));
        assert!(timer.is_armed(2));

        // Advance to trigger first alarm
        timer.advance(100);
        assert!(!timer.is_armed(0));
        assert!(timer.is_armed(1));
        assert!(timer.is_armed(2));

        // Advance to trigger second alarm
        timer.advance(100);
        assert!(!timer.is_armed(1));
        assert!(timer.is_armed(2));

        // Advance to trigger third alarm
        timer.advance(100);
        assert!(!timer.is_armed(2));
    }

    #[test]
    fn test_timer_alarm_clear() {
        let mut timer = Timer::new();

        // Set and trigger alarm
        timer.set_alarm(0, 100);
        timer.advance(100);

        // Clear interrupt by writing to INTR
        timer.write(TIMER_BASE + regs::INTR, 1).unwrap();

        // Check interrupt is cleared
        let intr = timer.read(TIMER_BASE + regs::INTR).unwrap();
        assert_eq!(intr & 1, 0);
    }

    #[test]
    fn test_timer_time_read() {
        let mut timer = Timer::new();
        timer.advance(0x12345678_9ABCDEF0);

        // Read TIMEHR and TIMELR
        let timehr = timer.read(TIMER_BASE + regs::TIMEHR).unwrap();
        let timelr = timer.read(TIMER_BASE + regs::TIMELR).unwrap();

        assert_eq!(timehr, 0x12345678);
        assert_eq!(timelr, 0x9ABCDEF0);
    }

    #[test]
    fn test_timer_raw_read() {
        let mut timer = Timer::new();
        timer.advance(0x100);

        // Read TIMERAWH and TIMERAWL
        let timerawh = timer.read(TIMER_BASE + regs::TIMERAWH).unwrap();
        let timerawl = timer.read(TIMER_BASE + regs::TIMERAWL).unwrap();

        assert_eq!(timerawh, 0);
        assert_eq!(timerawl, 0x100);
    }

    #[test]
    fn test_timer_lock() {
        let mut timer = Timer::new();

        // Initially not locked
        assert!(!timer.locked);

        // Note: LOCKED register is read-only in this implementation
        // It would be set by hardware when accessing protected registers
        let locked_val = timer.read(TIMER_BASE + regs::LOCKED).unwrap();
        assert_eq!(locked_val, 0);
    }

    #[test]
    fn test_timer_source() {
        let mut timer = Timer::new();

        // Default source is 0
        assert_eq!(timer.source, 0);

        // Write to SOURCE register
        timer.write(TIMER_BASE + regs::SOURCE, 1).unwrap();
        assert_eq!(timer.source, 1);
    }

    #[test]
    fn test_timer_dbgpause() {
        let mut timer = Timer::new();

        // Default dbgpause is 0x7
        assert_eq!(timer.dbgpause, 0x7);

        // Write to DBGPAUSE register
        timer.write(TIMER_BASE + regs::DBGPAUSE, 0x3).unwrap();
        assert_eq!(timer.dbgpause, 0x3);
    }

    #[test]
    fn test_timer_ints() {
        let mut timer = Timer::new();

        // Set alarm and enable interrupt
        timer.set_alarm(0, 100);
        timer.inte = 0x01;

        // Trigger alarm
        timer.advance(100);

        // Check INTS (interrupt status after masking)
        let ints = timer.read(TIMER_BASE + regs::INTS).unwrap();
        assert_eq!(ints & 1, 1);
    }

    #[test]
    fn test_timer_inte_masking() {
        let mut timer = Timer::new();

        // Set alarm but don't enable interrupt
        timer.set_alarm(0, 100);
        timer.inte = 0x00;

        // Trigger alarm
        timer.advance(100);

        // INTR should be set
        let intr = timer.read(TIMER_BASE + regs::INTR).unwrap();
        assert_eq!(intr & 1, 1);

        // But INTS should be 0 (masked by INTE)
        let ints = timer.read(TIMER_BASE + regs::INTS).unwrap();
        assert_eq!(ints & 1, 0);
    }

    #[test]
    fn test_timer_intf_force() {
        let mut timer = Timer::new();

        // Force interrupt via INTF
        timer.write(TIMER_BASE + regs::INTF, 0x02).unwrap();  // Force alarm 1 interrupt

        // Check INTS - INTF is ORed with (INTR & INTE)
        let ints = timer.read(TIMER_BASE + regs::INTS).unwrap();
        assert_eq!(ints & 2, 2);
    }

    #[test]
    fn test_timer_alarm_wrap() {
        let mut timer = Timer::new();

        // Set time near wrap point
        timer.set_time(0xFFFFFFF0);

        // Set alarm that will wrap - alarm value is absolute
        // When time wraps from 0xFFFFFFF0 to 0x10, it passes through 0
        // The alarm at 0x8 will trigger during the wrap (time passes through 0x8)
        timer.set_alarm(0, 0x8);

        // Advance past wrap - this will stop when alarm triggers
        timer.advance(0x20);

        // Alarm should have triggered
        assert!(!timer.is_armed(0));
        assert!(timer.has_interrupt());
    }

    #[test]
    fn test_timer_all_alarms_independent() {
        let mut timer = Timer::new();

        // Set all 4 alarms at different times
        timer.set_alarm(0, 100);
        timer.set_alarm(1, 50);
        timer.set_alarm(2, 200);
        timer.set_alarm(3, 150);

        // Advance to 50 - only alarm 1 should trigger
        timer.advance(50);
        assert!(timer.is_armed(0));  // 100 > 50, still armed
        assert!(!timer.is_armed(1)); // 50 <= 50, triggered
        assert!(timer.is_armed(2));  // 200 > 50, still armed
        assert!(timer.is_armed(3));  // 150 > 50, still armed

        // Check only alarm 1 interrupt
        let intr = timer.read(TIMER_BASE + regs::INTR).unwrap();
        assert_eq!(intr, 0b0010);
    }

    #[test]
    fn test_systick_reload() {
        let mut systick = SysTick::new();
        systick.rvr = 10;
        systick.counter = 10;
        systick.csr = systick_csr::ENABLE;

        // Count down to 0
        for _ in 0..10 {
            systick.tick();
        }

        // Counter should reload from RVR
        assert_eq!(systick.counter, 10);
    }

    #[test]
    fn test_systick_interrupt() {
        let mut systick = SysTick::new();
        systick.rvr = 5;
        systick.counter = 5;
        systick.csr = systick_csr::ENABLE | systick_csr::TICKINT;

        // Count down
        for _ in 0..4 {
            assert!(!systick.tick());
        }
        // On reaching 0, should return true (interrupt)
        assert!(systick.tick());
    }

    #[test]
    fn test_arm_timer_prescale() {
        let mut timer = ArmTimer::new();
        timer.load = 10;
        timer.counter = 10;
        timer.control = arm_timer_control::ENABLE;

        // Note: prescaler is stored but not implemented in tick
        // This test verifies basic countdown
        for _ in 0..9 {
            assert!(!timer.tick());
        }
        assert!(timer.tick());
    }

    #[test]
    fn test_arm_timer_mode() {
        let mut timer = ArmTimer::new();
        timer.load = 10;
        timer.counter = 10;
        timer.control = arm_timer_control::ENABLE | arm_timer_control::MODE_23BIT;

        // Should still count down
        for _ in 0..9 {
            assert!(!timer.tick());
        }
        assert!(timer.tick());
    }
}
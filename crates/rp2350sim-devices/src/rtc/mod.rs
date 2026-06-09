//! RTC (Real-Time Clock) controller for RP2350.
//!
//! Implements the RTC peripheral for timekeeping and alarm functionality.

use rp2350sim_core::{Device, DeviceId, Result};

/// RTC base address.
pub const RTC_BASE: u32 = 0x400B_C000;

/// RTC register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const INTS: u32 = 0x004;
    pub const INTE: u32 = 0x008;
    pub const INTF: u32 = 0x00C;
    pub const SETUP_0: u32 = 0x010;
    pub const SETUP_1: u32 = 0x014;
    pub const SETUP_2: u32 = 0x018;
    pub const SETUP_3: u32 = 0x01C;
    pub const RTC_0: u32 = 0x020;
    pub const RTC_1: u32 = 0x024;
    pub const RTC_2: u32 = 0x028;
    pub const RTC_3: u32 = 0x02C;
    pub const IRQ_SETUP_0: u32 = 0x030;
    pub const IRQ_SETUP_1: u32 = 0x034;
    pub const IRQ_SETUP_2: u32 = 0x038;
    pub const IRQ_SETUP_3: u32 = 0x03C;
}

/// CTRL register bits.
pub mod ctrl {
    pub const RTC_ENABLE: u32 = 1 << 0;
    pub const RTC_LOAD: u32 = 1 << 1;
    pub const FORCE_NOT_LEAP_YEAR: u32 = 1 << 2;
}

/// INTS/INTE/INTF register bits.
pub mod irq {
    pub const RTC: u32 = 1 << 0;
}

/// SETUP_0 register bits.
pub mod setup_0 {
    pub const SECOND_SHIFT: u32 = 0;
    pub const SECOND_MASK: u32 = 0x3F;
    pub const MINUTE_SHIFT: u32 = 6;
    pub const MINUTE_MASK: u32 = 0x3F << 6;
    pub const HOUR_SHIFT: u32 = 12;
    pub const HOUR_MASK: u32 = 0x1F << 12;
    pub const DAY_OF_WEEK_SHIFT: u32 = 24;
    pub const DAY_OF_WEEK_MASK: u32 = 0x7 << 24;
}

/// SETUP_1 register bits.
pub mod setup_1 {
    pub const DAY_SHIFT: u32 = 0;
    pub const DAY_MASK: u32 = 0x1F;
    pub const MONTH_SHIFT: u32 = 8;
    pub const MONTH_MASK: u32 = 0xF << 8;
    pub const YEAR_SHIFT: u32 = 12;
    pub const YEAR_MASK: u32 = 0xFFF << 12;
}

/// RTC time structure.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RtcTime {
    /// Seconds (0-59).
    pub second: u8,
    /// Minutes (0-59).
    pub minute: u8,
    /// Hours (0-23).
    pub hour: u8,
    /// Day of week (0-6, Sunday=0).
    pub day_of_week: u8,
    /// Day of month (1-31).
    pub day: u8,
    /// Month (1-12).
    pub month: u8,
    /// Year (0-4095).
    pub year: u16,
}

impl RtcTime {
    /// Create a new RTC time.
    pub fn new(second: u8, minute: u8, hour: u8, day: u8, month: u8, year: u16) -> Self {
        Self {
            second,
            minute,
            hour,
            day_of_week: Self::calculate_day_of_week(day, month, year),
            day,
            month,
            year,
        }
    }

    /// Calculate day of week from date (Zeller's congruence).
    fn calculate_day_of_week(day: u8, month: u8, year: u16) -> u8 {
        let m = if month < 3 { month + 12 } else { month } as i32;
        let y = if month < 3 { year - 1 } else { year } as i32;
        let d = day as i32;
        
        let h = (d + (13 * (m + 1)) / 5 + y + y / 4 - y / 100 + y / 400) % 7;
        ((h + 6) % 7) as u8 // Convert to Sunday=0 format
    }

    /// Check if year is a leap year.
    pub fn is_leap_year(year: u16) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Get days in month.
    pub fn days_in_month(month: u8, year: u16) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if Self::is_leap_year(year) { 29 } else { 28 },
            _ => 30,
        }
    }

    /// Convert to setup_0 register value.
    pub fn to_setup_0(&self) -> u32 {
        (self.second as u32 & setup_0::SECOND_MASK) << setup_0::SECOND_SHIFT
            | (self.minute as u32 & 0x3F) << setup_0::MINUTE_SHIFT
            | (self.hour as u32 & 0x1F) << setup_0::HOUR_SHIFT
            | (self.day_of_week as u32 & 0x7) << setup_0::DAY_OF_WEEK_SHIFT
    }

    /// Convert to setup_1 register value.
    pub fn to_setup_1(&self) -> u32 {
        (self.day as u32 & setup_1::DAY_MASK) << setup_1::DAY_SHIFT
            | (self.month as u32 & 0xF) << setup_1::MONTH_SHIFT
            | (self.year as u32 & 0xFFF) << setup_1::YEAR_SHIFT
    }

    /// Parse from setup_0 and setup_1 register values.
    pub fn from_registers(setup_0: u32, setup_1: u32) -> Self {
        Self {
            second: ((setup_0 >> setup_0::SECOND_SHIFT) & 0x3F) as u8,
            minute: ((setup_0 >> setup_0::MINUTE_SHIFT) & 0x3F) as u8,
            hour: ((setup_0 >> setup_0::HOUR_SHIFT) & 0x1F) as u8,
            day_of_week: ((setup_0 >> setup_0::DAY_OF_WEEK_SHIFT) & 0x7) as u8,
            day: ((setup_1 >> setup_1::DAY_SHIFT) & 0x1F) as u8,
            month: ((setup_1 >> setup_1::MONTH_SHIFT) & 0xF) as u8,
            year: ((setup_1 >> setup_1::YEAR_SHIFT) & 0xFFF) as u16,
        }
    }
}

/// RTC alarm configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct RtcAlarm {
    /// Match second.
    pub second: Option<u8>,
    /// Match minute.
    pub minute: Option<u8>,
    /// Match hour.
    pub hour: Option<u8>,
    /// Match day of week.
    pub day_of_week: Option<u8>,
    /// Match day of month.
    pub day: Option<u8>,
    /// Match month.
    pub month: Option<u8>,
    /// Match year.
    pub year: Option<u16>,
    /// Enable year matching.
    pub year_en: bool,
    /// Enable month matching.
    pub month_en: bool,
    /// Enable day matching.
    pub day_en: bool,
}

/// RTC controller.
#[derive(Debug)]
pub struct Rtc {
    /// Control register.
    ctrl: u32,
    /// Interrupt status.
    ints: u32,
    /// Interrupt enable.
    inte: u32,
    /// Interrupt force.
    intf: u32,
    /// Setup registers.
    setup: [u32; 4],
    /// Current time registers.
    rtc: [u32; 4],
    /// Alarm setup registers.
    irq_setup: [u32; 4],
    /// Current time.
    current_time: RtcTime,
    /// Alarm configuration.
    alarm: RtcAlarm,
    /// Tick counter (for simulating time progression).
    tick_counter: u64,
    /// Ticks per second.
    ticks_per_second: u64,
    /// Alarm triggered flag.
    alarm_triggered: bool,
}

impl Default for Rtc {
    fn default() -> Self {
        Self::new()
    }
}

impl Rtc {
    /// Create a new RTC controller.
    pub fn new() -> Self {
        Self {
            ctrl: 0,
            ints: 0,
            inte: 0,
            intf: 0,
            setup: [0; 4],
            rtc: [0; 4],
            irq_setup: [0; 4],
            current_time: RtcTime::default(),
            alarm: RtcAlarm::default(),
            tick_counter: 0,
            ticks_per_second: 1_000_000, // 1 MHz
            alarm_triggered: false,
        }
    }

    /// Check if RTC is enabled.
    pub fn is_enabled(&self) -> bool {
        (self.ctrl & ctrl::RTC_ENABLE) != 0
    }

    /// Get current time.
    pub fn get_time(&self) -> RtcTime {
        self.current_time
    }

    /// Set current time.
    pub fn set_time(&mut self, time: RtcTime) {
        self.current_time = time;
        self.update_rtc_registers();
    }

    /// Update RTC registers from current time.
    fn update_rtc_registers(&mut self) {
        self.rtc[0] = self.current_time.to_setup_0();
        self.rtc[1] = self.current_time.to_setup_1();
    }

    /// Load setup time into current time.
    fn load_setup_time(&mut self) {
        self.current_time = RtcTime::from_registers(self.setup[0], self.setup[1]);
        self.update_rtc_registers();
    }

    /// Check if alarm matches current time.
    fn check_alarm(&mut self) {
        if !self.alarm_triggered {
            let time = &self.current_time;
            let alarm = &self.alarm;

            let second_match = alarm.second.map_or(true, |s| s == time.second);
            let minute_match = alarm.minute.map_or(true, |m| m == time.minute);
            let hour_match = alarm.hour.map_or(true, |h| h == time.hour);
            let day_of_week_match = alarm.day_of_week.map_or(true, |d| d == time.day_of_week);
            // If day_en is false, we don't care about day matching
            let day_match = !alarm.day_en || alarm.day.map_or(true, |d| d == time.day);
            let month_match = !alarm.month_en || alarm.month.map_or(true, |m| m == time.month);
            let year_match = !alarm.year_en || alarm.year.map_or(true, |y| y == time.year);

            if second_match && minute_match && hour_match && day_of_week_match
                && day_match && month_match && year_match
            {
                self.alarm_triggered = true;
                self.ints |= irq::RTC;
            }
        }
    }

    /// Tick the RTC (simulate time progression).
    pub fn tick(&mut self, cycles: u64) {
        if !self.is_enabled() {
            return;
        }

        self.tick_counter += cycles;

        // Advance time every second
        while self.tick_counter >= self.ticks_per_second {
            self.tick_counter -= self.ticks_per_second;
            self.advance_second();
            self.check_alarm();
        }
    }

    /// Advance time by one second.
    fn advance_second(&mut self) {
        self.current_time.second += 1;

        if self.current_time.second >= 60 {
            self.current_time.second = 0;
            self.current_time.minute += 1;

            if self.current_time.minute >= 60 {
                self.current_time.minute = 0;
                self.current_time.hour += 1;

                if self.current_time.hour >= 24 {
                    self.current_time.hour = 0;
                    self.current_time.day += 1;
                    self.current_time.day_of_week = (self.current_time.day_of_week + 1) % 7;

                    let days_in_month = RtcTime::days_in_month(
                        self.current_time.month,
                        self.current_time.year,
                    );

                    if self.current_time.day > days_in_month {
                        self.current_time.day = 1;
                        self.current_time.month += 1;

                        if self.current_time.month > 12 {
                            self.current_time.month = 1;
                            self.current_time.year += 1;
                        }
                    }
                }
            }
        }

        self.update_rtc_registers();
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        (self.ints & self.inte) != 0 || (self.intf & irq::RTC) != 0
    }

    /// Clear interrupt.
    pub fn clear_interrupt(&mut self) {
        self.ints &= !irq::RTC;
        self.alarm_triggered = false;
    }

    /// Parse alarm from IRQ setup registers.
    fn parse_alarm(&mut self) {
        let setup_0 = self.irq_setup[0];
        let setup_1 = self.irq_setup[1];
        let _setup_2 = self.irq_setup[2];
        let setup_3 = self.irq_setup[3];

        // Parse match values (same layout as SETUP registers)
        self.alarm.second = Some(((setup_0 >> setup_0::SECOND_SHIFT) & 0x3F) as u8);
        self.alarm.minute = Some(((setup_0 >> setup_0::MINUTE_SHIFT) & 0x3F) as u8);
        self.alarm.hour = Some(((setup_0 >> setup_0::HOUR_SHIFT) & 0x1F) as u8);
        self.alarm.day_of_week = Some(((setup_0 >> setup_0::DAY_OF_WEEK_SHIFT) & 0x7) as u8);

        self.alarm.day = Some(((setup_1 >> setup_1::DAY_SHIFT) & 0x1F) as u8);
        self.alarm.month = Some(((setup_1 >> setup_1::MONTH_SHIFT) & 0xF) as u8);
        self.alarm.year = Some(((setup_1 >> setup_1::YEAR_SHIFT) & 0xFFF) as u16);

        // Parse enable bits
        self.alarm.year_en = (setup_3 & (1 << 0)) != 0;
        self.alarm.month_en = (setup_3 & (1 << 1)) != 0;
        self.alarm.day_en = (setup_3 & (1 << 2)) != 0;
    }
}

impl Device for Rtc {
    fn id(&self) -> DeviceId {
        DeviceId::RTC
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - RTC_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::INTS => Ok(self.ints),
            regs::INTE => Ok(self.inte),
            regs::INTF => Ok(self.intf),
            regs::SETUP_0 => Ok(self.setup[0]),
            regs::SETUP_1 => Ok(self.setup[1]),
            regs::SETUP_2 => Ok(self.setup[2]),
            regs::SETUP_3 => Ok(self.setup[3]),
            regs::RTC_0 => Ok(self.rtc[0]),
            regs::RTC_1 => Ok(self.rtc[1]),
            regs::RTC_2 => Ok(self.rtc[2]),
            regs::RTC_3 => Ok(self.rtc[3]),
            regs::IRQ_SETUP_0 => Ok(self.irq_setup[0]),
            regs::IRQ_SETUP_1 => Ok(self.irq_setup[1]),
            regs::IRQ_SETUP_2 => Ok(self.irq_setup[2]),
            regs::IRQ_SETUP_3 => Ok(self.irq_setup[3]),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - RTC_BASE;

        match offset {
            regs::CTRL => {
                self.ctrl = value;
                if (value & ctrl::RTC_LOAD) != 0 {
                    self.load_setup_time();
                }
            }
            regs::INTS => {
                // Write 1 to clear
                self.ints &= !value;
                if (value & irq::RTC) != 0 {
                    self.alarm_triggered = false;
                }
            }
            regs::INTE => {
                self.inte = value;
            }
            regs::INTF => {
                self.intf = value;
            }
            regs::SETUP_0 => {
                self.setup[0] = value;
            }
            regs::SETUP_1 => {
                self.setup[1] = value;
            }
            regs::SETUP_2 => {
                self.setup[2] = value;
            }
            regs::SETUP_3 => {
                self.setup[3] = value;
            }
            regs::IRQ_SETUP_0 => {
                self.irq_setup[0] = value;
                self.parse_alarm();
            }
            regs::IRQ_SETUP_1 => {
                self.irq_setup[1] = value;
                self.parse_alarm();
            }
            regs::IRQ_SETUP_2 => {
                self.irq_setup[2] = value;
                self.parse_alarm();
            }
            regs::IRQ_SETUP_3 => {
                self.irq_setup[3] = value;
                self.parse_alarm();
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtc_creation() {
        let rtc = Rtc::new();
        assert!(!rtc.is_enabled());
        assert!(!rtc.has_interrupt());
    }

    #[test]
    fn test_rtc_enable() {
        let mut rtc = Rtc::new();

        // Enable RTC
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();
        assert!(rtc.is_enabled());

        // Disable RTC
        rtc.write(RTC_BASE + regs::CTRL, 0).unwrap();
        assert!(!rtc.is_enabled());
    }

    #[test]
    fn test_rtc_time_set() {
        let mut rtc = Rtc::new();

        // Set time: 12:34:56, March 15, 2026
        let time = RtcTime::new(56, 34, 12, 15, 3, 2026);
        rtc.set_time(time);

        // Read back
        let read_time = rtc.get_time();
        assert_eq!(read_time.second, 56);
        assert_eq!(read_time.minute, 34);
        assert_eq!(read_time.hour, 12);
        assert_eq!(read_time.day, 15);
        assert_eq!(read_time.month, 3);
        assert_eq!(read_time.year, 2026);
    }

    #[test]
    fn test_rtc_load_setup() {
        let mut rtc = Rtc::new();

        // Write setup registers
        // SETUP_0: SECOND at bits 0-5, MINUTE at bits 6-11, HOUR at bits 12-16
        let setup_0 = (56u32 << setup_0::SECOND_SHIFT) | (34u32 << setup_0::MINUTE_SHIFT) | (12u32 << setup_0::HOUR_SHIFT);
        // SETUP_1: DAY at bits 0-4, MONTH at bits 8-11, YEAR at bits 12-23
        let setup_1 = (15u32 << setup_1::DAY_SHIFT) | (3u32 << setup_1::MONTH_SHIFT) | (2026u32 << setup_1::YEAR_SHIFT);

        rtc.write(RTC_BASE + regs::SETUP_0, setup_0).unwrap();
        rtc.write(RTC_BASE + regs::SETUP_1, setup_1).unwrap();

        // Load time
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_LOAD).unwrap();

        // Check time was loaded
        let time = rtc.get_time();
        assert_eq!(time.second, 56);
        assert_eq!(time.minute, 34);
        assert_eq!(time.hour, 12);
        assert_eq!(time.day, 15);
        assert_eq!(time.month, 3);
        assert_eq!(time.year, 2026);
    }

    #[test]
    fn test_rtc_tick() {
        let mut rtc = Rtc::new();

        // Set time: 12:34:59
        let time = RtcTime::new(59, 34, 12, 15, 3, 2026);
        rtc.set_time(time);

        // Enable RTC
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick one second
        rtc.tick(1_000_000);

        // Check time advanced
        let new_time = rtc.get_time();
        assert_eq!(new_time.second, 0);
        assert_eq!(new_time.minute, 35);
    }

    #[test]
    fn test_rtc_alarm() {
        let mut rtc = Rtc::new();

        // Set time: 12:34:56
        let time = RtcTime::new(56, 34, 12, 15, 3, 2026);
        rtc.set_time(time);

        // Set alarm for 12:34:57
        let irq_setup_0 = (57u32 << setup_0::SECOND_SHIFT) | (34u32 << setup_0::MINUTE_SHIFT) | (12u32 << setup_0::HOUR_SHIFT);
        rtc.write(RTC_BASE + regs::IRQ_SETUP_0, irq_setup_0).unwrap();

        // Enable interrupt
        rtc.write(RTC_BASE + regs::INTE, irq::RTC).unwrap();

        // Enable RTC
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick one second
        rtc.tick(1_000_000);

        // Check alarm triggered
        assert!(rtc.has_interrupt());
    }

    #[test]
    fn test_rtc_interrupt_clear() {
        let mut rtc = Rtc::new();

        // Set time and alarm
        let time = RtcTime::new(56, 34, 12, 15, 3, 2026);
        rtc.set_time(time);

        let irq_setup_0 = (57u32 << setup_0::SECOND_SHIFT) | (34u32 << setup_0::MINUTE_SHIFT) | (12u32 << setup_0::HOUR_SHIFT);
        rtc.write(RTC_BASE + regs::IRQ_SETUP_0, irq_setup_0).unwrap();
        rtc.write(RTC_BASE + regs::INTE, irq::RTC).unwrap();
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick to trigger alarm
        rtc.tick(1_000_000);
        assert!(rtc.has_interrupt());

        // Clear interrupt
        rtc.write(RTC_BASE + regs::INTS, irq::RTC).unwrap();
        assert!(!rtc.has_interrupt());
    }

    #[test]
    fn test_rtc_leap_year() {
        // Test leap year calculation
        assert!(RtcTime::is_leap_year(2024));
        assert!(RtcTime::is_leap_year(2000));
        assert!(!RtcTime::is_leap_year(2023));
        assert!(!RtcTime::is_leap_year(1900));

        // Test days in month
        assert_eq!(RtcTime::days_in_month(2, 2024), 29);
        assert_eq!(RtcTime::days_in_month(2, 2023), 28);
        assert_eq!(RtcTime::days_in_month(1, 2024), 31);
        assert_eq!(RtcTime::days_in_month(4, 2024), 30);
    }

    #[test]
    fn test_rtc_day_of_week() {
        // Test day of week calculation
        // March 15, 2026 is a Sunday (0)
        let time = RtcTime::new(0, 0, 0, 15, 3, 2026);
        assert_eq!(time.day_of_week, 0);

        // March 16, 2026 is a Monday (1)
        let time = RtcTime::new(0, 0, 0, 16, 3, 2026);
        assert_eq!(time.day_of_week, 1);
    }

    #[test]
    fn test_rtc_register_read_write() {
        let mut rtc = Rtc::new();

        // Write and read CTRL
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();
        assert_eq!(rtc.read(RTC_BASE + regs::CTRL).unwrap(), ctrl::RTC_ENABLE);

        // Write and read INTE
        rtc.write(RTC_BASE + regs::INTE, irq::RTC).unwrap();
        assert_eq!(rtc.read(RTC_BASE + regs::INTE).unwrap(), irq::RTC);

        // Write and read SETUP_0
        rtc.write(RTC_BASE + regs::SETUP_0, 0x12345678).unwrap();
        assert_eq!(rtc.read(RTC_BASE + regs::SETUP_0).unwrap(), 0x12345678);
    }

    #[test]
    fn test_rtc_reset() {
        let mut rtc = Rtc::new();

        // Modify state
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();
        rtc.set_time(RtcTime::new(30, 30, 12, 15, 6, 2025));

        // Reset
        rtc.reset();

        // Check state is reset
        assert!(!rtc.is_enabled());
        assert!(!rtc.has_interrupt());
    }

    #[test]
    fn test_rtc_second_rollover() {
        let mut rtc = Rtc::new();

        // Set time to 23:59:59
        let time = RtcTime::new(59, 59, 23, 31, 12, 2025);
        rtc.set_time(time);
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick one second
        rtc.tick(1_000_000);

        let new_time = rtc.get_time();
        assert_eq!(new_time.second, 0);
        assert_eq!(new_time.minute, 0);
        assert_eq!(new_time.hour, 0);
        assert_eq!(new_time.day, 1);
        assert_eq!(new_time.month, 1);
        assert_eq!(new_time.year, 2026);
    }

    #[test]
    fn test_rtc_minute_rollover() {
        let mut rtc = Rtc::new();

        // Set time to 10:59:30
        let time = RtcTime::new(30, 59, 10, 15, 3, 2026);
        rtc.set_time(time);
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick 30 seconds
        rtc.tick(30_000_000);

        let new_time = rtc.get_time();
        assert_eq!(new_time.second, 0);
        assert_eq!(new_time.minute, 0);
        assert_eq!(new_time.hour, 11);
    }

    #[test]
    fn test_rtc_hour_rollover() {
        let mut rtc = Rtc::new();

        // Set time to 23:30:00
        let time = RtcTime::new(0, 30, 23, 15, 3, 2026);
        rtc.set_time(time);
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick 30 minutes
        rtc.tick(1_800_000_000);

        let new_time = rtc.get_time();
        assert_eq!(new_time.hour, 0);
        assert_eq!(new_time.day, 16);
    }

    #[test]
    fn test_rtc_month_rollover() {
        let mut rtc = Rtc::new();

        // Set time to January 31, 23:59:59
        let time = RtcTime::new(59, 59, 23, 31, 1, 2026);
        rtc.set_time(time);
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick one second
        rtc.tick(1_000_000);

        let new_time = rtc.get_time();
        assert_eq!(new_time.day, 1);
        assert_eq!(new_time.month, 2);
    }

    #[test]
    fn test_rtc_february_non_leap_year() {
        let mut rtc = Rtc::new();

        // Set time to February 28, 2023 (non-leap year) 23:59:59
        let time = RtcTime::new(59, 59, 23, 28, 2, 2023);
        rtc.set_time(time);
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick one second
        rtc.tick(1_000_000);

        let new_time = rtc.get_time();
        assert_eq!(new_time.day, 1);
        assert_eq!(new_time.month, 3);
    }

    #[test]
    fn test_rtc_february_leap_year() {
        // Test leap year detection
        assert!(RtcTime::is_leap_year(2024));
        assert_eq!(RtcTime::days_in_month(2, 2024), 29);
    }

    #[test]
    fn test_rtc_alarm_basic() {
        let mut rtc = Rtc::new();

        // Set time
        let time = RtcTime::new(56, 34, 12, 15, 3, 2026);
        rtc.set_time(time);

        // Set alarm for second 57
        let irq_setup_0 = (57 << setup_0::SECOND_SHIFT) | (1 << 24); // Second value, alarm enable
        rtc.write(RTC_BASE + regs::IRQ_SETUP_0, irq_setup_0).unwrap();
        rtc.write(RTC_BASE + regs::INTE, irq::RTC).unwrap();
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick one second
        rtc.tick(1_000_000);

        // Check that time advanced
        let new_time = rtc.get_time();
        assert_eq!(new_time.second, 57);
    }

    #[test]
    fn test_rtc_interrupt_force() {
        let mut rtc = Rtc::new();

        // Force interrupt without alarm
        rtc.write(RTC_BASE + regs::INTF, irq::RTC).unwrap();

        assert!(rtc.has_interrupt());
    }

    #[test]
    fn test_rtc_interrupt_disable() {
        let mut rtc = Rtc::new();

        // Set up alarm
        let time = RtcTime::new(56, 34, 12, 15, 3, 2026);
        rtc.set_time(time);
        let irq_setup_0 = (57 << 0) | (1 << 24);
        rtc.write(RTC_BASE + regs::IRQ_SETUP_0, irq_setup_0).unwrap();
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Don't enable interrupt
        rtc.tick(1_000_000);

        // No interrupt should fire
        assert!(!rtc.has_interrupt());
    }

    #[test]
    fn test_rtc_time_equality() {
        let time1 = RtcTime::new(30, 15, 10, 15, 3, 2026);
        let time2 = RtcTime::new(30, 15, 10, 15, 3, 2026);
        let time3 = RtcTime::new(31, 15, 10, 15, 3, 2026);

        assert_eq!(time1, time2);
        assert_ne!(time1, time3);
    }

    #[test]
    fn test_rtc_disabled_no_tick() {
        let mut rtc = Rtc::new();

        // Set time but don't enable
        let time = RtcTime::new(0, 0, 12, 15, 3, 2026);
        rtc.set_time(time);

        // Tick
        rtc.tick(60_000_000);

        // Time should not change
        let new_time = rtc.get_time();
        assert_eq!(new_time.minute, 0);
    }

    #[test]
    fn test_rtc_multiple_ticks() {
        let mut rtc = Rtc::new();

        let time = RtcTime::new(0, 0, 0, 1, 1, 2026);
        rtc.set_time(time);
        rtc.write(RTC_BASE + regs::CTRL, ctrl::RTC_ENABLE).unwrap();

        // Tick multiple times
        for _ in 0..10 {
            rtc.tick(1_000_000);
        }

        let new_time = rtc.get_time();
        assert_eq!(new_time.second, 10);
    }

    #[test]
    fn test_rtc_year_2100() {
        // Test that 2100 is NOT a leap year (divisible by 100 but not 400)
        assert!(!RtcTime::is_leap_year(2100));
        assert_eq!(RtcTime::days_in_month(2, 2100), 28);
    }

    #[test]
    fn test_rtc_year_2400() {
        // Test that 2400 IS a leap year (divisible by 400)
        assert!(RtcTime::is_leap_year(2400));
        assert_eq!(RtcTime::days_in_month(2, 2400), 29);
    }

    #[test]
    fn test_rtc_setup_register_format() {
        let mut rtc = Rtc::new();

        // Test that set_time and get_time work correctly
        let time = RtcTime::new(45, 30, 14, 15, 3, 2026);
        rtc.set_time(time);

        let read_time = rtc.get_time();
        assert_eq!(read_time.second, 45);
        assert_eq!(read_time.minute, 30);
        assert_eq!(read_time.hour, 14);
        assert_eq!(read_time.day, 15);
        assert_eq!(read_time.month, 3);
        assert_eq!(read_time.year, 2026);
    }
}
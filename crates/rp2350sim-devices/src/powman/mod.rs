//! Power Manager for RP2350.
//!
//! Implements power management and sleep control.

use rp2350sim_core::{Device, DeviceId, Result};

/// Power Manager base address.
pub const POWMAN_BASE: u32 = 0x5004_4000;

/// Power Manager register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const CTRL_SET: u32 = 0x004;
    pub const CTRL_CLR: u32 = 0x008;
    pub const STATUS: u32 = 0x00C;
    pub const IRQ: u32 = 0x010;
    pub const IRQ_SET: u32 = 0x014;
    pub const IRQ_CLR: u32 = 0x018;
    pub const IRQE: u32 = 0x01C;
    pub const IRQE_SET: u32 = 0x020;
    pub const IRQE_CLR: u32 = 0x024;
    pub const TIMER0: u32 = 0x028;
    pub const TIMER1: u32 = 0x02C;
    pub const TIMER2: u32 = 0x030;
    pub const TIMER3: u32 = 0x034;
    pub const TIMER0_ALARM: u32 = 0x038;
    pub const TIMER1_ALARM: u32 = 0x03C;
    pub const TIMER2_ALARM: u32 = 0x040;
    pub const TIMER3_ALARM: u32 = 0x044;
    pub const VREG_CTRL: u32 = 0x048;
    pub const VREG_CTRL_SET: u32 = 0x04C;
    pub const VREG_CTRL_CLR: u32 = 0x050;
    pub const BOD_CTRL: u32 = 0x054;
    pub const BOD_CTRL_SET: u32 = 0x058;
    pub const BOD_CTRL_CLR: u32 = 0x05C;
    pub const LPOSC_CTRL: u32 = 0x060;
    pub const LPOSC_CTRL_SET: u32 = 0x064;
    pub const LPOSC_CTRL_CLR: u32 = 0x068;
    pub const LPOSC_FREQ: u32 = 0x06C;
    pub const DBG_FORCE: u32 = 0x070;
    pub const DBG_FORCE_SET: u32 = 0x074;
    pub const DBG_FORCE_CLR: u32 = 0x078;
    pub const POW_READY: u32 = 0x07C;
    pub const POW_COUNT: u32 = 0x080;
    pub const POW_SEQ: u32 = 0x084;
    pub const POW_SEQ_SET: u32 = 0x088;
    pub const POW_SEQ_CLR: u32 = 0x08C;
    pub const GPIO0: u32 = 0x090;
    pub const GPIO1: u32 = 0x094;
    pub const GPIO2: u32 = 0x098;
    pub const GPIO3: u32 = 0x09C;
    pub const GPIO_WDSEL: u32 = 0x0A0;
    pub const GPIO_WDSEL_SET: u32 = 0x0A4;
    pub const GPIO_WDSEL_CLR: u32 = 0x0A8;
    pub const GPIO_WAKE: u32 = 0x0AC;
    pub const GPIO_WAKE_SET: u32 = 0x0B0;
    pub const GPIO_WAKE_CLR: u32 = 0x0B4;
    pub const PROC0_WAKE: u32 = 0x0B8;
    pub const PROC0_WAKE_SET: u32 = 0x0BC;
    pub const PROC0_WAKE_CLR: u32 = 0x0C0;
    pub const PROC1_WAKE: u32 = 0x0C4;
    pub const PROC1_WAKE_SET: u32 = 0x0C8;
    pub const PROC1_WAKE_CLR: u32 = 0x0CC;
    pub const PROC0_CONFIG: u32 = 0x0D0;
    pub const PROC0_CONFIG_SET: u32 = 0x0D4;
    pub const PROC0_CONFIG_CLR: u32 = 0x0D8;
    pub const PROC1_CONFIG: u32 = 0x0DC;
    pub const PROC1_CONFIG_SET: u32 = 0x0E0;
    pub const PROC1_CONFIG_CLR: u32 = 0x0E4;
}

/// CTRL register bits.
pub mod ctrl {
    pub const SLEEP: u32 = 1 << 0;
    pub const LOWPWR: u32 = 1 << 1;
    pub const DEEPSLEEP: u32 = 1 << 2;
    pub const DBG_FORCE: u32 = 1 << 3;
    pub const WAKE_EN: u32 = 1 << 4;
    pub const TIMER_EN: u32 = 1 << 5;
    pub const LPOSC_EN: u32 = 1 << 6;
    pub const BOD_EN: u32 = 1 << 7;
}

/// STATUS register bits.
pub mod status {
    pub const SLEEPING: u32 = 1 << 0;
    pub const LOWPWR: u32 = 1 << 1;
    pub const DEEPSLEEP: u32 = 1 << 2;
    pub const WAKE_PENDING: u32 = 1 << 3;
    pub const TIMER0_ALARM: u32 = 1 << 4;
    pub const TIMER1_ALARM: u32 = 1 << 5;
    pub const TIMER2_ALARM: u32 = 1 << 6;
    pub const TIMER3_ALARM: u32 = 1 << 7;
    pub const BOD: u32 = 1 << 8;
    pub const VREG_OK: u32 = 1 << 9;
    pub const LPOSC_OK: u32 = 1 << 10;
}

/// IRQ register bits.
pub mod irq {
    pub const TIMER0: u32 = 1 << 0;
    pub const TIMER1: u32 = 1 << 1;
    pub const TIMER2: u32 = 1 << 2;
    pub const TIMER3: u32 = 1 << 3;
    pub const GPIO0: u32 = 1 << 4;
    pub const GPIO1: u32 = 1 << 5;
    pub const GPIO2: u32 = 1 << 6;
    pub const GPIO3: u32 = 1 << 7;
    pub const PROC0: u32 = 1 << 8;
    pub const PROC1: u32 = 1 << 9;
    pub const BOD: u32 = 1 << 10;
    pub const WAKE: u32 = 1 << 11;
}

/// VREG_CTRL register bits.
pub mod vreg_ctrl {
    pub const EN: u32 = 1 << 0;
    pub const HIZ: u32 = 1 << 1;
    pub const VSEL_SHIFT: u32 = 4;
    pub const VSEL_MASK: u32 = 0xF << 4;
}

/// BOD_CTRL register bits.
pub mod bod_ctrl {
    pub const EN: u32 = 1 << 0;
    pub const VSEL_SHIFT: u32 = 4;
    pub const VSEL_MASK: u32 = 0xF << 4;
}

/// Power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PowerState {
    #[default]
    Active,
    LowPower,
    Sleep,
    DeepSleep,
}

/// Power Manager peripheral.
#[derive(Debug)]
pub struct PowerManager {
    /// Control register.
    ctrl: u32,
    /// Status register.
    status: u32,
    /// IRQ status.
    irq: u32,
    /// IRQ enable.
    irqe: u32,
    /// Timers.
    timers: [u32; 4],
    /// Timer alarms.
    timer_alarms: [u32; 4],
    /// Timer alarm triggered flags.
    timer_triggered: [bool; 4],
    /// Voltage regulator control.
    vreg_ctrl: u32,
    /// Brown-out detection control.
    bod_ctrl: u32,
    /// Low-power oscillator control.
    lposc_ctrl: u32,
    /// Low-power oscillator frequency.
    lposc_freq: u32,
    /// Debug force.
    dbg_force: u32,
    /// Power ready.
    pow_ready: u32,
    /// Power count.
    pow_count: u32,
    /// Power sequence.
    pow_seq: u32,
    /// GPIO wake configuration.
    gpio_wakeup: [u32; 4],
    /// GPIO watchdog select.
    gpio_wdsel: u32,
    /// GPIO wake enable.
    gpio_wake: u32,
    /// Processor 0 wake.
    proc0_wake: u32,
    /// Processor 1 wake.
    proc1_wake: u32,
    /// Processor 0 config.
    proc0_config: u32,
    /// Processor 1 config.
    proc1_config: u32,
    /// Current power state.
    power_state: PowerState,
    /// Cycle counter for timers.
    cycle_counter: u64,
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerManager {
    /// Create a new Power Manager.
    pub fn new() -> Self {
        Self {
            ctrl: ctrl::WAKE_EN | ctrl::TIMER_EN,
            status: status::VREG_OK,
            irq: 0,
            irqe: 0,
            timers: [0; 4],
            timer_alarms: [0; 4],
            timer_triggered: [false; 4],
            vreg_ctrl: vreg_ctrl::EN | (0x5 << vreg_ctrl::VSEL_SHIFT),  // Default 1.1V
            bod_ctrl: 0,
            lposc_ctrl: 0,
            lposc_freq: 32768,  // 32.768 kHz
            dbg_force: 0,
            pow_ready: 0,
            pow_count: 0,
            pow_seq: 0,
            gpio_wakeup: [0; 4],
            gpio_wdsel: 0,
            gpio_wake: 0,
            proc0_wake: 0,
            proc1_wake: 0,
            proc0_config: 0,
            proc1_config: 0,
            power_state: PowerState::Active,
            cycle_counter: 0,
        }
    }

    /// Check if sleeping.
    pub fn is_sleeping(&self) -> bool {
        self.power_state == PowerState::Sleep || self.power_state == PowerState::DeepSleep
    }

    /// Get power state.
    pub fn get_power_state(&self) -> PowerState {
        self.power_state
    }

    /// Update power state based on control register.
    fn update_power_state(&mut self) {
        if (self.ctrl & ctrl::DEEPSLEEP) != 0 {
            self.power_state = PowerState::DeepSleep;
        } else if (self.ctrl & ctrl::SLEEP) != 0 {
            self.power_state = PowerState::Sleep;
        } else if (self.ctrl & ctrl::LOWPWR) != 0 {
            self.power_state = PowerState::LowPower;
        } else {
            self.power_state = PowerState::Active;
        }
    }

    /// Wake up from sleep.
    pub fn wakeup(&mut self, source: u32) {
        self.ctrl &= !(ctrl::SLEEP | ctrl::DEEPSLEEP);
        self.power_state = PowerState::Active;
        self.status |= status::WAKE_PENDING;
        self.irq |= irq::WAKE | source;
    }

    /// Update timers (called each cycle).
    pub fn tick(&mut self, cycles: u64) {
        self.cycle_counter += cycles;

        // Check timer alarms
        for i in 0..4 {
            if !self.timer_triggered[i] && self.timers[i] >= self.timer_alarms[i] && self.timer_alarms[i] != 0 {
                self.timer_triggered[i] = true;
                self.irq |= 1 << i;
                self.status |= 1 << (4 + i);
            }
        }
    }

    /// Get voltage regulator voltage.
    pub fn get_voltage(&self) -> u32 {
        let vsel = (self.vreg_ctrl >> vreg_ctrl::VSEL_SHIFT) & 0xF;
        // VSEL 0-15 maps to 0.8V - 1.35V in 37.5mV steps
        800 + vsel as u32 * 37
    }

    /// Check brown-out detection.
    pub fn check_bod(&self) -> bool {
        if (self.bod_ctrl & bod_ctrl::EN) == 0 {
            return false;
        }

        let bod_vsel = (self.bod_ctrl >> bod_ctrl::VSEL_SHIFT) & 0xF;
        let bod_voltage = 800 + bod_vsel as u32 * 37;
        let current_voltage = self.get_voltage();

        current_voltage < bod_voltage
    }

    /// Get low-power oscillator frequency.
    pub fn get_lposc_freq(&self) -> u32 {
        self.lposc_freq
    }
}

impl Device for PowerManager {
    fn id(&self) -> DeviceId {
        DeviceId::POWMAN
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - POWMAN_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::STATUS => {
                let mut status = self.status;
                if self.power_state == PowerState::Sleep {
                    status |= status::SLEEPING;
                }
                if self.power_state == PowerState::DeepSleep {
                    status |= status::SLEEPING | status::DEEPSLEEP;
                }
                if self.power_state == PowerState::LowPower {
                    status |= status::LOWPWR;
                }
                Ok(status)
            }
            regs::IRQ => Ok(self.irq),
            regs::IRQE => Ok(self.irqe),
            regs::TIMER0 => Ok(self.timers[0]),
            regs::TIMER1 => Ok(self.timers[1]),
            regs::TIMER2 => Ok(self.timers[2]),
            regs::TIMER3 => Ok(self.timers[3]),
            regs::TIMER0_ALARM => Ok(self.timer_alarms[0]),
            regs::TIMER1_ALARM => Ok(self.timer_alarms[1]),
            regs::TIMER2_ALARM => Ok(self.timer_alarms[2]),
            regs::TIMER3_ALARM => Ok(self.timer_alarms[3]),
            regs::VREG_CTRL => Ok(self.vreg_ctrl),
            regs::BOD_CTRL => Ok(self.bod_ctrl),
            regs::LPOSC_CTRL => Ok(self.lposc_ctrl),
            regs::LPOSC_FREQ => Ok(self.lposc_freq),
            regs::DBG_FORCE => Ok(self.dbg_force),
            regs::POW_READY => Ok(self.pow_ready),
            regs::POW_COUNT => Ok(self.pow_count),
            regs::POW_SEQ => Ok(self.pow_seq),
            regs::GPIO0 => Ok(self.gpio_wakeup[0]),
            regs::GPIO1 => Ok(self.gpio_wakeup[1]),
            regs::GPIO2 => Ok(self.gpio_wakeup[2]),
            regs::GPIO3 => Ok(self.gpio_wakeup[3]),
            regs::GPIO_WDSEL => Ok(self.gpio_wdsel),
            regs::GPIO_WAKE => Ok(self.gpio_wake),
            regs::PROC0_WAKE => Ok(self.proc0_wake),
            regs::PROC1_WAKE => Ok(self.proc1_wake),
            regs::PROC0_CONFIG => Ok(self.proc0_config),
            regs::PROC1_CONFIG => Ok(self.proc1_config),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - POWMAN_BASE;

        match offset {
            regs::CTRL | regs::CTRL_SET => {
                self.ctrl |= value;
                self.update_power_state();
            }
            regs::CTRL_CLR => {
                self.ctrl &= !value;
                self.update_power_state();
            }
            regs::IRQ => {
                self.irq = value;
            }
            regs::IRQ_SET => {
                self.irq |= value;
            }
            regs::IRQ_CLR => {
                self.irq &= !value;
            }
            regs::IRQE | regs::IRQE_SET => {
                self.irqe |= value;
            }
            regs::IRQE_CLR => {
                self.irqe &= !value;
            }
            regs::TIMER0 => {
                self.timers[0] = value;
            }
            regs::TIMER1 => {
                self.timers[1] = value;
            }
            regs::TIMER2 => {
                self.timers[2] = value;
            }
            regs::TIMER3 => {
                self.timers[3] = value;
            }
            regs::TIMER0_ALARM => {
                self.timer_alarms[0] = value;
                self.timer_triggered[0] = false;
            }
            regs::TIMER1_ALARM => {
                self.timer_alarms[1] = value;
                self.timer_triggered[1] = false;
            }
            regs::TIMER2_ALARM => {
                self.timer_alarms[2] = value;
                self.timer_triggered[2] = false;
            }
            regs::TIMER3_ALARM => {
                self.timer_alarms[3] = value;
                self.timer_triggered[3] = false;
            }
            regs::VREG_CTRL | regs::VREG_CTRL_SET => {
                self.vreg_ctrl |= value;
            }
            regs::VREG_CTRL_CLR => {
                self.vreg_ctrl &= !value;
            }
            regs::BOD_CTRL | regs::BOD_CTRL_SET => {
                self.bod_ctrl |= value;
            }
            regs::BOD_CTRL_CLR => {
                self.bod_ctrl &= !value;
            }
            regs::LPOSC_CTRL | regs::LPOSC_CTRL_SET => {
                self.lposc_ctrl |= value;
            }
            regs::LPOSC_CTRL_CLR => {
                self.lposc_ctrl &= !value;
            }
            regs::LPOSC_FREQ => {
                self.lposc_freq = value;
            }
            regs::DBG_FORCE | regs::DBG_FORCE_SET => {
                self.dbg_force |= value;
            }
            regs::DBG_FORCE_CLR => {
                self.dbg_force &= !value;
            }
            regs::POW_READY => {
                self.pow_ready = value;
            }
            regs::POW_COUNT => {
                self.pow_count = value;
            }
            regs::POW_SEQ | regs::POW_SEQ_SET => {
                self.pow_seq |= value;
            }
            regs::POW_SEQ_CLR => {
                self.pow_seq &= !value;
            }
            regs::GPIO0 => {
                self.gpio_wakeup[0] = value;
            }
            regs::GPIO1 => {
                self.gpio_wakeup[1] = value;
            }
            regs::GPIO2 => {
                self.gpio_wakeup[2] = value;
            }
            regs::GPIO3 => {
                self.gpio_wakeup[3] = value;
            }
            regs::GPIO_WDSEL | regs::GPIO_WDSEL_SET => {
                self.gpio_wdsel |= value;
            }
            regs::GPIO_WDSEL_CLR => {
                self.gpio_wdsel &= !value;
            }
            regs::GPIO_WAKE | regs::GPIO_WAKE_SET => {
                self.gpio_wake |= value;
            }
            regs::GPIO_WAKE_CLR => {
                self.gpio_wake &= !value;
            }
            regs::PROC0_WAKE | regs::PROC0_WAKE_SET => {
                self.proc0_wake |= value;
            }
            regs::PROC0_WAKE_CLR => {
                self.proc0_wake &= !value;
            }
            regs::PROC1_WAKE | regs::PROC1_WAKE_SET => {
                self.proc1_wake |= value;
            }
            regs::PROC1_WAKE_CLR => {
                self.proc1_wake &= !value;
            }
            regs::PROC0_CONFIG | regs::PROC0_CONFIG_SET => {
                self.proc0_config |= value;
            }
            regs::PROC0_CONFIG_CLR => {
                self.proc0_config &= !value;
            }
            regs::PROC1_CONFIG | regs::PROC1_CONFIG_SET => {
                self.proc1_config |= value;
            }
            regs::PROC1_CONFIG_CLR => {
                self.proc1_config &= !value;
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

    const BASE: u32 = POWMAN_BASE;

    // ==================== Basic Creation Tests ====================

    #[test]
    fn test_powman_creation() {
        let pm = PowerManager::new();

        // Default control bits
        assert_eq!(pm.ctrl & ctrl::WAKE_EN, ctrl::WAKE_EN);
        assert_eq!(pm.ctrl & ctrl::TIMER_EN, ctrl::TIMER_EN);

        // Default status
        assert_eq!(pm.status & status::VREG_OK, status::VREG_OK);

        // No IRQs
        assert_eq!(pm.irq, 0);
        assert_eq!(pm.irqe, 0);

        // Power state is Active
        assert_eq!(pm.power_state, PowerState::Active);
        assert!(!pm.is_sleeping());
    }

    #[test]
    fn test_powman_default() {
        let pm = PowerManager::default();
        assert_eq!(pm.power_state, PowerState::Active);
    }

    // ==================== Power State Tests ====================

    #[test]
    fn test_power_state_default() {
        let state = PowerState::default();
        assert_eq!(state, PowerState::Active);
    }

    #[test]
    fn test_power_state_transitions() {
        let mut pm = PowerManager::new();

        // Active -> LowPower
        pm.ctrl = ctrl::LOWPWR;
        pm.update_power_state();
        assert_eq!(pm.get_power_state(), PowerState::LowPower);

        // LowPower -> Sleep
        pm.ctrl = ctrl::SLEEP;
        pm.update_power_state();
        assert_eq!(pm.get_power_state(), PowerState::Sleep);
        assert!(pm.is_sleeping());

        // Sleep -> DeepSleep
        pm.ctrl = ctrl::DEEPSLEEP;
        pm.update_power_state();
        assert_eq!(pm.get_power_state(), PowerState::DeepSleep);
        assert!(pm.is_sleeping());

        // DeepSleep -> Active
        pm.ctrl = 0;
        pm.update_power_state();
        assert_eq!(pm.get_power_state(), PowerState::Active);
        assert!(!pm.is_sleeping());
    }

    #[test]
    fn test_deepsleep_overrides_sleep() {
        let mut pm = PowerManager::new();

        pm.ctrl = ctrl::SLEEP | ctrl::DEEPSLEEP;
        pm.update_power_state();
        assert_eq!(pm.get_power_state(), PowerState::DeepSleep);
    }

    // ==================== Wakeup Tests ====================

    #[test]
    fn test_wakeup() {
        let mut pm = PowerManager::new();

        pm.ctrl = ctrl::SLEEP;
        pm.update_power_state();
        assert_eq!(pm.power_state, PowerState::Sleep);

        pm.wakeup(irq::GPIO0);

        assert_eq!(pm.power_state, PowerState::Active);
        assert_eq!(pm.irq & irq::WAKE, irq::WAKE);
        assert_eq!(pm.irq & irq::GPIO0, irq::GPIO0);
        assert_eq!(pm.status & status::WAKE_PENDING, status::WAKE_PENDING);
    }

    #[test]
    fn test_wakeup_from_deepsleep() {
        let mut pm = PowerManager::new();

        pm.ctrl = ctrl::DEEPSLEEP;
        pm.update_power_state();
        assert!(pm.is_sleeping());

        pm.wakeup(irq::PROC0);

        assert_eq!(pm.power_state, PowerState::Active);
        assert!(!pm.is_sleeping());
    }

    // ==================== Voltage Regulator Tests ====================

    #[test]
    fn test_vreg_default() {
        let pm = PowerManager::new();

        // Default VSEL is 5 (1.1V)
        assert_eq!(pm.vreg_ctrl & vreg_ctrl::EN, vreg_ctrl::EN);
        assert_eq!(pm.get_voltage(), 800 + 5 * 37); // 985mV
    }

    #[test]
    fn test_get_voltage() {
        let mut pm = PowerManager::new();

        // VSEL 0 = 0.8V
        pm.vreg_ctrl = vreg_ctrl::EN | (0 << vreg_ctrl::VSEL_SHIFT);
        assert_eq!(pm.get_voltage(), 800);

        // VSEL 15 = max voltage
        pm.vreg_ctrl = vreg_ctrl::EN | (15 << vreg_ctrl::VSEL_SHIFT);
        assert_eq!(pm.get_voltage(), 800 + 15 * 37);
    }

    #[test]
    fn test_vreg_ctrl_bits() {
        let mut pm = PowerManager::new();

        pm.vreg_ctrl = 0;
        assert_eq!(pm.vreg_ctrl & vreg_ctrl::EN, 0);

        pm.vreg_ctrl = vreg_ctrl::EN | vreg_ctrl::HIZ;
        assert_eq!(pm.vreg_ctrl & vreg_ctrl::EN, vreg_ctrl::EN);
        assert_eq!(pm.vreg_ctrl & vreg_ctrl::HIZ, vreg_ctrl::HIZ);
    }

    // ==================== Brown-Out Detection Tests ====================

    #[test]
    fn test_bod_disabled() {
        let pm = PowerManager::new();
        assert!(!pm.check_bod()); // BOD disabled by default
    }

    #[test]
    fn test_bod_no_trigger() {
        let mut pm = PowerManager::new();

        pm.vreg_ctrl = vreg_ctrl::EN | (10 << vreg_ctrl::VSEL_SHIFT);
        pm.bod_ctrl = bod_ctrl::EN | (5 << bod_ctrl::VSEL_SHIFT);

        // BOD voltage (985mV) < current voltage (1170mV), no trigger
        assert!(!pm.check_bod());
    }

    #[test]
    fn test_bod_trigger() {
        let mut pm = PowerManager::new();

        pm.vreg_ctrl = vreg_ctrl::EN | (2 << vreg_ctrl::VSEL_SHIFT); // 874mV
        pm.bod_ctrl = bod_ctrl::EN | (10 << bod_ctrl::VSEL_SHIFT); // 1170mV

        // BOD voltage > current voltage, should trigger
        assert!(pm.check_bod());
    }

    // ==================== Low-Power Oscillator Tests ====================

    #[test]
    fn test_lposc_default() {
        let pm = PowerManager::new();
        assert_eq!(pm.get_lposc_freq(), 32768);
    }

    #[test]
    fn test_lposc_set_freq() {
        let mut pm = PowerManager::new();
        pm.lposc_freq = 65536;
        assert_eq!(pm.get_lposc_freq(), 65536);
    }

    // ==================== Timer Tests ====================

    #[test]
    fn test_timer_initial() {
        let pm = PowerManager::new();
        for i in 0..4 {
            assert_eq!(pm.timers[i], 0);
            assert_eq!(pm.timer_alarms[i], 0);
            assert!(!pm.timer_triggered[i]);
        }
    }

    #[test]
    fn test_timer_tick_no_alarm() {
        let mut pm = PowerManager::new();

        pm.timers[0] = 100;
        pm.tick(10);

        // No alarm set, no trigger
        assert_eq!(pm.irq & irq::TIMER0, 0);
    }

    #[test]
    fn test_timer_alarm_trigger() {
        let mut pm = PowerManager::new();

        pm.timers[0] = 100;
        pm.timer_alarms[0] = 50;
        pm.tick(0); // Check happens in tick

        assert!(pm.timer_triggered[0]);
        assert_eq!(pm.irq & irq::TIMER0, irq::TIMER0);
        assert_eq!(pm.status & status::TIMER0_ALARM, status::TIMER0_ALARM);
    }

    #[test]
    fn test_timer_multiple_alarms() {
        let mut pm = PowerManager::new();

        pm.timers[0] = 100;
        pm.timers[1] = 200;
        pm.timer_alarms[0] = 50;
        pm.timer_alarms[1] = 150;
        pm.tick(0);

        assert!(pm.timer_triggered[0]);
        assert!(pm.timer_triggered[1]);
        assert_eq!(pm.irq & (irq::TIMER0 | irq::TIMER1), irq::TIMER0 | irq::TIMER1);
    }

    // ==================== Register Read Tests ====================

    #[test]
    fn test_read_ctrl() {
        let mut pm = PowerManager::new();
        pm.ctrl = 0x12345678;
        assert_eq!(pm.read(BASE + regs::CTRL).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_status() {
        let mut pm = PowerManager::new();

        let status = pm.read(BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::VREG_OK, status::VREG_OK);
    }

    #[test]
    fn test_read_status_sleeping() {
        let mut pm = PowerManager::new();

        pm.power_state = PowerState::Sleep;
        let status = pm.read(BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::SLEEPING, status::SLEEPING);

        pm.power_state = PowerState::DeepSleep;
        let status = pm.read(BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::DEEPSLEEP, status::DEEPSLEEP);
    }

    #[test]
    fn test_read_irq() {
        let mut pm = PowerManager::new();
        pm.irq = irq::TIMER0 | irq::GPIO0;
        assert_eq!(pm.read(BASE + regs::IRQ).unwrap(), irq::TIMER0 | irq::GPIO0);
    }

    #[test]
    fn test_read_timers() {
        let mut pm = PowerManager::new();

        pm.timers[0] = 100;
        pm.timers[3] = 500;

        assert_eq!(pm.read(BASE + regs::TIMER0).unwrap(), 100);
        assert_eq!(pm.read(BASE + regs::TIMER3).unwrap(), 500);
    }

    #[test]
    fn test_read_vreg_ctrl() {
        let mut pm = PowerManager::new();
        pm.vreg_ctrl = 0xABCD;
        assert_eq!(pm.read(BASE + regs::VREG_CTRL).unwrap(), 0xABCD);
    }

    #[test]
    fn test_read_gpio_wakeup() {
        let mut pm = PowerManager::new();

        pm.gpio_wakeup[0] = 0x11111111;
        pm.gpio_wakeup[3] = 0x44444444;

        assert_eq!(pm.read(BASE + regs::GPIO0).unwrap(), 0x11111111);
        assert_eq!(pm.read(BASE + regs::GPIO3).unwrap(), 0x44444444);
    }

    // ==================== Register Write Tests ====================

    #[test]
    fn test_write_ctrl() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::CTRL, ctrl::SLEEP).unwrap();
        assert_eq!(pm.ctrl & ctrl::SLEEP, ctrl::SLEEP);
        assert_eq!(pm.power_state, PowerState::Sleep);
    }

    #[test]
    fn test_write_ctrl_set() {
        let mut pm = PowerManager::new();

        pm.ctrl = 0;
        pm.write(BASE + regs::CTRL_SET, ctrl::SLEEP | ctrl::LOWPWR).unwrap();
        assert_eq!(pm.ctrl, ctrl::SLEEP | ctrl::LOWPWR);
    }

    #[test]
    fn test_write_ctrl_clr() {
        let mut pm = PowerManager::new();

        pm.ctrl = ctrl::SLEEP | ctrl::LOWPWR;
        pm.write(BASE + regs::CTRL_CLR, ctrl::SLEEP).unwrap();
        assert_eq!(pm.ctrl, ctrl::LOWPWR);
    }

    #[test]
    fn test_write_irq_set_clr() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::IRQ_SET, irq::TIMER0).unwrap();
        assert_eq!(pm.irq, irq::TIMER0);

        pm.write(BASE + regs::IRQ_CLR, irq::TIMER0).unwrap();
        assert_eq!(pm.irq, 0);
    }

    #[test]
    fn test_write_timer() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::TIMER0, 1000).unwrap();
        assert_eq!(pm.timers[0], 1000);

        pm.write(BASE + regs::TIMER3, 5000).unwrap();
        assert_eq!(pm.timers[3], 5000);
    }

    #[test]
    fn test_write_timer_alarm() {
        let mut pm = PowerManager::new();

        pm.timer_triggered[0] = true;
        pm.write(BASE + regs::TIMER0_ALARM, 100).unwrap();

        assert_eq!(pm.timer_alarms[0], 100);
        assert!(!pm.timer_triggered[0]); // Writing alarm clears trigger
    }

    #[test]
    fn test_write_vreg_ctrl() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::VREG_CTRL, 0xFFFF).unwrap();
        assert_eq!(pm.vreg_ctrl, 0xFFFF);

        pm.write(BASE + regs::VREG_CTRL_CLR, vreg_ctrl::HIZ).unwrap();
        assert_eq!(pm.vreg_ctrl & vreg_ctrl::HIZ, 0);
    }

    #[test]
    fn test_write_gpio_wakeup() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::GPIO0, 0x12345678).unwrap();
        assert_eq!(pm.gpio_wakeup[0], 0x12345678);
    }

    #[test]
    fn test_write_proc_wake() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::PROC0_WAKE, 0x1111).unwrap();
        assert_eq!(pm.proc0_wake, 0x1111);

        pm.write(BASE + regs::PROC1_WAKE_SET, 0x2222).unwrap();
        assert_eq!(pm.proc1_wake, 0x2222);

        pm.write(BASE + regs::PROC1_WAKE_CLR, 0x2222).unwrap();
        assert_eq!(pm.proc1_wake, 0);
    }

    #[test]
    fn test_write_proc_config() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::PROC0_CONFIG, 0xAAAA).unwrap();
        assert_eq!(pm.proc0_config, 0xAAAA);

        pm.write(BASE + regs::PROC1_CONFIG_SET, 0xBBBB).unwrap();
        assert_eq!(pm.proc1_config, 0xBBBB);

        pm.write(BASE + regs::PROC1_CONFIG_CLR, 0xBBBB).unwrap();
        assert_eq!(pm.proc1_config, 0);
    }

    // ==================== Device Trait Tests ====================

    #[test]
    fn test_device_id() {
        let pm = PowerManager::new();
        assert_eq!(pm.id(), DeviceId::POWMAN);
    }

    #[test]
    fn test_device_reset() {
        let mut pm = PowerManager::new();

        pm.ctrl = 0xFFFFFFFF;
        pm.irq = 0xFFFFFFFF;
        pm.power_state = PowerState::DeepSleep;
        pm.vreg_ctrl = 0;

        pm.reset();

        assert_eq!(pm.power_state, PowerState::Active);
        assert_eq!(pm.irq, 0);
    }

    // ==================== Invalid Register Tests ====================

    // ==================== Edge Case: Timer Alarm at 0 (Disabled) ====================

    #[test]
    fn test_timer_alarm_disabled_at_zero() {
        let mut pm = PowerManager::new();

        pm.timers[0] = 100;
        pm.timer_alarms[0] = 0; // Alarm disabled
        pm.tick(0);

        // Alarm at 0 should not trigger
        assert!(!pm.timer_triggered[0]);
        assert_eq!(pm.irq & irq::TIMER0, 0);
    }

    // ==================== Wakeup Sources Tests ====================

    #[test]
    fn test_wakeup_sources_all_gpio() {
        let mut pm = PowerManager::new();

        pm.ctrl = ctrl::DEEPSLEEP;
        pm.update_power_state();

        // Test all GPIO wakeup sources
        pm.wakeup(irq::GPIO0);
        assert_eq!(pm.irq & irq::GPIO0, irq::GPIO0);

        pm.wakeup(irq::GPIO1);
        assert_eq!(pm.irq & irq::GPIO1, irq::GPIO1);

        pm.wakeup(irq::GPIO2);
        assert_eq!(pm.irq & irq::GPIO2, irq::GPIO2);

        pm.wakeup(irq::GPIO3);
        assert_eq!(pm.irq & irq::GPIO3, irq::GPIO3);
    }

    #[test]
    fn test_wakeup_proc_sources() {
        let mut pm = PowerManager::new();

        pm.ctrl = ctrl::SLEEP;
        pm.update_power_state();

        pm.wakeup(irq::PROC0 | irq::PROC1);
        assert_eq!(pm.irq & irq::PROC0, irq::PROC0);
        assert_eq!(pm.irq & irq::PROC1, irq::PROC1);
        assert_eq!(pm.power_state, PowerState::Active);
    }

    #[test]
    fn test_wakeup_bod_source() {
        let mut pm = PowerManager::new();

        pm.ctrl = ctrl::SLEEP;
        pm.update_power_state();

        pm.wakeup(irq::BOD);
        assert_eq!(pm.irq & irq::BOD, irq::BOD);
        assert_eq!(pm.irq & irq::WAKE, irq::WAKE);
    }

    // ==================== IRQ Generation Tests ====================

    #[test]
    fn test_irq_generation_timer_all() {
        let mut pm = PowerManager::new();

        // Test all 4 timers trigger their respective IRQs
        for i in 0..4 {
            pm.timers[i] = 100;
            pm.timer_alarms[i] = 50;
        }
        pm.tick(0);

        assert_eq!(pm.irq & irq::TIMER0, irq::TIMER0);
        assert_eq!(pm.irq & irq::TIMER1, irq::TIMER1);
        assert_eq!(pm.irq & irq::TIMER2, irq::TIMER2);
        assert_eq!(pm.irq & irq::TIMER3, irq::TIMER3);
    }

    #[test]
    fn test_irq_no_retrigger() {
        let mut pm = PowerManager::new();

        pm.timers[0] = 100;
        pm.timer_alarms[0] = 50;
        pm.tick(0);

        assert!(pm.timer_triggered[0]);
        assert_eq!(pm.irq & irq::TIMER0, irq::TIMER0);

        // Clear the IRQ but timer still triggered
        pm.irq &= !irq::TIMER0;

        // Second tick should not re-trigger
        pm.tick(0);
        assert_eq!(pm.irq & irq::TIMER0, 0);
    }

    #[test]
    fn test_irq_wake_on_wakeup() {
        let mut pm = PowerManager::new();

        pm.wakeup(0);
        assert_eq!(pm.irq & irq::WAKE, irq::WAKE);
    }

    #[test]
    fn test_irqe_enable() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::IRQE_SET, irq::TIMER0 | irq::GPIO0).unwrap();
        assert_eq!(pm.irqe, irq::TIMER0 | irq::GPIO0);

        pm.write(BASE + regs::IRQE_CLR, irq::TIMER0).unwrap();
        assert_eq!(pm.irqe, irq::GPIO0);
    }

    // ==================== Voltage Regulator Edge Cases ====================

    #[test]
    fn test_voltage_minimum_vsel() {
        let mut pm = PowerManager::new();

        pm.vreg_ctrl = vreg_ctrl::EN | (0 << vreg_ctrl::VSEL_SHIFT);
        assert_eq!(pm.get_voltage(), 800); // Minimum 0.8V
    }

    #[test]
    fn test_voltage_maximum_vsel() {
        let mut pm = PowerManager::new();

        pm.vreg_ctrl = vreg_ctrl::EN | (15 << vreg_ctrl::VSEL_SHIFT);
        assert_eq!(pm.get_voltage(), 800 + 15 * 37); // Maximum
    }

    #[test]
    fn test_voltage_all_vsel_values() {
        let mut pm = PowerManager::new();

        // Test voltage formula for all VSEL values
        for vsel in 0..=15u32 {
            pm.vreg_ctrl = vreg_ctrl::EN | (vsel << vreg_ctrl::VSEL_SHIFT);
            let expected = 800 + vsel * 37;
            assert_eq!(pm.get_voltage(), expected, "VSEL {}", vsel);
        }
    }

    // ==================== Sleep Mode Edge Cases ====================

    #[test]
    fn test_sleep_mode_via_device_write() {
        let mut pm = PowerManager::new();

        // Enter sleep via write
        pm.write(BASE + regs::CTRL, ctrl::SLEEP).unwrap();
        assert_eq!(pm.get_power_state(), PowerState::Sleep);
        assert!(pm.is_sleeping());

        // Check status reflects sleep
        let status = pm.read(BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::SLEEPING, status::SLEEPING);
    }

    #[test]
    fn test_lowpwr_mode_via_device_write() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::CTRL, ctrl::LOWPWR).unwrap();
        assert_eq!(pm.get_power_state(), PowerState::LowPower);

        let status = pm.read(BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::LOWPWR, status::LOWPWR);
    }

    #[test]
    fn test_deepsleep_mode_via_device_write() {
        let mut pm = PowerManager::new();

        pm.write(BASE + regs::CTRL, ctrl::DEEPSLEEP).unwrap();
        assert_eq!(pm.get_power_state(), PowerState::DeepSleep);
        assert!(pm.is_sleeping());

        let status = pm.read(BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::DEEPSLEEP, status::DEEPSLEEP);
        assert_eq!(status & status::SLEEPING, status::SLEEPING);
    }

    #[test]
    fn test_sleep_priority_order() {
        let mut pm = PowerManager::new();

        // DeepSleep has priority over Sleep
        pm.write(BASE + regs::CTRL, ctrl::SLEEP | ctrl::DEEPSLEEP).unwrap();
        assert_eq!(pm.get_power_state(), PowerState::DeepSleep);

        // Clear DeepSleep, should go to Sleep
        pm.write(BASE + regs::CTRL_CLR, ctrl::DEEPSLEEP).unwrap();
        assert_eq!(pm.get_power_state(), PowerState::Sleep);

        // Clear Sleep first, then set LowPower
        pm.write(BASE + regs::CTRL_CLR, ctrl::SLEEP).unwrap();
        pm.write(BASE + regs::CTRL, ctrl::LOWPWR).unwrap();
        assert_eq!(pm.get_power_state(), PowerState::LowPower);

        // Clear all, back to Active
        pm.write(BASE + regs::CTRL_CLR, ctrl::LOWPWR).unwrap();
        assert_eq!(pm.get_power_state(), PowerState::Active);
    }

    // ==================== Brown-Out Detection Edge Cases ====================

    #[test]
    fn test_bod_voltage_at_threshold() {
        let mut pm = PowerManager::new();

        // Set VREG and BOD to same voltage
        pm.vreg_ctrl = vreg_ctrl::EN | (5 << vreg_ctrl::VSEL_SHIFT); // 985mV
        pm.bod_ctrl = bod_ctrl::EN | (5 << bod_ctrl::VSEL_SHIFT); // 985mV

        // At threshold, should not trigger (< not <=)
        assert!(!pm.check_bod());
    }

    #[test]
    fn test_bod_minimum_bod_vsel() {
        let mut pm = PowerManager::new();

        pm.vreg_ctrl = vreg_ctrl::EN | (0 << vreg_ctrl::VSEL_SHIFT); // 800mV
        pm.bod_ctrl = bod_ctrl::EN | (0 << bod_ctrl::VSEL_SHIFT); // 800mV BOD threshold

        // Same voltage, no trigger
        assert!(!pm.check_bod());
    }

    #[test]
    fn test_bod_maximum_bod_vsel() {
        let mut pm = PowerManager::new();

        pm.vreg_ctrl = vreg_ctrl::EN | (0 << vreg_ctrl::VSEL_SHIFT); // 800mV (lowest)
        pm.bod_ctrl = bod_ctrl::EN | (15 << bod_ctrl::VSEL_SHIFT); // 1355mV threshold

        // Current < threshold, should trigger
        assert!(pm.check_bod());
    }


    #[test]
    fn test_read_invalid_register() {
        let mut pm = PowerManager::new();
        assert_eq!(pm.read(BASE + 0x1000).unwrap(), 0);
    }

    #[test]
    fn test_write_invalid_register() {
        let mut pm = PowerManager::new();
        // Should not panic
        pm.write(BASE + 0x1000, 0xFFFFFFFF).unwrap();
    }
}
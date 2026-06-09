//! NVIC (Nested Vectored Interrupt Controller) for RP2350.
//!
//! Implements the ARM Cortex-M33 NVIC.

use rp2350sim_core::{Device, DeviceId, Result};

/// NVIC base addresses (per core).
pub const NVIC_BASE_CORE0: u32 = 0xE000_E100;
pub const NVIC_BASE_CORE1: u32 = 0xE002_E100;

/// NVIC register offsets.
pub mod regs {
    // Interrupt Set Enable
    pub const ISER0: u32 = 0x000;
    pub const ISER1: u32 = 0x004;
    pub const ISER2: u32 = 0x008;
    pub const ISER3: u32 = 0x00C;
    // Interrupt Clear Enable
    pub const ICER0: u32 = 0x080;
    pub const ICER1: u32 = 0x084;
    pub const ICER2: u32 = 0x088;
    pub const ICER3: u32 = 0x08C;
    // Interrupt Set Pending
    pub const ISPR0: u32 = 0x100;
    pub const ISPR1: u32 = 0x104;
    pub const ISPR2: u32 = 0x108;
    pub const ISPR3: u32 = 0x10C;
    // Interrupt Clear Pending
    pub const ICPR0: u32 = 0x180;
    pub const ICPR1: u32 = 0x184;
    pub const ICPR2: u32 = 0x188;
    pub const ICPR3: u32 = 0x18C;
    // Interrupt Active Bit
    pub const IABR0: u32 = 0x200;
    pub const IABR1: u32 = 0x204;
    pub const IABR2: u32 = 0x208;
    pub const IABR3: u32 = 0x20C;
    // Interrupt Priority
    pub const IPR0: u32 = 0x300;
    pub const IPR1: u32 = 0x304;
    pub const IPR2: u32 = 0x308;
    pub const IPR3: u32 = 0x30C;
    pub const IPR4: u32 = 0x310;
    pub const IPR5: u32 = 0x314;
    pub const IPR6: u32 = 0x318;
    pub const IPR7: u32 = 0x31C;
    pub const IPR8: u32 = 0x320;
    pub const IPR9: u32 = 0x324;
    pub const IPR10: u32 = 0x328;
    pub const IPR11: u32 = 0x32C;
    pub const IPR12: u32 = 0x330;
    pub const IPR13: u32 = 0x334;
    pub const IPR14: u32 = 0x338;
    pub const IPR15: u32 = 0x33C;
    // Software Triggered Interrupt
    pub const STIR: u32 = 0xE00;
}

/// Number of external interrupts.
const NUM_IRQS: usize = 128;

/// NVIC for a single core.
#[derive(Debug)]
pub struct NvicCore {
    /// Interrupt enable registers.
    enable: [u32; 4],
    /// Pending registers.
    pending: [u32; 4],
    /// Active registers.
    active: [u32; 4],
    /// Priority registers (8-bit per IRQ, 4 IRQs per register).
    priority: [u32; 16],
    /// Vector table offset.
    vector_table: u32,
    /// Current interrupt number.
    current_irq: Option<u8>,
}

impl Default for NvicCore {
    fn default() -> Self {
        Self::new()
    }
}

impl NvicCore {
    /// Create a new NVIC core.
    pub fn new() -> Self {
        Self {
            enable: [0; 4],
            pending: [0; 4],
            active: [0; 4],
            priority: [0; 16],
            vector_table: 0,
            current_irq: None,
        }
    }

    /// Check if interrupt is enabled.
    pub fn is_enabled(&self, irq: u8) -> bool {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if reg < 4 {
            (self.enable[reg] & (1 << bit)) != 0
        } else {
            false
        }
    }

    /// Enable interrupt.
    pub fn enable_irq(&mut self, irq: u8) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if reg < 4 {
            self.enable[reg] |= 1 << bit;
        }
    }

    /// Disable interrupt.
    pub fn disable_irq(&mut self, irq: u8) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if reg < 4 {
            self.enable[reg] &= !(1 << bit);
        }
    }

    /// Check if interrupt is pending.
    pub fn is_pending(&self, irq: u8) -> bool {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if reg < 4 {
            (self.pending[reg] & (1 << bit)) != 0
        } else {
            false
        }
    }

    /// Set interrupt pending.
    pub fn set_pending(&mut self, irq: u8) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if reg < 4 {
            self.pending[reg] |= 1 << bit;
        }
    }

    /// Clear interrupt pending.
    pub fn clear_pending(&mut self, irq: u8) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if reg < 4 {
            self.pending[reg] &= !(1 << bit);
        }
    }

    /// Check if interrupt is active.
    pub fn is_active(&self, irq: u8) -> bool {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if reg < 4 {
            (self.active[reg] & (1 << bit)) != 0
        } else {
            false
        }
    }

    /// Set interrupt active.
    fn set_active(&mut self, irq: u8, active: bool) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if reg < 4 {
            if active {
                self.active[reg] |= 1 << bit;
            } else {
                self.active[reg] &= !(1 << bit);
            }
        }
    }

    /// Get interrupt priority (0-255, lower = higher priority).
    pub fn get_priority(&self, irq: u8) -> u8 {
        let reg = (irq / 4) as usize;
        let shift = (irq % 4) * 8;
        if reg < 16 {
            ((self.priority[reg] >> shift) & 0xFF) as u8
        } else {
            0
        }
    }

    /// Set interrupt priority.
    pub fn set_priority(&mut self, irq: u8, priority: u8) {
        let reg = (irq / 4) as usize;
        let shift = (irq % 4) * 8;
        if reg < 16 {
            self.priority[reg] = (self.priority[reg] & !(0xFF << shift)) | ((priority as u32) << shift);
        }
    }

    /// Get vector table address.
    pub fn get_vector_table(&self) -> u32 {
        self.vector_table
    }

    /// Set vector table address.
    pub fn set_vector_table(&mut self, addr: u32) {
        self.vector_table = addr;
    }

    /// Get the highest priority pending interrupt.
    pub fn get_highest_pending(&self) -> Option<u8> {
        let mut best_irq: Option<u8> = None;
        let mut best_priority: u8 = 255;

        for irq in 0..NUM_IRQS as u8 {
            if self.is_pending(irq) && self.is_enabled(irq) && !self.is_active(irq) {
                let priority = self.get_priority(irq);
                if priority < best_priority {
                    best_priority = priority;
                    best_irq = Some(irq);
                }
            }
        }

        best_irq
    }

    /// Acknowledge interrupt (start handling).
    pub fn acknowledge(&mut self, irq: u8) {
        self.clear_pending(irq);
        self.set_active(irq, true);
        self.current_irq = Some(irq);
    }

    /// End of interrupt.
    pub fn end_of_interrupt(&mut self, irq: u8) {
        self.set_active(irq, false);
        if self.current_irq == Some(irq) {
            self.current_irq = None;
        }
    }

    /// Get interrupt vector address.
    pub fn get_vector_address(&self, irq: u8) -> u32 {
        // Vector table contains: 0-15 = system exceptions, 16+ = external interrupts
        let vector_num = irq as u32 + 16;
        self.vector_table + vector_num * 4
    }

    /// Read register.
    fn read(&self, offset: u32) -> u32 {
        match offset {
            regs::ISER0..=regs::ISER3 => {
                let idx = (offset - regs::ISER0) as usize / 4;
                if idx < 4 { self.enable[idx] } else { 0 }
            }
            regs::ICER0..=regs::ICER3 => {
                let idx = (offset - regs::ICER0) as usize / 4;
                if idx < 4 { self.enable[idx] } else { 0 }
            }
            regs::ISPR0..=regs::ISPR3 => {
                let idx = (offset - regs::ISPR0) as usize / 4;
                if idx < 4 { self.pending[idx] } else { 0 }
            }
            regs::ICPR0..=regs::ICPR3 => {
                let idx = (offset - regs::ICPR0) as usize / 4;
                if idx < 4 { self.pending[idx] } else { 0 }
            }
            regs::IABR0..=regs::IABR3 => {
                let idx = (offset - regs::IABR0) as usize / 4;
                if idx < 4 { self.active[idx] } else { 0 }
            }
            regs::IPR0..=regs::IPR15 => {
                let idx = (offset - regs::IPR0) as usize / 4;
                if idx < 16 { self.priority[idx] } else { 0 }
            }
            _ => 0,
        }
    }

    /// Write register.
    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            regs::ISER0..=regs::ISER3 => {
                let idx = (offset - regs::ISER0) as usize / 4;
                if idx < 4 { self.enable[idx] |= value; }
            }
            regs::ICER0..=regs::ICER3 => {
                let idx = (offset - regs::ICER0) as usize / 4;
                if idx < 4 { self.enable[idx] &= !value; }
            }
            regs::ISPR0..=regs::ISPR3 => {
                let idx = (offset - regs::ISPR0) as usize / 4;
                if idx < 4 { self.pending[idx] |= value; }
            }
            regs::ICPR0..=regs::ICPR3 => {
                let idx = (offset - regs::ICPR0) as usize / 4;
                if idx < 4 { self.pending[idx] &= !value; }
            }
            regs::IPR0..=regs::IPR15 => {
                let idx = (offset - regs::IPR0) as usize / 4;
                if idx < 16 { self.priority[idx] = value; }
            }
            regs::STIR => {
                // Software triggered interrupt
                let irq = (value & 0x1FF) as u8;
                if irq < NUM_IRQS as u8 {
                    self.set_pending(irq);
                }
            }
            _ => {}
        }
    }

    /// Reset.
    fn reset(&mut self) {
        *self = Self::new();
    }
}

/// NVIC peripheral (dual core).
#[derive(Debug)]
pub struct Nvic {
    /// Core 0 NVIC.
    pub core0: NvicCore,
    /// Core 1 NVIC.
    pub core1: NvicCore,
}

impl Default for Nvic {
    fn default() -> Self {
        Self::new()
    }
}

impl Nvic {
    /// Create a new NVIC instance.
    pub fn new() -> Self {
        Self {
            core0: NvicCore::new(),
            core1: NvicCore::new(),
        }
    }

    /// Get core NVIC.
    pub fn get_core(&self, core: usize) -> &NvicCore {
        match core {
            0 => &self.core0,
            _ => &self.core1,
        }
    }

    /// Get mutable core NVIC.
    pub fn get_core_mut(&mut self, core: usize) -> &mut NvicCore {
        match core {
            0 => &mut self.core0,
            _ => &mut self.core1,
        }
    }

    /// Trigger interrupt on all cores.
    pub fn trigger_irq(&mut self, irq: u8) {
        self.core0.set_pending(irq);
        self.core1.set_pending(irq);
    }

    /// Determine core from address.
    fn get_core_index(&self, addr: u32) -> usize {
        if addr >= NVIC_BASE_CORE1 {
            1
        } else {
            0
        }
    }

    /// Get base address for core.
    fn get_base(&self, core: usize) -> u32 {
        match core {
            0 => NVIC_BASE_CORE0,
            _ => NVIC_BASE_CORE1,
        }
    }
}

impl Device for Nvic {
    fn id(&self) -> DeviceId {
        DeviceId::NVIC
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let core_idx = self.get_core_index(addr);
        let base = self.get_base(core_idx);
        let offset = addr - base;

        match core_idx {
            0 => Ok(self.core0.read(offset)),
            _ => Ok(self.core1.read(offset)),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let core_idx = self.get_core_index(addr);
        let base = self.get_base(core_idx);
        let offset = addr - base;

        match core_idx {
            0 => self.core0.write(offset, value),
            _ => self.core1.write(offset, value),
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.core0.reset();
        self.core1.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE0: u32 = NVIC_BASE_CORE0;
    const BASE1: u32 = NVIC_BASE_CORE1;

    // ==================== NvicCore Tests ====================

    #[test]
    fn test_nvic_core_creation() {
        let nvic = NvicCore::new();

        // All interrupts disabled
        assert!(!nvic.is_enabled(0));
        assert!(!nvic.is_enabled(127));

        // No pending interrupts
        assert!(!nvic.is_pending(0));

        // No active interrupts
        assert!(!nvic.is_active(0));
    }

    #[test]
    fn test_nvic_core_default() {
        let nvic = NvicCore::default();
        assert!(!nvic.is_enabled(0));
    }

    #[test]
    fn test_enable_disable_irq() {
        let mut nvic = NvicCore::new();

        // Enable IRQ 0
        nvic.enable_irq(0);
        assert!(nvic.is_enabled(0));

        // Disable IRQ 0
        nvic.disable_irq(0);
        assert!(!nvic.is_enabled(0));
    }

    #[test]
    fn test_enable_multiple_irqs() {
        let mut nvic = NvicCore::new();

        nvic.enable_irq(0);
        nvic.enable_irq(31);
        nvic.enable_irq(32);
        nvic.enable_irq(127);

        assert!(nvic.is_enabled(0));
        assert!(nvic.is_enabled(31));
        assert!(nvic.is_enabled(32));
        assert!(nvic.is_enabled(127));

        // IRQ 128 doesn't exist
        assert!(!nvic.is_enabled(128));
    }

    #[test]
    fn test_set_clear_pending() {
        let mut nvic = NvicCore::new();

        nvic.set_pending(5);
        assert!(nvic.is_pending(5));

        nvic.clear_pending(5);
        assert!(!nvic.is_pending(5));
    }

    #[test]
    fn test_active_state() {
        let mut nvic = NvicCore::new();

        assert!(!nvic.is_active(10));

        // Use acknowledge to set active
        nvic.set_pending(10);
        nvic.acknowledge(10);
        assert!(nvic.is_active(10));

        nvic.end_of_interrupt(10);
        assert!(!nvic.is_active(10));
    }

    #[test]
    fn test_priority() {
        let mut nvic = NvicCore::new();

        // Default priority is 0
        assert_eq!(nvic.get_priority(0), 0);

        // Set priority
        nvic.set_priority(0, 100);
        assert_eq!(nvic.get_priority(0), 100);

        nvic.set_priority(127, 255);
        assert_eq!(nvic.get_priority(127), 255);
    }

    #[test]
    fn test_priority_multiple_irqs_per_register() {
        let mut nvic = NvicCore::new();

        // Each IPR register holds 4 priorities
        nvic.set_priority(0, 0x11);
        nvic.set_priority(1, 0x22);
        nvic.set_priority(2, 0x33);
        nvic.set_priority(3, 0x44);

        assert_eq!(nvic.get_priority(0), 0x11);
        assert_eq!(nvic.get_priority(1), 0x22);
        assert_eq!(nvic.get_priority(2), 0x33);
        assert_eq!(nvic.get_priority(3), 0x44);
    }

    #[test]
    fn test_vector_table() {
        let mut nvic = NvicCore::new();

        assert_eq!(nvic.get_vector_table(), 0);

        nvic.set_vector_table(0x2000_0000);
        assert_eq!(nvic.get_vector_table(), 0x2000_0000);
    }

    #[test]
    fn test_get_vector_address() {
        let mut nvic = NvicCore::new();

        nvic.set_vector_table(0x2000_0000);

        // IRQ 0 = vector 16
        assert_eq!(nvic.get_vector_address(0), 0x2000_0000 + 16 * 4);

        // IRQ 5 = vector 21
        assert_eq!(nvic.get_vector_address(5), 0x2000_0000 + 21 * 4);
    }

    #[test]
    fn test_highest_pending() {
        let mut nvic = NvicCore::new();

        // No pending
        assert!(nvic.get_highest_pending().is_none());

        // Set pending but not enabled
        nvic.set_pending(10);
        assert!(nvic.get_highest_pending().is_none());

        // Enable and set pending
        nvic.enable_irq(10);
        assert_eq!(nvic.get_highest_pending(), Some(10));

        // Lower priority (lower value = higher priority)
        nvic.set_priority(10, 50);
        nvic.enable_irq(20);
        nvic.set_pending(20);
        nvic.set_priority(20, 30); // Higher priority
        assert_eq!(nvic.get_highest_pending(), Some(20));
    }

    #[test]
    fn test_acknowledge_and_eoi() {
        let mut nvic = NvicCore::new();

        nvic.enable_irq(5);
        nvic.set_pending(5);

        // Acknowledge
        nvic.acknowledge(5);
        assert!(!nvic.is_pending(5));
        assert!(nvic.is_active(5));
        assert_eq!(nvic.current_irq, Some(5));

        // End of interrupt
        nvic.end_of_interrupt(5);
        assert!(!nvic.is_active(5));
        assert_eq!(nvic.current_irq, None);
    }

    #[test]
    fn test_nvic_core_reset() {
        let mut nvic = NvicCore::new();

        nvic.enable_irq(10);
        nvic.set_pending(10);
        nvic.set_priority(10, 100);

        nvic.reset();

        assert!(!nvic.is_enabled(10));
        assert!(!nvic.is_pending(10));
        assert_eq!(nvic.get_priority(10), 0);
    }

    // ==================== Register Read/Write Tests ====================

    #[test]
    fn test_iser_read_write() {
        let mut nvic = NvicCore::new();

        // Write to ISER0
        nvic.write(regs::ISER0, 0xFFFFFFFF);

        // Read back - all bits set
        assert_eq!(nvic.read(regs::ISER0), 0xFFFFFFFF);

        // All IRQs 0-31 enabled
        for irq in 0..32 {
            assert!(nvic.is_enabled(irq));
        }
    }

    #[test]
    fn test_icer_clear_enable() {
        let mut nvic = NvicCore::new();

        nvic.write(regs::ISER0, 0xFFFFFFFF);
        assert_eq!(nvic.read(regs::ISER0), 0xFFFFFFFF);

        // Clear some bits via ICER0
        nvic.write(regs::ICER0, 0x0000_FFFF);

        assert_eq!(nvic.read(regs::ISER0), 0xFFFF_0000);
    }

    #[test]
    fn test_ispr_icpr() {
        let mut nvic = NvicCore::new();

        // Set pending via ISPR
        nvic.write(regs::ISPR0, 0x0000_0001);
        assert!(nvic.is_pending(0));

        // Read back
        assert_eq!(nvic.read(regs::ISPR0), 0x0000_0001);

        // Clear via ICPR
        nvic.write(regs::ICPR0, 0x0000_0001);
        assert!(!nvic.is_pending(0));
    }

    #[test]
    fn test_iabr_read_only() {
        let nvic = NvicCore::new();

        // Read active bits (should be 0)
        assert_eq!(nvic.read(regs::IABR0), 0);
    }

    #[test]
    fn test_ipr_read_write() {
        let mut nvic = NvicCore::new();

        // Write to IPR0
        nvic.write(regs::IPR0, 0x4433_2211);

        assert_eq!(nvic.get_priority(0), 0x11);
        assert_eq!(nvic.get_priority(1), 0x22);
        assert_eq!(nvic.get_priority(2), 0x33);
        assert_eq!(nvic.get_priority(3), 0x44);
    }

    #[test]
    fn test_stir() {
        let mut nvic = NvicCore::new();

        // Trigger software interrupt
        nvic.write(regs::STIR, 10);
        assert!(nvic.is_pending(10));
    }

    // ==================== Nvic (Dual Core) Tests ====================

    #[test]
    fn test_nvic_creation() {
        let nvic = Nvic::new();

        assert!(!nvic.get_core(0).is_enabled(0));
        assert!(!nvic.get_core(1).is_enabled(0));
    }

    #[test]
    fn test_nvic_default() {
        let nvic = Nvic::default();
        assert!(!nvic.core0.is_enabled(0));
    }

    #[test]
    fn test_nvic_core_independence() {
        let mut nvic = Nvic::new();

        // Enable IRQ 5 on core 0 only
        nvic.write(BASE0 + regs::ISER0, 1 << 5).unwrap();

        assert!(nvic.get_core(0).is_enabled(5));
        assert!(!nvic.get_core(1).is_enabled(5));
    }

    #[test]
    fn test_nvic_trigger_irq() {
        let mut nvic = Nvic::new();

        nvic.trigger_irq(10);

        assert!(nvic.get_core(0).is_pending(10));
        assert!(nvic.get_core(1).is_pending(10));
    }

    #[test]
    fn test_nvic_read_core0() {
        let mut nvic = Nvic::new();

        nvic.write(BASE0 + regs::ISER0, 0xFF).unwrap();
        let val = nvic.read(BASE0 + regs::ISER0).unwrap();
        assert_eq!(val, 0xFF);
    }

    #[test]
    fn test_nvic_read_core1() {
        let mut nvic = Nvic::new();

        nvic.write(BASE1 + regs::ISER1, 0xFF).unwrap();
        let val = nvic.read(BASE1 + regs::ISER1).unwrap();
        assert_eq!(val, 0xFF);
    }

    #[test]
    fn test_nvic_write_core0() {
        let mut nvic = Nvic::new();

        nvic.write(BASE0 + regs::ISER0, 0x12345678).unwrap();
        assert_eq!(nvic.core0.read(regs::ISER0), 0x12345678);
    }

    #[test]
    fn test_nvic_write_core1() {
        let mut nvic = Nvic::new();

        nvic.write(BASE1 + regs::ISER1, 0x12345678).unwrap();
        assert_eq!(nvic.core1.read(regs::ISER1), 0x12345678);
    }

    #[test]
    fn test_nvic_reset() {
        let mut nvic = Nvic::new();

        nvic.write(BASE0 + regs::ISER0, 0xFFFFFFFF).unwrap();
        nvic.write(BASE1 + regs::ISER0, 0xFFFFFFFF).unwrap();

        nvic.reset();

        assert!(!nvic.core0.is_enabled(0));
        assert!(!nvic.core1.is_enabled(0));
    }

    #[test]
    fn test_nvic_device_id() {
        let nvic = Nvic::new();
        assert_eq!(nvic.id(), DeviceId::NVIC);
    }

    #[test]
    fn test_nvic_get_core_mut() {
        let mut nvic = Nvic::new();

        let core0 = nvic.get_core_mut(0);
        core0.enable_irq(5);

        assert!(nvic.get_core(0).is_enabled(5));
    }

    #[test]
    fn test_nvic_all_iser_registers() {
        let mut nvic = NvicCore::new();

        nvic.write(regs::ISER0, 0xFFFFFFFF);
        nvic.write(regs::ISER1, 0xFFFFFFFF);
        nvic.write(regs::ISER2, 0xFFFFFFFF);
        nvic.write(regs::ISER3, 0xFFFFFFFF);

        // Check all 128 IRQs are enabled
        for irq in 0..128 {
            assert!(nvic.is_enabled(irq));
        }
    }

    #[test]
    fn test_nvic_all_ipr_registers() {
        let mut nvic = NvicCore::new();

        // Write to all priority registers
        for i in 0..16 {
            nvic.write(regs::IPR0 + i * 4, 0xFFFFFFFF);
        }

        // All priorities should be 0xFF
        for irq in 0..64 {
            assert_eq!(nvic.get_priority(irq), 0xFF);
        }
    }
}
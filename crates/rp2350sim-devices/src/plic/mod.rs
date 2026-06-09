//! PLIC (Platform Level Interrupt Controller) for RP2350.
//!
//! Implements the RISC-V PLIC for Hazard3 core.

use rp2350sim_core::{Device, DeviceId, Result};

/// PLIC base address.
pub const PLIC_BASE: u32 = 0x1000_0000;

/// PLIC register offsets.
pub mod regs {
    // Interrupt Pending
    pub const PENDING0: u32 = 0x0000;
    pub const PENDING1: u32 = 0x0004;
    pub const PENDING2: u32 = 0x0008;
    pub const PENDING3: u32 = 0x000C;
    // Interrupt Enable (per context)
    pub const ENABLE0_BASE: u32 = 0x0020;
    pub const ENABLE1_BASE: u32 = 0x0024;
    // Priority
    pub const PRIORITY_BASE: u32 = 0x1000;
    // Threshold
    pub const THRESHOLD0: u32 = 0x2000;
    pub const THRESHOLD1: u32 = 0x2010;
    // Claim/Complete
    pub const CLAIM0: u32 = 0x2004;
    pub const CLAIM1: u32 = 0x2014;
}

/// Number of interrupt sources.
const NUM_SOURCES: usize = 128;

/// Number of contexts (2 cores).
const NUM_CONTEXTS: usize = 2;

/// PLIC peripheral.
#[derive(Debug)]
pub struct Plic {
    /// Interrupt pending registers.
    pending: [u32; 4],
    /// Interrupt enable registers (per context).
    enable: [[u32; 4]; NUM_CONTEXTS],
    /// Interrupt priorities (per source).
    priority: [u32; NUM_SOURCES],
    /// Threshold (per context).
    threshold: [u32; NUM_CONTEXTS],
    /// Claim register (per context).
    claim: [u32; NUM_CONTEXTS],
}

impl Default for Plic {
    fn default() -> Self {
        Self::new()
    }
}

impl Plic {
    /// Create a new PLIC instance.
    pub fn new() -> Self {
        Self {
            pending: [0; 4],
            enable: [[0; 4]; NUM_CONTEXTS],
            priority: [0; NUM_SOURCES],
            threshold: [0; NUM_CONTEXTS],
            claim: [0; NUM_CONTEXTS],
        }
    }

    /// Check if interrupt is pending.
    pub fn is_pending(&self, source: u32) -> bool {
        if source == 0 || source >= NUM_SOURCES as u32 {
            return false;
        }
        let reg = (source / 32) as usize;
        let bit = source % 32;
        (self.pending[reg] & (1 << bit)) != 0
    }

    /// Set interrupt pending.
    pub fn set_pending(&mut self, source: u32) {
        if source == 0 || source >= NUM_SOURCES as u32 {
            return;
        }
        let reg = (source / 32) as usize;
        let bit = source % 32;
        self.pending[reg] |= 1 << bit;
    }

    /// Clear interrupt pending.
    pub fn clear_pending(&mut self, source: u32) {
        if source == 0 || source >= NUM_SOURCES as u32 {
            return;
        }
        let reg = (source / 32) as usize;
        let bit = source % 32;
        self.pending[reg] &= !(1 << bit);
    }

    /// Check if interrupt is enabled for context.
    pub fn is_enabled(&self, context: usize, source: u32) -> bool {
        if source == 0 || source >= NUM_SOURCES as u32 || context >= NUM_CONTEXTS {
            return false;
        }
        let reg = (source / 32) as usize;
        let bit = source % 32;
        (self.enable[context][reg] & (1 << bit)) != 0
    }

    /// Enable interrupt for context.
    pub fn enable(&mut self, context: usize, source: u32) {
        if source == 0 || source >= NUM_SOURCES as u32 || context >= NUM_CONTEXTS {
            return;
        }
        let reg = (source / 32) as usize;
        let bit = source % 32;
        self.enable[context][reg] |= 1 << bit;
    }

    /// Disable interrupt for context.
    pub fn disable(&mut self, context: usize, source: u32) {
        if source == 0 || source >= NUM_SOURCES as u32 || context >= NUM_CONTEXTS {
            return;
        }
        let reg = (source / 32) as usize;
        let bit = source % 32;
        self.enable[context][reg] &= !(1 << bit);
    }

    /// Get interrupt priority.
    pub fn get_priority(&self, source: u32) -> u32 {
        if source >= NUM_SOURCES as u32 {
            return 0;
        }
        self.priority[source as usize]
    }

    /// Set interrupt priority.
    pub fn set_priority(&mut self, source: u32, priority: u32) {
        if source >= NUM_SOURCES as u32 {
            return;
        }
        self.priority[source as usize] = priority & 0x7; // 3-bit priority
    }

    /// Get threshold for context.
    pub fn get_threshold(&self, context: usize) -> u32 {
        if context >= NUM_CONTEXTS {
            return 0;
        }
        self.threshold[context]
    }

    /// Set threshold for context.
    pub fn set_threshold(&mut self, context: usize, threshold: u32) {
        if context >= NUM_CONTEXTS {
            return;
        }
        self.threshold[context] = threshold & 0x7;
    }

    /// Claim the highest priority interrupt for context.
    pub fn claim(&mut self, context: usize) -> u32 {
        if context >= NUM_CONTEXTS {
            return 0;
        }

        let mut best_source: u32 = 0;
        let mut best_priority: u32 = 0;
        let threshold = self.threshold[context];

        for source in 1..NUM_SOURCES as u32 {
            if self.is_pending(source) && self.is_enabled(context, source) {
                let priority = self.get_priority(source);
                if priority > threshold && priority > best_priority {
                    best_priority = priority;
                    best_source = source;
                }
            }
        }

        if best_source > 0 {
            // Clear pending and store in claim register
            self.clear_pending(best_source);
            self.claim[context] = best_source;
        }

        best_source
    }

    /// Complete interrupt handling.
    pub fn complete(&mut self, context: usize, source: u32) {
        if context >= NUM_CONTEXTS || source >= NUM_SOURCES as u32 {
            return;
        }

        // Verify this is the claimed interrupt
        if self.claim[context] == source {
            self.claim[context] = 0;
        }
    }

    /// Check if there's a pending interrupt above threshold.
    pub fn has_pending(&self, context: usize) -> bool {
        if context >= NUM_CONTEXTS {
            return false;
        }

        let threshold = self.threshold[context];
        
        for source in 1..NUM_SOURCES as u32 {
            if self.is_pending(source) && self.is_enabled(context, source) {
                let priority = self.get_priority(source);
                if priority > threshold {
                    return true;
                }
            }
        }

        false
    }

    /// Read register.
    fn read_reg(&self, offset: u32) -> u32 {
        match offset {
            // Pending registers
            0x0000..=0x000F => {
                let idx = (offset / 4) as usize;
                if idx < 4 { self.pending[idx] } else { 0 }
            }
            // Enable registers for context 0
            0x0020..=0x002F => {
                let idx = ((offset - 0x0020) / 4) as usize;
                if idx < 4 { self.enable[0][idx] } else { 0 }
            }
            // Enable registers for context 1
            0x0030..=0x003F => {
                let idx = ((offset - 0x0030) / 4) as usize;
                if idx < 4 { self.enable[1][idx] } else { 0 }
            }
            // Priority registers
            0x1000..=0x11FC => {
                let source = ((offset - 0x1000) / 4) as usize;
                if source < NUM_SOURCES { self.priority[source] } else { 0 }
            }
            // Context 0 threshold
            0x2000 => self.threshold[0],
            // Context 0 claim
            0x2004 => self.claim[0],
            // Context 1 threshold
            0x2010 => self.threshold[1],
            // Context 1 claim
            0x2014 => self.claim[1],
            _ => 0,
        }
    }

    /// Write register.
    fn write_reg(&mut self, offset: u32, value: u32) {
        match offset {
            // Enable registers for context 0
            0x0020..=0x002F => {
                let idx = ((offset - 0x0020) / 4) as usize;
                if idx < 4 { self.enable[0][idx] = value; }
            }
            // Enable registers for context 1
            0x0030..=0x003F => {
                let idx = ((offset - 0x0030) / 4) as usize;
                if idx < 4 { self.enable[1][idx] = value; }
            }
            // Priority registers
            0x1000..=0x11FC => {
                let source = ((offset - 0x1000) / 4) as usize;
                if source < NUM_SOURCES { self.priority[source] = value & 0x7; }
            }
            // Context 0 threshold
            0x2000 => {
                self.threshold[0] = value & 0x7;
            }
            // Context 0 claim/complete
            0x2004 => {
                self.complete(0, value);
            }
            // Context 1 threshold
            0x2010 => {
                self.threshold[1] = value & 0x7;
            }
            // Context 1 claim/complete
            0x2014 => {
                self.complete(1, value);
            }
            _ => {}
        }
    }
}

impl Device for Plic {
    fn id(&self) -> DeviceId {
        DeviceId::PLIC
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - PLIC_BASE;
        Ok(self.read_reg(offset))
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - PLIC_BASE;
        self.write_reg(offset, value);
        Ok(())
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: u32 = PLIC_BASE;

    // ==================== Basic Creation Tests ====================

    #[test]
    fn test_plic_creation() {
        let plic = Plic::new();

        // No pending interrupts
        assert!(!plic.is_pending(1));
        assert!(!plic.is_pending(127));

        // No enabled interrupts
        assert!(!plic.is_enabled(0, 1));
        assert!(!plic.is_enabled(1, 1));

        // All priorities are 0
        assert_eq!(plic.get_priority(1), 0);
        assert_eq!(plic.get_priority(127), 0);

        // All thresholds are 0
        assert_eq!(plic.get_threshold(0), 0);
        assert_eq!(plic.get_threshold(1), 0);
    }

    #[test]
    fn test_plic_default() {
        let plic = Plic::default();
        assert!(!plic.is_pending(1));
    }

    // ==================== Pending Tests ====================

    #[test]
    fn test_set_pending() {
        let mut plic = Plic::new();

        plic.set_pending(1);
        assert!(plic.is_pending(1));

        plic.set_pending(64);
        assert!(plic.is_pending(64));

        plic.set_pending(127);
        assert!(plic.is_pending(127));
    }

    #[test]
    fn test_clear_pending() {
        let mut plic = Plic::new();

        plic.set_pending(10);
        assert!(plic.is_pending(10));

        plic.clear_pending(10);
        assert!(!plic.is_pending(10));
    }

    #[test]
    fn test_pending_invalid_source() {
        let mut plic = Plic::new();

        // Source 0 is invalid
        plic.set_pending(0);
        assert!(!plic.is_pending(0));

        // Source 128+ is out of range
        plic.set_pending(128);
        assert!(!plic.is_pending(128));
    }

    // ==================== Enable Tests ====================

    #[test]
    fn test_enable_disable() {
        let mut plic = Plic::new();

        // Enable for context 0
        plic.enable(0, 5);
        assert!(plic.is_enabled(0, 5));
        assert!(!plic.is_enabled(1, 5)); // Not enabled for context 1

        // Enable for context 1
        plic.enable(1, 5);
        assert!(plic.is_enabled(1, 5));

        // Disable for context 0
        plic.disable(0, 5);
        assert!(!plic.is_enabled(0, 5));
        assert!(plic.is_enabled(1, 5)); // Still enabled for context 1
    }

    #[test]
    fn test_enable_multiple_sources() {
        let mut plic = Plic::new();

        for source in 1..=32 {
            plic.enable(0, source);
        }

        for source in 1..=32 {
            assert!(plic.is_enabled(0, source));
        }
    }

    #[test]
    fn test_enable_invalid_params() {
        let mut plic = Plic::new();

        // Invalid source
        plic.enable(0, 0);
        assert!(!plic.is_enabled(0, 0));

        plic.enable(0, 128);
        assert!(!plic.is_enabled(0, 128));

        // Invalid context
        plic.enable(2, 5);
        assert!(!plic.is_enabled(2, 5));
    }

    // ==================== Priority Tests ====================

    #[test]
    fn test_priority() {
        let mut plic = Plic::new();

        plic.set_priority(1, 3);
        assert_eq!(plic.get_priority(1), 3);

        plic.set_priority(50, 7);
        assert_eq!(plic.get_priority(50), 7);

        plic.set_priority(127, 1);
        assert_eq!(plic.get_priority(127), 1);
    }

    #[test]
    fn test_priority_masked() {
        let mut plic = Plic::new();

        // Priority is 3-bit, should mask to 7
        plic.set_priority(1, 15);
        assert_eq!(plic.get_priority(1), 7);

        plic.set_priority(1, 8);
        assert_eq!(plic.get_priority(1), 0); // 8 & 0x7 = 0
    }

    #[test]
    fn test_priority_invalid_source() {
        let mut plic = Plic::new();

        plic.set_priority(128, 5);
        assert_eq!(plic.get_priority(128), 0);
    }

    // ==================== Threshold Tests ====================

    #[test]
    fn test_threshold() {
        let mut plic = Plic::new();

        plic.set_threshold(0, 3);
        assert_eq!(plic.get_threshold(0), 3);

        plic.set_threshold(1, 5);
        assert_eq!(plic.get_threshold(1), 5);
    }

    #[test]
    fn test_threshold_masked() {
        let mut plic = Plic::new();

        // Threshold is 3-bit, should mask
        plic.set_threshold(0, 15);
        assert_eq!(plic.get_threshold(0), 7);
    }

    #[test]
    fn test_threshold_invalid_context() {
        let mut plic = Plic::new();

        plic.set_threshold(2, 5);
        assert_eq!(plic.get_threshold(2), 0);
    }

    // ==================== Claim/Complete Tests ====================

    #[test]
    fn test_claim_no_interrupt() {
        let mut plic = Plic::new();

        // No pending interrupt
        assert_eq!(plic.claim(0), 0);
        assert_eq!(plic.claim(1), 0);
    }

    #[test]
    fn test_claim_single_interrupt() {
        let mut plic = Plic::new();

        plic.set_pending(5);
        plic.enable(0, 5);
        plic.set_priority(5, 3);

        let claimed = plic.claim(0);
        assert_eq!(claimed, 5);

        // Pending should be cleared
        assert!(!plic.is_pending(5));
    }

    #[test]
    fn test_claim_highest_priority() {
        let mut plic = Plic::new();

        plic.set_pending(5);
        plic.set_pending(10);
        plic.set_pending(20);

        plic.enable(0, 5);
        plic.enable(0, 10);
        plic.enable(0, 20);

        plic.set_priority(5, 2);
        plic.set_priority(10, 7); // Highest
        plic.set_priority(20, 4);

        let claimed = plic.claim(0);
        assert_eq!(claimed, 10); // Should claim highest priority
    }

    #[test]
    fn test_claim_respects_threshold() {
        let mut plic = Plic::new();

        plic.set_pending(5);
        plic.enable(0, 5);
        plic.set_priority(5, 3);

        // Set threshold higher than priority
        plic.set_threshold(0, 5);

        let claimed = plic.claim(0);
        assert_eq!(claimed, 0); // Should not claim

        // Lower threshold
        plic.set_threshold(0, 2);
        let claimed = plic.claim(0);
        assert_eq!(claimed, 5); // Should claim now
    }

    #[test]
    fn test_complete() {
        let mut plic = Plic::new();

        plic.set_pending(5);
        plic.enable(0, 5);
        plic.set_priority(5, 3);

        let claimed = plic.claim(0);
        assert_eq!(claimed, 5);

        // Complete the interrupt
        plic.complete(0, 5);

        // Claim register should be cleared
        assert_eq!(plic.read(BASE + 0x2004).unwrap(), 0);
    }

    // ==================== Has Pending Tests ====================

    #[test]
    fn test_has_pending() {
        let mut plic = Plic::new();

        assert!(!plic.has_pending(0));

        plic.set_pending(5);
        assert!(!plic.has_pending(0)); // Not enabled

        plic.enable(0, 5);
        plic.set_priority(5, 1); // Need priority > 0 for has_pending to be true
        assert!(plic.has_pending(0));

        // Set priority below threshold
        plic.set_threshold(0, 5);
        assert!(!plic.has_pending(0)); // Priority 1 <= threshold 5

        plic.set_priority(5, 7);
        assert!(plic.has_pending(0)); // Priority 7 > threshold 5
    }

    // ==================== Register Read/Write Tests ====================

    #[test]
    fn test_read_pending_registers() {
        let mut plic = Plic::new();

        plic.set_pending(1); // Bit 1 in pending[0]
        plic.set_pending(32); // Bit 0 in pending[1]
        plic.set_pending(64); // Bit 0 in pending[2]

        assert_eq!(plic.read_reg(0x0000), 0b10); // Bit 1 set
        assert_eq!(plic.read_reg(0x0004), 0b1); // Bit 0 set
        assert_eq!(plic.read_reg(0x0008), 0b1); // Bit 0 set
    }

    #[test]
    fn test_read_enable_registers() {
        let mut plic = Plic::new();

        plic.enable(0, 5);
        plic.enable(1, 10);

        // Context 0 enable
        assert_eq!(plic.read_reg(0x0020), 1 << 5);

        // Context 1 enable
        assert_eq!(plic.read_reg(0x0030), 1 << 10);
    }

    #[test]
    fn test_write_enable_registers() {
        let mut plic = Plic::new();

        plic.write_reg(0x0020, 0xFFFFFFFF);
        assert_eq!(plic.read_reg(0x0020), 0xFFFFFFFF);

        // All sources 1-31 should be enabled for context 0
        for source in 1..=31 {
            assert!(plic.is_enabled(0, source));
        }
    }

    #[test]
    fn test_read_priority_registers() {
        let mut plic = Plic::new();

        plic.set_priority(0, 5);
        plic.set_priority(1, 3);

        assert_eq!(plic.read_reg(0x1000), 5); // Priority 0
        assert_eq!(plic.read_reg(0x1004), 3); // Priority 1
    }

    #[test]
    fn test_write_priority_registers() {
        let mut plic = Plic::new();

        plic.write_reg(0x1000, 5);
        assert_eq!(plic.get_priority(0), 5);

        plic.write_reg(0x1100, 7);
        assert_eq!(plic.get_priority(64), 7);
    }

    #[test]
    fn test_read_threshold_claim() {
        let mut plic = Plic::new();

        plic.set_threshold(0, 3);
        plic.set_threshold(1, 5);

        assert_eq!(plic.read_reg(0x2000), 3);
        assert_eq!(plic.read_reg(0x2010), 5);
    }

    #[test]
    fn test_write_threshold() {
        let mut plic = Plic::new();

        plic.write_reg(0x2000, 4);
        assert_eq!(plic.get_threshold(0), 4);

        plic.write_reg(0x2010, 6);
        assert_eq!(plic.get_threshold(1), 6);
    }

    #[test]
    fn test_write_claim_complete() {
        let mut plic = Plic::new();

        plic.set_pending(5);
        plic.enable(0, 5);
        plic.set_priority(5, 3);

        let claimed = plic.claim(0);
        assert_eq!(claimed, 5);

        // Write to claim register completes the interrupt
        plic.write_reg(0x2004, 5);

        // Claim register should be 0
        assert_eq!(plic.read_reg(0x2004), 0);
    }

    // ==================== Device Trait Tests ====================

    #[test]
    fn test_device_read() {
        let mut plic = Plic::new();

        plic.set_pending(5);

        let val = plic.read(BASE + 0x0000).unwrap();
        assert_eq!(val, 1 << 5);
    }

    #[test]
    fn test_device_write() {
        let mut plic = Plic::new();

        plic.write(BASE + 0x0020, 0xFF).unwrap();
        assert_eq!(plic.read(BASE + 0x0020).unwrap(), 0xFF);
    }

    #[test]
    fn test_device_reset() {
        let mut plic = Plic::new();

        plic.set_pending(5);
        plic.enable(0, 5);
        plic.set_priority(5, 3);
        plic.set_threshold(0, 2);

        plic.reset();

        assert!(!plic.is_pending(5));
        assert!(!plic.is_enabled(0, 5));
        assert_eq!(plic.get_priority(5), 0);
        assert_eq!(plic.get_threshold(0), 0);
    }

    #[test]
    fn test_device_id() {
        let plic = Plic::new();
        assert_eq!(plic.id(), DeviceId::PLIC);
    }

    // ==================== Context Independence Tests ====================

    #[test]
    fn test_context_independence() {
        let mut plic = Plic::new();

        // Set up for context 0
        plic.set_pending(5);
        plic.enable(0, 5);
        plic.set_priority(5, 3);

        // Context 0 should see it
        assert!(plic.has_pending(0));

        // Context 1 should not (not enabled)
        assert!(!plic.has_pending(1));

        // Enable for context 1
        plic.enable(1, 5);
        assert!(plic.has_pending(1));

        // Claim from context 0
        let claimed = plic.claim(0);
        assert_eq!(claimed, 5);

        // Pending is cleared, context 1 should not see it
        assert!(!plic.has_pending(1));
    }

    #[test]
    fn test_both_contexts_same_interrupt() {
        let mut plic = Plic::new();

        plic.set_pending(10);
        plic.enable(0, 10);
        plic.enable(1, 10);
        plic.set_priority(10, 5);

        // Both should see pending
        assert!(plic.has_pending(0));
        assert!(plic.has_pending(1));

        // Context 0 claims
        assert_eq!(plic.claim(0), 10);

        // Pending cleared, context 1 should not claim
        assert_eq!(plic.claim(1), 0);
    }

    #[test]
    fn test_context_isolation() {
        let mut plic = Plic::new();

        // Enable different IRQs for each context
        plic.enable(0, 5);
        plic.enable(0, 10);
        plic.enable(1, 15);
        plic.enable(1, 20);

        // Context 0 should only see its enabled IRQs
        assert!(plic.is_enabled(0, 5));
        assert!(plic.is_enabled(0, 10));
        assert!(!plic.is_enabled(0, 15));
        assert!(!plic.is_enabled(0, 20));

        // Context 1 should only see its enabled IRQs
        assert!(!plic.is_enabled(1, 5));
        assert!(!plic.is_enabled(1, 10));
        assert!(plic.is_enabled(1, 15));
        assert!(plic.is_enabled(1, 20));

        // Set pending on all
        plic.set_pending(5);
        plic.set_pending(10);
        plic.set_pending(15);
        plic.set_pending(20);

        plic.set_priority(5, 2);
        plic.set_priority(10, 3);
        plic.set_priority(15, 4);
        plic.set_priority(20, 5);

        // Each context claims its highest priority enabled interrupt
        assert_eq!(plic.claim(0), 10); // Priority 3 (highest enabled for ctx 0)
        assert_eq!(plic.claim(1), 20); // Priority 5 (highest enabled for ctx 1)
    }

    #[test]
    fn test_priority_masking_comprehensive() {
        let mut plic = Plic::new();

        // Test all 3-bit priority values (0-7)
        for priority in 0..=7 {
            plic.set_priority(1, priority);
            assert_eq!(plic.get_priority(1), priority);
        }

        // Test values above 7 are masked
        for value in 8..=255 {
            plic.set_priority(1, value);
            assert_eq!(plic.get_priority(1), value & 0x7);
        }

        // Verify threshold is also 3-bit masked
        for threshold in 0..=7 {
            plic.set_threshold(0, threshold);
            assert_eq!(plic.get_threshold(0), threshold);
        }

        plic.set_threshold(0, 15);
        assert_eq!(plic.get_threshold(0), 7);
    }

    #[test]
    fn test_multiple_contexts_dual_core() {
        let mut plic = Plic::new();

        // Set up shared interrupt with different priorities per core
        plic.set_pending(10);
        plic.set_pending(20);
        plic.set_pending(30);

        // Core 0 (context 0) enables all three
        plic.enable(0, 10);
        plic.enable(0, 20);
        plic.enable(0, 30);

        // Core 1 (context 1) enables only two
        plic.enable(1, 10);
        plic.enable(1, 30);

        plic.set_priority(10, 5);
        plic.set_priority(20, 7); // Highest overall
        plic.set_priority(30, 3);

        // Set different thresholds
        plic.set_threshold(0, 2);
        plic.set_threshold(1, 4); // Higher threshold for core 1

        // Core 0 can claim all enabled (highest is 20 with priority 7)
        let claimed0 = plic.claim(0);
        assert_eq!(claimed0, 20);

        // Core 1 can only claim 10 (priority 5 > threshold 4), not 30 (priority 3 < threshold 4)
        plic.set_pending(10); // Re-pend since core 0 cleared it
        plic.set_pending(30);
        let claimed1 = plic.claim(1);
        assert_eq!(claimed1, 10); // 10 has priority 5 > threshold 4, 30 has priority 3 < threshold 4
    }

    #[test]
    fn test_has_pending_comprehensive() {
        let mut plic = Plic::new();

        // Empty state - no pending
        assert!(!plic.has_pending(0));
        assert!(!plic.has_pending(1));

        // Pending but not enabled
        plic.set_pending(5);
        plic.set_priority(5, 7);
        assert!(!plic.has_pending(0));

        // Enabled but priority below threshold
        plic.enable(0, 5);
        plic.set_threshold(0, 7);
        assert!(!plic.has_pending(0)); // Priority 7 not > threshold 7

        // Priority above threshold
        plic.set_threshold(0, 6);
        assert!(plic.has_pending(0));

        // Different context, different threshold
        plic.set_threshold(1, 7);
        plic.enable(1, 5);
        assert!(!plic.has_pending(1)); // Threshold 7 blocks priority 7

        plic.set_threshold(1, 5);
        assert!(plic.has_pending(1));
    }

    #[test]
    fn test_source_zero_ignored() {
        let mut plic = Plic::new();

        // Source 0 is invalid - all operations should be no-ops
        plic.set_pending(0);
        assert!(!plic.is_pending(0));

        plic.enable(0, 0);
        assert!(!plic.is_enabled(0, 0));

        plic.set_priority(0, 7);
        // Priority 0 can still be set/read since source 0 priority register exists
        assert_eq!(plic.get_priority(0), 7);

        // Clear pending on source 0 should also be no-op
        plic.clear_pending(0);

        // Disable source 0 should be no-op
        plic.disable(0, 0);

        // Claim should never return source 0
        plic.set_pending(1);
        plic.enable(0, 1);
        plic.set_priority(1, 1);
        assert_eq!(plic.claim(0), 1); // Returns 1, not 0
    }

    #[test]
    fn test_out_of_bounds_source() {
        let mut plic = Plic::new();

        // Source 128 and above are out of bounds
        plic.set_pending(128);
        assert!(!plic.is_pending(128));

        plic.set_pending(200);
        assert!(!plic.is_pending(200));

        plic.enable(0, 128);
        assert!(!plic.is_enabled(0, 128));

        plic.set_priority(128, 5);
        assert_eq!(plic.get_priority(128), 0);

        // Clear and disable should also be no-ops
        plic.clear_pending(128);
        plic.disable(0, 128);

        // Claim with out-of-bounds should return 0
        let mut plic2 = Plic::new();
        plic2.set_pending(127);
        plic2.enable(0, 127);
        plic2.set_priority(127, 1);
        assert_eq!(plic2.claim(0), 127); // Max valid source
    }

    #[test]
    fn test_empty_pending_state() {
        let mut plic = Plic::new();

        // With no pending interrupts, claim returns 0
        assert_eq!(plic.claim(0), 0);
        assert_eq!(plic.claim(1), 0);

        // With no pending, has_pending is false
        assert!(!plic.has_pending(0));
        assert!(!plic.has_pending(1));

        // Enable some interrupts but don't set pending
        plic.enable(0, 1);
        plic.enable(0, 50);
        plic.enable(0, 100);
        plic.set_priority(1, 7);
        plic.set_priority(50, 7);
        plic.set_priority(100, 7);

        // Still no pending
        assert!(!plic.has_pending(0));
        assert_eq!(plic.claim(0), 0);

        // Verify pending registers are all zero
        assert_eq!(plic.read_reg(0x0000), 0);
        assert_eq!(plic.read_reg(0x0004), 0);
        assert_eq!(plic.read_reg(0x0008), 0);
        assert_eq!(plic.read_reg(0x000C), 0);
    }

    #[test]
    fn test_complete_wrong_source() {
        let mut plic = Plic::new();

        // Set up and claim an interrupt
        plic.set_pending(5);
        plic.enable(0, 5);
        plic.set_priority(5, 3);
        let claimed = plic.claim(0);
        assert_eq!(claimed, 5);

        // Try to complete with wrong source - should be ignored
        plic.complete(0, 10);

        // Claim register should still have the original claim
        assert_eq!(plic.read_reg(0x2004), 5);

        // Complete with correct source
        plic.complete(0, 5);
        assert_eq!(plic.read_reg(0x2004), 0);
    }

    #[test]
    fn test_claim_clears_pending() {
        let mut plic = Plic::new();

        plic.set_pending(5);
        plic.enable(0, 5);
        plic.set_priority(5, 3);

        assert!(plic.is_pending(5));

        // Claim clears pending
        plic.claim(0);
        assert!(!plic.is_pending(5));

        // Complete the interrupt
        plic.complete(0, 5);

        // Re-trigger by setting pending again
        plic.set_pending(5);
        assert!(plic.is_pending(5));

        // Can claim again
        assert_eq!(plic.claim(0), 5);
    }
}
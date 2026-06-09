//! Bus Control for RP2350.
//!
//! Implements the bus fabric control and configuration.

use rp2350sim_core::{Device, DeviceId, Result};

/// Bus Control base address.
pub const BUSCTRL_BASE: u32 = 0x4003_0000;

/// Bus Control register offsets.
pub mod regs {
    pub const BUS_PRIORITY: u32 = 0x000;
    pub const BUS_PRIORITY_ACK: u32 = 0x004;
    pub const BUS_PRIORITY_PERF0: u32 = 0x008;
    pub const BUS_PRIORITY_PERF1: u32 = 0x00C;
    pub const BUS_PRIORITY_PERF2: u32 = 0x010;
    pub const BUS_PRIORITY_PERF3: u32 = 0x014;
    pub const BUS_PRIORITY_PERF4: u32 = 0x018;
    pub const BUS_PRIORITY_PERF5: u32 = 0x01C;
    pub const BUS_PRIORITY_PERF6: u32 = 0x020;
    pub const BUS_PRIORITY_PERF7: u32 = 0x024;
    pub const BUS_PRIORITY_PERF8: u32 = 0x028;
    pub const BUS_PRIORITY_PERF9: u32 = 0x02C;
    pub const BUS_PRIORITY_PERF10: u32 = 0x030;
    pub const BUS_PRIORITY_PERF11: u32 = 0x034;
    pub const PERFCTR0: u32 = 0x038;
    pub const PERFCTR1: u32 = 0x03C;
    pub const PERFCTR2: u32 = 0x040;
    pub const PERFCTR3: u32 = 0x044;
    pub const PERFSEL0: u32 = 0x048;
    pub const PERFSEL1: u32 = 0x04C;
    pub const PERFSEL2: u32 = 0x050;
    pub const PERFSEL3: u32 = 0x054;
}

/// BUS_PRIORITY register bits.
pub mod bus_priority {
    pub const PROC0_SHIFT: u32 = 0;
    pub const PROC0_MASK: u32 = 0xFF << 0;
    pub const PROC1_SHIFT: u32 = 8;
    pub const PROC1_MASK: u32 = 0xFF << 8;
    pub const DMA_R_SHIFT: u32 = 16;
    pub const DMA_R_MASK: u32 = 0xFF << 16;
    pub const DMA_W_SHIFT: u32 = 24;
    pub const DMA_W_MASK: u32 = 0xFF << 24;
}

/// Performance counter event types.
pub mod perf_event {
    pub const APB_CONTESTED: u32 = 0x00;
    pub const APB: u32 = 0x01;
    pub const FASTPERIPH: u32 = 0x02;
    pub const SRAM5: u32 = 0x03;
    pub const SRAM4: u32 = 0x04;
    pub const SRAM3: u32 = 0x05;
    pub const SRAM2: u32 = 0x06;
    pub const SRAM1: u32 = 0x07;
    pub const SRAM0: u32 = 0x08;
    pub const XIP_MAIN: u32 = 0x09;
    pub const ROM: u32 = 0x0A;
}

/// Bus master identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusMaster {
    Proc0 = 0,
    Proc1 = 1,
    DmaRead = 2,
    DmaWrite = 3,
}

/// Performance counter.
#[derive(Debug, Clone, Copy, Default)]
pub struct PerfCounter {
    /// Counter value.
    pub value: u32,
    /// Event selector.
    pub event: u32,
}

/// Bus Control peripheral.
#[derive(Debug)]
pub struct BusControl {
    /// Bus priority register.
    priority: u32,
    /// Priority acknowledge (read-only).
    priority_ack: u32,
    /// Performance priorities for each bus master.
    perf_priority: [u32; 12],
    /// Performance counters.
    perf_counters: [PerfCounter; 4],
    /// Bus access counters (internal).
    access_counters: [u32; 11],
}

impl Default for BusControl {
    fn default() -> Self {
        Self::new()
    }
}

impl BusControl {
    /// Create a new Bus Control instance.
    pub fn new() -> Self {
        Self {
            priority: 0,
            priority_ack: 0,
            perf_priority: [0; 12],
            perf_counters: [PerfCounter::default(); 4],
            access_counters: [0; 11],
        }
    }

    /// Get priority for a bus master.
    pub fn get_priority(&self, master: BusMaster) -> u8 {
        match master {
            BusMaster::Proc0 => ((self.priority >> bus_priority::PROC0_SHIFT) & 0xFF) as u8,
            BusMaster::Proc1 => ((self.priority >> bus_priority::PROC1_SHIFT) & 0xFF) as u8,
            BusMaster::DmaRead => ((self.priority >> bus_priority::DMA_R_SHIFT) & 0xFF) as u8,
            BusMaster::DmaWrite => ((self.priority >> bus_priority::DMA_W_SHIFT) & 0xFF) as u8,
        }
    }

    /// Set priority for a bus master.
    pub fn set_priority(&mut self, master: BusMaster, priority: u8) {
        let shift = match master {
            BusMaster::Proc0 => bus_priority::PROC0_SHIFT,
            BusMaster::Proc1 => bus_priority::PROC1_SHIFT,
            BusMaster::DmaRead => bus_priority::DMA_R_SHIFT,
            BusMaster::DmaWrite => bus_priority::DMA_W_SHIFT,
        };
        
        self.priority = (self.priority & !(0xFF << shift)) | ((priority as u32) << shift);
        self.priority_ack = priority as u32;
    }

    /// Record a bus access.
    pub fn record_access(&mut self, event: u32) {
        if (event as usize) < self.access_counters.len() {
            self.access_counters[event as usize] += 1;
            
            // Update performance counters
            for counter in &mut self.perf_counters {
                if counter.event == event {
                    counter.value = counter.value.wrapping_add(1);
                }
            }
        }
    }

    /// Get performance counter value.
    pub fn get_perf_counter(&self, index: usize) -> u32 {
        if index < 4 {
            self.perf_counters[index].value
        } else {
            0
        }
    }

    /// Set performance event selector.
    pub fn set_perf_event(&mut self, index: usize, event: u32) {
        if index < 4 {
            self.perf_counters[index].event = event & 0x1F;
        }
    }

    /// Get performance event selector.
    pub fn get_perf_event(&self, index: usize) -> u32 {
        if index < 4 {
            self.perf_counters[index].event
        } else {
            0
        }
    }

    /// Check if bus is contested (multiple masters requesting).
    pub fn is_contested(&self) -> bool {
        // Simplified: check if more than one master has non-zero priority
        let priorities = [
            self.get_priority(BusMaster::Proc0),
            self.get_priority(BusMaster::Proc1),
            self.get_priority(BusMaster::DmaRead),
            self.get_priority(BusMaster::DmaWrite),
        ];
        
        priorities.iter().filter(|&&p| p > 0).count() > 1
    }

    /// Arbitrate bus access.
    pub fn arbitrate(&self) -> Option<BusMaster> {
        let priorities = [
            (BusMaster::Proc0, self.get_priority(BusMaster::Proc0)),
            (BusMaster::Proc1, self.get_priority(BusMaster::Proc1)),
            (BusMaster::DmaRead, self.get_priority(BusMaster::DmaRead)),
            (BusMaster::DmaWrite, self.get_priority(BusMaster::DmaWrite)),
        ];

        priorities
            .iter()
            .filter(|(_, p)| *p > 0)
            .max_by_key(|(_, p)| *p)
            .map(|(m, _)| *m)
    }
}

impl Device for BusControl {
    fn id(&self) -> DeviceId {
        DeviceId::BUSCTRL
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - BUSCTRL_BASE;

        match offset {
            regs::BUS_PRIORITY => Ok(self.priority),
            regs::BUS_PRIORITY_ACK => Ok(self.priority_ack),
            regs::BUS_PRIORITY_PERF0 => Ok(self.perf_priority[0]),
            regs::BUS_PRIORITY_PERF1 => Ok(self.perf_priority[1]),
            regs::BUS_PRIORITY_PERF2 => Ok(self.perf_priority[2]),
            regs::BUS_PRIORITY_PERF3 => Ok(self.perf_priority[3]),
            regs::BUS_PRIORITY_PERF4 => Ok(self.perf_priority[4]),
            regs::BUS_PRIORITY_PERF5 => Ok(self.perf_priority[5]),
            regs::BUS_PRIORITY_PERF6 => Ok(self.perf_priority[6]),
            regs::BUS_PRIORITY_PERF7 => Ok(self.perf_priority[7]),
            regs::BUS_PRIORITY_PERF8 => Ok(self.perf_priority[8]),
            regs::BUS_PRIORITY_PERF9 => Ok(self.perf_priority[9]),
            regs::BUS_PRIORITY_PERF10 => Ok(self.perf_priority[10]),
            regs::BUS_PRIORITY_PERF11 => Ok(self.perf_priority[11]),
            regs::PERFCTR0 => Ok(self.get_perf_counter(0)),
            regs::PERFCTR1 => Ok(self.get_perf_counter(1)),
            regs::PERFCTR2 => Ok(self.get_perf_counter(2)),
            regs::PERFCTR3 => Ok(self.get_perf_counter(3)),
            regs::PERFSEL0 => Ok(self.get_perf_event(0)),
            regs::PERFSEL1 => Ok(self.get_perf_event(1)),
            regs::PERFSEL2 => Ok(self.get_perf_event(2)),
            regs::PERFSEL3 => Ok(self.get_perf_event(3)),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - BUSCTRL_BASE;

        match offset {
            regs::BUS_PRIORITY => {
                self.priority = value;
            }
            regs::BUS_PRIORITY_PERF0 => self.perf_priority[0] = value,
            regs::BUS_PRIORITY_PERF1 => self.perf_priority[1] = value,
            regs::BUS_PRIORITY_PERF2 => self.perf_priority[2] = value,
            regs::BUS_PRIORITY_PERF3 => self.perf_priority[3] = value,
            regs::BUS_PRIORITY_PERF4 => self.perf_priority[4] = value,
            regs::BUS_PRIORITY_PERF5 => self.perf_priority[5] = value,
            regs::BUS_PRIORITY_PERF6 => self.perf_priority[6] = value,
            regs::BUS_PRIORITY_PERF7 => self.perf_priority[7] = value,
            regs::BUS_PRIORITY_PERF8 => self.perf_priority[8] = value,
            regs::BUS_PRIORITY_PERF9 => self.perf_priority[9] = value,
            regs::BUS_PRIORITY_PERF10 => self.perf_priority[10] = value,
            regs::BUS_PRIORITY_PERF11 => self.perf_priority[11] = value,
            regs::PERFCTR0 => self.perf_counters[0].value = value,
            regs::PERFCTR1 => self.perf_counters[1].value = value,
            regs::PERFCTR2 => self.perf_counters[2].value = value,
            regs::PERFCTR3 => self.perf_counters[3].value = value,
            regs::PERFSEL0 => self.set_perf_event(0, value),
            regs::PERFSEL1 => self.set_perf_event(1, value),
            regs::PERFSEL2 => self.set_perf_event(2, value),
            regs::PERFSEL3 => self.set_perf_event(3, value),
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

    const BASE: u32 = BUSCTRL_BASE;

    // ==================== Basic Creation Tests ====================

    #[test]
    fn test_busctrl_creation() {
        let bus = BusControl::new();

        assert_eq!(bus.priority, 0);
        assert_eq!(bus.priority_ack, 0);

        // All priorities are 0
        assert_eq!(bus.get_priority(BusMaster::Proc0), 0);
        assert_eq!(bus.get_priority(BusMaster::Proc1), 0);
        assert_eq!(bus.get_priority(BusMaster::DmaRead), 0);
        assert_eq!(bus.get_priority(BusMaster::DmaWrite), 0);

        // All performance counters are 0
        for i in 0..4 {
            assert_eq!(bus.get_perf_counter(i), 0);
        }
    }

    #[test]
    fn test_busctrl_default() {
        let bus = BusControl::default();
        assert_eq!(bus.priority, 0);
    }

    // ==================== Priority Tests ====================

    #[test]
    fn test_set_priority_proc0() {
        let mut bus = BusControl::new();

        bus.set_priority(BusMaster::Proc0, 50);
        assert_eq!(bus.get_priority(BusMaster::Proc0), 50);
        assert_eq!(bus.priority_ack, 50);
    }

    #[test]
    fn test_set_priority_proc1() {
        let mut bus = BusControl::new();

        bus.set_priority(BusMaster::Proc1, 75);
        assert_eq!(bus.get_priority(BusMaster::Proc1), 75);
    }

    #[test]
    fn test_set_priority_dma_read() {
        let mut bus = BusControl::new();

        bus.set_priority(BusMaster::DmaRead, 100);
        assert_eq!(bus.get_priority(BusMaster::DmaRead), 100);
    }

    #[test]
    fn test_set_priority_dma_write() {
        let mut bus = BusControl::new();

        bus.set_priority(BusMaster::DmaWrite, 200);
        assert_eq!(bus.get_priority(BusMaster::DmaWrite), 200);
    }

    #[test]
    fn test_priority_all_masters() {
        let mut bus = BusControl::new();

        bus.set_priority(BusMaster::Proc0, 10);
        bus.set_priority(BusMaster::Proc1, 20);
        bus.set_priority(BusMaster::DmaRead, 30);
        bus.set_priority(BusMaster::DmaWrite, 40);

        assert_eq!(bus.get_priority(BusMaster::Proc0), 10);
        assert_eq!(bus.get_priority(BusMaster::Proc1), 20);
        assert_eq!(bus.get_priority(BusMaster::DmaRead), 30);
        assert_eq!(bus.get_priority(BusMaster::DmaWrite), 40);
    }

    #[test]
    fn test_priority_register_encoding() {
        let mut bus = BusControl::new();

        // Set all priorities
        bus.set_priority(BusMaster::Proc0, 0x11);
        bus.set_priority(BusMaster::Proc1, 0x22);
        bus.set_priority(BusMaster::DmaRead, 0x33);
        bus.set_priority(BusMaster::DmaWrite, 0x44);

        // Check raw register value
        assert_eq!(bus.priority, 0x44332211);
    }

    // ==================== Arbitration Tests ====================

    #[test]
    fn test_arbitrate_no_master() {
        let bus = BusControl::new();
        assert!(bus.arbitrate().is_none());
    }

    #[test]
    fn test_arbitrate_single_master() {
        let mut bus = BusControl::new();

        bus.set_priority(BusMaster::Proc0, 10);
        assert_eq!(bus.arbitrate(), Some(BusMaster::Proc0));
    }

    #[test]
    fn test_arbitrate_highest_priority() {
        let mut bus = BusControl::new();

        bus.set_priority(BusMaster::Proc0, 10);
        bus.set_priority(BusMaster::Proc1, 50);
        bus.set_priority(BusMaster::DmaRead, 30);

        assert_eq!(bus.arbitrate(), Some(BusMaster::Proc1));
    }

    #[test]
    fn test_is_contested() {
        let mut bus = BusControl::new();

        // No masters
        assert!(!bus.is_contested());

        // Single master
        bus.set_priority(BusMaster::Proc0, 10);
        assert!(!bus.is_contested());

        // Two masters
        bus.set_priority(BusMaster::Proc1, 20);
        assert!(bus.is_contested());
    }

    // ==================== Performance Counter Tests ====================

    #[test]
    fn test_perf_counter_initial() {
        let bus = BusControl::new();

        for i in 0..4 {
            assert_eq!(bus.get_perf_counter(i), 0);
            assert_eq!(bus.get_perf_event(i), 0);
        }
    }

    #[test]
    fn test_set_perf_event() {
        let mut bus = BusControl::new();

        bus.set_perf_event(0, perf_event::APB);
        assert_eq!(bus.get_perf_event(0), perf_event::APB);

        bus.set_perf_event(1, perf_event::SRAM0);
        assert_eq!(bus.get_perf_event(1), perf_event::SRAM0);

        bus.set_perf_event(2, perf_event::XIP_MAIN);
        assert_eq!(bus.get_perf_event(2), perf_event::XIP_MAIN);

        bus.set_perf_event(3, perf_event::ROM);
        assert_eq!(bus.get_perf_event(3), perf_event::ROM);
    }

    #[test]
    fn test_perf_event_masked() {
        let mut bus = BusControl::new();

        // Event is 5-bit (0x1F mask)
        bus.set_perf_event(0, 0xFF);
        assert_eq!(bus.get_perf_event(0), 0x1F);
    }

    #[test]
    fn test_perf_event_invalid_index() {
        let mut bus = BusControl::new();

        bus.set_perf_event(4, 5);
        assert_eq!(bus.get_perf_event(4), 0);
    }

    #[test]
    fn test_record_access() {
        let mut bus = BusControl::new();

        bus.set_perf_event(0, perf_event::APB);

        bus.record_access(perf_event::APB);
        assert_eq!(bus.get_perf_counter(0), 1);

        bus.record_access(perf_event::APB);
        assert_eq!(bus.get_perf_counter(0), 2);
    }

    #[test]
    fn test_record_access_multiple_counters() {
        let mut bus = BusControl::new();

        bus.set_perf_event(0, perf_event::APB);
        bus.set_perf_event(1, perf_event::SRAM0);

        bus.record_access(perf_event::APB);
        bus.record_access(perf_event::SRAM0);
        bus.record_access(perf_event::APB);

        assert_eq!(bus.get_perf_counter(0), 2);
        assert_eq!(bus.get_perf_counter(1), 1);
    }

    #[test]
    fn test_record_access_wrapping() {
        let mut bus = BusControl::new();

        bus.set_perf_event(0, perf_event::APB);
        bus.perf_counters[0].value = 0xFFFFFFFF;

        bus.record_access(perf_event::APB);
        assert_eq!(bus.get_perf_counter(0), 0); // Wraps to 0
    }

    // ==================== Register Read/Write Tests ====================

    #[test]
    fn test_read_bus_priority() {
        let mut bus = BusControl::new();

        bus.priority = 0x12345678;
        assert_eq!(bus.read(BASE + regs::BUS_PRIORITY).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_bus_priority_ack() {
        let mut bus = BusControl::new();

        bus.priority_ack = 0x99;
        assert_eq!(bus.read(BASE + regs::BUS_PRIORITY_ACK).unwrap(), 0x99);
    }

    #[test]
    fn test_read_perf_priority() {
        let mut bus = BusControl::new();

        bus.perf_priority[0] = 100;
        bus.perf_priority[5] = 200;
        bus.perf_priority[11] = 300;

        assert_eq!(bus.read(BASE + regs::BUS_PRIORITY_PERF0).unwrap(), 100);
        assert_eq!(bus.read(BASE + regs::BUS_PRIORITY_PERF5).unwrap(), 200);
        assert_eq!(bus.read(BASE + regs::BUS_PRIORITY_PERF11).unwrap(), 300);
    }

    #[test]
    fn test_write_bus_priority() {
        let mut bus = BusControl::new();

        bus.write(BASE + regs::BUS_PRIORITY, 0x12345678).unwrap();
        assert_eq!(bus.priority, 0x12345678);
    }

    #[test]
    fn test_write_perf_priority() {
        let mut bus = BusControl::new();

        bus.write(BASE + regs::BUS_PRIORITY_PERF0, 100).unwrap();
        assert_eq!(bus.perf_priority[0], 100);

        bus.write(BASE + regs::BUS_PRIORITY_PERF11, 300).unwrap();
        assert_eq!(bus.perf_priority[11], 300);
    }

    #[test]
    fn test_read_perf_counter() {
        let mut bus = BusControl::new();

        bus.perf_counters[0].value = 1000;
        bus.perf_counters[3].value = 5000;

        assert_eq!(bus.read(BASE + regs::PERFCTR0).unwrap(), 1000);
        assert_eq!(bus.read(BASE + regs::PERFCTR3).unwrap(), 5000);
    }

    #[test]
    fn test_write_perf_counter() {
        let mut bus = BusControl::new();

        bus.write(BASE + regs::PERFCTR0, 1000).unwrap();
        assert_eq!(bus.perf_counters[0].value, 1000);

        bus.write(BASE + regs::PERFCTR3, 5000).unwrap();
        assert_eq!(bus.perf_counters[3].value, 5000);
    }

    #[test]
    fn test_read_perfsel() {
        let mut bus = BusControl::new();

        bus.set_perf_event(0, perf_event::APB);
        bus.set_perf_event(3, perf_event::ROM);

        assert_eq!(bus.read(BASE + regs::PERFSEL0).unwrap(), perf_event::APB);
        assert_eq!(bus.read(BASE + regs::PERFSEL3).unwrap(), perf_event::ROM);
    }

    #[test]
    fn test_write_perfsel() {
        let mut bus = BusControl::new();

        bus.write(BASE + regs::PERFSEL0, perf_event::SRAM5).unwrap();
        assert_eq!(bus.get_perf_event(0), perf_event::SRAM5);
    }

    #[test]
    fn test_read_invalid_register() {
        let mut bus = BusControl::new();
        assert_eq!(bus.read(BASE + 0x1000).unwrap(), 0);
    }

    #[test]
    fn test_write_invalid_register() {
        let mut bus = BusControl::new();
        // Should not panic
        bus.write(BASE + 0x1000, 0xFFFFFFFF).unwrap();
    }

    // ==================== Device Trait Tests ====================

    #[test]
    fn test_device_id() {
        let bus = BusControl::new();
        assert_eq!(bus.id(), DeviceId::BUSCTRL);
    }

    #[test]
    fn test_device_reset() {
        let mut bus = BusControl::new();

        bus.priority = 0x12345678;
        bus.perf_counters[0].value = 1000;
        bus.set_perf_event(0, perf_event::APB);

        bus.reset();

        assert_eq!(bus.priority, 0);
        assert_eq!(bus.get_perf_counter(0), 0);
        assert_eq!(bus.get_perf_event(0), 0);
    }

    // ==================== BusMaster Enum Tests ====================

    #[test]
    fn test_bus_master_values() {
        assert_eq!(BusMaster::Proc0 as u8, 0);
        assert_eq!(BusMaster::Proc1 as u8, 1);
        assert_eq!(BusMaster::DmaRead as u8, 2);
        assert_eq!(BusMaster::DmaWrite as u8, 3);
    }

    #[test]
    fn test_bus_master_equality() {
        assert_eq!(BusMaster::Proc0, BusMaster::Proc0);
        assert_ne!(BusMaster::Proc0, BusMaster::Proc1);
    }

    // ==================== PerfCounter Tests ====================

    #[test]
    fn test_perf_counter_default() {
        let counter = PerfCounter::default();
        assert_eq!(counter.value, 0);
        assert_eq!(counter.event, 0);
    }

    #[test]
    fn test_perf_counter_clone() {
        let counter = PerfCounter { value: 100, event: 5 };
        let cloned = counter.clone();
        assert_eq!(cloned.value, 100);
        assert_eq!(cloned.event, 5);
    }
}
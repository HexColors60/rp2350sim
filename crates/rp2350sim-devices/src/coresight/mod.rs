//! CoreSight Debug Components for RP2350.
//!
//! Implements CoreSight debug and trace components.

use rp2350sim_core::{Device, DeviceId, Result};

/// CoreSight base addresses.
pub mod base {
    pub const PPB: u32 = 0xE000_0000;
    pub const DWT: u32 = 0xE000_1000;
    pub const FPB: u32 = 0xE000_2000;
    pub const ITM: u32 = 0xE000_0000;
    pub const TPIU: u32 = 0xE004_0000;
    pub const ETM: u32 = 0xE004_1000;
    pub const CTI: u32 = 0xE004_2000;
    pub const DEBUG: u32 = 0xE005_0000;
}

/// DWT (Data Watchpoint and Trace) registers.
pub mod dwt {
    pub const CTRL: u32 = 0x000;
    pub const CYCCNT: u32 = 0x004;
    pub const CPICNT: u32 = 0x008;
    pub const EXCCNT: u32 = 0x00C;
    pub const SLEEPCNT: u32 = 0x010;
    pub const LSUCNT: u32 = 0x014;
    pub const FOLDCNT: u32 = 0x018;
    pub const PCSR: u32 = 0x01C;
    pub const COMP0: u32 = 0x020;
    pub const MASK0: u32 = 0x024;
    pub const FUNCTION0: u32 = 0x028;
    pub const COMP1: u32 = 0x030;
    pub const MASK1: u32 = 0x034;
    pub const FUNCTION1: u32 = 0x038;
    pub const COMP2: u32 = 0x040;
    pub const MASK2: u32 = 0x044;
    pub const FUNCTION2: u32 = 0x048;
    pub const COMP3: u32 = 0x050;
    pub const MASK3: u32 = 0x054;
    pub const FUNCTION3: u32 = 0x058;
}

/// DWT CTRL bits.
pub mod dwt_ctrl {
    pub const NUMCOMP_SHIFT: u32 = 28;
    pub const NUMCOMP_MASK: u32 = 0xF << 28;
    pub const TRCEN: u32 = 1 << 12;
    pub const CYCCNTENA: u32 = 1 << 0;
}

/// FPB (Flash Patch and Breakpoint) registers.
pub mod fpb {
    pub const CTRL: u32 = 0x000;
    pub const REMAP: u32 = 0x004;
    pub const COMP0: u32 = 0x008;
    pub const COMP1: u32 = 0x00C;
    pub const COMP2: u32 = 0x010;
    pub const COMP3: u32 = 0x014;
    pub const COMP4: u32 = 0x018;
    pub const COMP5: u32 = 0x01C;
    pub const COMP6: u32 = 0x020;
    pub const COMP7: u32 = 0x024;
    pub const LAR: u32 = 0xFB0;
    pub const LSR: u32 = 0xFB4;
}

/// FPB CTRL bits.
pub mod fpb_ctrl {
    pub const REV_SHIFT: u32 = 28;
    pub const REV_MASK: u32 = 0xF << 28;
    pub const NUM_CODE_SHIFT: u32 = 8;
    pub const NUM_CODE_MASK: u32 = 0x7 << 8;
    pub const NUM_LIT_SHIFT: u32 = 4;
    pub const NUM_LIT_MASK: u32 = 0xF << 4;
    pub const KEY: u32 = 1 << 1;
    pub const ENABLE: u32 = 1 << 0;
}

/// ITM (Instrumentation Trace Macrocell) registers.
pub mod itm {
    pub const STIM0: u32 = 0x000;
    pub const STIM255: u32 = 0x3FC;
    pub const TER: u32 = 0xE00;
    pub const TPR: u32 = 0xE40;
    pub const TCR: u32 = 0xE80;
    pub const LAR: u32 = 0xFB0;
    pub const LSR: u32 = 0xFB4;
}

/// ITM TCR bits.
pub mod itm_tcr {
    pub const BUSY: u32 = 1 << 23;
    pub const ATBID_SHIFT: u32 = 16;
    pub const ATBID_MASK: u32 = 0x7F << 16;
    pub const TXENA: u32 = 1 << 0;
}

/// DWT (Data Watchpoint and Trace) unit.
#[derive(Debug)]
pub struct Dwt {
    /// Control register.
    ctrl: u32,
    /// Cycle counter.
    cyccnt: u32,
    /// CPI counter.
    cpicnt: u32,
    /// Exception counter.
    exccnt: u32,
    /// Sleep counter.
    sleepcnt: u32,
    /// LSU counter.
    lsucnt: u32,
    /// Fold counter.
    foldcnt: u32,
    /// Program Counter Sample.
    pcsr: u32,
    /// Comparators.
    comparators: [DwtComparator; 4],
}

/// DWT Comparator.
#[derive(Debug, Clone, Copy, Default)]
struct DwtComparator {
    comp: u32,
    mask: u32,
    function: u32,
}

impl Default for Dwt {
    fn default() -> Self {
        Self::new()
    }
}

impl Dwt {
    /// Create a new DWT instance.
    pub fn new() -> Self {
        Self {
            ctrl: 4 << dwt_ctrl::NUMCOMP_SHIFT, // 4 comparators
            cyccnt: 0,
            cpicnt: 0,
            exccnt: 0,
            sleepcnt: 0,
            lsucnt: 0,
            foldcnt: 0,
            pcsr: 0,
            comparators: [DwtComparator::default(); 4],
        }
    }

    /// Check if cycle counter enabled.
    pub fn is_cyccnt_enabled(&self) -> bool {
        (self.ctrl & dwt_ctrl::CYCCNTENA) != 0
    }

    /// Tick cycle counter.
    pub fn tick(&mut self) {
        if self.is_cyccnt_enabled() {
            self.cyccnt = self.cyccnt.wrapping_add(1);
        }
    }

    /// Get cycle count.
    pub fn get_cyccnt(&self) -> u32 {
        self.cyccnt
    }

    /// Set PC sample.
    pub fn set_pcsr(&mut self, pc: u32) {
        self.pcsr = pc;
    }

    /// Read register.
    fn read(&mut self, offset: u32) -> u32 {
        match offset {
            dwt::CTRL => self.ctrl,
            dwt::CYCCNT => self.cyccnt,
            dwt::CPICNT => self.cpicnt,
            dwt::EXCCNT => self.exccnt,
            dwt::SLEEPCNT => self.sleepcnt,
            dwt::LSUCNT => self.lsucnt,
            dwt::FOLDCNT => self.foldcnt,
            dwt::PCSR => self.pcsr,
            dwt::COMP0 => self.comparators[0].comp,
            dwt::MASK0 => self.comparators[0].mask,
            dwt::FUNCTION0 => self.comparators[0].function,
            dwt::COMP1 => self.comparators[1].comp,
            dwt::MASK1 => self.comparators[1].mask,
            dwt::FUNCTION1 => self.comparators[1].function,
            dwt::COMP2 => self.comparators[2].comp,
            dwt::MASK2 => self.comparators[2].mask,
            dwt::FUNCTION2 => self.comparators[2].function,
            dwt::COMP3 => self.comparators[3].comp,
            dwt::MASK3 => self.comparators[3].mask,
            dwt::FUNCTION3 => self.comparators[3].function,
            _ => 0,
        }
    }

    /// Write register.
    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            dwt::CTRL => {
                self.ctrl = (self.ctrl & dwt_ctrl::NUMCOMP_MASK) | (value & !dwt_ctrl::NUMCOMP_MASK);
            }
            dwt::CYCCNT => {
                self.cyccnt = value;
            }
            dwt::CPICNT => {
                self.cpicnt = value & 0xFF;
            }
            dwt::EXCCNT => {
                self.exccnt = value & 0xFF;
            }
            dwt::SLEEPCNT => {
                self.sleepcnt = value & 0xFF;
            }
            dwt::LSUCNT => {
                self.lsucnt = value & 0xFF;
            }
            dwt::FOLDCNT => {
                self.foldcnt = value & 0xFF;
            }
            dwt::COMP0 => self.comparators[0].comp = value,
            dwt::MASK0 => self.comparators[0].mask = value & 0xF,
            dwt::FUNCTION0 => self.comparators[0].function = value & 0xF,
            dwt::COMP1 => self.comparators[1].comp = value,
            dwt::MASK1 => self.comparators[1].mask = value & 0xF,
            dwt::FUNCTION1 => self.comparators[1].function = value & 0xF,
            dwt::COMP2 => self.comparators[2].comp = value,
            dwt::MASK2 => self.comparators[2].mask = value & 0xF,
            dwt::FUNCTION2 => self.comparators[2].function = value & 0xF,
            dwt::COMP3 => self.comparators[3].comp = value,
            dwt::MASK3 => self.comparators[3].mask = value & 0xF,
            dwt::FUNCTION3 => self.comparators[3].function = value & 0xF,
            _ => {}
        }
    }

    /// Reset.
    fn reset(&mut self) {
        let ctrl = self.ctrl & dwt_ctrl::NUMCOMP_MASK;
        *self = Self::new();
        self.ctrl = ctrl;
    }
}

/// FPB (Flash Patch and Breakpoint) unit.
#[derive(Debug)]
pub struct Fpb {
    /// Control register.
    ctrl: u32,
    /// Remap register.
    remap: u32,
    /// Comparators.
    comparators: [u32; 8],
    /// Lock status.
    locked: bool,
}

impl Default for Fpb {
    fn default() -> Self {
        Self::new()
    }
}

impl Fpb {
    /// Create a new FPB instance.
    pub fn new() -> Self {
        Self {
            ctrl: (1 << fpb_ctrl::REV_SHIFT) | (6 << fpb_ctrl::NUM_CODE_SHIFT) | (2 << fpb_ctrl::NUM_LIT_SHIFT),
            remap: 0,
            comparators: [0; 8],
            locked: false,
        }
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.ctrl & fpb_ctrl::ENABLE) != 0
    }

    /// Check if breakpoint matches.
    pub fn check_breakpoint(&self, addr: u32) -> bool {
        if !self.is_enabled() {
            return false;
        }

        for comp in &self.comparators {
            let comp_addr = comp & 0x1FFFFFFC;
            let enabled = (comp & 1) != 0;
            if enabled && addr == comp_addr {
                return true;
            }
        }

        false
    }

    /// Read register.
    fn read(&self, offset: u32) -> u32 {
        match offset {
            fpb::CTRL => self.ctrl,
            fpb::REMAP => self.remap,
            fpb::COMP0 => self.comparators[0],
            fpb::COMP1 => self.comparators[1],
            fpb::COMP2 => self.comparators[2],
            fpb::COMP3 => self.comparators[3],
            fpb::COMP4 => self.comparators[4],
            fpb::COMP5 => self.comparators[5],
            fpb::COMP6 => self.comparators[6],
            fpb::COMP7 => self.comparators[7],
            fpb::LSR => if self.locked { 1 } else { 0 },
            _ => 0,
        }
    }

    /// Write register.
    fn write(&mut self, offset: u32, value: u32) {
        if self.locked && offset != fpb::LAR {
            return;
        }

        match offset {
            fpb::CTRL => {
                self.ctrl = (self.ctrl & (fpb_ctrl::REV_MASK | fpb_ctrl::NUM_CODE_MASK | fpb_ctrl::NUM_LIT_MASK))
                    | (value & !(fpb_ctrl::REV_MASK | fpb_ctrl::NUM_CODE_MASK | fpb_ctrl::NUM_LIT_MASK));
            }
            fpb::REMAP => {
                self.remap = value & 0x1FFFFFFF;
            }
            fpb::COMP0 => self.comparators[0] = value,
            fpb::COMP1 => self.comparators[1] = value,
            fpb::COMP2 => self.comparators[2] = value,
            fpb::COMP3 => self.comparators[3] = value,
            fpb::COMP4 => self.comparators[4] = value,
            fpb::COMP5 => self.comparators[5] = value,
            fpb::COMP6 => self.comparators[6] = value,
            fpb::COMP7 => self.comparators[7] = value,
            fpb::LAR => {
                if value == 0xC5AC_CE55 {
                    self.locked = false;
                }
            }
            _ => {}
        }
    }

    /// Reset.
    fn reset(&mut self) {
        let ctrl = self.ctrl & (fpb_ctrl::REV_MASK | fpb_ctrl::NUM_CODE_MASK | fpb_ctrl::NUM_LIT_MASK);
        *self = Self::new();
        self.ctrl = ctrl;
    }
}

/// ITM (Instrumentation Trace Macrocell) unit.
#[derive(Debug)]
pub struct Itm {
    /// Trace Enable Register.
    ter: u32,
    /// Trace Privilege Register.
    tpr: u32,
    /// Trace Control Register.
    tcr: u32,
    /// Stimulus ports.
    stim: [u32; 256],
    /// Lock status.
    locked: bool,
}

impl Default for Itm {
    fn default() -> Self {
        Self::new()
    }
}

impl Itm {
    /// Create a new ITM instance.
    pub fn new() -> Self {
        Self {
            ter: 0,
            tpr: 0,
            tcr: 0,
            stim: [0; 256],
            locked: false,
        }
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.tcr & itm_tcr::TXENA) != 0
    }

    /// Write stimulus port.
    pub fn write_stim(&mut self, port: usize, value: u32) {
        if port < 256 && (self.ter & (1 << port)) != 0 {
            self.stim[port] = value;
        }
    }

    /// Read register.
    fn read(&self, offset: u32) -> u32 {
        match offset {
            itm::TER => self.ter,
            itm::TPR => self.tpr,
            itm::TCR => self.tcr | itm_tcr::BUSY,
            itm::LSR => if self.locked { 1 } else { 0 },
            _ => {
                if offset < 0x400 {
                    let port = (offset >> 2) as usize;
                    if port < 256 {
                        return self.stim[port];
                    }
                }
                0
            }
        }
    }

    /// Write register.
    fn write(&mut self, offset: u32, value: u32) {
        if self.locked && offset != itm::LAR {
            return;
        }

        match offset {
            itm::TER => self.ter = value,
            itm::TPR => self.tpr = value & 0xF,
            itm::TCR => self.tcr = value & 0x007F_0001,
            itm::LAR => {
                if value == 0xC5AC_CE55 {
                    self.locked = false;
                }
            }
            _ => {
                if offset < 0x400 {
                    let port = (offset >> 2) as usize;
                    self.write_stim(port, value);
                }
            }
        }
    }

    /// Reset.
    fn reset(&mut self) {
        self.ter = 0;
        self.tpr = 0;
        self.tcr = 0;
        self.stim.fill(0);
    }
}

/// CoreSight debug components.
#[derive(Debug)]
pub struct Coresight {
    /// DWT unit.
    pub dwt: Dwt,
    /// FPB unit.
    pub fpb: Fpb,
    /// ITM unit.
    pub itm: Itm,
}

impl Default for Coresight {
    fn default() -> Self {
        Self::new()
    }
}

impl Coresight {
    /// Create a new CoreSight instance.
    pub fn new() -> Self {
        Self {
            dwt: Dwt::new(),
            fpb: Fpb::new(),
            itm: Itm::new(),
        }
    }

    /// Tick DWT cycle counter.
    pub fn tick(&mut self) {
        self.dwt.tick();
    }

    /// Check FPB breakpoint.
    pub fn check_breakpoint(&self, addr: u32) -> bool {
        self.fpb.check_breakpoint(addr)
    }

    /// Set DWT PC sample.
    pub fn set_pcsr(&mut self, pc: u32) {
        self.dwt.set_pcsr(pc);
    }
}

impl Device for Coresight {
    fn id(&self) -> DeviceId {
        DeviceId::CORESIGHT
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        // Determine which component based on address
        if addr >= base::ITM && addr < base::TPIU {
            Ok(self.itm.read(addr - base::ITM))
        } else if addr >= base::DWT && addr < base::FPB {
            Ok(self.dwt.read(addr - base::DWT))
        } else if addr >= base::FPB && addr < 0xE000_3000 {
            Ok(self.fpb.read(addr - base::FPB))
        } else {
            Ok(0)
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        if addr >= base::ITM && addr < base::TPIU {
            self.itm.write(addr - base::ITM, value);
        } else if addr >= base::DWT && addr < base::FPB {
            self.dwt.write(addr - base::DWT, value);
        } else if addr >= base::FPB && addr < 0xE000_3000 {
            self.fpb.write(addr - base::FPB, value);
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.dwt.reset();
        self.fpb.reset();
        self.itm.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== DWT Tests ====================

    #[test]
    fn test_dwt_creation() {
        let dwt = Dwt::new();

        // 4 comparators indicated in TYPE
        assert_eq!((dwt.ctrl >> dwt_ctrl::NUMCOMP_SHIFT) & 0xF, 4);

        assert!(!dwt.is_cyccnt_enabled());
        assert_eq!(dwt.get_cyccnt(), 0);
    }

    #[test]
    fn test_dwt_default() {
        let dwt = Dwt::default();
        assert_eq!((dwt.ctrl >> dwt_ctrl::NUMCOMP_SHIFT) & 0xF, 4);
    }

    #[test]
    fn test_dwt_cycle_counter_enable() {
        let mut dwt = Dwt::new();

        assert!(!dwt.is_cyccnt_enabled());

        dwt.ctrl |= dwt_ctrl::CYCCNTENA;
        assert!(dwt.is_cyccnt_enabled());
    }

    #[test]
    fn test_dwt_cycle_counter_tick() {
        let mut dwt = Dwt::new();

        // Disabled - no tick
        dwt.tick();
        assert_eq!(dwt.get_cyccnt(), 0);

        // Enabled - tick increments
        dwt.ctrl |= dwt_ctrl::CYCCNTENA;
        dwt.tick();
        assert_eq!(dwt.get_cyccnt(), 1);

        dwt.tick();
        assert_eq!(dwt.get_cyccnt(), 2);
    }

    #[test]
    fn test_dwt_cycle_counter_wrapping() {
        let mut dwt = Dwt::new();

        dwt.ctrl |= dwt_ctrl::CYCCNTENA;
        dwt.cyccnt = 0xFFFFFFFF;

        dwt.tick();
        assert_eq!(dwt.get_cyccnt(), 0);
    }

    #[test]
    fn test_dwt_pcsr() {
        let mut dwt = Dwt::new();

        dwt.set_pcsr(0x2000_1234);
        assert_eq!(dwt.pcsr, 0x2000_1234);
    }

    #[test]
    fn test_dwt_counters_masked() {
        let mut dwt = Dwt::new();

        dwt.write(dwt::CPICNT, 0x12345678);
        assert_eq!(dwt.cpicnt, 0x78); // Only lower 8 bits

        dwt.write(dwt::EXCCNT, 0xFFFFFFFF);
        assert_eq!(dwt.exccnt, 0xFF);

        dwt.write(dwt::SLEEPCNT, 0x100);
        assert_eq!(dwt.sleepcnt, 0);
    }

    #[test]
    fn test_dwt_comparators() {
        let mut dwt = Dwt::new();

        dwt.write(dwt::COMP0, 0x12345678);
        assert_eq!(dwt.comparators[0].comp, 0x12345678);

        dwt.write(dwt::MASK0, 0xFF);
        assert_eq!(dwt.comparators[0].mask, 0xF); // 4-bit mask

        dwt.write(dwt::FUNCTION0, 0xFF);
        assert_eq!(dwt.comparators[0].function, 0xF); // 4-bit function
    }

    #[test]
    fn test_dwt_read_registers() {
        let mut dwt = Dwt::new();

        dwt.ctrl |= dwt_ctrl::CYCCNTENA;
        dwt.cyccnt = 100;
        dwt.pcsr = 0x2000_0000;

        assert_eq!(dwt.read(dwt::CTRL), dwt.ctrl);
        assert_eq!(dwt.read(dwt::CYCCNT), 100);
        assert_eq!(dwt.read(dwt::PCSR), 0x2000_0000);
    }

    #[test]
    fn test_dwt_reset_preserves_numcomp() {
        let mut dwt = Dwt::new();

        dwt.ctrl |= dwt_ctrl::CYCCNTENA;
        dwt.cyccnt = 0xFFFFFFFF;
        dwt.comparators[0].comp = 0x12345678;

        dwt.reset();

        // NUMCOMP preserved
        assert_eq!((dwt.ctrl >> dwt_ctrl::NUMCOMP_SHIFT) & 0xF, 4);
        // Counter reset
        assert_eq!(dwt.cyccnt, 0);
        assert!(!dwt.is_cyccnt_enabled());
        // Comparators reset
        assert_eq!(dwt.comparators[0].comp, 0);
    }

    // ==================== FPB Tests ====================

    #[test]
    fn test_fpb_creation() {
        let fpb = Fpb::new();

        assert!(!fpb.is_enabled());
        assert!(!fpb.locked);

        // Check NUM_CODE and NUM_LIT
        assert_eq!((fpb.ctrl >> fpb_ctrl::NUM_CODE_SHIFT) & 0x7, 6);
        assert_eq!((fpb.ctrl >> fpb_ctrl::NUM_LIT_SHIFT) & 0xF, 2);
    }

    #[test]
    fn test_fpb_default() {
        let fpb = Fpb::default();
        assert!(!fpb.is_enabled());
    }

    #[test]
    fn test_fpb_enable() {
        let mut fpb = Fpb::new();

        fpb.ctrl |= fpb_ctrl::ENABLE;
        assert!(fpb.is_enabled());
    }

    #[test]
    fn test_fpb_check_breakpoint_disabled() {
        let fpb = Fpb::new();

        // Set comparator but FPB disabled
        let comp_addr = 0x0800_1000;
        let comp_val = comp_addr | 1; // Enable bit

        // No match when disabled
        assert!(!fpb.check_breakpoint(comp_addr));
    }

    #[test]
    fn test_fpb_check_breakpoint_enabled() {
        let mut fpb = Fpb::new();

        fpb.ctrl |= fpb_ctrl::ENABLE;

        let comp_addr = 0x0800_1000;
        fpb.comparators[0] = comp_addr | 1; // Enable bit

        assert!(fpb.check_breakpoint(comp_addr));
        assert!(!fpb.check_breakpoint(0x0000_0000));
    }

    #[test]
    fn test_fpb_multiple_breakpoints() {
        let mut fpb = Fpb::new();

        fpb.ctrl |= fpb_ctrl::ENABLE;

        fpb.comparators[0] = 0x0800_1000 | 1;
        fpb.comparators[3] = 0x0800_2000 | 1;

        assert!(fpb.check_breakpoint(0x0800_1000));
        assert!(fpb.check_breakpoint(0x0800_2000));
        assert!(!fpb.check_breakpoint(0x0800_3000));
    }

    #[test]
    fn test_fpb_read_write_registers() {
        let mut fpb = Fpb::new();

        fpb.write(fpb::CTRL, fpb_ctrl::ENABLE);
        assert_eq!(fpb.read(fpb::CTRL) & fpb_ctrl::ENABLE, fpb_ctrl::ENABLE);

        fpb.write(fpb::REMAP, 0x12345678);
        assert_eq!(fpb.read(fpb::REMAP), 0x12345678 & 0x1FFFFFFF);

        fpb.write(fpb::COMP0, 0x12345678);
        assert_eq!(fpb.read(fpb::COMP0), 0x12345678);
    }

    #[test]
    fn test_fpb_lock_unlock() {
        let mut fpb = Fpb::new();

        fpb.locked = true;

        // Locked - write should fail
        fpb.write(fpb::COMP0, 0x12345678);
        assert_eq!(fpb.comparators[0], 0);

        // Unlock with key
        fpb.write(fpb::LAR, 0xC5AC_CE55);
        assert!(!fpb.locked);

        // Now write should work
        fpb.write(fpb::COMP0, 0x12345678);
        assert_eq!(fpb.comparators[0], 0x12345678);
    }

    #[test]
    fn test_fpb_lsr() {
        let mut fpb = Fpb::new();

        assert_eq!(fpb.read(fpb::LSR), 0); // Not locked

        fpb.locked = true;
        assert_eq!(fpb.read(fpb::LSR), 1); // Locked
    }

    #[test]
    fn test_fpb_reset_preserves_ctrl_info() {
        let mut fpb = Fpb::new();

        fpb.ctrl |= fpb_ctrl::ENABLE;
        fpb.comparators[0] = 0xFFFFFFFF;

        fpb.reset();

        assert!(!fpb.is_enabled());
        assert_eq!(fpb.comparators[0], 0);
        // REV, NUM_CODE, NUM_LIT preserved
        assert_eq!((fpb.ctrl >> fpb_ctrl::REV_SHIFT) & 0xF, 1);
    }

    // ==================== ITM Tests ====================

    #[test]
    fn test_itm_creation() {
        let itm = Itm::new();

        assert!(!itm.is_enabled());
        assert_eq!(itm.ter, 0);
        assert_eq!(itm.tpr, 0);
        assert_eq!(itm.tcr, 0);
    }

    #[test]
    fn test_itm_default() {
        let itm = Itm::default();
        assert!(!itm.is_enabled());
    }

    #[test]
    fn test_itm_enable() {
        let mut itm = Itm::new();

        itm.tcr = itm_tcr::TXENA;
        assert!(itm.is_enabled());
    }

    #[test]
    fn test_itm_stim_port_write() {
        let mut itm = Itm::new();

        // Port disabled - no write
        itm.write_stim(0, 0x12345678);
        assert_eq!(itm.stim[0], 0);

        // Enable port 0
        itm.ter = 1;
        itm.write_stim(0, 0x12345678);
        assert_eq!(itm.stim[0], 0x12345678);
    }

    #[test]
    fn test_itm_stim_port_range() {
        let mut itm = Itm::new();
        itm.ter = 0xFFFFFFFF;

        // Port 255
        itm.write_stim(255, 0xDEADBEEF);
        assert_eq!(itm.stim[255], 0xDEADBEEF);

        // Port 256 out of range - should be ignored
        itm.write_stim(256, 0x12345678);
    }

    #[test]
    fn test_itm_read_registers() {
        let mut itm = Itm::new();

        itm.ter = 0x1234;
        itm.tpr = 0xF;
        itm.tcr = itm_tcr::TXENA;

        assert_eq!(itm.read(itm::TER), 0x1234);
        assert_eq!(itm.read(itm::TPR), 0xF);
        assert_eq!(itm.read(itm::TCR) & itm_tcr::TXENA, itm_tcr::TXENA);
        // BUSY bit should be set in read
        assert_eq!(itm.read(itm::TCR) & itm_tcr::BUSY, itm_tcr::BUSY);
    }

    #[test]
    fn test_itm_write_registers() {
        let mut itm = Itm::new();

        itm.write(itm::TER, 0xFFFFFFFF);
        assert_eq!(itm.ter, 0xFFFFFFFF);

        itm.write(itm::TPR, 0xFF);
        assert_eq!(itm.tpr, 0xF); // Only 4 bits

        itm.write(itm::TCR, 0xFFFFFFFF);
        assert_eq!(itm.tcr, 0x007F_0001); // Masked
    }

    #[test]
    fn test_itm_lock_unlock() {
        let mut itm = Itm::new();

        itm.locked = true;

        // Locked - write should fail
        itm.write(itm::TER, 0xFFFFFFFF);
        assert_eq!(itm.ter, 0);

        // Unlock with key
        itm.write(itm::LAR, 0xC5AC_CE55);
        assert!(!itm.locked);

        // Now write should work
        itm.write(itm::TER, 0xFFFFFFFF);
        assert_eq!(itm.ter, 0xFFFFFFFF);
    }

    #[test]
    fn test_itm_lsr() {
        let mut itm = Itm::new();

        assert_eq!(itm.read(itm::LSR), 0);

        itm.locked = true;
        assert_eq!(itm.read(itm::LSR), 1);
    }

    #[test]
    fn test_itm_reset() {
        let mut itm = Itm::new();

        itm.ter = 0xFFFFFFFF;
        itm.tpr = 0xF;
        itm.tcr = itm_tcr::TXENA;
        itm.stim[0] = 0x12345678;

        itm.reset();

        assert_eq!(itm.ter, 0);
        assert_eq!(itm.tpr, 0);
        assert_eq!(itm.tcr, 0);
        assert_eq!(itm.stim[0], 0);
    }

    // ==================== Coresight Tests ====================

    #[test]
    fn test_coresight_creation() {
        let cs = Coresight::new();

        assert!(!cs.dwt.is_cyccnt_enabled());
        assert!(!cs.fpb.is_enabled());
        assert!(!cs.itm.is_enabled());
    }

    #[test]
    fn test_coresight_default() {
        let cs = Coresight::default();
        assert!(!cs.dwt.is_cyccnt_enabled());
    }

    #[test]
    fn test_coresight_tick() {
        let mut cs = Coresight::new();

        cs.dwt.ctrl |= dwt_ctrl::CYCCNTENA;
        cs.tick();
        assert_eq!(cs.dwt.get_cyccnt(), 1);
    }

    #[test]
    fn test_coresight_check_breakpoint() {
        let mut cs = Coresight::new();

        cs.fpb.ctrl |= fpb_ctrl::ENABLE;
        cs.fpb.comparators[0] = 0x0800_1000 | 1;

        assert!(cs.check_breakpoint(0x0800_1000));
    }

    #[test]
    fn test_coresight_set_pcsr() {
        let mut cs = Coresight::new();

        cs.set_pcsr(0x2000_1234);
        assert_eq!(cs.dwt.pcsr, 0x2000_1234);
    }

    #[test]
    fn test_coresight_device_id() {
        let cs = Coresight::new();
        assert_eq!(cs.id(), DeviceId::CORESIGHT);
    }

    #[test]
    fn test_coresight_read_dwt() {
        let mut cs = Coresight::new();

        cs.dwt.cyccnt = 100;
        let val = cs.read(base::DWT + dwt::CYCCNT).unwrap();
        assert_eq!(val, 100);
    }

    #[test]
    fn test_coresight_read_fpb() {
        let mut cs = Coresight::new();

        cs.fpb.comparators[0] = 0x12345678;
        let val = cs.read(base::FPB + fpb::COMP0).unwrap();
        assert_eq!(val, 0x12345678);
    }

    #[test]
    fn test_coresight_read_itm() {
        let mut cs = Coresight::new();

        cs.itm.ter = 0x12345678;
        let val = cs.read(base::ITM + itm::TER).unwrap();
        assert_eq!(val, 0x12345678);
    }

    #[test]
    fn test_coresight_write_dwt() {
        let mut cs = Coresight::new();

        cs.write(base::DWT + dwt::CYCCNT, 100).unwrap();
        assert_eq!(cs.dwt.cyccnt, 100);
    }

    #[test]
    fn test_coresight_write_fpb() {
        let mut cs = Coresight::new();

        cs.write(base::FPB + fpb::COMP0, 0x12345678).unwrap();
        assert_eq!(cs.fpb.comparators[0], 0x12345678);
    }

    #[test]
    fn test_coresight_write_itm() {
        let mut cs = Coresight::new();

        cs.write(base::ITM + itm::TER, 0xFFFFFFFF).unwrap();
        assert_eq!(cs.itm.ter, 0xFFFFFFFF);
    }

    #[test]
    fn test_coresight_reset() {
        let mut cs = Coresight::new();

        cs.dwt.ctrl |= dwt_ctrl::CYCCNTENA;
        cs.dwt.cyccnt = 100;
        cs.fpb.ctrl |= fpb_ctrl::ENABLE;
        cs.itm.ter = 0xFFFFFFFF;

        cs.reset();

        assert!(!cs.dwt.is_cyccnt_enabled());
        assert!(!cs.fpb.is_enabled());
        assert_eq!(cs.itm.ter, 0);
    }

    #[test]
    fn test_coresight_read_invalid() {
        let mut cs = Coresight::new();
        assert_eq!(cs.read(0xE010_0000).unwrap(), 0);
    }
}
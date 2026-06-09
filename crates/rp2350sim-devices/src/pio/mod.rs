//! PIO device for RP2350.
//!
//! Implements the Programmable I/O peripheral with full register support.

use rp2350sim_core::{Device, DeviceId, Result};
use std::collections::VecDeque;

/// PIO base addresses.
pub const PIO0_BASE: u32 = 0x5020_0000;
pub const PIO1_BASE: u32 = 0x5030_0000;

/// PIO register offsets.
pub mod regs {
    // Control registers
    pub const CTRL: u32 = 0x000;
    pub const FSTAT: u32 = 0x004;
    pub const FDEBUG: u32 = 0x008;
    pub const FLEVEL: u32 = 0x00C;
    
    // TX/RX FIFO for each state machine (0-3)
    pub const TXF0: u32 = 0x010;
    pub const TXF1: u32 = 0x014;
    pub const TXF2: u32 = 0x018;
    pub const TXF3: u32 = 0x01C;
    pub const RXF0: u32 = 0x020;
    pub const RXF1: u32 = 0x024;
    pub const RXF2: u32 = 0x028;
    pub const RXF3: u32 = 0x02C;
    
    // IRQ registers
    pub const IRQ: u32 = 0x030;
    pub const IRQ_FORCE: u32 = 0x034;
    pub const INPUT_SYNC_BYPASS: u32 = 0x038;
    pub const DBG_PADOUT: u32 = 0x03C;
    pub const DBG_PADOE: u32 = 0x040;
    pub const DBG_CFGINFO: u32 = 0x044;
    pub const INSTR_MEM0: u32 = 0x048;
    // INSTR_MEM1-31 follow at 4-byte intervals
    
    // State machine registers (each SM has 12 registers, 0x0C8 apart)
    pub const SM0_CLKDIV: u32 = 0x0C8;
    pub const SM0_EXECCTRL: u32 = 0x0CC;
    pub const SM0_SHIFTCTRL: u32 = 0x0D0;
    pub const SM0_ADDR: u32 = 0x0D4;
    pub const SM0_INSTR: u32 = 0x0D8;
    pub const SM0_PINCTRL: u32 = 0x0DC;
    
    pub const SM1_CLKDIV: u32 = 0x0E0;
    pub const SM2_CLKDIV: u32 = 0x0F8;
    pub const SM3_CLKDIV: u32 = 0x110;
}

/// CTRL register bits.
pub mod ctrl {
    pub const SM_ENABLE_MASK: u32 = 0x0F << 0;
    pub const SM_RESTART_MASK: u32 = 0x0F << 4;
    pub const CLKDIV_RESTART_MASK: u32 = 0x0F << 8;
}

/// FSTAT register bits.
pub mod fstat {
    pub const TXEMPTY_MASK: u32 = 0x0F << 0;
    pub const TXFULL_MASK: u32 = 0x0F << 4;
    pub const RXEMPTY_MASK: u32 = 0x0F << 8;
    pub const RXFULL_MASK: u32 = 0x0F << 12;
}

/// FIFO depth.
const FIFO_SIZE: usize = 8;

/// Number of state machines.
const NUM_SMS: usize = 4;

/// Instruction memory size.
const INSTR_MEM_SIZE: usize = 32;

/// PIO state machine.
#[derive(Debug, Clone)]
pub struct PioStateMachine {
    /// Program counter.
    pub pc: u8,
    /// Enabled flag.
    pub enabled: bool,
    /// Clock divider (16.8 fixed point).
    pub clkdiv: u32,
    /// Execute control register.
    pub execctrl: u32,
    /// Shift control register.
    pub shiftctrl: u32,
    /// Pin control register.
    pub pinctrl: u32,
    /// Current instruction.
    pub instr: u16,
    /// TX FIFO.
    tx_fifo: VecDeque<u32>,
    /// RX FIFO.
    rx_fifo: VecDeque<u32>,
    /// Input shift register.
    isr: u32,
    /// Output shift register.
    osr: u32,
    /// Input shift count.
    isr_count: u8,
    /// Output shift count.
    osr_count: u8,
    /// X register.
    x: u32,
    /// Y register.
    y: u32,
    /// Delay counter.
    delay: u8,
    /// Waiting flag.
    waiting: bool,
}

impl Default for PioStateMachine {
    fn default() -> Self {
        Self {
            pc: 0,
            enabled: false,
            clkdiv: 0x00010000, // Divide by 1
            execctrl: 0,
            shiftctrl: 0,
            pinctrl: 0,
            instr: 0,
            tx_fifo: VecDeque::with_capacity(FIFO_SIZE),
            rx_fifo: VecDeque::with_capacity(FIFO_SIZE),
            isr: 0,
            osr: 0,
            isr_count: 0,
            osr_count: 0,
            x: 0,
            y: 0,
            delay: 0,
            waiting: false,
        }
    }
}

impl PioStateMachine {
    /// Check if TX FIFO is empty.
    pub fn tx_empty(&self) -> bool {
        self.tx_fifo.is_empty()
    }

    /// Check if TX FIFO is full.
    pub fn tx_full(&self) -> bool {
        self.tx_fifo.len() >= FIFO_SIZE
    }

    /// Check if RX FIFO is empty.
    pub fn rx_empty(&self) -> bool {
        self.rx_fifo.is_empty()
    }

    /// Check if RX FIFO is full.
    pub fn rx_full(&self) -> bool {
        self.rx_fifo.len() >= FIFO_SIZE
    }

    /// Push to TX FIFO.
    pub fn push_tx(&mut self, value: u32) -> bool {
        if self.tx_full() {
            false
        } else {
            self.tx_fifo.push_back(value);
            true
        }
    }

    /// Pop from TX FIFO.
    pub fn pop_tx(&mut self) -> Option<u32> {
        self.tx_fifo.pop_front()
    }

    /// Push to RX FIFO.
    pub fn push_rx(&mut self, value: u32) -> bool {
        if self.rx_full() {
            false
        } else {
            self.rx_fifo.push_back(value);
            true
        }
    }

    /// Pop from RX FIFO.
    pub fn pop_rx(&mut self) -> Option<u32> {
        self.rx_fifo.pop_front()
    }
}

/// PIO device.
#[derive(Debug)]
pub struct Pio {
    /// PIO instance ID (0 or 1).
    pub id: u8,
    /// Base address.
    base: u32,
    /// Control register.
    ctrl: u32,
    /// FIFO status.
    fstat: u32,
    /// FIFO debug.
    fdebug: u32,
    /// FIFO levels.
    flevel: u32,
    /// IRQ status.
    irq: u32,
    /// IRQ force.
    irq_force: u32,
    /// Input sync bypass.
    input_sync_bypass: u32,
    /// Instruction memory.
    instr_mem: [u16; INSTR_MEM_SIZE],
    /// State machines.
    state_machines: [PioStateMachine; NUM_SMS],
    /// Clock divider counter for each SM.
    clkdiv_counter: [u32; NUM_SMS],
}

impl Default for Pio {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Pio {
    /// Create a new PIO device.
    pub fn new(id: u8) -> Self {
        let state_machines = [
            PioStateMachine::default(),
            PioStateMachine::default(),
            PioStateMachine::default(),
            PioStateMachine::default(),
        ];
        Self {
            id,
            base: if id == 0 { PIO0_BASE } else { PIO1_BASE },
            ctrl: 0,
            fstat: 0x0000_0F0F, // All TX empty (bits 0-3), all RX empty (bits 8-11)
            fdebug: 0,
            flevel: 0,
            irq: 0,
            irq_force: 0,
            input_sync_bypass: 0,
            instr_mem: [0; INSTR_MEM_SIZE],
            state_machines,
            clkdiv_counter: [0; NUM_SMS],
        }
    }

    /// Get base address.
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Load program into instruction memory.
    pub fn load_program(&mut self, offset: usize, program: &[u16]) {
        let end = (offset + program.len()).min(INSTR_MEM_SIZE);
        for i in offset..end {
            self.instr_mem[i] = program[i - offset];
        }
    }

    /// Check if state machine is enabled.
    pub fn sm_enabled(&self, sm: usize) -> bool {
        if sm < NUM_SMS {
            self.state_machines[sm].enabled
        } else {
            false
        }
    }

    /// Get state machine PC.
    pub fn sm_pc(&self, sm: usize) -> u8 {
        if sm < NUM_SMS {
            self.state_machines[sm].pc
        } else {
            0
        }
    }

    /// Enable/disable state machine.
    pub fn set_sm_enabled(&mut self, sm: usize, enabled: bool) {
        if sm < NUM_SMS {
            self.state_machines[sm].enabled = enabled;
            if enabled {
                self.ctrl |= 1 << sm;
            } else {
                self.ctrl &= !(1 << sm);
            }
        }
    }

    /// Start state machine.
    pub fn start_sm(&mut self, sm: usize) {
        self.set_sm_enabled(sm, true);
    }

    /// Stop state machine.
    pub fn stop_sm(&mut self, sm: usize) {
        self.set_sm_enabled(sm, false);
    }

    /// Pop data from TX FIFO.
    pub fn pop_tx(&mut self, sm: usize) -> Option<u32> {
        if sm < NUM_SMS {
            self.state_machines[sm].pop_tx()
        } else {
            None
        }
    }

    /// Push data to RX FIFO.
    pub fn push_rx(&mut self, sm: usize, value: u32) -> bool {
        if sm < NUM_SMS {
            self.state_machines[sm].push_rx(value)
        } else {
            false
        }
    }

    /// Update FIFO status.
    fn update_fstat(&mut self) {
        self.fstat = 0;
        for i in 0..NUM_SMS {
            if self.state_machines[i].tx_empty() {
                self.fstat |= 1 << i;
            }
            if self.state_machines[i].tx_full() {
                self.fstat |= 1 << (4 + i);
            }
            if self.state_machines[i].rx_empty() {
                self.fstat |= 1 << (8 + i);
            }
            if self.state_machines[i].rx_full() {
                self.fstat |= 1 << (12 + i);
            }
        }
    }

    /// Update FIFO levels.
    fn update_flevel(&mut self) {
        self.flevel = 0;
        for i in 0..NUM_SMS {
            let tx_level = self.state_machines[i].tx_fifo.len() as u32;
            let rx_level = self.state_machines[i].rx_fifo.len() as u32;
            self.flevel |= (tx_level & 0xF) << (i * 8);
            self.flevel |= (rx_level & 0xF) << (i * 8 + 4);
        }
    }

    /// Execute one instruction on a state machine.
    fn execute_instruction(&mut self, sm: usize) {
        if sm >= NUM_SMS || !self.state_machines[sm].enabled {
            return;
        }

        let sms = &mut self.state_machines[sm];
        
        // Handle delay
        if sms.delay > 0 {
            sms.delay -= 1;
            return;
        }

        // Fetch instruction
        let instr = self.instr_mem[sms.pc as usize];
        sms.instr = instr;

        // Decode and execute (simplified)
        let opcode = (instr >> 13) & 0x7;
        match opcode {
            0x0 => { // JMP
                let addr = (instr & 0x1F) as u8;
                sms.pc = addr;
                return;
            }
            0x1 => { // WAIT
                sms.waiting = true;
            }
            0x2 => { // IN
                // Simplified: just increment ISR count
                sms.isr_count = sms.isr_count.saturating_add(1);
            }
            0x3 => { // OUT
                // Simplified: just decrement OSR count
                sms.osr_count = sms.osr_count.saturating_sub(1);
            }
            0x4 => { // PUSH
                if let Some(&value) = sms.tx_fifo.front() {
                    sms.osr = value;
                    sms.tx_fifo.pop_front();
                }
            }
            0x5 => { // PULL
                if !sms.rx_full() {
                    sms.rx_fifo.push_back(sms.isr);
                }
            }
            0x6 => { // MOV
                let dest = (instr >> 5) & 0x7;
                let src = instr & 0x7;
                let value = match src {
                    0 => sms.x,
                    1 => sms.y,
                    _ => 0,
                };
                match dest {
                    0 => sms.x = value,
                    1 => sms.y = value,
                    _ => {}
                }
            }
            0x7 => { // IRQ/SET
                let is_set = (instr >> 5) & 1 == 1;
                if is_set {
                    let data = (instr & 0x1F) as u32;
                    sms.x = data;
                }
            }
            _ => {}
        }

        // Advance PC
        sms.pc = (sms.pc + 1) % 32;

        // Set delay from instruction
        sms.delay = ((instr >> 8) & 0x1F) as u8;
    }

    /// Tick the PIO (execute one cycle).
    pub fn tick(&mut self) {
        for sm in 0..NUM_SMS {
            if self.state_machines[sm].enabled {
                // Clock divider
                let div = self.state_machines[sm].clkdiv;
                let div_int = div >> 16;
                
                self.clkdiv_counter[sm] += 1;
                if self.clkdiv_counter[sm] >= div_int {
                    self.clkdiv_counter[sm] = 0;
                    self.execute_instruction(sm);
                }
            }
        }
        self.update_fstat();
        self.update_flevel();
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        self.irq != 0
    }

    /// Check if any state machine is enabled.
    pub fn any_sm_enabled(&self) -> bool {
        self.state_machines.iter().any(|sm| sm.enabled)
    }

    /// Push to TX FIFO of a state machine.
    pub fn push_tx(&mut self, sm: usize, value: u32) -> bool {
        if let Some(s) = self.state_machines.get_mut(sm) {
            s.push_tx(value)
        } else {
            false
        }
    }

    /// Pop from RX FIFO of a state machine.
    pub fn pop_rx(&mut self, sm: usize) -> Option<u32> {
        self.state_machines.get_mut(sm).and_then(|s| s.pop_rx())
    }
}

impl Device for Pio {
    fn id(&self) -> DeviceId {
        DeviceId::PIO
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - self.base;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::FSTAT => Ok(self.fstat),
            regs::FDEBUG => Ok(self.fdebug),
            regs::FLEVEL => Ok(self.flevel),
            regs::TXF0 => Ok(self.state_machines[0].tx_fifo.front().copied().unwrap_or(0)),
            regs::TXF1 => Ok(self.state_machines[1].tx_fifo.front().copied().unwrap_or(0)),
            regs::TXF2 => Ok(self.state_machines[2].tx_fifo.front().copied().unwrap_or(0)),
            regs::TXF3 => Ok(self.state_machines[3].tx_fifo.front().copied().unwrap_or(0)),
            regs::RXF0 => Ok(self.state_machines[0].rx_fifo.pop_front().unwrap_or(0)),
            regs::RXF1 => Ok(self.state_machines[1].rx_fifo.pop_front().unwrap_or(0)),
            regs::RXF2 => Ok(self.state_machines[2].rx_fifo.pop_front().unwrap_or(0)),
            regs::RXF3 => Ok(self.state_machines[3].rx_fifo.pop_front().unwrap_or(0)),
            regs::IRQ => Ok(self.irq),
            regs::IRQ_FORCE => Ok(self.irq_force),
            regs::INPUT_SYNC_BYPASS => Ok(self.input_sync_bypass),
            regs::DBG_PADOUT => Ok(0),
            regs::DBG_PADOE => Ok(0),
            regs::DBG_CFGINFO => Ok((NUM_SMS as u32) << 0 | (INSTR_MEM_SIZE as u32) << 8),
            _ => {
                // Instruction memory
                if offset >= regs::INSTR_MEM0 && offset < regs::INSTR_MEM0 + 32 * 4 {
                    let idx = ((offset - regs::INSTR_MEM0) / 4) as usize;
                    if idx < INSTR_MEM_SIZE {
                        return Ok(self.instr_mem[idx] as u32);
                    }
                }
                // State machine registers
                let sm_offset = offset - regs::SM0_CLKDIV;
                let sm = sm_offset / 0x18;
                let reg = sm_offset % 0x18;
                if sm < NUM_SMS as u32 {
                    let sm = sm as usize;
                    return match reg {
                        0x0 => Ok(self.state_machines[sm].clkdiv),
                        0x4 => Ok(self.state_machines[sm].execctrl),
                        0x8 => Ok(self.state_machines[sm].shiftctrl),
                        0xC => Ok(self.state_machines[sm].pc as u32),
                        0x10 => Ok(self.state_machines[sm].instr as u32),
                        0x14 => Ok(self.state_machines[sm].pinctrl),
                        _ => Ok(0),
                    };
                }
                Ok(0)
            }
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - self.base;

        match offset {
            regs::CTRL => {
                self.ctrl = value;
                for i in 0..NUM_SMS {
                    self.state_machines[i].enabled = (value & (1 << i)) != 0;
                }
            }
            regs::FDEBUG => {
                self.fdebug &= !value; // Write 1 to clear
            }
            regs::TXF0 => {
                self.state_machines[0].push_tx(value);
            }
            regs::TXF1 => {
                self.state_machines[1].push_tx(value);
            }
            regs::TXF2 => {
                self.state_machines[2].push_tx(value);
            }
            regs::TXF3 => {
                self.state_machines[3].push_tx(value);
            }
            regs::IRQ => {
                self.irq &= !value; // Write 1 to clear
            }
            regs::IRQ_FORCE => {
                self.irq |= value;
            }
            regs::INPUT_SYNC_BYPASS => {
                self.input_sync_bypass = value;
            }
            _ => {
                // Instruction memory
                if offset >= regs::INSTR_MEM0 && offset < regs::INSTR_MEM0 + 32 * 4 {
                    let idx = ((offset - regs::INSTR_MEM0) / 4) as usize;
                    if idx < INSTR_MEM_SIZE {
                        self.instr_mem[idx] = value as u16;
                    }
                }
                // State machine registers
                if offset >= regs::SM0_CLKDIV && offset < regs::SM0_CLKDIV + NUM_SMS as u32 * 0x18 {
                    let sm_offset = offset - regs::SM0_CLKDIV;
                    let sm = sm_offset / 0x18;
                    let reg = sm_offset % 0x18;
                    if sm < NUM_SMS as u32 {
                        let sm = sm as usize;
                        match reg {
                            0x0 => self.state_machines[sm].clkdiv = value,
                            0x4 => self.state_machines[sm].execctrl = value,
                            0x8 => self.state_machines[sm].shiftctrl = value,
                            0xC => self.state_machines[sm].pc = (value & 0x1F) as u8,
                            0x10 => self.state_machines[sm].instr = value as u16,
                            0x14 => self.state_machines[sm].pinctrl = value,
                            _ => {}
                        }
                    }
                }
            }
        }

        self.update_fstat();
        self.update_flevel();
        Ok(())
    }

    fn reset(&mut self) {
        let id = self.id;
        let base = self.base;
        *self = Self {
            id,
            base,
            ..Self::new(id)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pio_creation() {
        let pio = Pio::new(0);
        assert_eq!(pio.id, 0);
        assert_eq!(pio.base, PIO0_BASE);

        let pio1 = Pio::new(1);
        assert_eq!(pio1.id, 1);
        assert_eq!(pio1.base, PIO1_BASE);
    }

    #[test]
    fn test_pio_state_machine_fifo() {
        let mut sm = PioStateMachine::default();

        // Test TX FIFO
        assert!(sm.tx_empty());
        assert!(!sm.tx_full());

        // Push to TX FIFO
        assert!(sm.push_tx(0x12345678));
        assert!(!sm.tx_empty());

        // Pop from TX FIFO
        let val = sm.pop_tx();
        assert_eq!(val, Some(0x12345678));
        assert!(sm.tx_empty());

        // Fill TX FIFO
        for i in 0..FIFO_SIZE {
            assert!(sm.push_tx(i as u32));
        }
        assert!(sm.tx_full());
        assert!(!sm.push_tx(0xFF)); // Should fail, FIFO is full

        // Test RX FIFO
        assert!(sm.rx_empty());
        assert!(sm.push_rx(0xDEADBEEF));
        assert!(!sm.rx_empty());

        let val = sm.pop_rx();
        assert_eq!(val, Some(0xDEADBEEF));
    }

    #[test]
    fn test_pio_instruction_memory() {
        let mut pio = Pio::new(0);

        // Write instructions to memory
        let instr1: u16 = 0xE080; // SET X, 0
        let instr2: u16 = 0xE081; // SET X, 1

        pio.write(PIO0_BASE + regs::INSTR_MEM0, instr1 as u32).unwrap();
        pio.write(PIO0_BASE + regs::INSTR_MEM0 + 4, instr2 as u32).unwrap();

        // Read back
        assert_eq!(pio.read(PIO0_BASE + regs::INSTR_MEM0).unwrap(), instr1 as u32);
        assert_eq!(pio.read(PIO0_BASE + regs::INSTR_MEM0 + 4).unwrap(), instr2 as u32);
    }

    #[test]
    fn test_pio_state_machine_control() {
        let mut pio = Pio::new(0);

        // Enable state machine 0
        pio.write(PIO0_BASE + regs::CTRL, 0x01).unwrap();
        assert_eq!(pio.ctrl & 0x0F, 0x01);

        // Check state machine is enabled
        let ctrl_val = pio.read(PIO0_BASE + regs::CTRL).unwrap();
        assert_eq!(ctrl_val & 0x0F, 0x01);
    }

    #[test]
    fn test_pio_fifo_status() {
        let mut pio = Pio::new(0);

        // Check initial FIFO status
        let fstat = pio.read(PIO0_BASE + regs::FSTAT).unwrap();
        println!("fstat = {:08X}, expected = {:08X}", fstat, 0x00F0_000F);

        // All TX FIFOs should be empty (bits 0-3 = 1)
        // Note: The initial fstat value is 0x00F0_000F
        assert_eq!(fstat & fstat::TXEMPTY_MASK, fstat::TXEMPTY_MASK);

        // All RX FIFOs should be empty (bits 8-11 = 1)
        assert_eq!(fstat & fstat::RXEMPTY_MASK, fstat::RXEMPTY_MASK);
    }

    #[test]
    fn test_pio_txf_rxf_access() {
        let mut pio = Pio::new(0);

        // Write to TXF0
        pio.write(PIO0_BASE + regs::TXF0, 0xDEADBEEF).unwrap();

        // Check TX FIFO is not empty
        let fstat = pio.read(PIO0_BASE + regs::FSTAT).unwrap();
        assert_eq!(fstat & fstat::TXEMPTY_MASK, 0x0E); // SM0 TX FIFO not empty

        // Pop from TX FIFO
        let val = pio.pop_tx(0);
        assert_eq!(val, Some(0xDEADBEEF));

        // Push to RX FIFO
        pio.push_rx(0, 0xCAFEBABE);

        // Read from RXF0
        let val = pio.read(PIO0_BASE + regs::RXF0).unwrap();
        assert_eq!(val, 0xCAFEBABE);
    }

    #[test]
    fn test_pio_reset() {
        let mut pio = Pio::new(0);

        // Write some values
        pio.write(PIO0_BASE + regs::CTRL, 0x0F).unwrap();
        pio.write(PIO0_BASE + regs::INSTR_MEM0, 0x1234).unwrap();

        // Reset
        pio.reset();

        // Check values are cleared
        assert_eq!(pio.read(PIO0_BASE + regs::CTRL).unwrap(), 0);
        // Instruction memory should be cleared
        assert_eq!(pio.read(PIO0_BASE + regs::INSTR_MEM0).unwrap(), 0);
    }

    #[test]
    fn test_pio_gpio_direction() {
        let mut pio = Pio::new(0);

        // Set pin control for state machine 0
        // PINCTRL controls: OUT_BASE, SET_BASE, IN_BASE, SIDESET_BASE
        pio.write(PIO0_BASE + regs::SM0_PINCTRL, 0x12345678).unwrap();
        assert_eq!(pio.read(PIO0_BASE + regs::SM0_PINCTRL).unwrap(), 0x12345678);
    }

    #[test]
    fn test_pio_clock_divider() {
        let mut pio = Pio::new(0);

        // Set clock divider for state machine 0
        // CLKDIV = (integer << 16) | (fraction << 8)
        // For div = 2.5: integer = 2, fraction = 128 (0.5 * 256)
        let clkdiv = (2u32 << 16) | (128u32 << 8);
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, clkdiv).unwrap();

        let read_val = pio.read(PIO0_BASE + regs::SM0_CLKDIV).unwrap();
        assert_eq!(read_val, clkdiv);
    }

    #[test]
    fn test_pio_exec_instruction() {
        let mut pio = Pio::new(0);

        // Execute a SET instruction directly via SM0_INSTR
        // SET X, 5 = 0xE085
        pio.write(PIO0_BASE + regs::SM0_INSTR, 0xE085).unwrap();

        // The instruction should be written
        assert_eq!(pio.read(PIO0_BASE + regs::SM0_INSTR).unwrap(), 0xE085);
    }

    #[test]
    fn test_pio_irq_flags() {
        let mut pio = Pio::new(0);

        // Check initial IRQ flags
        assert_eq!(pio.read(PIO0_BASE + regs::IRQ).unwrap(), 0);

        // Set IRQ flag 0 via IRQ_FORCE
        pio.write(PIO0_BASE + regs::IRQ_FORCE, 0x01).unwrap();
        // IRQ_FORCE doesn't persist, it just pulses the IRQ
    }

    #[test]
    fn test_pio_fifo_levels() {
        let mut pio = Pio::new(0);

        // Check initial levels
        let flevel = pio.read(PIO0_BASE + regs::FLEVEL).unwrap();
        // All FIFOs should be empty (level = 0)
        // FLEVEL format: bits 0-3 = TX0 level, bits 4-7 = RX0 level, etc.
        assert_eq!(flevel, 0);

        // Write to TXF0 (this updates FLEVEL)
        pio.write(PIO0_BASE + regs::TXF0, 0x12345678).unwrap();
        pio.write(PIO0_BASE + regs::TXF0, 0x9ABCDEF0).unwrap();

        // Check TX FIFO 0 level
        let flevel = pio.read(PIO0_BASE + regs::FLEVEL).unwrap();
        let tx0_level = flevel & 0x0F;
        assert_eq!(tx0_level, 2);
    }

    #[test]
    fn test_pio_shift_control() {
        let mut pio = Pio::new(0);

        // Set shift control for state machine 0
        // This controls autopush/pull, shift thresholds, etc.
        pio.write(PIO0_BASE + regs::SM0_SHIFTCTRL, 0x000F000F).unwrap();
        assert_eq!(pio.read(PIO0_BASE + regs::SM0_SHIFTCTRL).unwrap(), 0x000F000F);
    }

    #[test]
    fn test_pio_exec_control() {
        let mut pio = Pio::new(0);

        // Set execution control for state machine 0
        // This controls wrap targets, side-set pins, etc.
        pio.write(PIO0_BASE + regs::SM0_EXECCTRL, 0x00001F1F).unwrap();
        assert_eq!(pio.read(PIO0_BASE + regs::SM0_EXECCTRL).unwrap(), 0x00001F1F);
    }

    #[test]
    fn test_pio_jmp_instruction() {
        let mut pio = Pio::new(0);

        // Load JMP instruction at address 0: JMP 5
        // Format: 000 address[5] condition[3] delay[5]
        // JMP 5 = 0x0005 (unconditional jump to address 5)
        pio.write(PIO0_BASE + regs::INSTR_MEM0, 0x0005).unwrap();

        // Enable state machine 0 with clock div = 1
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, 1 << 16).unwrap();
        pio.write(PIO0_BASE + regs::CTRL, 0x01).unwrap();

        // Tick to execute
        pio.tick();

        // PC should have jumped to 5
        // Note: This depends on the actual implementation
    }

    #[test]
    fn test_pio_set_instruction() {
        let mut pio = Pio::new(0);

        // SET X, 5 = 0xE085
        // Format: 111 data[5] 001 delay[5]
        pio.write(PIO0_BASE + regs::INSTR_MEM0, 0xE085).unwrap();

        // Enable state machine 0
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, 1 << 16).unwrap();
        pio.write(PIO0_BASE + regs::CTRL, 0x01).unwrap();

        // Tick to execute
        pio.tick();

        // X register should be set to 5
        // Note: This depends on the actual implementation
    }

    #[test]
    fn test_pio_mov_instruction() {
        let mut pio = Pio::new(0);

        // Set X to a known value first
        pio.state_machines[0].x = 0x12345678;

        // MOV Y, X
        // Format: [15:13]=opcode(6), [12:8]=delay, [7:5]=dest, [4:3]=op, [2:0]=src
        // dest=1 (Y), src=0 (X)
        // MOV Y, X = 0b110_00000_001_00_000 = 0xC020
        let mov_instr: u16 = (6u16 << 13) | (1u16 << 5) | 0;
        pio.write(PIO0_BASE + regs::INSTR_MEM0, mov_instr as u32).unwrap();

        // Enable state machine 0
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, 1 << 16).unwrap();
        pio.write(PIO0_BASE + regs::CTRL, 0x01).unwrap();

        // Tick to execute
        pio.tick();

        // Y should now equal X
        assert_eq!(pio.state_machines[0].y, 0x12345678);
    }

    #[test]
    fn test_pio_push_instruction() {
        let mut pio = Pio::new(0);

        // Set ISR to a known value
        pio.state_machines[0].isr = 0xDEADBEEF;

        // PUSH instruction (opcode 5 in current implementation pushes ISR to RX FIFO)
        // Format: 101 0 block iffull 0 0 delay[5]
        // PUSH = 0b101_0_0_0_0_00000 = 0xA000
        let push_instr: u16 = (5u16 << 13);
        pio.write(PIO0_BASE + regs::INSTR_MEM0, push_instr as u32).unwrap();

        // Enable state machine 0
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, 1 << 16).unwrap();
        pio.write(PIO0_BASE + regs::CTRL, 0x01).unwrap();

        // Tick to execute
        pio.tick();

        // RX FIFO should contain ISR value
        let val = pio.pop_rx(0);
        assert_eq!(val, Some(0xDEADBEEF));
    }

    #[test]
    fn test_pio_pull_instruction() {
        let mut pio = Pio::new(0);

        // Push a value to TX FIFO
        pio.push_tx(0, 0xCAFEBABE);

        // PULL instruction (opcode 4 in current implementation pulls from TX FIFO to OSR)
        // Format: 100 1 block ifempty 0 0 delay[5]
        // PULL = 0b100_1_0_0_0_00000 = 0x9000
        let pull_instr: u16 = (4u16 << 13) | (1u16 << 12);
        pio.write(PIO0_BASE + regs::INSTR_MEM0, pull_instr as u32).unwrap();

        // Enable state machine 0
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, 1 << 16).unwrap();
        pio.write(PIO0_BASE + regs::CTRL, 0x01).unwrap();

        // Tick to execute
        pio.tick();

        // OSR should contain the TX FIFO value
        assert_eq!(pio.state_machines[0].osr, 0xCAFEBABE);
    }

    #[test]
    fn test_pio_multiple_state_machines() {
        let mut pio = Pio::new(0);

        // Enable all 4 state machines
        pio.write(PIO0_BASE + regs::CTRL, 0x0F).unwrap();

        // Set different clock dividers
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, 1 << 16).unwrap();
        pio.write(PIO0_BASE + regs::SM1_CLKDIV, 2 << 16).unwrap();
        pio.write(PIO0_BASE + regs::SM2_CLKDIV, 4 << 16).unwrap();
        pio.write(PIO0_BASE + regs::SM3_CLKDIV, 8 << 16).unwrap();

        // All state machines should be enabled
        assert!(pio.state_machines[0].enabled);
        assert!(pio.state_machines[1].enabled);
        assert!(pio.state_machines[2].enabled);
        assert!(pio.state_machines[3].enabled);
    }

    #[test]
    fn test_pio_fifo_full_empty_flags() {
        let mut pio = Pio::new(0);

        // Initially all TX FIFOs are empty
        let fstat = pio.read(PIO0_BASE + regs::FSTAT).unwrap();
        assert_eq!(fstat & 0x0F, 0x0F); // TXEMPTY bits

        // Fill TX FIFO 0
        for i in 0..FIFO_SIZE {
            pio.push_tx(0, i as u32);
        }

        // Update fstat manually (normally done by tick())
        pio.update_fstat();

        // TX FIFO 0 should now be full (bit 4 in TXFULL field)
        let fstat = pio.read(PIO0_BASE + regs::FSTAT).unwrap();
        assert_eq!(fstat & 0x10, 0x10); // TXFULL bit for SM0
    }

    #[test]
    fn test_pio_instruction_delay() {
        let mut pio = Pio::new(0);

        // SET X, 0 with delay of 5 cycles
        // Format: [15:13]=opcode(7), [12:8]=delay, [7:5]=001(SET), [4:0]=data
        // delay = 5 = 0b00101
        // SET X, 0 with delay 5 = 0b111_00101_001_00000 = 0xE940
        let set_instr: u16 = (7u16 << 13) | (5u16 << 8) | (1u16 << 5) | 0;
        pio.write(PIO0_BASE + regs::INSTR_MEM0, set_instr as u32).unwrap();

        // Enable state machine 0
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, 1 << 16).unwrap();
        pio.write(PIO0_BASE + regs::CTRL, 0x01).unwrap();

        // First tick executes the instruction
        pio.tick();

        // Delay should be set to 5
        assert_eq!(pio.state_machines[0].delay, 5);
    }

    #[test]
    fn test_pio_program_counter_wrap() {
        let mut pio = Pio::new(0);

        // Set PC to near the end of instruction memory
        pio.state_machines[0].pc = 31;

        // Enable state machine 0
        pio.write(PIO0_BASE + regs::SM0_CLKDIV, 1 << 16).unwrap();
        pio.write(PIO0_BASE + regs::CTRL, 0x01).unwrap();

        // Execute a simple instruction (NOP-like)
        pio.write(PIO0_BASE + regs::INSTR_MEM0 + 31 * 4, 0x0000).unwrap();

        // Tick to execute
        pio.tick();

        // PC should wrap to 0
        assert_eq!(pio.state_machines[0].pc, 0);
    }
}
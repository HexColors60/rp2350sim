//! I2C device for RP2350.
//!
//! Implements the I2C peripheral with full register support.

use rp2350sim_core::{Device, DeviceId, Result};
use std::collections::VecDeque;

/// I2C base addresses.
pub const I2C0_BASE: u32 = 0x4004_4000;
pub const I2C1_BASE: u32 = 0x4004_8000;

/// I2C register offsets.
pub mod regs {
    pub const IC_CON: u32 = 0x000;
    pub const IC_TAR: u32 = 0x004;
    pub const IC_SAR: u32 = 0x008;
    pub const IC_HS_MADDR: u32 = 0x00C;
    pub const IC_DATA_CMD: u32 = 0x010;
    pub const IC_SS_SCL_HCNT: u32 = 0x014;
    pub const IC_SS_SCL_LCNT: u32 = 0x018;
    pub const IC_FS_SCL_HCNT: u32 = 0x01C;
    pub const IC_FS_SCL_LCNT: u32 = 0x020;
    pub const IC_INTR_STAT: u32 = 0x02C;
    pub const IC_INTR_MASK: u32 = 0x030;
    pub const IC_RAW_INTR_STAT: u32 = 0x034;
    pub const IC_RX_TL: u32 = 0x038;
    pub const IC_TX_TL: u32 = 0x03C;
    pub const IC_CLR_INTR: u32 = 0x040;
    pub const IC_CLR_RX_UNDER: u32 = 0x044;
    pub const IC_CLR_RX_OVER: u32 = 0x048;
    pub const IC_CLR_TX_OVER: u32 = 0x04C;
    pub const IC_CLR_RD_REQ: u32 = 0x050;
    pub const IC_CLR_TX_ABRT: u32 = 0x054;
    pub const IC_CLR_RX_DONE: u32 = 0x058;
    pub const IC_CLR_ACTIVITY: u32 = 0x05C;
    pub const IC_CLR_STOP_DET: u32 = 0x060;
    pub const IC_CLR_START_DET: u32 = 0x064;
    pub const IC_CLR_GEN_CALL: u32 = 0x068;
    pub const IC_ENABLE: u32 = 0x06C;
    pub const IC_STATUS: u32 = 0x070;
    pub const IC_TXFLR: u32 = 0x074;
    pub const IC_RXFLR: u32 = 0x078;
    pub const IC_SDA_HOLD: u32 = 0x07C;
    pub const IC_TX_ABRT_SOURCE: u32 = 0x080;
    pub const IC_SLV_DATA_NACK_ONLY: u32 = 0x084;
    pub const IC_DMA_CR: u32 = 0x088;
    pub const IC_DMA_TDLR: u32 = 0x08C;
    pub const IC_DMA_RDLR: u32 = 0x090;
    pub const IC_SDA_SETUP: u32 = 0x094;
    pub const IC_ACK_GENERAL_CALL: u32 = 0x098;
    pub const IC_ENABLE_STATUS: u32 = 0x09C;
    pub const IC_FS_SPKLEN: u32 = 0x0A0;
    pub const IC_HS_SPKLEN: u32 = 0x0A4;
    pub const IC_CLR_RESTART_DET: u32 = 0x0A8;
    pub const IC_COMP_PARAM_1: u32 = 0x0F4;
    pub const IC_COMP_VERSION: u32 = 0x0F8;
    pub const IC_COMP_TYPE: u32 = 0x0FC;
}

/// IC_CON bits.
pub mod con {
    pub const MASTER_MODE: u32 = 1 << 0;
    pub const SPEED_STD: u32 = 0x01 << 1;
    pub const SPEED_FAST: u32 = 0x02 << 1;
    pub const SPEED_HIGH: u32 = 0x03 << 1;
    pub const SPEED_MASK: u32 = 0x03 << 1;
    pub const IC_10BITADDR_SLAVE: u32 = 1 << 3;
    pub const IC_10BITADDR_MASTER: u32 = 1 << 4;
    pub const IC_RESTART_EN: u32 = 1 << 5;
    pub const IC_SLAVE_DISABLE: u32 = 1 << 6;
    pub const STOP_DET_IFADDRESSED: u32 = 1 << 7;
    pub const TX_EMPTY_CTRL: u32 = 1 << 8;
    pub const RX_FIFO_FULL_HLD_CTRL: u32 = 1 << 9;
}

/// IC_DATA_CMD bits.
pub mod data_cmd {
    pub const CMD_READ: u16 = 1 << 8;
    pub const CMD_STOP: u16 = 1 << 9;
    pub const CMD_RESTART: u16 = 1 << 10;
}

/// IC_STATUS bits.
pub mod status {
    pub const ACTIVITY: u32 = 1 << 0;
    pub const TFNF: u32 = 1 << 1;  // TX FIFO not full
    pub const TFE: u32 = 1 << 2;   // TX FIFO empty
    pub const RFNE: u32 = 1 << 3;  // RX FIFO not empty
    pub const RFF: u32 = 1 << 4;   // RX FIFO full
    pub const MST_ACTIVITY: u32 = 1 << 5;
    pub const SLV_ACTIVITY: u32 = 1 << 6;
}

/// Interrupt bits.
pub mod intr {
    pub const RX_UNDER: u32 = 1 << 0;
    pub const RX_OVER: u32 = 1 << 1;
    pub const RX_FULL: u32 = 1 << 2;
    pub const TX_OVER: u32 = 1 << 3;
    pub const TX_EMPTY: u32 = 1 << 4;
    pub const RD_REQ: u32 = 1 << 5;
    pub const TX_ABRT: u32 = 1 << 6;
    pub const RX_DONE: u32 = 1 << 7;
    pub const ACTIVITY: u32 = 1 << 8;
    pub const STOP_DET: u32 = 1 << 9;
    pub const START_DET: u32 = 1 << 10;
    pub const GEN_CALL: u32 = 1 << 11;
    pub const RESTART_DET: u32 = 1 << 12;
}

/// FIFO depth.
const FIFO_SIZE: usize = 16;

/// I2C device.
pub struct I2c {
    /// Base address.
    base: u32,
    /// Control register.
    con: u32,
    /// Target address.
    tar: u16,
    /// Slave address.
    sar: u16,
    /// High speed master address.
    hs_maddr: u8,
    /// SS SCL high count.
    ss_scl_hcnt: u16,
    /// SS SCL low count.
    ss_scl_lcnt: u16,
    /// FS SCL high count.
    fs_scl_hcnt: u16,
    /// FS SCL low count.
    fs_scl_lcnt: u16,
    /// Interrupt mask.
    intr_mask: u32,
    /// Raw interrupt status.
    raw_intr_stat: u32,
    /// RX threshold.
    rx_tl: u8,
    /// TX threshold.
    tx_tl: u8,
    /// Enable register.
    enable: u32,
    /// TX abort source.
    tx_abrt_source: u32,
    /// SDA hold time.
    sda_hold: u16,
    /// TX FIFO.
    tx_fifo: VecDeque<u16>,
    /// RX FIFO.
    rx_fifo: VecDeque<u16>,
    /// Virtual slave devices.
    slaves: Vec<Box<dyn I2cSlave + Send + Sync>>,
    /// Current state.
    state: I2cState,
    /// Current transaction address.
    current_addr: u8,
    /// Transaction in progress.
    transaction_active: bool,
}

impl std::fmt::Debug for I2c {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I2c")
            .field("base", &self.base)
            .field("con", &self.con)
            .field("tar", &self.tar)
            .field("sar", &self.sar)
            .field("slaves_count", &self.slaves.len())
            .field("state", &self.state)
            .finish()
    }
}

/// I2C state machine state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum I2cState {
    Idle,
    Start,
    #[allow(dead_code)]
    Address,
    ReadData,
    WriteData,
    Stop,
}

impl Default for I2c {
    fn default() -> Self {
        Self::new(0)
    }
}

impl I2c {
    /// Create a new I2C device.
    pub fn new(id: u8) -> Self {
        Self {
            base: if id == 0 { I2C0_BASE } else { I2C1_BASE },
            con: con::MASTER_MODE | con::SPEED_FAST | con::IC_SLAVE_DISABLE | con::IC_RESTART_EN,
            tar: 0x55,
            sar: 0x55,
            hs_maddr: 0,
            ss_scl_hcnt: 0,
            ss_scl_lcnt: 0,
            fs_scl_hcnt: 0,
            fs_scl_lcnt: 0,
            intr_mask: 0,
            raw_intr_stat: 0,
            rx_tl: 0,
            tx_tl: 0,
            enable: 0,
            tx_abrt_source: 0,
            sda_hold: 0,
            tx_fifo: VecDeque::with_capacity(FIFO_SIZE),
            rx_fifo: VecDeque::with_capacity(FIFO_SIZE),
            slaves: Vec::new(),
            state: I2cState::Idle,
            current_addr: 0,
            transaction_active: false,
        }
    }

    /// Create I2C0.
    pub fn i2c0() -> Self {
        Self::new(0)
    }

    /// Create I2C1.
    pub fn i2c1() -> Self {
        Self::new(1)
    }

    /// Get base address.
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.enable & 1) != 0
    }

    /// Check if master mode.
    pub fn is_master(&self) -> bool {
        (self.con & con::MASTER_MODE) != 0
    }

    /// Check if busy.
    pub fn is_busy(&self) -> bool {
        self.transaction_active || !self.tx_fifo.is_empty()
    }

    /// Get target address.
    pub fn get_target_address(&self) -> u8 {
        (self.tar & 0x7F) as u8
    }

    /// Enable the I2C controller.
    pub fn enable(&mut self, enable: bool) {
        self.enable = if enable { 1 } else { 0 };
    }

    /// Generate start condition.
    pub fn start(&mut self) -> Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        self.state = I2cState::Start;
        self.transaction_active = true;
        Ok(())
    }

    /// Generate stop condition.
    pub fn stop(&mut self) -> Result<()> {
        self.state = I2cState::Stop;
        self.transaction_active = false;
        self.raw_intr_stat |= intr::STOP_DET;
        Ok(())
    }

    /// Write address byte.
    pub fn write_address(&mut self, addr: u8, read: bool) -> Result<bool> {
        self.current_addr = addr;

        // Check if any slave responds
        let ack = self.slaves.iter_mut().any(|s| s.match_address(addr));

        if !ack {
            self.tx_abrt_source |= 1 << 7; // TX_ABRT_7B_ADDR_NOACK
            self.raw_intr_stat |= intr::TX_ABRT;
        }

        self.state = if read { I2cState::ReadData } else { I2cState::WriteData };
        Ok(ack)
    }

    /// Write data byte.
    pub fn write_byte(&mut self, data: u8) -> Result<bool> {
        let ack = self.slaves.iter_mut().any(|s| {
            if s.match_address(self.current_addr) {
                s.write(data);
                true
            } else {
                false
            }
        });

        if !ack {
            self.tx_abrt_source |= 1 << 8; // TX_ABRT_ACKED
        }

        Ok(ack)
    }

    /// Read data byte.
    pub fn read_byte(&mut self, ack: bool) -> u8 {
        for slave in &mut self.slaves {
            if slave.match_address(self.current_addr) {
                return slave.read(ack);
            }
        }
        0xFF
    }

    /// Add a slave device.
    pub fn add_slave<S: I2cSlave + Send + 'static>(&mut self, slave: S) {
        self.slaves.push(Box::new(slave));
    }

    /// Update status register.
    fn get_status(&self) -> u32 {
        let mut status = 0;

        if self.transaction_active {
            status |= status::ACTIVITY;
            if self.is_master() {
                status |= status::MST_ACTIVITY;
            } else {
                status |= status::SLV_ACTIVITY;
            }
        }

        if self.tx_fifo.len() < FIFO_SIZE {
            status |= status::TFNF;
        }
        if self.tx_fifo.is_empty() {
            status |= status::TFE;
        }
        if !self.rx_fifo.is_empty() {
            status |= status::RFNE;
        }
        if self.rx_fifo.len() >= FIFO_SIZE {
            status |= status::RFF;
        }

        status
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        (self.raw_intr_stat & self.intr_mask) != 0
    }

    /// Process TX FIFO.
    fn process_tx_fifo(&mut self) {
        while let Some(cmd) = self.tx_fifo.pop_front() {
            let is_read = (cmd & data_cmd::CMD_READ) != 0;
            let is_stop = (cmd & data_cmd::CMD_STOP) != 0;
            let is_restart = (cmd & data_cmd::CMD_RESTART) != 0;
            let data = (cmd & 0xFF) as u8;

            if is_restart || !self.transaction_active {
                self.start().ok();
                self.write_address(self.get_target_address(), is_read).ok();
            }

            if is_read {
                let byte = self.read_byte(!is_stop);
                self.rx_fifo.push_back(byte as u16);
                self.raw_intr_stat |= intr::RX_FULL;
            } else {
                self.write_byte(data).ok();
            }

            if is_stop {
                self.stop().ok();
            }
        }
    }
}

impl Device for I2c {
    fn id(&self) -> DeviceId {
        DeviceId::I2C
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - self.base;

        match offset {
            regs::IC_CON => Ok(self.con),
            regs::IC_TAR => Ok(self.tar as u32),
            regs::IC_SAR => Ok(self.sar as u32),
            regs::IC_HS_MADDR => Ok(self.hs_maddr as u32),
            regs::IC_DATA_CMD => {
                let data = self.rx_fifo.pop_front().unwrap_or(0);
                Ok(data as u32)
            }
            regs::IC_SS_SCL_HCNT => Ok(self.ss_scl_hcnt as u32),
            regs::IC_SS_SCL_LCNT => Ok(self.ss_scl_lcnt as u32),
            regs::IC_FS_SCL_HCNT => Ok(self.fs_scl_hcnt as u32),
            regs::IC_FS_SCL_LCNT => Ok(self.fs_scl_lcnt as u32),
            regs::IC_INTR_STAT => Ok(self.raw_intr_stat & self.intr_mask),
            regs::IC_INTR_MASK => Ok(self.intr_mask),
            regs::IC_RAW_INTR_STAT => Ok(self.raw_intr_stat),
            regs::IC_RX_TL => Ok(self.rx_tl as u32),
            regs::IC_TX_TL => Ok(self.tx_tl as u32),
            regs::IC_CLR_INTR => {
                let val = self.raw_intr_stat;
                self.raw_intr_stat = 0;
                Ok(val)
            }
            regs::IC_ENABLE => Ok(self.enable),
            regs::IC_STATUS => Ok(self.get_status()),
            regs::IC_TXFLR => Ok(self.tx_fifo.len() as u32),
            regs::IC_RXFLR => Ok(self.rx_fifo.len() as u32),
            regs::IC_SDA_HOLD => Ok(self.sda_hold as u32),
            regs::IC_TX_ABRT_SOURCE => Ok(self.tx_abrt_source),
            regs::IC_ENABLE_STATUS => Ok(self.enable),
            regs::IC_COMP_PARAM_1 => Ok(0x00),
            regs::IC_COMP_VERSION => Ok(0x3130322A),
            regs::IC_COMP_TYPE => Ok(0x44570140),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - self.base;

        match offset {
            regs::IC_CON => {
                self.con = value & 0x3FF;
            }
            regs::IC_TAR => {
                self.tar = (value & 0x3FF) as u16;
            }
            regs::IC_SAR => {
                self.sar = (value & 0x3FF) as u16;
            }
            regs::IC_HS_MADDR => {
                self.hs_maddr = (value & 0x7F) as u8;
            }
            regs::IC_DATA_CMD => {
                if self.tx_fifo.len() < FIFO_SIZE {
                    self.tx_fifo.push_back((value & 0x7FF) as u16);
                    self.process_tx_fifo();
                }
            }
            regs::IC_SS_SCL_HCNT => {
                self.ss_scl_hcnt = (value & 0xFFFF) as u16;
            }
            regs::IC_SS_SCL_LCNT => {
                self.ss_scl_lcnt = (value & 0xFFFF) as u16;
            }
            regs::IC_FS_SCL_HCNT => {
                self.fs_scl_hcnt = (value & 0xFFFF) as u16;
            }
            regs::IC_FS_SCL_LCNT => {
                self.fs_scl_lcnt = (value & 0xFFFF) as u16;
            }
            regs::IC_INTR_MASK => {
                self.intr_mask = value & 0x1FFF;
            }
            regs::IC_RX_TL => {
                self.rx_tl = (value & 0xFF) as u8;
            }
            regs::IC_TX_TL => {
                self.tx_tl = (value & 0xFF) as u8;
            }
            regs::IC_ENABLE => {
                self.enable = value & 1;
            }
            regs::IC_SDA_HOLD => {
                self.sda_hold = (value & 0xFFFF) as u16;
            }
            regs::IC_CLR_TX_ABRT => {
                self.tx_abrt_source = 0;
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        let base = self.base;
        *self = Self {
            base,
            ..Self::new(if base == I2C0_BASE { 0 } else { 1 })
        };
    }
}

/// I2C slave device trait.
pub trait I2cSlave: Send + Sync {
    /// Check if address matches.
    fn match_address(&self, addr: u8) -> bool;

    /// Write data to slave.
    fn write(&mut self, data: u8);

    /// Read data from slave.
    fn read(&mut self, ack: bool) -> u8;
}

/// Simple I2C EEPROM slave.
#[derive(Debug)]
pub struct I2cEeprom {
    address: u8,
    data: Vec<u8>,
    pointer: usize,
    /// Whether we're still receiving address bytes.
    address_mode: bool,
}

impl I2cEeprom {
    pub fn new(address: u8, size: usize) -> Self {
        Self {
            address,
            data: vec![0xFF; size],
            pointer: 0,
            address_mode: true,
        }
    }

    pub fn load(&mut self, offset: usize, data: &[u8]) {
        let end = std::cmp::min(offset + data.len(), self.data.len());
        self.data[offset..end].copy_from_slice(&data[..end - offset]);
    }

    pub fn read_all(&self) -> &[u8] {
        &self.data
    }
}

impl I2cSlave for I2cEeprom {
    fn match_address(&self, addr: u8) -> bool {
        self.address == addr
    }

    fn write(&mut self, data: u8) {
        if self.address_mode {
            // First byte(s) set the address pointer
            self.pointer = (self.pointer << 8) | (data as usize);
            self.address_mode = false;
        } else if self.pointer < self.data.len() {
            self.data[self.pointer] = data;
            self.pointer += 1;
        }
    }

    fn read(&mut self, _ack: bool) -> u8 {
        let data = if self.pointer < self.data.len() {
            self.data[self.pointer]
        } else {
            0xFF
        };
        self.pointer += 1;
        data
    }
}

/// I2C temperature sensor slave (e.g., TMP102).
#[derive(Debug)]
pub struct I2cTempSensor {
    address: u8,
    temperature: i16,
    pointer: u8,
}

impl I2cTempSensor {
    pub fn new(address: u8) -> Self {
        Self {
            address,
            temperature: 25 << 4, // 25°C in TMP102 format
            pointer: 0,
        }
    }

    pub fn set_temperature(&mut self, temp_celsius: f32) {
        self.temperature = (temp_celsius * 16.0) as i16;
    }
}

impl I2cSlave for I2cTempSensor {
    fn match_address(&self, addr: u8) -> bool {
        self.address == addr
    }

    fn write(&mut self, data: u8) {
        self.pointer = data;
    }

    fn read(&mut self, _ack: bool) -> u8 {
        let data = match self.pointer {
            0 => (self.temperature >> 8) as u8,
            1 => (self.temperature & 0xFF) as u8,
            _ => 0,
        };
        self.pointer += 1;
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i2c_creation() {
        let i2c = I2c::i2c0();
        assert_eq!(i2c.base(), I2C0_BASE);
    }

    #[test]
    fn test_i2c_enable() {
        let mut i2c = I2c::i2c0();
        assert!(!i2c.is_enabled());
        i2c.enable(true);
        assert!(i2c.is_enabled());
    }

    #[test]
    fn test_i2c_master_mode() {
        let i2c = I2c::i2c0();
        assert!(i2c.is_master());
    }

    #[test]
    fn test_i2c_eeprom() {
        let mut eeprom = I2cEeprom::new(0x50, 256);

        // Write address pointer
        eeprom.write(0x00);
        eeprom.write(0x10);

        // Read back
        eeprom.pointer = 0;
        let data = eeprom.read(true);
        assert_eq!(data, 0x10);
    }

    #[test]
    fn test_i2c_temp_sensor() {
        let mut sensor = I2cTempSensor::new(0x48);
        sensor.set_temperature(25.0);

        sensor.write(0); // Read temperature register
        let msb = sensor.read(true);
        let lsb = sensor.read(true);

        let temp = ((msb as i16) << 8) | (lsb as i16);
        // Temperature is 25.0 * 16 = 400 in TMP102 format
        assert_eq!(temp, 400);
        // Temperature in Celsius is temp >> 4 = 25
        assert_eq!(temp >> 4, 25);
    }

    #[test]
    fn test_i2c_register_read_write() {
        let mut i2c = I2c::i2c0();

        // Test IC_CON register
        i2c.write(I2C0_BASE + regs::IC_CON, con::MASTER_MODE | con::IC_SLAVE_DISABLE).unwrap();
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_CON).unwrap() & con::MASTER_MODE, con::MASTER_MODE);

        // Test IC_TAR register (target address)
        i2c.write(I2C0_BASE + regs::IC_TAR, 0x50).unwrap();
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_TAR).unwrap() & 0x3FF, 0x50);

        // Test IC_ENABLE register
        i2c.write(I2C0_BASE + regs::IC_ENABLE, 1).unwrap();
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_ENABLE).unwrap(), 1);
    }

    #[test]
    fn test_i2c_status_register() {
        let mut i2c = I2c::i2c0();

        // Check initial status
        let status = i2c.read(I2C0_BASE + regs::IC_STATUS).unwrap();
        // TX FIFO should be empty initially
        assert_eq!(status & status::TFE, status::TFE);
    }

    #[test]
    fn test_i2c_slave_address() {
        let mut i2c = I2c::i2c0();

        // Set slave address (for slave mode)
        i2c.write(I2C0_BASE + regs::IC_SAR, 0x55).unwrap();
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_SAR).unwrap(), 0x55);
    }

    #[test]
    fn test_i2c_fifo_levels() {
        let mut i2c = I2c::i2c0();
        i2c.enable(true);

        // Check initial FIFO levels
        let txflr = i2c.read(I2C0_BASE + regs::IC_TXFLR).unwrap();
        let rxflr = i2c.read(I2C0_BASE + regs::IC_RXFLR).unwrap();
        assert_eq!(txflr, 0);
        assert_eq!(rxflr, 0);
    }

    #[test]
    fn test_i2c_clock_speed() {
        let mut i2c = I2c::i2c0();

        // Set standard speed SCL timing
        i2c.write(I2C0_BASE + regs::IC_SS_SCL_HCNT, 100).unwrap();
        i2c.write(I2C0_BASE + regs::IC_SS_SCL_LCNT, 120).unwrap();

        assert_eq!(i2c.read(I2C0_BASE + regs::IC_SS_SCL_HCNT).unwrap(), 100);
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_SS_SCL_LCNT).unwrap(), 120);

        // Set fast speed SCL timing
        i2c.write(I2C0_BASE + regs::IC_FS_SCL_HCNT, 30).unwrap();
        i2c.write(I2C0_BASE + regs::IC_FS_SCL_LCNT, 40).unwrap();

        assert_eq!(i2c.read(I2C0_BASE + regs::IC_FS_SCL_HCNT).unwrap(), 30);
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_FS_SCL_LCNT).unwrap(), 40);
    }

    #[test]
    fn test_i2c_interrupt_mask() {
        let mut i2c = I2c::i2c0();

        // Enable various interrupts
        i2c.write(I2C0_BASE + regs::IC_INTR_MASK, intr::RX_FULL | intr::TX_EMPTY | intr::STOP_DET).unwrap();

        let mask = i2c.read(I2C0_BASE + regs::IC_INTR_MASK).unwrap();
        assert_eq!(mask & intr::RX_FULL, intr::RX_FULL);
        assert_eq!(mask & intr::TX_EMPTY, intr::TX_EMPTY);
        assert_eq!(mask & intr::STOP_DET, intr::STOP_DET);
    }

    #[test]
    fn test_i2c_fifo_thresholds() {
        let mut i2c = I2c::i2c0();

        // Set RX threshold
        i2c.write(I2C0_BASE + regs::IC_RX_TL, 8).unwrap();
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_RX_TL).unwrap(), 8);

        // Set TX threshold
        i2c.write(I2C0_BASE + regs::IC_TX_TL, 4).unwrap();
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_TX_TL).unwrap(), 4);
    }

    #[test]
    fn test_i2c_sda_hold_time() {
        let mut i2c = I2c::i2c0();

        // Set SDA hold time (16-bit value)
        i2c.write(I2C0_BASE + regs::IC_SDA_HOLD, 0x1234).unwrap();
        let hold = i2c.read(I2C0_BASE + regs::IC_SDA_HOLD).unwrap();
        assert_eq!(hold, 0x1234);
    }

    #[test]
    fn test_i2c_10bit_addressing() {
        let mut i2c = I2c::i2c0();

        // Enable 10-bit addressing for master
        let con_val = con::MASTER_MODE | con::SPEED_FAST | con::IC_SLAVE_DISABLE | con::IC_10BITADDR_MASTER;
        i2c.write(I2C0_BASE + regs::IC_CON, con_val).unwrap();

        let con_read = i2c.read(I2C0_BASE + regs::IC_CON).unwrap();
        assert_eq!(con_read & con::IC_10BITADDR_MASTER, con::IC_10BITADDR_MASTER);

        // Set 10-bit target address
        i2c.write(I2C0_BASE + regs::IC_TAR, 0x3FF).unwrap();
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_TAR).unwrap() & 0x3FF, 0x3FF);
    }

    #[test]
    fn test_i2c_speed_modes() {
        let mut i2c = I2c::i2c0();

        // Standard speed (100 kHz)
        i2c.write(I2C0_BASE + regs::IC_CON, con::MASTER_MODE | con::SPEED_STD | con::IC_SLAVE_DISABLE).unwrap();
        let con = i2c.read(I2C0_BASE + regs::IC_CON).unwrap();
        assert_eq!((con >> 1) & 0x3, 1);

        // Fast speed (400 kHz)
        i2c.write(I2C0_BASE + regs::IC_CON, con::MASTER_MODE | con::SPEED_FAST | con::IC_SLAVE_DISABLE).unwrap();
        let con = i2c.read(I2C0_BASE + regs::IC_CON).unwrap();
        assert_eq!((con >> 1) & 0x3, 2);

        // High speed (3.4 MHz)
        i2c.write(I2C0_BASE + regs::IC_CON, con::MASTER_MODE | con::SPEED_HIGH | con::IC_SLAVE_DISABLE).unwrap();
        let con = i2c.read(I2C0_BASE + regs::IC_CON).unwrap();
        assert_eq!((con >> 1) & 0x3, 3);
    }

    #[test]
    fn test_i2c_enable_status() {
        let mut i2c = I2c::i2c0();

        // Check enable status register
        let status = i2c.read(I2C0_BASE + regs::IC_ENABLE_STATUS).unwrap();
        // IC_EN bit 0 should be 0 initially
        assert_eq!(status & 0x01, 0);

        // Enable I2C
        i2c.write(I2C0_BASE + regs::IC_ENABLE, 1).unwrap();
        let status = i2c.read(I2C0_BASE + regs::IC_ENABLE_STATUS).unwrap();
        assert_eq!(status & 0x01, 1);
    }

    #[test]
    fn test_i2c_reset() {
        let mut i2c = I2c::i2c0();

        // Modify state
        i2c.write(I2C0_BASE + regs::IC_ENABLE, 1).unwrap();
        i2c.write(I2C0_BASE + regs::IC_TAR, 0x70).unwrap();
        i2c.write(I2C0_BASE + regs::IC_SS_SCL_HCNT, 200).unwrap();

        // Reset
        i2c.reset();

        // Check state is reset
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_ENABLE).unwrap(), 0);
        // TAR should reset to default
        assert_eq!(i2c.read(I2C0_BASE + regs::IC_TAR).unwrap() & 0x3FF, 0x55);
    }
}
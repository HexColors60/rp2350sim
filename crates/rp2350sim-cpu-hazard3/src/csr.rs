//! RISC-V CSR (Control and Status Registers).


/// CSR addresses.
pub const MSTATUS: u16 = 0x300;
pub const MISA: u16 = 0x301;
pub const MIE: u16 = 0x304;
pub const MTVEC: u16 = 0x305;
pub const MSCRATCH: u16 = 0x340;
pub const MEPC: u16 = 0x341;
pub const MCAUSE: u16 = 0x342;
pub const MTVAL: u16 = 0x343;
pub const MIP: u16 = 0x344;

/// CSR state.
#[derive(Debug, Clone, Default)]
pub struct CsrState {
    pub mstatus: u32,
    pub misa: u32,
    pub mie: u32,
    pub mtvec: u32,
    pub mscratch: u32,
    pub mepc: u32,
    pub mcause: u32,
    pub mtval: u32,
    pub mip: u32,
}

impl CsrState {
    pub fn new() -> Self {
        Self {
            mstatus: 0,
            misa: 0x40101104, // RV32IMC
            mie: 0,
            mtvec: 0,
            mscratch: 0,
            mepc: 0,
            mcause: 0,
            mtval: 0,
            mip: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u32 {
        match addr {
            MSTATUS => self.mstatus,
            MISA => self.misa,
            MIE => self.mie,
            MTVEC => self.mtvec,
            MSCRATCH => self.mscratch,
            MEPC => self.mepc,
            MCAUSE => self.mcause,
            MTVAL => self.mtval,
            MIP => self.mip,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u32) {
        match addr {
            MSTATUS => self.mstatus = value,
            MIE => self.mie = value,
            MTVEC => self.mtvec = value & !0x2, // Align to 4 bytes
            MSCRATCH => self.mscratch = value,
            MEPC => self.mepc = value & !0x1, // Clear low bit
            MCAUSE => self.mcause = value,
            MTVAL => self.mtval = value,
            MIP => self.mip = value,
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
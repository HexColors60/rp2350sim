//! RV32 instruction executor.


use crate::state::Hazard3CoreState;
use rp2350sim_core::Result;

use super::decode::{Rv32Instruction, Rv32Kind};

/// Execute an RV32 instruction.
pub fn execute(core: &mut Hazard3CoreState, instr: &Rv32Instruction, memory: &mut impl MemoryAccess) -> Result<u8> {
    let mut cycles = 1u8;

    match instr.kind {
        // LUI
        Rv32Kind::Lui => {
            core.set_reg(instr.rd as usize, (instr.imm as u32) & 0xFFFFF000);
        }

        // AUIPC
        Rv32Kind::Auipc => {
            let pc = core.pc;
            core.set_reg(instr.rd as usize, pc.wrapping_add(instr.imm as u32));
        }

        // JAL
        Rv32Kind::Jal => {
            let pc = core.pc;
            core.set_reg(instr.rd as usize, pc + 4);
            core.pc = pc.wrapping_add(instr.imm as u32);
            cycles = 2;
        }

        // JALR
        Rv32Kind::Jalr => {
            let pc = core.pc;
            let target = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32) & !1;
            core.set_reg(instr.rd as usize, pc + 4);
            core.pc = target;
            cycles = 2;
        }

        // Branch instructions
        Rv32Kind::Beq => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            if a == b {
                core.pc = core.pc.wrapping_add(instr.imm as u32);
                cycles = 2;
            } else {
                core.pc += instr.length as u32;
            }
        }
        Rv32Kind::Bne => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            if a != b {
                core.pc = core.pc.wrapping_add(instr.imm as u32);
                cycles = 2;
            } else {
                core.pc += instr.length as u32;
            }
        }
        Rv32Kind::Blt => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            let b = core.get_reg(instr.rs2 as usize) as i32;
            if a < b {
                core.pc = core.pc.wrapping_add(instr.imm as u32);
                cycles = 2;
            } else {
                core.pc += instr.length as u32;
            }
        }
        Rv32Kind::Bge => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            let b = core.get_reg(instr.rs2 as usize) as i32;
            if a >= b {
                core.pc = core.pc.wrapping_add(instr.imm as u32);
                cycles = 2;
            } else {
                core.pc += instr.length as u32;
            }
        }
        Rv32Kind::Bltu => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            if a < b {
                core.pc = core.pc.wrapping_add(instr.imm as u32);
                cycles = 2;
            } else {
                core.pc += instr.length as u32;
            }
        }
        Rv32Kind::Bgeu => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            if a >= b {
                core.pc = core.pc.wrapping_add(instr.imm as u32);
                cycles = 2;
            } else {
                core.pc += instr.length as u32;
            }
        }

        // Load instructions
        Rv32Kind::Lb => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = memory.read_byte(addr)? as i8 as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        Rv32Kind::Lh => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = memory.read_half(addr)? as i16 as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        Rv32Kind::Lw => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        Rv32Kind::Lbu => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = memory.read_byte(addr)? as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        Rv32Kind::Lhu => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = memory.read_half(addr)? as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }

        // Store instructions
        Rv32Kind::Sb => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = core.get_reg(instr.rs2 as usize) as u8;
            memory.write_byte(addr, value)?;
            cycles = 2;
        }
        Rv32Kind::Sh => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = core.get_reg(instr.rs2 as usize) as u16;
            memory.write_half(addr, value)?;
            cycles = 2;
        }
        Rv32Kind::Sw => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = core.get_reg(instr.rs2 as usize);
            memory.write_word(addr, value)?;
            cycles = 2;
        }

        // Immediate arithmetic
        Rv32Kind::Addi => {
            let a = core.get_reg(instr.rs1 as usize);
            core.set_reg(instr.rd as usize, a.wrapping_add(instr.imm as u32));
        }
        Rv32Kind::Slti => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            core.set_reg(instr.rd as usize, if a < instr.imm { 1 } else { 0 });
        }
        Rv32Kind::Sltiu => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = instr.imm as u32;
            core.set_reg(instr.rd as usize, if a < b { 1 } else { 0 });
        }
        Rv32Kind::Xori => {
            let a = core.get_reg(instr.rs1 as usize);
            core.set_reg(instr.rd as usize, a ^ (instr.imm as u32));
        }
        Rv32Kind::Ori => {
            let a = core.get_reg(instr.rs1 as usize);
            core.set_reg(instr.rd as usize, a | (instr.imm as u32));
        }
        Rv32Kind::Andi => {
            let a = core.get_reg(instr.rs1 as usize);
            core.set_reg(instr.rd as usize, a & (instr.imm as u32));
        }
        Rv32Kind::Slli => {
            let a = core.get_reg(instr.rs1 as usize);
            let shamt = (instr.imm as u32) & 0x1F;
            core.set_reg(instr.rd as usize, a << shamt);
        }
        Rv32Kind::Srli => {
            let a = core.get_reg(instr.rs1 as usize);
            let shamt = (instr.imm as u32) & 0x1F;
            core.set_reg(instr.rd as usize, a >> shamt);
        }
        Rv32Kind::Srai => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            let shamt = (instr.imm as u32) & 0x1F;
            core.set_reg(instr.rd as usize, (a >> shamt) as u32);
        }

        // Register arithmetic
        Rv32Kind::Add => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rd as usize, a.wrapping_add(b));
        }
        Rv32Kind::Sub => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rd as usize, a.wrapping_sub(b));
        }
        Rv32Kind::Sll => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize) & 0x1F;
            core.set_reg(instr.rd as usize, a << b);
        }
        Rv32Kind::Slt => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            let b = core.get_reg(instr.rs2 as usize) as i32;
            core.set_reg(instr.rd as usize, if a < b { 1 } else { 0 });
        }
        Rv32Kind::Sltu => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rd as usize, if a < b { 1 } else { 0 });
        }
        Rv32Kind::Xor => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rd as usize, a ^ b);
        }
        Rv32Kind::Srl => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize) & 0x1F;
            core.set_reg(instr.rd as usize, a >> b);
        }
        Rv32Kind::Sra => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            let b = core.get_reg(instr.rs2 as usize) & 0x1F;
            core.set_reg(instr.rd as usize, (a >> b) as u32);
        }
        Rv32Kind::Or => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rd as usize, a | b);
        }
        Rv32Kind::And => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rd as usize, a & b);
        }

        // Multiply/divide
        Rv32Kind::Mul => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            let b = core.get_reg(instr.rs2 as usize) as i32;
            core.set_reg(instr.rd as usize, a.wrapping_mul(b) as u32);
            cycles = 3;
        }
        Rv32Kind::Mulh => {
            let a = core.get_reg(instr.rs1 as usize) as i32 as i64;
            let b = core.get_reg(instr.rs2 as usize) as i32 as i64;
            core.set_reg(instr.rd as usize, ((a * b) >> 32) as u32);
            cycles = 3;
        }
        Rv32Kind::Mulhsu => {
            let a = core.get_reg(instr.rs1 as usize) as i32 as i64;
            let b = core.get_reg(instr.rs2 as usize) as u64;
            core.set_reg(instr.rd as usize, ((a * b as i64) >> 32) as u32);
            cycles = 3;
        }
        Rv32Kind::Mulhu => {
            let a = core.get_reg(instr.rs1 as usize) as u64;
            let b = core.get_reg(instr.rs2 as usize) as u64;
            core.set_reg(instr.rd as usize, ((a * b) >> 32) as u32);
            cycles = 3;
        }
        Rv32Kind::Div => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            let b = core.get_reg(instr.rs2 as usize) as i32;
            let result = if b == 0 {
                u32::MAX
            } else if a == i32::MIN && b == -1 {
                a as u32
            } else {
                (a / b) as u32
            };
            core.set_reg(instr.rd as usize, result);
            cycles = 20;
        }
        Rv32Kind::Divu => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            let result = if b == 0 { u32::MAX } else { a / b };
            core.set_reg(instr.rd as usize, result);
            cycles = 20;
        }
        Rv32Kind::Rem => {
            let a = core.get_reg(instr.rs1 as usize) as i32;
            let b = core.get_reg(instr.rs2 as usize) as i32;
            let result = if b == 0 {
                a as u32
            } else if a == i32::MIN && b == -1 {
                0
            } else {
                (a % b) as u32
            };
            core.set_reg(instr.rd as usize, result);
            cycles = 20;
        }
        Rv32Kind::Remu => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            let result = if b == 0 { a } else { a % b };
            core.set_reg(instr.rd as usize, result);
            cycles = 20;
        }

        // Fence
        Rv32Kind::Fence | Rv32Kind::FenceI => {}

        // System
        Rv32Kind::Ecall => {
            // Environment call - trigger trap
            core.pending_trap = true;
            core.trap_cause = 11; // Environment call from U-mode
        }
        Rv32Kind::Ebreak => {
            // Breakpoint - trigger trap
            core.pending_trap = true;
            core.trap_cause = 3; // Breakpoint
        }
        Rv32Kind::Csrrw => {
            let csr_val = core.csr.read((instr.imm as u32) as u16);
            let rs1_val = core.get_reg(instr.rs1 as usize);
            core.csr.write((instr.imm as u32) as u16, rs1_val);
            core.set_reg(instr.rd as usize, csr_val);
        }
        Rv32Kind::Csrrs => {
            let csr_val = core.csr.read((instr.imm as u32) as u16);
            let rs1_val = core.get_reg(instr.rs1 as usize);
            core.csr.write((instr.imm as u32) as u16, csr_val | rs1_val);
            core.set_reg(instr.rd as usize, csr_val);
        }
        Rv32Kind::Csrrc => {
            let csr_val = core.csr.read((instr.imm as u32) as u16);
            let rs1_val = core.get_reg(instr.rs1 as usize);
            core.csr.write((instr.imm as u32) as u16, csr_val & !rs1_val);
            core.set_reg(instr.rd as usize, csr_val);
        }
        Rv32Kind::Csrrwi => {
            let csr_val = core.csr.read((instr.imm as u32) as u16);
            let zimm = instr.rs1 as u32; // Zero-extended immediate
            core.csr.write((instr.imm as u32) as u16, zimm);
            core.set_reg(instr.rd as usize, csr_val);
        }
        Rv32Kind::Csrrsi => {
            let csr_val = core.csr.read((instr.imm as u32) as u16);
            let zimm = instr.rs1 as u32;
            core.csr.write((instr.imm as u32) as u16, csr_val | zimm);
            core.set_reg(instr.rd as usize, csr_val);
        }
        Rv32Kind::Csrrci => {
            let csr_val = core.csr.read((instr.imm as u32) as u16);
            let zimm = instr.rs1 as u32;
            core.csr.write((instr.imm as u32) as u16, csr_val & !zimm);
            core.set_reg(instr.rd as usize, csr_val);
        }

        // Additional compressed instructions
        Rv32Kind::CLi2 => {
            core.set_reg(instr.rd as usize, instr.imm as u32);
        }
        Rv32Kind::CAddi16sp2 => {
            let sp = core.get_reg(2);
            core.set_reg(2, sp.wrapping_add(instr.imm as u32));
        }
        Rv32Kind::CLui2 => {
            core.set_reg(instr.rd as usize, instr.imm as u32);
        }

        // Compressed instructions
        Rv32Kind::CAddi4spn => {
            let sp = core.get_reg(2);
            core.set_reg(instr.rd as usize, sp.wrapping_add(instr.imm as u32));
        }
        Rv32Kind::CLw => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        Rv32Kind::CSw => {
            let addr = core.get_reg(instr.rs1 as usize).wrapping_add(instr.imm as u32);
            let value = core.get_reg(instr.rs2 as usize);
            memory.write_word(addr, value)?;
            cycles = 2;
        }
        Rv32Kind::CAddi => {
            let val = core.get_reg(instr.rd as usize);
            core.set_reg(instr.rd as usize, val.wrapping_add(instr.imm as u32));
        }
        Rv32Kind::CLi => {
            core.set_reg(instr.rd as usize, instr.imm as u32);
        }
        Rv32Kind::CAddi16sp => {
            let sp = core.get_reg(2);
            core.set_reg(2, sp.wrapping_add(instr.imm as u32));
        }
        Rv32Kind::CLui => {
            core.set_reg(instr.rd as usize, instr.imm as u32);
        }
        Rv32Kind::CSrli => {
            let val = core.get_reg(instr.rs1 as usize);
            let shamt = (instr.imm as u32) & 0x1F;
            core.set_reg(instr.rs1 as usize, val >> shamt);
        }
        Rv32Kind::CSrai => {
            let val = core.get_reg(instr.rs1 as usize) as i32;
            let shamt = (instr.imm as u32) & 0x1F;
            core.set_reg(instr.rs1 as usize, (val >> shamt) as u32);
        }
        Rv32Kind::CAndi => {
            let val = core.get_reg(instr.rs1 as usize);
            core.set_reg(instr.rs1 as usize, val & (instr.imm as u32));
        }
        Rv32Kind::CSub => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rs1 as usize, a.wrapping_sub(b));
        }
        Rv32Kind::CXor => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rs1 as usize, a ^ b);
        }
        Rv32Kind::COr => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rs1 as usize, a | b);
        }
        Rv32Kind::CAnd => {
            let a = core.get_reg(instr.rs1 as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rs1 as usize, a & b);
        }
        Rv32Kind::CJ => {
            core.pc = core.pc.wrapping_add(instr.imm as u32);
            cycles = 2;
        }
        Rv32Kind::CBeqz => {
            let val = core.get_reg(instr.rs1 as usize);
            if val == 0 {
                core.pc = core.pc.wrapping_add(instr.imm as u32);
                cycles = 2;
            }
        }
        Rv32Kind::CBnez => {
            let val = core.get_reg(instr.rs1 as usize);
            if val != 0 {
                core.pc = core.pc.wrapping_add(instr.imm as u32);
                cycles = 2;
            }
        }
        Rv32Kind::CSlli => {
            let val = core.get_reg(instr.rd as usize);
            let shamt = (instr.imm as u32) & 0x1F;
            core.set_reg(instr.rd as usize, val << shamt);
        }
        Rv32Kind::CLwsp => {
            let addr = core.get_reg(2).wrapping_add(instr.imm as u32);
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        Rv32Kind::CJr => {
            core.pc = core.get_reg(instr.rs1 as usize) & !1;
            cycles = 2;
        }
        Rv32Kind::CMv => {
            core.set_reg(instr.rd as usize, core.get_reg(instr.rs2 as usize));
        }
        Rv32Kind::CEbreak => {}
        Rv32Kind::CJalr => {
            let target = core.get_reg(instr.rs1 as usize) & !1;
            core.set_reg(1, core.pc + 2);
            core.pc = target;
            cycles = 2;
        }
        Rv32Kind::CAdd => {
            let a = core.get_reg(instr.rd as usize);
            let b = core.get_reg(instr.rs2 as usize);
            core.set_reg(instr.rd as usize, a.wrapping_add(b));
        }
        Rv32Kind::CSwsp => {
            let addr = core.get_reg(2).wrapping_add(instr.imm as u32);
            let value = core.get_reg(instr.rs2 as usize);
            memory.write_word(addr, value)?;
            cycles = 2;
        }

        // Unknown instruction - should not happen with complete decoder
        Rv32Kind::Unknown => {
            // Invalid instruction - could raise exception in real hardware
            // For now, treat as NOP to allow simulation to continue
        }
    }

    // Advance PC for non-branch, non-jump instructions
    // Branch and jump instructions update PC directly
    if !instr.is_branch && !matches!(instr.kind, Rv32Kind::Jal | Rv32Kind::Jalr) {
        core.pc += instr.length as u32;
    }

    core.cycles += cycles as u64;
    core.instructions += 1;

    Ok(cycles)
}

/// Memory access trait.
pub trait MemoryAccess {
    fn read_byte(&self, addr: u32) -> Result<u8>;
    fn read_half(&self, addr: u32) -> Result<u16>;
    fn read_word(&self, addr: u32) -> Result<u32>;
    fn write_byte(&mut self, addr: u32, value: u8) -> Result<()>;
    fn write_half(&mut self, addr: u32, value: u16) -> Result<()>;
    fn write_word(&mut self, addr: u32, value: u32) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Hazard3CoreState;
    use rp2350sim_core::CoreId;

    /// Mock memory for testing
    struct MockMemory {
        data: [u8; 1024],
    }

    impl MockMemory {
        fn new() -> Self {
            Self { data: [0; 1024] }
        }
    }

    impl MemoryAccess for MockMemory {
        fn read_byte(&self, addr: u32) -> Result<u8> {
            Ok(self.data[addr as usize % 1024])
        }
        fn read_half(&self, addr: u32) -> Result<u16> {
            let lo = self.data[addr as usize % 1024] as u16;
            let hi = self.data[(addr as usize + 1) % 1024] as u16;
            Ok(lo | (hi << 8))
        }
        fn read_word(&self, addr: u32) -> Result<u32> {
            let b0 = self.data[addr as usize % 1024] as u32;
            let b1 = self.data[(addr as usize + 1) % 1024] as u32;
            let b2 = self.data[(addr as usize + 2) % 1024] as u32;
            let b3 = self.data[(addr as usize + 3) % 1024] as u32;
            Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
        }
        fn write_byte(&mut self, addr: u32, value: u8) -> Result<()> {
            self.data[addr as usize % 1024] = value;
            Ok(())
        }
        fn write_half(&mut self, addr: u32, value: u16) -> Result<()> {
            self.data[addr as usize % 1024] = (value & 0xFF) as u8;
            self.data[(addr as usize + 1) % 1024] = ((value >> 8) & 0xFF) as u8;
            Ok(())
        }
        fn write_word(&mut self, addr: u32, value: u32) -> Result<()> {
            self.data[addr as usize % 1024] = (value & 0xFF) as u8;
            self.data[(addr as usize + 1) % 1024] = ((value >> 8) & 0xFF) as u8;
            self.data[(addr as usize + 2) % 1024] = ((value >> 16) & 0xFF) as u8;
            self.data[(addr as usize + 3) % 1024] = ((value >> 24) & 0xFF) as u8;
            Ok(())
        }
    }

    fn create_core() -> Hazard3CoreState {
        Hazard3CoreState::new(CoreId::CORE0)
    }

    #[test]
    fn test_lui() {
        let mut core = create_core();
        let instr = Rv32Instruction {
            kind: Rv32Kind::Lui,
            rd: 5,
            imm: 0x12345000,  // LUI immediate is already shifted
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(5), 0x12345000);
    }

    #[test]
    fn test_auipc() {
        let mut core = create_core();
        core.pc = 0x1000;
        let instr = Rv32Instruction {
            kind: Rv32Kind::Auipc,
            rd: 5,
            imm: 0x1000,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(5), 0x2000);
    }

    #[test]
    fn test_addi() {
        let mut core = create_core();
        core.set_reg(1, 100);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Addi,
            rd: 2,
            rs1: 1,
            imm: 50,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(2), 150);
    }

    #[test]
    fn test_add() {
        let mut core = create_core();
        core.set_reg(1, 100);
        core.set_reg(2, 50);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Add,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 150);
    }

    #[test]
    fn test_sub() {
        let mut core = create_core();
        core.set_reg(1, 100);
        core.set_reg(2, 30);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Sub,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 70);
    }

    #[test]
    fn test_and() {
        let mut core = create_core();
        core.set_reg(1, 0xFF);
        core.set_reg(2, 0x0F);
        let instr = Rv32Instruction {
            kind: Rv32Kind::And,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 0x0F);
    }

    #[test]
    fn test_or() {
        let mut core = create_core();
        core.set_reg(1, 0xF0);
        core.set_reg(2, 0x0F);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Or,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 0xFF);
    }

    #[test]
    fn test_xor() {
        let mut core = create_core();
        core.set_reg(1, 0xFF);
        core.set_reg(2, 0x0F);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Xor,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 0xF0);
    }

    #[test]
    fn test_slli() {
        let mut core = create_core();
        core.set_reg(1, 1);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Slli,
            rd: 2,
            rs1: 1,
            imm: 4,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(2), 16);
    }

    #[test]
    fn test_srli() {
        let mut core = create_core();
        core.set_reg(1, 16);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Srli,
            rd: 2,
            rs1: 1,
            imm: 4,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(2), 1);
    }

    #[test]
    fn test_srai() {
        let mut core = create_core();
        core.set_reg(1, (-16i32) as u32);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Srai,
            rd: 2,
            rs1: 1,
            imm: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(2) as i32, -4);
    }

    #[test]
    fn test_slt() {
        let mut core = create_core();
        core.set_reg(1, 5);
        core.set_reg(2, 10);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Slt,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 1);

        core.set_reg(1, 10);
        core.set_reg(2, 5);
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 0);
    }

    #[test]
    fn test_sltu() {
        let mut core = create_core();
        core.set_reg(1, 5);
        core.set_reg(2, 10);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Sltu,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 1);
    }

    #[test]
    fn test_beq_taken() {
        let mut core = create_core();
        core.pc = 0x1000;
        core.set_reg(1, 5);
        core.set_reg(2, 5);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Beq,
            rs1: 1,
            rs2: 2,
            imm: 16,
            is_branch: true,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.pc, 0x1010);
    }

    #[test]
    fn test_beq_not_taken() {
        let mut core = create_core();
        core.pc = 0x1000;
        core.set_reg(1, 5);
        core.set_reg(2, 3);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Beq,
            rs1: 1,
            rs2: 2,
            imm: 16,
            is_branch: true,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        // PC advances by instruction length
        assert_eq!(core.pc, 0x1004);
    }

    #[test]
    fn test_bne_taken() {
        let mut core = create_core();
        core.pc = 0x1000;
        core.set_reg(1, 5);
        core.set_reg(2, 3);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Bne,
            rs1: 1,
            rs2: 2,
            imm: 16,
            is_branch: true,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.pc, 0x1010);
    }

    #[test]
    fn test_blt_signed() {
        let mut core = create_core();
        core.pc = 0x1000;
        core.set_reg(1, (-5i32) as u32);
        core.set_reg(2, 5);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Blt,
            rs1: 1,
            rs2: 2,
            imm: 16,
            is_branch: true,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.pc, 0x1010);
    }

    #[test]
    fn test_jal() {
        let mut core = create_core();
        core.pc = 0x1000;
        let instr = Rv32Instruction {
            kind: Rv32Kind::Jal,
            rd: 1,
            imm: 0x100,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.pc, 0x1100);
        assert_eq!(core.get_reg(1), 0x1004);
    }

    #[test]
    fn test_jalr() {
        let mut core = create_core();
        core.pc = 0x1000;
        core.set_reg(1, 0x2000);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Jalr,
            rd: 2,
            rs1: 1,
            imm: 0x10,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.pc, 0x2010);
        assert_eq!(core.get_reg(2), 0x1004);
    }

    #[test]
    fn test_load_store_word() {
        let mut core = create_core();
        let mut mem = MockMemory::new();

        // Store word
        core.set_reg(1, 0x100);  // Address
        core.set_reg(2, 0xDEADBEEF);  // Value
        let store_instr = Rv32Instruction {
            kind: Rv32Kind::Sw,
            rs1: 1,
            rs2: 2,
            imm: 0,
            ..Default::default()
        };
        execute(&mut core, &store_instr, &mut mem).unwrap();

        // Load word
        let load_instr = Rv32Instruction {
            kind: Rv32Kind::Lw,
            rd: 3,
            rs1: 1,
            imm: 0,
            ..Default::default()
        };
        execute(&mut core, &load_instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 0xDEADBEEF);
    }

    #[test]
    fn test_mul() {
        let mut core = create_core();
        core.set_reg(1, 100);
        core.set_reg(2, 7);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Mul,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 700);
    }

    #[test]
    fn test_divu() {
        let mut core = create_core();
        core.set_reg(1, 100);
        core.set_reg(2, 7);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Divu,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 14);
    }

    #[test]
    fn test_divu_by_zero() {
        let mut core = create_core();
        core.set_reg(1, 100);
        core.set_reg(2, 0);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Divu,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 0xFFFFFFFF);
    }

    #[test]
    fn test_div() {
        let mut core = create_core();
        core.set_reg(1, (-100i32) as u32);
        core.set_reg(2, 7);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Div,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3) as i32, -14);
    }

    #[test]
    fn test_remu() {
        let mut core = create_core();
        core.set_reg(1, 100);
        core.set_reg(2, 7);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Remu,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 2);  // 100 % 7 = 2
    }

    #[test]
    fn test_slti() {
        let mut core = create_core();
        core.set_reg(1, 5);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Slti,
            rd: 2,
            rs1: 1,
            imm: 10,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(2), 1);  // 5 < 10
    }

    #[test]
    fn test_sll() {
        let mut core = create_core();
        core.set_reg(1, 0x00000001);
        core.set_reg(2, 4);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Sll,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 0x00000010);
    }

    #[test]
    fn test_srl() {
        let mut core = create_core();
        core.set_reg(1, 0x00000010);
        core.set_reg(2, 4);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Srl,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3), 0x00000001);
    }

    #[test]
    fn test_sra() {
        let mut core = create_core();
        core.set_reg(1, (-16i32) as u32);  // -16
        core.set_reg(2, 2);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Sra,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(3) as i32, -4);  // -16 >> 2 = -4 (arithmetic)
    }

    #[test]
    fn test_mulh() {
        let mut core = create_core();
        core.set_reg(1, 0x10000);
        core.set_reg(2, 0x10000);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Mulh,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        // 0x10000 * 0x10000 = 0x100000000, high 32 bits = 1
        assert_eq!(core.get_reg(3), 1);
    }

    #[test]
    fn test_rem() {
        let mut core = create_core();
        core.set_reg(1, (-100i32) as u32);
        core.set_reg(2, 7);
        let instr = Rv32Instruction {
            kind: Rv32Kind::Rem,
            rd: 3,
            rs1: 1,
            rs2: 2,
            ..Default::default()
        };
        let mut mem = MockMemory::new();
        execute(&mut core, &instr, &mut mem).unwrap();
        // -100 % 7 = -2 (RISC-V uses truncated division)
        assert_eq!(core.get_reg(3) as i32, -2);
    }

    #[test]
    fn test_lb_sign_extend() {
        let mut core = create_core();
        core.set_reg(1, 0x1000);
        let mut mem = MockMemory::new();
        mem.write_byte(0x1000, 0x80).unwrap();  // -128 as signed byte

        let instr = Rv32Instruction {
            kind: Rv32Kind::Lb,
            rd: 2,
            rs1: 1,
            imm: 0,
            ..Default::default()
        };
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(2) as i32, -128);  // Sign extended
    }

    #[test]
    fn test_lbu_zero_extend() {
        let mut core = create_core();
        core.set_reg(1, 0x1000);
        let mut mem = MockMemory::new();
        mem.write_byte(0x1000, 0x80).unwrap();

        let instr = Rv32Instruction {
            kind: Rv32Kind::Lbu,
            rd: 2,
            rs1: 1,
            imm: 0,
            ..Default::default()
        };
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(2), 0x80);  // Zero extended
    }

    #[test]
    fn test_lh_sign_extend() {
        let mut core = create_core();
        core.set_reg(1, 0x1000);
        let mut mem = MockMemory::new();
        mem.write_half(0x1000, 0x8000).unwrap();  // -32768 as signed halfword

        let instr = Rv32Instruction {
            kind: Rv32Kind::Lh,
            rd: 2,
            rs1: 1,
            imm: 0,
            ..Default::default()
        };
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(core.get_reg(2) as i32, -32768);  // Sign extended
    }

    #[test]
    fn test_sb() {
        let mut core = create_core();
        core.set_reg(1, 0x1000);
        core.set_reg(2, 0xAB);
        let mut mem = MockMemory::new();

        let instr = Rv32Instruction {
            kind: Rv32Kind::Sb,
            rs1: 1,
            rs2: 2,
            imm: 0,
            ..Default::default()
        };
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(mem.read_byte(0x1000).unwrap(), 0xAB);
    }

    #[test]
    fn test_sh() {
        let mut core = create_core();
        core.set_reg(1, 0x1000);
        core.set_reg(2, 0x1234);
        let mut mem = MockMemory::new();

        let instr = Rv32Instruction {
            kind: Rv32Kind::Sh,
            rs1: 1,
            rs2: 2,
            imm: 0,
            ..Default::default()
        };
        execute(&mut core, &instr, &mut mem).unwrap();
        assert_eq!(mem.read_half(0x1000).unwrap(), 0x1234);
    }
}
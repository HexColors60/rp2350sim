//! ARM Thumb instruction executor.

use crate::state::ArmCoreState;
use rp2350sim_core::Result;

use super::decode::{Condition, ThumbInstruction, ThumbKind};

/// Execute a Thumb instruction.
pub fn execute(
    core: &mut ArmCoreState,
    instr: &ThumbInstruction,
    memory: &mut impl MemoryAccess,
) -> Result<u8> {
    let mut cycles = 1u8;

    // Check if we're in an IT block
    let in_it_block = core.is_it_block_active();

    // Check condition for IT block (but NOT for BCond - it handles its own condition)
    let should_execute = if in_it_block && instr.kind != ThumbKind::It {
        // Get the condition from IT block state
        let (cond, _) = core.get_it_condition();
        let result = check_condition(core, cond);
        // Advance IT state after checking condition
        core.advance_it_state();
        result
    } else {
        true
    };

    if !should_execute {
        return Ok(cycles);
    }

    let mut branch_taken = false;

    match instr.kind {
        // Shift instructions
        ThumbKind::Lsl => {
            let value = core.get_reg(instr.rm as usize);
            let result = if instr.shift == 0 {
                value
            } else {
                value << instr.shift
            };
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(
                core,
                result,
                instr.shift != 0 && (value >> (32 - instr.shift)) & 1 != 0,
            );
        }
        ThumbKind::Lsr => {
            let value = core.get_reg(instr.rm as usize);
            let result = if instr.shift == 0 {
                0
            } else {
                value >> instr.shift
            };
            let carry = instr.shift != 0 && (value >> (instr.shift - 1)) & 1 != 0;
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, carry);
        }
        ThumbKind::Asr => {
            let value = core.get_reg(instr.rm as usize) as i32;
            let result = if instr.shift == 0 {
                if value < 0 {
                    0xFFFFFFFF
                } else {
                    0
                }
            } else {
                (value >> instr.shift) as u32
            };
            let carry = instr.shift != 0 && ((value as u32) >> (instr.shift - 1)) & 1 != 0;
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, carry);
        }

        // Add instructions
        ThumbKind::AddReg => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let (result, carry, overflow) = add_with_carry(a, b, false);
            core.set_reg(instr.rd as usize, result);
            update_flags_arithmetic(core, result, carry, overflow);
        }
        ThumbKind::AddImm3 | ThumbKind::AddImm8 => {
            let a = core.get_reg(instr.rn as usize);
            let b = instr.imm;
            let (result, carry, overflow) = add_with_carry(a, b, false);
            core.set_reg(instr.rd as usize, result);
            update_flags_arithmetic(core, result, carry, overflow);
        }

        // Subtract instructions
        ThumbKind::SubReg => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let (result, carry, overflow) = sub_with_carry(a, b, true);
            core.set_reg(instr.rd as usize, result);
            update_flags_arithmetic(core, result, carry, overflow);
        }
        ThumbKind::SubImm3 | ThumbKind::SubImm8 => {
            let a = core.get_reg(instr.rn as usize);
            let b = instr.imm;
            let (result, carry, overflow) = sub_with_carry(a, b, true);
            core.set_reg(instr.rd as usize, result);
            update_flags_arithmetic(core, result, carry, overflow);
        }

        // Move instructions
        ThumbKind::MovImm => {
            core.set_reg(instr.rd as usize, instr.imm);
            update_flags_logical(core, instr.imm, false);
        }

        // Compare instructions
        ThumbKind::CmpImm | ThumbKind::Cmp => {
            let a = core.get_reg(instr.rd as usize);
            let b = if instr.kind == ThumbKind::CmpImm {
                instr.imm
            } else {
                core.get_reg(instr.rm as usize)
            };
            let (result, carry, overflow) = sub_with_carry(a, b, true);
            update_flags_arithmetic(core, result, carry, overflow);
        }
        ThumbKind::Cmn => {
            let a = core.get_reg(instr.rd as usize);
            let b = core.get_reg(instr.rm as usize);
            let (result, carry, overflow) = add_with_carry(a, b, false);
            update_flags_arithmetic(core, result, carry, overflow);
        }

        // Logical instructions
        ThumbKind::And => {
            let result = core.get_reg(instr.rd as usize) & core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, false);
        }
        ThumbKind::Eor => {
            let result = core.get_reg(instr.rd as usize) ^ core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, false);
        }
        ThumbKind::Orr => {
            let result = core.get_reg(instr.rd as usize) | core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, false);
        }
        ThumbKind::Orn => {
            let result = core.get_reg(instr.rd as usize) | !core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, false);
        }

        ThumbKind::Bic => {
            let result = core.get_reg(instr.rd as usize) & !core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, false);
        }
        ThumbKind::Mvn => {
            let result = !core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, false);
        }
        ThumbKind::Tst => {
            let result = core.get_reg(instr.rd as usize) & core.get_reg(instr.rm as usize);
            update_flags_logical(core, result, false);
        }

        // Arithmetic with carry
        ThumbKind::Adc => {
            let a = core.get_reg(instr.rd as usize);
            let b = core.get_reg(instr.rm as usize);
            let c = core.xpsr.c() as u32;
            let (result, carry, overflow) = add_with_carry(a, b, c != 0);
            core.set_reg(instr.rd as usize, result);
            update_flags_arithmetic(core, result, carry, overflow);
        }
        ThumbKind::Sbc => {
            let a = core.get_reg(instr.rd as usize);
            let b = core.get_reg(instr.rm as usize);
            let c = core.xpsr.c() as u32;
            let (result, carry, overflow) = sub_with_carry(a, b, c != 0);
            core.set_reg(instr.rd as usize, result);
            update_flags_arithmetic(core, result, carry, overflow);
        }

        // Rotate
        ThumbKind::Ror => {
            let value = core.get_reg(instr.rd as usize);
            let shift = (core.get_reg(instr.rm as usize) & 0xFF) as u32;
            let shift = shift & 0x1F;
            let result = if shift == 0 {
                value
            } else {
                (value >> shift) | (value << (32 - shift))
            };
            let carry = if shift == 0 {
                false
            } else {
                (value >> (shift - 1)) & 1 != 0
            };
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, carry);
        }

        // Negate
        ThumbKind::Neg => {
            let value = core.get_reg(instr.rm as usize);
            let (result, carry, overflow) = sub_with_carry(0, value, true);
            core.set_reg(instr.rd as usize, result);
            update_flags_arithmetic(core, result, carry, overflow);
        }

        // Multiply
        ThumbKind::Mul => {
            let a = core.get_reg(instr.rd as usize);
            let b = core.get_reg(instr.rm as usize);
            let result = a.wrapping_mul(b);
            core.set_reg(instr.rd as usize, result);
            cycles = 3; // Multiply takes more cycles
        }

        // Register-based shift instructions
        // ARM architecture: shift amount is bottom byte, with special cases for >= 32
        ThumbKind::LslReg => {
            let value = core.get_reg(instr.rd as usize);
            let shift = (core.get_reg(instr.rm as usize) & 0xFF) as u32;
            let (result, carry) = if shift == 0 {
                (value, false)
            } else if shift < 32 {
                (value << shift, (value >> (32 - shift)) & 1 != 0)
            } else if shift == 32 {
                (0, value & 1 != 0)
            } else {
                (0, false)
            };
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, carry);
        }
        ThumbKind::LsrReg => {
            let value = core.get_reg(instr.rd as usize);
            let shift = (core.get_reg(instr.rm as usize) & 0xFF) as u32;
            let (result, carry) = if shift == 0 {
                (value, false)
            } else if shift < 32 {
                (value >> shift, (value >> (shift - 1)) & 1 != 0)
            } else if shift == 32 {
                (0, (value >> 31) & 1 != 0)
            } else {
                (0, false)
            };
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, carry);
        }
        ThumbKind::AsrReg => {
            let value = core.get_reg(instr.rd as usize);
            let shift = (core.get_reg(instr.rm as usize) & 0xFF) as u32;
            let (result, carry) = if shift == 0 {
                (value, false)
            } else if shift < 32 {
                let signed_val = value as i32;
                ((signed_val >> shift) as u32, (value >> (shift - 1)) & 1 != 0)
            } else {
                // shift >= 32: fill with sign bit
                let sign_bit = (value >> 31) & 1;
                let result = if sign_bit != 0 { 0xFFFFFFFFu32 } else { 0u32 };
                (result, sign_bit != 0)
            };
            core.set_reg(instr.rd as usize, result);
            update_flags_logical(core, result, carry);
        }

        // Long multiply with accumulate
        ThumbKind::Umlal => {
            let a = core.get_reg(instr.rn as usize) as u64;
            let b = core.get_reg(instr.rm as usize) as u64;
            let acc_lo = core.get_reg(instr.rd as usize) as u64;
            let acc_hi = core.get_reg(instr.rd as usize + 1) as u64;
            let acc = (acc_hi << 32) | acc_lo;
            let result = a.wrapping_mul(b).wrapping_add(acc);
            core.set_reg(instr.rd as usize, result as u32);
            core.set_reg(instr.rd as usize + 1, (result >> 32) as u32);
            cycles = 4;
        }
        ThumbKind::Smlal => {
            let a = core.get_reg(instr.rn as usize) as i32 as i64;
            let b = core.get_reg(instr.rm as usize) as i32 as i64;
            let acc_lo = core.get_reg(instr.rd as usize) as u64;
            let acc_hi = core.get_reg(instr.rd as usize + 1) as u64;
            let acc = ((acc_hi << 32) | acc_lo) as i64;
            let result = a.wrapping_mul(b).wrapping_add(acc) as u64;
            core.set_reg(instr.rd as usize, result as u32);
            core.set_reg(instr.rd as usize + 1, (result >> 32) as u32);
            cycles = 4;
        }

        // DSP multiply instructions
        ThumbKind::Smlad => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let acc = core.get_reg(instr.rd as usize) as i32;
            let a_lo = (a & 0xFFFF) as i16 as i32;
            let a_hi = ((a >> 16) & 0xFFFF) as i16 as i32;
            let b_lo = (b & 0xFFFF) as i16 as i32;
            let b_hi = ((b >> 16) & 0xFFFF) as i16 as i32;
            let result = a_lo
                .wrapping_mul(b_lo)
                .wrapping_add(a_hi.wrapping_mul(b_hi))
                .wrapping_add(acc);
            core.set_reg(instr.rd as usize, result as u32);
            cycles = 2;
        }
        ThumbKind::Smlsd => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let acc = core.get_reg(instr.rd as usize) as i32;
            let a_lo = (a & 0xFFFF) as i16 as i32;
            let a_hi = ((a >> 16) & 0xFFFF) as i16 as i32;
            let b_lo = (b & 0xFFFF) as i16 as i32;
            let b_hi = ((b >> 16) & 0xFFFF) as i16 as i32;
            let result = a_lo
                .wrapping_mul(b_lo)
                .wrapping_sub(a_hi.wrapping_mul(b_hi))
                .wrapping_add(acc);
            core.set_reg(instr.rd as usize, result as u32);
            cycles = 2;
        }
        ThumbKind::Smuad => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let a_lo = (a & 0xFFFF) as i16 as i32;
            let a_hi = ((a >> 16) & 0xFFFF) as i16 as i32;
            let b_lo = (b & 0xFFFF) as i16 as i32;
            let b_hi = ((b >> 16) & 0xFFFF) as i16 as i32;
            let result = a_lo
                .wrapping_mul(b_lo)
                .wrapping_add(a_hi.wrapping_mul(b_hi));
            core.set_reg(instr.rd as usize, result as u32);
            cycles = 2;
        }
        ThumbKind::Smusd => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let a_lo = (a & 0xFFFF) as i16 as i32;
            let a_hi = ((a >> 16) & 0xFFFF) as i16 as i32;
            let b_lo = (b & 0xFFFF) as i16 as i32;
            let b_hi = ((b >> 16) & 0xFFFF) as i16 as i32;
            let result = a_lo
                .wrapping_mul(b_lo)
                .wrapping_sub(a_hi.wrapping_mul(b_hi));
            core.set_reg(instr.rd as usize, result as u32);
            cycles = 2;
        }
        ThumbKind::Smlald => {
            // Signed multiply accumulate dual (64-bit result)
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let acc_lo = core.get_reg(instr.rd as usize) as u64;
            let acc_hi = core.get_reg(instr.rd as usize + 1) as u64;
            let acc = ((acc_hi << 32) | acc_lo) as i64;
            let a_lo = (a & 0xFFFF) as i16 as i64;
            let a_hi = ((a >> 16) & 0xFFFF) as i16 as i64;
            let b_lo = (b & 0xFFFF) as i16 as i64;
            let b_hi = ((b >> 16) & 0xFFFF) as i16 as i64;
            let result = a_lo * b_lo + a_hi * b_hi + acc;
            core.set_reg(instr.rd as usize, result as u32);
            core.set_reg(instr.rd as usize + 1, (result >> 32) as u32);
            cycles = 2;
        }
        ThumbKind::Smlsld => {
            // Signed multiply subtract dual (64-bit result)
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let acc_lo = core.get_reg(instr.rd as usize) as u64;
            let acc_hi = core.get_reg(instr.rd as usize + 1) as u64;
            let acc = ((acc_hi << 32) | acc_lo) as i64;
            let a_lo = (a & 0xFFFF) as i16 as i64;
            let a_hi = ((a >> 16) & 0xFFFF) as i16 as i64;
            let b_lo = (b & 0xFFFF) as i16 as i64;
            let b_hi = ((b >> 16) & 0xFFFF) as i16 as i64;
            let result = a_lo * b_lo - a_hi * b_hi + acc;
            core.set_reg(instr.rd as usize, result as u32);
            core.set_reg(instr.rd as usize + 1, (result >> 32) as u32);
            cycles = 2;
        }
        ThumbKind::Smmla => {
            // Signed most significant word multiply accumulate
            let a = core.get_reg(instr.rn as usize) as i32 as i64;
            let b = core.get_reg(instr.rm as usize) as i32 as i64;
            let acc = core.get_reg(instr.rd as usize) as i32 as i64;
            let result = ((a * b + (acc << 32) + (1i64 << 31)) >> 32) as i32;
            core.set_reg(instr.rd as usize, result as u32);
            cycles = 2;
        }
        ThumbKind::Smmls => {
            // Signed most significant word multiply subtract
            let a = core.get_reg(instr.rn as usize) as i32 as i64;
            let b = core.get_reg(instr.rm as usize) as i32 as i64;
            let acc = core.get_reg(instr.rd as usize) as i32 as i64;
            let result = ((a * b - (acc << 32) + (1i64 << 31)) >> 32) as i32;
            core.set_reg(instr.rd as usize, result as u32);
            cycles = 2;
        }
        ThumbKind::Smmul => {
            // Signed most significant word multiply
            let a = core.get_reg(instr.rn as usize) as i32 as i64;
            let b = core.get_reg(instr.rm as usize) as i32 as i64;
            let result = ((a * b + (1i64 << 31)) >> 32) as i32;
            core.set_reg(instr.rd as usize, result as u32);
            cycles = 2;
        }


        // Load instructions
        ThumbKind::LdrLit => {
            let addr = ((core.pc + 4) & !3) + instr.imm;
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::LdrSp => {
            let addr = core.msp + instr.imm;
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }

        // Store instructions
        ThumbKind::StrSp => {
            let addr = core.msp + instr.imm;
            let value = core.get_reg(instr.rd as usize);
            memory.write_word(addr, value)?;
            cycles = 2;
        }

        // Push/Pop
        ThumbKind::Push => {
            let mut sp = core.msp;
            for i in (0..16).rev() {
                if (instr.imm >> i) & 1 != 0 {
                    sp -= 4;
                    memory.write_word(sp, core.get_reg(i))?;
                }
            }
            core.msp = sp;
            cycles = 2;
        }
        ThumbKind::Pop => {
            let mut sp = core.msp;
            for i in 0..16 {
                if (instr.imm >> i) & 1 != 0 {
                    let value = memory.read_word(sp)?;
                    sp += 4;
                    if i == 15 {
                        core.pc = value & !1;
                        core.xpsr.set_t(value & 1 != 0);
                    } else {
                        core.set_reg(i, value);
                    }
                }
            }
            core.msp = sp;
            cycles = 2;
        }

        // Branch instructions
        ThumbKind::B => {
            core.pc = (core.pc as i32 + 4 + instr.imm as i32) as u32;
            branch_taken = true;
            cycles = 2;
        }
        ThumbKind::BCond => {
            if check_condition(core, instr.cond) {
                core.pc = (core.pc as i32 + 4 + instr.imm as i32) as u32;
                branch_taken = true;
                cycles = 2;
            }
        }
        ThumbKind::Cbz => {
            if core.get_reg(instr.rn as usize) == 0 {
                core.pc = (core.pc as i32 + 4 + instr.imm as i32) as u32;
                branch_taken = true;
                cycles = 2;
            }
        }
        ThumbKind::Cbnz => {
            if core.get_reg(instr.rn as usize) != 0 {
                core.pc = (core.pc as i32 + 4 + instr.imm as i32) as u32;
                branch_taken = true;
                cycles = 2;
            }
        }

        // IT block (If-Then)
        ThumbKind::It => {
            let firstcond = instr.cond;
            let mask = instr.imm as u8;
            core.set_it_state(firstcond, mask);
        }

        // NOP and other hints
        ThumbKind::Nop => {}
        ThumbKind::Wfi | ThumbKind::Wfe | ThumbKind::Sev | ThumbKind::Yield => {}

        // Branch with link
        ThumbKind::Bl => {
            let next_pc = core.pc + 4;
            core.lr = next_pc | 1; // Set Thumb bit
            core.pc = (core.pc as i32 + 4 + instr.imm as i32) as u32;
            branch_taken = true;
            cycles = 2;
        }
        ThumbKind::BlxReg => {
            let target = core.get_reg(instr.rm as usize);
            let next_pc = core.pc + 2;
            core.lr = next_pc | 1;
            core.pc = target & !1;
            core.xpsr.set_t(target & 1 != 0);
            branch_taken = true;
            cycles = 2;
        }

        // Branch and exchange
        ThumbKind::Bx => {
            let target = core.get_reg(instr.rm as usize);
            core.pc = target & !1;
            core.xpsr.set_t(target & 1 != 0);
            branch_taken = true;
            cycles = 2;
        }
        ThumbKind::Blx => {
            // 16-bit BLX - branch with link and exchange
            let target = core.get_reg(instr.rm as usize);
            let next_pc = core.pc + 2;
            core.lr = next_pc | 1;
            core.pc = target & !1;
            core.xpsr.set_t(target & 1 != 0);
            branch_taken = true;
            cycles = 2;
        }
        ThumbKind::Setend => {
            // SETEND - set endianness (rarely used)
            // Would need to track endianness state for full support
        }
        ThumbKind::Cps => {
            // CPS - change processor state (enable/disable interrupts)
            // For simulation purposes, treat as NOP
        }


        // Load/store register offset
        ThumbKind::LdrReg => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::LdrbReg => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = memory.read_byte(addr)? as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::LdrhReg => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = memory.read_half(addr)? as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::LdrsbReg => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = memory.read_byte(addr)? as i8 as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::LdrshReg => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = memory.read_half(addr)? as i16 as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::StrReg => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = core.get_reg(instr.rd as usize);
            memory.write_word(addr, value)?;
            cycles = 2;
        }
        ThumbKind::StrbReg => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = core.get_reg(instr.rd as usize) as u8;
            memory.write_byte(addr, value)?;
            cycles = 2;
        }
        ThumbKind::StrhReg => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = core.get_reg(instr.rd as usize) as u16;
            memory.write_half(addr, value)?;
            cycles = 2;
        }

        // Load/store immediate
        ThumbKind::LdrImm5 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm * 4);
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::StrImm5 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm * 4);
            let value = core.get_reg(instr.rd as usize);
            memory.write_word(addr, value)?;
            cycles = 2;
        }
        ThumbKind::LdrbImm5 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = memory.read_byte(addr)? as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::StrbImm5 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = core.get_reg(instr.rd as usize) as u8;
            memory.write_byte(addr, value)?;
            cycles = 2;
        }
        ThumbKind::LdrhImm5 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm * 2);
            let value = memory.read_half(addr)? as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::StrhImm5 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm * 2);
            let value = core.get_reg(instr.rd as usize) as u16;
            memory.write_half(addr, value)?;
            cycles = 2;
        }

        // Address calculation
        ThumbKind::Adr => {
            let addr = (core.pc & !3) + instr.imm * 4;
            core.set_reg(instr.rd as usize, addr);
        }
        ThumbKind::AddSpImm => {
            let sp = core.msp;
            core.set_reg(instr.rd as usize, sp.wrapping_add(instr.imm * 4));
        }
        ThumbKind::AddSpReg => {
            let sp = core.msp;
            core.msp = sp.wrapping_add(instr.imm * 4);
        }

        // Sign extend
        ThumbKind::Sxth => {
            let value = core.get_reg(instr.rm as usize) as i16 as u32;
            core.set_reg(instr.rd as usize, value);
        }
        ThumbKind::Sxtb => {
            let value = core.get_reg(instr.rm as usize) as i8 as u32;
            core.set_reg(instr.rd as usize, value);
        }
        ThumbKind::Uxth => {
            let value = core.get_reg(instr.rm as usize) & 0xFFFF;
            core.set_reg(instr.rd as usize, value);
        }
        ThumbKind::Uxtb => {
            let value = core.get_reg(instr.rm as usize) & 0xFF;
            core.set_reg(instr.rd as usize, value);
        }

        // High register operations
        ThumbKind::AddHigh => {
            let a = core.get_reg(instr.rd as usize);
            let b = core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, a.wrapping_add(b));
        }
        ThumbKind::CmpHigh => {
            let a = core.get_reg(instr.rd as usize);
            let b = core.get_reg(instr.rm as usize);
            let (result, carry, overflow) = sub_with_carry(a, b, false);
            update_flags_arithmetic(core, result, carry, overflow);
        }
        ThumbKind::MovHigh => {
            core.set_reg(instr.rd as usize, core.get_reg(instr.rm as usize));
        }

        // Long multiply
        ThumbKind::Umull => {
            let a = core.get_reg(instr.rn as usize) as u64;
            let b = core.get_reg(instr.rm as usize) as u64;
            let result = a * b;
            core.set_reg(instr.rd as usize, result as u32);
            core.set_reg(instr.rd as usize + 1, (result >> 32) as u32);
            cycles = 4;
        }
        ThumbKind::Smull => {
            let a = core.get_reg(instr.rn as usize) as i32 as i64;
            let b = core.get_reg(instr.rm as usize) as i32 as i64;
            let result = a * b;
            core.set_reg(instr.rd as usize, result as u32);
            core.set_reg(instr.rd as usize + 1, (result >> 32) as u32);
            cycles = 4;
        }

        // Memory barriers
        ThumbKind::Dmb | ThumbKind::Dsb | ThumbKind::Isb => {}

        // ==================== 32-bit Thumb-2 instructions ====================

        // Data processing (32-bit) - immediate
        ThumbKind::MovImm32 => {
            core.set_reg(instr.rd as usize, instr.imm);
        }
        ThumbKind::MvnImm32 => {
            core.set_reg(instr.rd as usize, !instr.imm);
        }
        ThumbKind::AddImm32 => {
            let a = core.get_reg(instr.rn as usize);
            core.set_reg(instr.rd as usize, a.wrapping_add(instr.imm));
        }
        ThumbKind::SubImm32 => {
            let a = core.get_reg(instr.rn as usize);
            core.set_reg(instr.rd as usize, a.wrapping_sub(instr.imm));
        }
        ThumbKind::AndImm32 => {
            let a = core.get_reg(instr.rn as usize);
            core.set_reg(instr.rd as usize, a & instr.imm);
        }
        ThumbKind::OrrImm32 => {
            let a = core.get_reg(instr.rn as usize);
            core.set_reg(instr.rd as usize, a | instr.imm);
        }
        ThumbKind::EorImm32 => {
            let a = core.get_reg(instr.rn as usize);
            core.set_reg(instr.rd as usize, a ^ instr.imm);
        }
        ThumbKind::BicImm32 => {
            let a = core.get_reg(instr.rn as usize);
            core.set_reg(instr.rd as usize, a & !instr.imm);
        }

        // Data processing (32-bit) - register
        ThumbKind::MovReg32 => {
            core.set_reg(instr.rd as usize, core.get_reg(instr.rm as usize));
        }
        ThumbKind::MvnReg32 => {
            core.set_reg(instr.rd as usize, !core.get_reg(instr.rm as usize));
        }
        ThumbKind::AddReg32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, a.wrapping_add(b));
        }
        ThumbKind::SubReg32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, a.wrapping_sub(b));
        }
        ThumbKind::AndReg32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, a & b);
        }
        ThumbKind::OrrReg32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, a | b);
        }
        ThumbKind::EorReg32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, a ^ b);
        }
        ThumbKind::BicReg32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            core.set_reg(instr.rd as usize, a & !b);
        }

        // Shift operations (32-bit)
        ThumbKind::LslImm32 => {
            let a = core.get_reg(instr.rm as usize);
            let shift = instr.imm as u32 & 0x1F;
            core.set_reg(instr.rd as usize, a << shift);
        }
        ThumbKind::LsrImm32 => {
            let a = core.get_reg(instr.rm as usize);
            let shift = instr.imm as u32 & 0x1F;
            core.set_reg(instr.rd as usize, if shift == 0 { 0 } else { a >> shift });
        }
        ThumbKind::AsrImm32 => {
            let a = core.get_reg(instr.rm as usize) as i32;
            let shift = instr.imm as u32 & 0x1F;
            core.set_reg(
                instr.rd as usize,
                if shift == 0 {
                    if a < 0 {
                        0xFFFFFFFF
                    } else {
                        0
                    }
                } else {
                    (a >> shift) as u32
                },
            );
        }
        ThumbKind::RorImm32 => {
            let value = core.get_reg(instr.rm as usize);
            let shift = instr.imm as u32 & 0x1F;
            let result = if shift == 0 {
                value
            } else {
                (value >> shift) | (value << (32 - shift))
            };
            core.set_reg(instr.rd as usize, result);
        }

        // Load/store (32-bit)
        ThumbKind::LdrImm32 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::StrImm32 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = core.get_reg(instr.rd as usize);
            memory.write_word(addr, value)?;
            cycles = 2;
        }
        ThumbKind::LdrbImm32 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = memory.read_byte(addr)? as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::StrbImm32 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = core.get_reg(instr.rd as usize) as u8;
            memory.write_byte(addr, value)?;
            cycles = 2;
        }
        ThumbKind::LdrhImm32 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = memory.read_half(addr)? as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::LdrsbImm32 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = memory.read_byte(addr)? as i8 as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::LdrshImm32 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = memory.read_half(addr)? as i16 as u32;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }

        ThumbKind::StrhImm32 => {
            let addr = core.get_reg(instr.rn as usize).wrapping_add(instr.imm);
            let value = core.get_reg(instr.rd as usize) as u16;
            memory.write_half(addr, value)?;
            cycles = 2;
        }
        ThumbKind::LdrReg32 => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = memory.read_word(addr)?;
            core.set_reg(instr.rd as usize, value);
            cycles = 2;
        }
        ThumbKind::StrReg32 => {
            let addr = core
                .get_reg(instr.rn as usize)
                .wrapping_add(core.get_reg(instr.rm as usize));
            let value = core.get_reg(instr.rd as usize);
            memory.write_word(addr, value)?;
            cycles = 2;
        }

        // Branch (32-bit)
        ThumbKind::Bl32 => {
            let next_pc = core.pc + 4;
            core.lr = next_pc | 1;
            core.pc = (core.pc as i32 + 4 + instr.imm as i32) as u32;
            branch_taken = true;
            cycles = 2;
        }
        ThumbKind::Blx32 => {
            let next_pc = core.pc + 4;
            core.lr = next_pc | 1;
            core.pc = (core.pc as i32 + 4 + instr.imm as i32) as u32 & !1;
            branch_taken = true;
            cycles = 2;
        }
        ThumbKind::Beq32
        | ThumbKind::Bne32
        | ThumbKind::Bge32
        | ThumbKind::Blt32
        | ThumbKind::Bgt32
        | ThumbKind::Ble32 => {
            if check_condition(core, instr.cond) {
                core.pc = (core.pc as i32 + 4 + instr.imm as i32) as u32;
                branch_taken = true;
                cycles = 2;
            }
        }

        // Division (ARMv7-M)
        ThumbKind::Udiv => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let result = if b == 0 { 0 } else { a / b };
            core.set_reg(instr.rd as usize, result);
            cycles = 4;
        }
        ThumbKind::Sdiv => {
            let a = core.get_reg(instr.rn as usize) as i32;
            let b = core.get_reg(instr.rm as usize) as i32;
            let result = if b == 0 { 0 } else { a / b };
            core.set_reg(instr.rd as usize, result as u32);
            cycles = 4;
        }

        // Count leading zeros
        ThumbKind::Clz => {
            let value = core.get_reg(instr.rm as usize);
            let result = if value == 0 {
                32u32
            } else {
                value.leading_zeros()
            };
            core.set_reg(instr.rd as usize, result);
        }

        // Reverse operations
        ThumbKind::Rev => {
            let value = core.get_reg(instr.rm as usize);
            let result = value.swap_bytes();
            core.set_reg(instr.rd as usize, result);
        }
        ThumbKind::Rev16 => {
            let value = core.get_reg(instr.rm as usize);
            let result = ((value & 0x00FF00FF) << 8) | ((value & 0xFF00FF00) >> 8);
            core.set_reg(instr.rd as usize, result);
        }
        ThumbKind::Revsh => {
            let value = core.get_reg(instr.rm as usize) as u16;
            let result = (value >> 8) | ((value & 0xFF) << 8);
            core.set_reg(instr.rd as usize, result as i16 as i32 as u32);
        }
        ThumbKind::Rbit => {
            let value = core.get_reg(instr.rm as usize);
            let result = value.reverse_bits();
            core.set_reg(instr.rd as usize, result);
        }

        // Bit field operations
        ThumbKind::Ubfx => {
            let value = core.get_reg(instr.rn as usize);
            let lsb = instr.shift as u32;
            let width = instr.imm as u32;
            let mask = (1u32 << width) - 1;
            core.set_reg(instr.rd as usize, (value >> lsb) & mask);
        }
        ThumbKind::Sbfx => {
            let value = core.get_reg(instr.rn as usize);
            let lsb = instr.shift as u32;
            let width = instr.imm as u32;
            let mask = (1u32 << width) - 1;
            let extracted = (value >> lsb) & mask;
            // Sign extend
            let sign_bit = 1u32 << (width - 1);
            let result = if extracted & sign_bit != 0 {
                extracted | (!mask)
            } else {
                extracted
            };
            core.set_reg(instr.rd as usize, result);
        }

        // 32-bit multiply instructions
        ThumbKind::Mul32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let result = a.wrapping_mul(b);
            core.set_reg(instr.rd as usize, result);
            cycles = 2;
        }
        ThumbKind::Mla32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let c = core.get_reg(instr.shift as usize); // Ra stored in shift field
            let result = a.wrapping_mul(b).wrapping_add(c);
            core.set_reg(instr.rd as usize, result);
            cycles = 2;
        }
        ThumbKind::Umlal32 => {
            let a = core.get_reg(instr.rn as usize);
            let b = core.get_reg(instr.rm as usize);
            let mut acc = (core.get_reg(instr.rd as usize) as u64)
                | ((core.get_reg(instr.shift as usize) as u64) << 32);
            acc += (a as u64) * (b as u64);
            core.set_reg(instr.rd as usize, acc as u32);
            core.set_reg(instr.shift as usize, (acc >> 32) as u32);
            cycles = 3;
        }
        ThumbKind::Smlal32 => {
            let a = core.get_reg(instr.rn as usize) as i32;
            let b = core.get_reg(instr.rm as usize) as i32;
            let mut acc = (core.get_reg(instr.rd as usize) as i64)
                | ((core.get_reg(instr.shift as usize) as i64) << 32);
            acc += (a as i64) * (b as i64);
            core.set_reg(instr.rd as usize, acc as u32);
            core.set_reg(instr.shift as usize, (acc >> 32) as u32);
            cycles = 3;
        }

        // 32-bit load/store multiple
        ThumbKind::Ldm32 => {
            let base = core.get_reg(instr.rn as usize);
            let mut addr = base;
            let mut count = 0u32;

            // Register list is in imm field
            for i in 0..16 {
                if (instr.imm >> i) & 1 != 0 {
                    let value = memory.read_word(addr)?;
                    core.set_reg(i, value);
                    addr += 4;
                    count += 1;
                }
            }

            // Write back if not loading PC
            if (instr.imm & (1 << instr.rn)) == 0 {
                core.set_reg(instr.rn as usize, base + count * 4);
            }
            cycles = (1 + count) as u8;
        }
        ThumbKind::Stm32 => {
            let base = core.get_reg(instr.rn as usize);
            let mut addr = base;
            let mut count = 0u32;

            // Register list is in imm field
            for i in 0..16 {
                if (instr.imm >> i) & 1 != 0 {
                    let value = core.get_reg(i);
                    memory.write_word(addr, value)?;
                    addr += 4;
                    count += 1;
                }
            }

            // Write back
            core.set_reg(instr.rn as usize, base + count * 4);
            cycles = (1 + count) as u8;
        }
        ThumbKind::Push32 => {
            let mut sp = core.get_reg(13); // SP
            let mut count = 0u32;

            // Register list is in imm field
            for i in (0..16).rev() {
                if (instr.imm >> i) & 1 != 0 {
                    sp -= 4;
                    let value = core.get_reg(i);
                    memory.write_word(sp, value)?;
                    count += 1;
                }
            }

            core.set_reg(13, sp);
            cycles = (1 + count) as u8;
        }
        ThumbKind::Pop32 => {
            let mut sp = core.get_reg(13); // SP
            let mut count = 0u32;

            // Register list is in imm field
            for i in 0..16 {
                if (instr.imm >> i) & 1 != 0 {
                    let value = memory.read_word(sp)?;
                    if i == 15 {
                        // Loading PC - mask to preserve Thumb bit
                        core.pc = value & !1;
                    } else {
                        core.set_reg(i, value);
                    }
                    sp += 4;
                    count += 1;
                }
            }

            core.set_reg(13, sp);
            cycles = (1 + count) as u8;
        }

        // Bit field insert/clear
        ThumbKind::Bfi => {
            let value = core.get_reg(instr.rm as usize);
            let dest = core.get_reg(instr.rd as usize);
            let lsb = instr.shift as u32;
            let width = instr.imm as u32;
            let mask = ((1u32 << width) - 1) << lsb;
            let result = (dest & !mask) | ((value << lsb) & mask);
            core.set_reg(instr.rd as usize, result);
        }
        ThumbKind::Bfc => {
            let dest = core.get_reg(instr.rd as usize);
            let lsb = instr.shift as u32;
            let width = instr.imm as u32;
            let mask = !(((1u32 << width) - 1) << lsb);
            core.set_reg(instr.rd as usize, dest & mask);
        }

        // Placeholder for 32-bit Thumb-2 that needs more decoding
        ThumbKind::Thumb2 => {
            // This is a placeholder - actual Thumb-2 instruction was not fully decoded
            // In a real implementation, this would need to decode the second halfword
        }
        
        // Unknown instruction - should not happen with complete decoder
        ThumbKind::Unknown => {
            // Invalid instruction - could raise exception in real hardware
        }
    }

    // Advance PC for instructions that didn't branch
    if !branch_taken && instr.kind != ThumbKind::Pop {
        core.pc += instr.length as u32;
    }

    core.cycles += cycles as u64;
    core.instructions += 1;

    Ok(cycles)
}

/// Check condition code.
fn check_condition(core: &ArmCoreState, cond: u8) -> bool {
    let n = core.xpsr.n();
    let z = core.xpsr.z();
    let c = core.xpsr.c();
    let v = core.xpsr.v();

    match Condition::from_bits(cond) {
        Condition::Eq => z,
        Condition::Ne => !z,
        Condition::Cs => c,
        Condition::Cc => !c,
        Condition::Mi => n,
        Condition::Pl => !n,
        Condition::Vs => v,
        Condition::Vc => !v,
        Condition::Hi => c && !z,
        Condition::Ls => !c || z,
        Condition::Ge => n == v,
        Condition::Lt => n != v,
        Condition::Gt => !z && n == v,
        Condition::Le => z || n != v,
        Condition::Al => true,
        Condition::Nv => false,
    }
}

/// Add with carry.
fn add_with_carry(a: u32, b: u32, c: bool) -> (u32, bool, bool) {
    let c_val = if c { 1u64 } else { 0u64 };
    let result = (a as u64).wrapping_add(b as u64).wrapping_add(c_val);
    let carry = result > 0xFFFFFFFF;
    let overflow = (!(a ^ b) & (a ^ (result as u32))) >> 31 != 0;
    (result as u32, carry, overflow)
}

/// Subtract with carry.
/// For SUB (without carry), pass c = true to compute a - b.
/// For SBC (with carry), pass c = C flag to compute a - b - !C.
fn sub_with_carry(a: u32, b: u32, c: bool) -> (u32, bool, bool) {
    // When c = true, we compute a - b (no extra subtraction)
    // When c = false, we compute a - b - 1 (used for SBC with C=0)
    let c_val = if c { 0u64 } else { 1u64 };
    let result = (a as u64).wrapping_sub(b as u64).wrapping_sub(c_val);
    let carry = a as u64 >= (b as u64 + c_val);
    let overflow = ((a ^ b) & (a ^ (result as u32))) >> 31 != 0;
    (result as u32, carry, overflow)
}

/// Update flags for logical operations.
fn update_flags_logical(core: &mut ArmCoreState, result: u32, carry: bool) {
    core.xpsr.set_n((result >> 31) != 0);
    core.xpsr.set_z(result == 0);
    core.xpsr.set_c(carry);
}

/// Update flags for arithmetic operations.
fn update_flags_arithmetic(core: &mut ArmCoreState, result: u32, carry: bool, overflow: bool) {
    core.xpsr.set_n((result >> 31) != 0);
    core.xpsr.set_z(result == 0);
    core.xpsr.set_c(carry);
    core.xpsr.set_v(overflow);
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
    use crate::state::ArmCoreState;
    use rp2350sim_core::CoreId;

    /// Mock memory for testing
    struct MockMemory;

    impl MemoryAccess for MockMemory {
        fn read_byte(&self, _addr: u32) -> Result<u8> {
            Ok(0)
        }
        fn read_half(&self, _addr: u32) -> Result<u16> {
            Ok(0)
        }
        fn read_word(&self, _addr: u32) -> Result<u32> {
            Ok(0)
        }
        fn write_byte(&mut self, _addr: u32, _value: u8) -> Result<()> {
            Ok(())
        }
        fn write_half(&mut self, _addr: u32, _value: u16) -> Result<()> {
            Ok(())
        }
        fn write_word(&mut self, _addr: u32, _value: u32) -> Result<()> {
            Ok(())
        }
    }

    fn create_core() -> ArmCoreState {
        ArmCoreState::new(CoreId::CORE0)
    }

    #[test]
    fn test_udiv_normal() {
        let mut core = create_core();
        core.r[0] = 100;
        core.r[1] = 7;

        let instr = ThumbInstruction {
            kind: ThumbKind::Udiv,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 14); // 100 / 7 = 14
    }

    #[test]
    fn test_udiv_by_zero() {
        let mut core = create_core();
        core.r[0] = 100;
        core.r[1] = 0;

        let instr = ThumbInstruction {
            kind: ThumbKind::Udiv,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 0); // Division by zero returns 0
    }

    #[test]
    fn test_sdiv_positive() {
        let mut core = create_core();
        core.r[0] = 100;
        core.r[1] = 7;

        let instr = ThumbInstruction {
            kind: ThumbKind::Sdiv,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 14);
    }

    #[test]
    fn test_sdiv_negative() {
        let mut core = create_core();
        core.r[0] = (-100i32) as u32; // -100
        core.r[1] = 7;

        let instr = ThumbInstruction {
            kind: ThumbKind::Sdiv,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2] as i32, -14); // -100 / 7 = -14
    }

    #[test]
    fn test_sdiv_both_negative() {
        let mut core = create_core();
        core.r[0] = (-100i32) as u32;
        core.r[1] = (-7i32) as u32;

        let instr = ThumbInstruction {
            kind: ThumbKind::Sdiv,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 14); // -100 / -7 = 14
    }

    #[test]
    fn test_clz_normal() {
        let mut core = create_core();
        core.r[0] = 0x00FF0000; // 8 leading zeros

        let instr = ThumbInstruction {
            kind: ThumbKind::Clz,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 8);
    }

    #[test]
    fn test_clz_zero() {
        let mut core = create_core();
        core.r[0] = 0;

        let instr = ThumbInstruction {
            kind: ThumbKind::Clz,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 32);
    }

    #[test]
    fn test_clz_all_ones() {
        let mut core = create_core();
        core.r[0] = 0xFFFFFFFF;

        let instr = ThumbInstruction {
            kind: ThumbKind::Clz,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 0);
    }

    #[test]
    fn test_rev() {
        let mut core = create_core();
        core.r[0] = 0x12345678;

        let instr = ThumbInstruction {
            kind: ThumbKind::Rev,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 0x78563412);
    }

    #[test]
    fn test_rev16() {
        let mut core = create_core();
        core.r[0] = 0x12345678;

        let instr = ThumbInstruction {
            kind: ThumbKind::Rev16,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 0x34127856);
    }

    #[test]
    fn test_rbit() {
        let mut core = create_core();
        core.r[0] = 0x80808080;

        let instr = ThumbInstruction {
            kind: ThumbKind::Rbit,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 0x01010101);
    }

    #[test]
    fn test_it_block_execution() {
        let mut core = create_core();
        core.r[0] = 5;
        core.r[1] = 5;

        // Set up IT block for EQ condition (Z=1)
        core.xpsr.set_z(true);
        core.set_it_state(0x0, 0x8); // IT EQ (firstcond=0, mask=8)

        // First instruction in IT block should execute
        let (cond, _) = core.get_it_condition();
        assert_eq!(cond, 0); // EQ condition
        assert!(check_condition(&core, cond));
    }

    #[test]
    fn test_it_block_skip() {
        let mut core = create_core();
        core.r[0] = 5;
        core.r[1] = 3;

        // Set up IT block for EQ condition (Z=0, so condition fails)
        core.xpsr.set_z(false);
        core.set_it_state(0x0, 0x8); // IT EQ

        // First instruction in IT block should NOT execute
        let (cond, _) = core.get_it_condition();
        assert_eq!(cond, 0); // EQ condition
        assert!(!check_condition(&core, cond));
    }

    #[test]
    fn test_check_condition_eq() {
        let mut core = create_core();
        core.xpsr.set_z(true);
        assert!(check_condition(&core, 0x0)); // EQ
        assert!(!check_condition(&core, 0x1)); // NE
    }

    #[test]
    fn test_check_condition_ne() {
        let mut core = create_core();
        core.xpsr.set_z(false);
        assert!(!check_condition(&core, 0x0)); // EQ
        assert!(check_condition(&core, 0x1)); // NE
    }

    #[test]
    fn test_check_condition_cs_cc() {
        let mut core = create_core();
        core.xpsr.set_c(true);
        assert!(check_condition(&core, 0x2)); // CS
        assert!(!check_condition(&core, 0x3)); // CC

        core.xpsr.set_c(false);
        assert!(!check_condition(&core, 0x2)); // CS
        assert!(check_condition(&core, 0x3)); // CC
    }

    #[test]
    fn test_check_condition_mi_pl() {
        let mut core = create_core();
        core.xpsr.set_n(true);
        assert!(check_condition(&core, 0x4)); // MI
        assert!(!check_condition(&core, 0x5)); // PL

        core.xpsr.set_n(false);
        assert!(!check_condition(&core, 0x4)); // MI
        assert!(check_condition(&core, 0x5)); // PL
    }

    #[test]
    fn test_check_condition_ge_lt() {
        let mut core = create_core();
        core.xpsr.set_n(true);
        core.xpsr.set_v(true);
        assert!(check_condition(&core, 0xA)); // GE (N == V)
        assert!(!check_condition(&core, 0xB)); // LT

        core.xpsr.set_n(true);
        core.xpsr.set_v(false);
        assert!(!check_condition(&core, 0xA)); // GE
        assert!(check_condition(&core, 0xB)); // LT
    }

    #[test]
    fn test_check_condition_gt_le() {
        let mut core = create_core();
        // GT: Z==0 && N==V
        core.xpsr.set_z(false);
        core.xpsr.set_n(true);
        core.xpsr.set_v(true);
        assert!(check_condition(&core, 0xC)); // GT
        assert!(!check_condition(&core, 0xD)); // LE

        // LE: Z==1 || N!=V
        core.xpsr.set_z(true);
        assert!(!check_condition(&core, 0xC)); // GT
        assert!(check_condition(&core, 0xD)); // LE
    }

    #[test]
    fn test_check_condition_al() {
        let mut core = create_core();
        assert!(check_condition(&core, 0xE)); // AL (always)
    }

    #[test]
    fn test_ldr_imm32() {
        let mut core = create_core();
        core.r[0] = 0x20000000; // Base address

        let instr = ThumbInstruction {
            kind: ThumbKind::LdrImm32,
            rn: 0,
            rd: 1,
            imm: 0x100,
            ..Default::default()
        };

        let mut mem = MockMemory;
        // Note: MockMemory returns 0 for reads
        execute(&mut core, &instr, &mut mem).unwrap();

        // R1 should be loaded from address 0x20000100
        assert_eq!(core.r[1], 0);
    }

    #[test]
    fn test_str_imm32() {
        let mut core = create_core();
        core.r[0] = 0x20000000; // Base address
        core.r[1] = 0xDEADBEEF; // Value to store

        let instr = ThumbInstruction {
            kind: ThumbKind::StrImm32,
            rn: 0,
            rd: 1,
            imm: 0x100,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // Instruction should complete without error
        // (MockMemory accepts all writes)
    }

    #[test]
    fn test_ldrb_imm32() {
        let mut core = create_core();
        core.r[0] = 0x20000000;

        let instr = ThumbInstruction {
            kind: ThumbKind::LdrbImm32,
            rn: 0,
            rd: 1,
            imm: 0x50,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // R1 should contain a byte value (0 from MockMemory)
        assert_eq!(core.r[1], 0);
    }

    #[test]
    fn test_strb_imm32() {
        let mut core = create_core();
        core.r[0] = 0x20000000;
        core.r[1] = 0xAB;

        let instr = ThumbInstruction {
            kind: ThumbKind::StrbImm32,
            rn: 0,
            rd: 1,
            imm: 0x50,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();
    }

    #[test]
    fn test_ldrh_imm32() {
        let mut core = create_core();
        core.r[0] = 0x20000000;

        let instr = ThumbInstruction {
            kind: ThumbKind::LdrhImm32,
            rn: 0,
            rd: 1,
            imm: 0x80,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // R1 should contain a halfword value
        assert_eq!(core.r[1], 0);
    }

    #[test]
    fn test_strh_imm32() {
        let mut core = create_core();
        core.r[0] = 0x20000000;
        core.r[1] = 0x1234;

        let instr = ThumbInstruction {
            kind: ThumbKind::StrhImm32,
            rn: 0,
            rd: 1,
            imm: 0x80,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();
    }

    #[test]
    fn test_ldr_reg32() {
        let mut core = create_core();
        core.r[0] = 0x20000000;
        core.r[1] = 0x100; // Offset

        let instr = ThumbInstruction {
            kind: ThumbKind::LdrReg32,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // R2 should be loaded from address 0x20000100
        assert_eq!(core.r[2], 0);
    }

    #[test]
    fn test_str_reg32() {
        let mut core = create_core();
        core.r[0] = 0x20000000;
        core.r[1] = 0x100;
        core.r[2] = 0xCAFEBABE;

        let instr = ThumbInstruction {
            kind: ThumbKind::StrReg32,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();
    }

    #[test]
    fn test_ldr_pc_imm32() {
        let mut core = create_core();
        core.pc = 0x1000; // PC

        let instr = ThumbInstruction {
            kind: ThumbKind::LdrImm32,
            rn: 15, // PC-relative load
            rd: 0,
            imm: 0x100,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // R0 should be loaded from PC + 0x100 (aligned)
        assert_eq!(core.r[0], 0);
    }

    #[test]
    fn test_sxth() {
        let mut core = create_core();
        core.r[0] = 0x0000FF80; // Low halfword is -128 (signed)

        let instr = ThumbInstruction {
            kind: ThumbKind::Sxth,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // Should sign-extend the low halfword
        assert_eq!(core.r[1] as i32, -128);
    }

    #[test]
    fn test_sxtb() {
        let mut core = create_core();
        core.r[0] = 0x00000080; // Low byte is -128 (signed)

        let instr = ThumbInstruction {
            kind: ThumbKind::Sxtb,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // Should sign-extend the low byte
        assert_eq!(core.r[1] as i32, -128);
    }

    #[test]
    fn test_uxth() {
        let mut core = create_core();
        core.r[0] = 0xFFFF8000; // High halfword should be ignored

        let instr = ThumbInstruction {
            kind: ThumbKind::Uxth,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // Should zero-extend the low halfword
        assert_eq!(core.r[1], 0x8000);
    }

    #[test]
    fn test_uxtb() {
        let mut core = create_core();
        core.r[0] = 0xFFFFFF80; // High bytes should be ignored

        let instr = ThumbInstruction {
            kind: ThumbKind::Uxtb,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // Should zero-extend the low byte
        assert_eq!(core.r[1], 0x80);
    }

    #[test]
    fn test_mvn_reg32() {
        let mut core = create_core();
        core.r[0] = 0x0000FFFF;

        let instr = ThumbInstruction {
            kind: ThumbKind::MvnReg32,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 0xFFFF0000);
    }

    #[test]
    fn test_mov_reg32() {
        let mut core = create_core();
        core.r[0] = 0x12345678;

        let instr = ThumbInstruction {
            kind: ThumbKind::MovReg32,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 0x12345678);
    }

    #[test]
    fn test_mov_imm32() {
        let mut core = create_core();

        let instr = ThumbInstruction {
            kind: ThumbKind::MovImm32,
            rd: 0,
            imm: 0x1234,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[0], 0x1234);
    }

    #[test]
    fn test_orr_reg32() {
        let mut core = create_core();
        core.r[0] = 0xFF00;
        core.r[1] = 0x00FF;

        let instr = ThumbInstruction {
            kind: ThumbKind::OrrReg32,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 0xFFFF);
    }

    #[test]
    fn test_eor_reg32() {
        let mut core = create_core();
        core.r[0] = 0xFF0F;
        core.r[1] = 0x0FFF;

        let instr = ThumbInstruction {
            kind: ThumbKind::EorReg32,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 0xF0F0);
    }

    #[test]
    fn test_and_reg32() {
        let mut core = create_core();
        core.r[0] = 0xFF0F;
        core.r[1] = 0x0FFF;

        let instr = ThumbInstruction {
            kind: ThumbKind::AndReg32,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 0x0F0F);
    }

    #[test]
    fn test_bic_reg32() {
        let mut core = create_core();
        core.r[0] = 0xFFFF;
        core.r[1] = 0x0F0F;

        let instr = ThumbInstruction {
            kind: ThumbKind::BicReg32,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // BIC clears bits where the mask is 1
        assert_eq!(core.r[2], 0xF0F0);
    }

    #[test]
    fn test_add_reg32() {
        let mut core = create_core();
        core.r[0] = 100;
        core.r[1] = 50;

        let instr = ThumbInstruction {
            kind: ThumbKind::AddReg32,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 150);
    }

    #[test]
    fn test_sub_reg32() {
        let mut core = create_core();
        core.r[0] = 100;
        core.r[1] = 30;

        let instr = ThumbInstruction {
            kind: ThumbKind::SubReg32,
            rn: 0,
            rm: 1,
            rd: 2,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[2], 70);
    }

    #[test]
    fn test_add_imm32() {
        let mut core = create_core();
        core.r[0] = 100;

        let instr = ThumbInstruction {
            kind: ThumbKind::AddImm32,
            rn: 0,
            rd: 1,
            imm: 50,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 150);
    }

    #[test]
    fn test_sub_imm32() {
        let mut core = create_core();
        core.r[0] = 100;

        let instr = ThumbInstruction {
            kind: ThumbKind::SubImm32,
            rn: 0,
            rd: 1,
            imm: 30,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1], 70);
    }

    #[test]
    fn test_lsl_reg() {
        let mut core = create_core();
        core.r[0] = 0x00000001;
        core.r[1] = 4;

        // LslReg uses rd for value and rm for shift
        let instr = ThumbInstruction {
            kind: ThumbKind::LslReg,
            rd: 0, // value to shift (and destination)
            rm: 1, // shift amount
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[0], 0x00000010);
    }

    #[test]
    fn test_lsr_reg() {
        let mut core = create_core();
        core.r[0] = 0x00000010;
        core.r[1] = 4;

        // LsrReg uses rd for value and rm for shift
        let instr = ThumbInstruction {
            kind: ThumbKind::LsrReg,
            rd: 0, // value to shift (and destination)
            rm: 1, // shift amount
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[0], 0x00000001);
    }

    #[test]
    fn test_asr_reg() {
        let mut core = create_core();
        core.r[0] = (-16i32) as u32; // -16
        core.r[1] = 2;

        // AsrReg uses rd for value and rm for shift
        let instr = ThumbInstruction {
            kind: ThumbKind::AsrReg,
            rd: 0, // value to shift (and destination)
            rm: 1, // shift amount
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[0] as i32, -4); // -16 >> 2 = -4 (arithmetic)
    }

    #[test]
    fn test_ror() {
        let mut core = create_core();
        core.r[0] = 0x00000001;
        core.r[1] = 4;

        // Ror uses rd for value and rm for shift
        let instr = ThumbInstruction {
            kind: ThumbKind::Ror,
            rd: 0, // value to rotate (and destination)
            rm: 1, // rotate amount
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        // Rotate right by 4: 0x00000001 -> 0x10000000
        assert_eq!(core.r[0], 0x10000000);
    }

    #[test]
    fn test_neg() {
        let mut core = create_core();
        core.r[0] = 100;

        let instr = ThumbInstruction {
            kind: ThumbKind::Neg,
            rm: 0,
            rd: 1,
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[1] as i32, -100);
    }

    #[test]
    fn test_mul() {
        let mut core = create_core();
        core.r[0] = 100;
        core.r[1] = 7;

        // Mul uses rd for first operand and destination, rm for second operand
        let instr = ThumbInstruction {
            kind: ThumbKind::Mul,
            rd: 0, // first operand (and destination)
            rm: 1, // second operand
            ..Default::default()
        };

        let mut mem = MockMemory;
        execute(&mut core, &instr, &mut mem).unwrap();

        assert_eq!(core.r[0], 700);
    }
}

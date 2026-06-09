//! Tests for RP2350 simulator.

use rp2350sim_cpu_arm::thumb::{decode, disassemble, ThumbKind};
use rp2350sim_cpu_hazard3::rv32::{decode as rv32_decode, disassemble as rv32_disassemble, Rv32Kind};

#[cfg(test)]
mod thumb_tests {
    use super::*;

    // ==================== Move and Compare ====================

    #[test]
    fn test_decode_mov_imm() {
        // MOVS R0, #0x42
        // Format: 00100 Rd[2:0] imm8[7:0]
        let opcode = 0x2042;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::MovImm);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.imm, 0x42);
    }

    #[test]
    fn test_decode_mov_imm_max() {
        // MOVS R7, #0xFF
        let opcode = 0x27FF;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::MovImm);
        assert_eq!(instr.rd, 7);
        assert_eq!(instr.imm, 0xFF);
    }

    #[test]
    fn test_decode_cmp_imm() {
        // CMP R0, #0x42
        let opcode = 0x2842;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::CmpImm);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.imm, 0x42);
    }

    #[test]
    fn test_decode_cmp_reg() {
        // CMP R0, R1
        let opcode = 0x4288;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Cmp);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_cmn() {
        // CMN R0, R1
        let opcode = 0x42C8;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Cmn);
    }

    #[test]
    fn test_decode_tst() {
        // TST R0, R1
        let opcode = 0x4208;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Tst);
    }

    // ==================== Arithmetic ====================

    #[test]
    fn test_decode_add_reg() {
        // ADDS R0, R1, R2
        let opcode = 0x1888;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::AddReg);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rn, 1);
        assert_eq!(instr.rm, 2);
    }

    #[test]
    fn test_decode_sub_reg() {
        // SUBS R0, R1, R2
        let opcode = 0x1A88;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::SubReg);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rn, 1);
        assert_eq!(instr.rm, 2);
    }

    #[test]
    fn test_decode_add_imm3() {
        // ADDS R0, R1, #5
        // Format: 000111 imm3 Rn Rd where bit 10=1 (imm), bit 9=0 (ADD)
        // imm3=5=101, Rn=1=001, Rd=0=000
        // Opcode: 0001 1101 0100 1000 = 0x1D48
        let opcode = 0x1D48;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::AddImm3);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rn, 1);
        assert_eq!(instr.imm, 5);
    }

    #[test]
    fn test_decode_sub_imm3() {
        // SUBS R0, R1, #5
        // Format: 000111 imm3 Rn Rd where bit 10=1 (imm), bit 9=1 (SUB)
        // imm3=5=101, Rn=1=001, Rd=0=000
        // Opcode: 0001 1111 0100 1000 = 0x1F48
        let opcode = 0x1F48;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::SubImm3);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rn, 1);
        assert_eq!(instr.imm, 5);
    }

    #[test]
    fn test_decode_add_imm8() {
        // ADDS R0, #0x42
        let opcode = 0x3042;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::AddImm8);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.imm, 0x42);
    }

    #[test]
    fn test_decode_sub_imm8() {
        // SUBS R0, #0x42
        let opcode = 0x3842;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::SubImm8);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.imm, 0x42);
    }

    #[test]
    fn test_decode_adc() {
        // ADCS R0, R1
        let opcode = 0x4148;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Adc);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_sbc() {
        // SBCS R0, R1
        let opcode = 0x4188;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Sbc);
    }

    #[test]
    fn test_decode_neg() {
        // NEGS R0, R1 (RSB)
        let opcode = 0x4248;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Neg);
    }

    #[test]
    fn test_decode_mul() {
        // MULS R0, R1
        let opcode = 0x4348;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Mul);
    }

    // ==================== Shift ====================

    #[test]
    fn test_decode_lsl() {
        // LSLS R0, R1, #4
        let opcode = 0x0108;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Lsl);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.shift, 4);
    }

    #[test]
    fn test_decode_lsr() {
        // LSRS R0, R1, #4
        let opcode = 0x0908;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Lsr);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.shift, 4);
    }

    #[test]
    fn test_decode_asr() {
        // ASRS R0, R1, #4
        let opcode = 0x1108;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Asr);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.shift, 4);
    }

    #[test]
    fn test_decode_lsl_reg() {
        // LSLS R0, R1
        let opcode = 0x4088;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::LslReg);
    }

    #[test]
    fn test_decode_lsr_reg() {
        // LSRS R0, R1
        let opcode = 0x40C8;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::LsrReg);
    }

    #[test]
    fn test_decode_asr_reg() {
        // ASRS R0, R1
        let opcode = 0x4108;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::AsrReg);
    }

    #[test]
    fn test_decode_ror() {
        // RORS R0, R1
        let opcode = 0x41C8;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Ror);
    }

    // ==================== Logical ====================

    #[test]
    fn test_decode_and() {
        // ANDS R0, R1
        let opcode = 0x4008;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::And);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_eor() {
        // EORS R0, R1
        let opcode = 0x4048;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Eor);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_orr() {
        // ORRS R0, R1
        let opcode = 0x4308;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Orr);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_bic() {
        // BICS R0, R1
        let opcode = 0x4388;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Bic);
    }

    #[test]
    fn test_decode_mvn() {
        // MVNS R0, R1
        let opcode = 0x43C8;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Mvn);
    }

    // ==================== Load/Store ====================

    #[test]
    fn test_decode_ldr_lit() {
        // LDR R0, [PC, #0x40]
        let opcode = 0x4810;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::LdrLit);
        assert_eq!(instr.rd, 0);
        assert!(instr.reads_mem);
    }

    #[test]
    fn test_decode_ldr_sp() {
        // LDR R0, [SP, #0x40]
        // Format: 01011 Rd imm8
        // Opcode: 0101 1000 0001 0000 = 0x9810
        let opcode = 0x9810;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::LdrSp);
        assert_eq!(instr.rd, 0);
        assert!(instr.reads_mem);
    }

    #[test]
    fn test_decode_str_sp() {
        // STR R0, [SP, #0x40]
        // Format: 01010 Rd imm8
        // Opcode: 0101 0000 0001 0000 = 0x9010
        let opcode = 0x9010;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::StrSp);
        assert_eq!(instr.rd, 0);
        assert!(instr.writes_mem);
    }

    #[test]
    fn test_decode_ldr_reg() {
        // LDR R0, [R1, R2]
        // Format: 0101 100 Rm Rn Rt
        // Opcode: 0101 100 010 001 000 = 0x5888
        let opcode = 0x5888;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::LdrReg);
        assert!(instr.reads_mem);
    }

    #[test]
    fn test_decode_str_reg() {
        // STR R0, [R1, R2]
        // Format: 0101 000 Rm Rn Rt
        // Opcode: 0101 000 010 001 000 = 0x5088
        let opcode = 0x5088;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::StrReg);
        assert!(instr.writes_mem);
    }

    #[test]
    fn test_decode_ldrb_reg() {
        // LDRB R0, [R1, R2]
        // Format: 0101 110 Rm Rn Rt
        // Opcode: 0101 110 010 001 000 = 0x5C88
        let opcode = 0x5C88;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::LdrbReg);
    }

    #[test]
    fn test_decode_strb_reg() {
        // STRB R0, [R1, R2]
        // Format: 0101 010 Rm Rn Rt
        // Opcode: 0101 010 010 001 000 = 0x5488
        let opcode = 0x5488;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::StrbReg);
    }

    #[test]
    fn test_decode_ldrh_reg() {
        // LDRH R0, [R1, R2]
        // Format: 0101 101 Rm Rn Rt
        // Opcode: 0101 101 010 001 000 = 0x5A88
        let opcode = 0x5A88;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::LdrhReg);
    }

    #[test]
    fn test_decode_strh_reg() {
        // STRH R0, [R1, R2]
        // Format: 0101 001 Rm Rn Rt
        // Opcode: 0101 001 010 001 000 = 0x5288
        let opcode = 0x5288;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::StrhReg);
    }

    // ==================== Push/Pop ====================

    #[test]
    fn test_decode_push() {
        // PUSH {R4, R5, LR}
        let opcode = 0xB470;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Push);
    }

    #[test]
    fn test_decode_push_simple() {
        // PUSH {R4}
        let opcode = 0xB410;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Push);
    }

    #[test]
    fn test_decode_pop() {
        // POP {R4, R5, PC}
        let opcode = 0xBC70;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Pop);
    }

    #[test]
    fn test_decode_pop_simple() {
        // POP {R4}
        let opcode = 0xBC10;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Pop);
    }

    // ==================== Branch ====================

    #[test]
    fn test_decode_b() {
        // B #0x100
        let opcode = 0xE040;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::B);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_beq() {
        // BEQ #0x10
        let opcode = 0xD008;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x0); // EQ
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_bne() {
        // BNE #0x10
        let opcode = 0xD108;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x1); // NE
    }

    #[test]
    fn test_decode_bcs() {
        // BCS #0x10
        let opcode = 0xD208;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x2); // CS
    }

    #[test]
    fn test_decode_bcc() {
        // BCC #0x10
        let opcode = 0xD308;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x3); // CC
    }

    #[test]
    fn test_decode_bmi() {
        // BMI #0x10
        let opcode = 0xD408;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x4); // MI
    }

    #[test]
    fn test_decode_bpl() {
        // BPL #0x10
        let opcode = 0xD508;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x5); // PL
    }

    #[test]
    fn test_decode_bvs() {
        // BVS #0x10
        let opcode = 0xD608;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x6); // VS
    }

    #[test]
    fn test_decode_bvc() {
        // BVC #0x10
        let opcode = 0xD708;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x7); // VC
    }

    #[test]
    fn test_decode_bhi() {
        // BHI #0x10
        let opcode = 0xD808;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x8); // HI
    }

    #[test]
    fn test_decode_bls() {
        // BLS #0x10
        let opcode = 0xD908;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0x9); // LS
    }

    #[test]
    fn test_decode_bge() {
        // BGE #0x10
        let opcode = 0xDA08;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0xA); // GE
    }

    #[test]
    fn test_decode_blt() {
        // BLT #0x10
        let opcode = 0xDB08;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0xB); // LT
    }

    #[test]
    fn test_decode_bgt() {
        // BGT #0x10
        let opcode = 0xDC08;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0xC); // GT
    }

    #[test]
    fn test_decode_ble() {
        // BLE #0x10
        let opcode = 0xDD08;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::BCond);
        assert_eq!(instr.cond, 0xD); // LE
    }

    #[test]
    fn test_decode_cbz() {
        // CBZ R0, #0x10
        // Format: 1011 0x1x imm5 Rn where x=0 for CBZ
        // Opcode: 1011 0001 0000 1000 = 0xB108
        let opcode = 0xB108;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Cbz);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_cbnz() {
        // CBNZ R0, #0x10
        // Format: 1011 0x1x imm5 Rn where x=1 for CBNZ
        // Opcode: 1011 1001 0000 1000 = 0xB908
        let opcode = 0xB908;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Cbnz);
        assert!(instr.is_branch);
    }

    // ==================== Extend ====================

    #[test]
    fn test_decode_sxth() {
        // SXTH R0, R1
        // Format: 1011 0010 01 Rm Rd
        // Opcode: 1011 0010 0100 1000 = 0xB248
        let opcode = 0xB248;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Sxth);
    }

    #[test]
    fn test_decode_sxtb() {
        // SXTB R0, R1
        // Format: 1011 0010 11 Rm Rd
        // Opcode: 1011 0010 1100 1000 = 0xB2C8
        let opcode = 0xB2C8;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Sxtb);
    }

    #[test]
    fn test_decode_uxth() {
        // UXTH R0, R1
        // Format: 1011 0010 00 Rm Rd
        // Opcode: 1011 0010 0000 1000 = 0xB208
        let opcode = 0xB208;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Uxth);
    }

    #[test]
    fn test_decode_uxtb() {
        // UXTB R0, R1
        // Format: 1011 0010 10 Rm Rd
        // Opcode: 1011 0010 1000 1000 = 0xB288
        let opcode = 0xB288;
        let instr = decode(opcode);
        assert_eq!(instr.kind, ThumbKind::Uxtb);
    }

    // ==================== Disassembly ====================

    #[test]
    fn test_disassemble_mov() {
        let opcode = 0x2242;
        let result = disassemble(opcode, 0);
        assert!(result.contains("movs"));
    }

    #[test]
    fn test_disassemble_add() {
        let opcode = 0x1888;
        let result = disassemble(opcode, 0);
        assert!(result.contains("adds"));
    }

    #[test]
    fn test_disassemble_sub() {
        let opcode = 0x1A88;
        let result = disassemble(opcode, 0);
        assert!(result.contains("subs"));
    }

    #[test]
    fn test_disassemble_lsl() {
        let opcode = 0x0108;
        let result = disassemble(opcode, 0);
        assert!(result.contains("lsls"));
    }

    #[test]
    fn test_disassemble_lsr() {
        let opcode = 0x0908;
        let result = disassemble(opcode, 0);
        assert!(result.contains("lsrs"));
    }

    #[test]
    fn test_disassemble_asr() {
        let opcode = 0x1108;
        let result = disassemble(opcode, 0);
        assert!(result.contains("asrs"));
    }

    #[test]
    fn test_disassemble_and() {
        let opcode = 0x4008;
        let result = disassemble(opcode, 0);
        assert!(result.contains("ands"));
    }

    #[test]
    fn test_disassemble_eor() {
        let opcode = 0x4048;
        let result = disassemble(opcode, 0);
        assert!(result.contains("eors"));
    }

    #[test]
    fn test_disassemble_orr() {
        let opcode = 0x4308;
        let result = disassemble(opcode, 0);
        assert!(result.contains("orrs"));
    }

    #[test]
    fn test_disassemble_push() {
        let opcode = 0xB410;
        let result = disassemble(opcode, 0);
        assert!(result.contains("push"));
    }

    #[test]
    fn test_disassemble_pop() {
        let opcode = 0xBC10;
        let result = disassemble(opcode, 0);
        assert!(result.contains("pop"));
    }

    #[test]
    fn test_disassemble_b() {
        let opcode = 0xE040;
        let result = disassemble(opcode, 0x1000);
        assert!(result.contains("b 0x"));
    }

    #[test]
    fn test_disassemble_beq() {
        let opcode = 0xD008;
        let result = disassemble(opcode, 0x1000);
        assert!(result.contains("beq"));
    }

    #[test]
    fn test_disassemble_bne() {
        let opcode = 0xD108;
        let result = disassemble(opcode, 0x1000);
        assert!(result.contains("bne"));
    }

    #[test]
    fn test_disassemble_ldr_lit() {
        let opcode = 0x4810;
        let result = disassemble(opcode, 0);
        assert!(result.contains("ldr"));
    }
}

#[cfg(test)]
mod rv32_tests {
    use super::*;

    // ==================== LUI/AUIPC ====================

    #[test]
    fn test_decode_lui() {
        // LUI x1, 0x12345
        // Format: imm[31:12] | rd[11:7] | opcode[6:0]
        // Encoding: 0x12345 << 12 | 1 << 7 | 0x37 = 0x123450B7
        let opcode = 0x123450B7;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lui);
        assert_eq!(instr.rd, 1);
    }

    #[test]
    fn test_decode_lui_zero() {
        // LUI x0, 0
        let opcode = 0x00000037;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lui);
        assert_eq!(instr.rd, 0);
    }

    #[test]
    fn test_decode_auipc() {
        // AUIPC x1, 0x12345
        // Format: imm[31:12] | rd[11:7] | opcode[6:0]
        // Encoding: 0x12345 << 12 | 1 << 7 | 0x17 = 0x12345097
        let opcode = 0x12345097;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Auipc);
        assert_eq!(instr.rd, 1);
    }

    // ==================== JAL/JALR ====================

    #[test]
    fn test_decode_jal() {
        // JAL x1, 0x1000
        let opcode = 0x001005EF;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Jal);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_jal_link() {
        // JAL x1, ... (call)
        let opcode = 0x000000EF; // JAL x1, 0
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Jal);
        assert!(instr.is_call);
    }

    #[test]
    fn test_decode_jalr() {
        // JALR x1, x2, 0
        let opcode = 0x000100E7;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Jalr);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_jalr_return() {
        // JALR x0, x1, 0 (ret)
        let opcode = 0x00008067;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Jalr);
        assert!(instr.is_return);
    }

    // ==================== Branch ====================

    #[test]
    fn test_decode_beq() {
        // BEQ x1, x2, 0x10
        let opcode = 0x00208663;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Beq);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_bne() {
        // BNE x1, x2, 0x10
        // B-type format: imm[12|10:5] | rs2 | rs1 | funct3 | imm[4:1|11] | opcode
        // For BNE: funct3=001, imm=16
        let opcode = 0x00209863;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Bne);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_blt() {
        // BLT x1, x2, 0x10
        let opcode = 0x0020C663;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Blt);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_bge() {
        // BGE x1, x2, 0x10
        let opcode = 0x0020D663;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Bge);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_bltu() {
        // BLTU x1, x2, 0x10
        let opcode = 0x0020E663;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Bltu);
        assert!(instr.is_branch);
    }

    #[test]
    fn test_decode_bgeu() {
        // BGEU x1, x2, 0x10
        let opcode = 0x0020F663;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Bgeu);
        assert!(instr.is_branch);
    }

    // ==================== Load ====================

    #[test]
    fn test_decode_lb() {
        // LB x1, 0(x2)
        let opcode = 0x00010083;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lb);
        assert!(instr.reads_mem);
    }

    #[test]
    fn test_decode_lh() {
        // LH x1, 0(x2)
        let opcode = 0x00011083;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lh);
        assert!(instr.reads_mem);
    }

    #[test]
    fn test_decode_lw() {
        // LW x1, 0(x2)
        let opcode = 0x00012083;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lw);
        assert!(instr.reads_mem);
    }

    #[test]
    fn test_decode_lbu() {
        // LBU x1, 0(x2)
        let opcode = 0x00014083;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lbu);
        assert!(instr.reads_mem);
    }

    #[test]
    fn test_decode_lhu() {
        // LHU x1, 0(x2)
        let opcode = 0x00015083;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Lhu);
        assert!(instr.reads_mem);
    }

    // ==================== Store ====================

    #[test]
    fn test_decode_sb() {
        // SB x1, 0(x2)
        let opcode = 0x00110023;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sb);
        assert!(instr.writes_mem);
    }

    #[test]
    fn test_decode_sh() {
        // SH x1, 0(x2)
        let opcode = 0x00111023;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sh);
        assert!(instr.writes_mem);
    }

    #[test]
    fn test_decode_sw() {
        // SW x1, 0(x2)
        let opcode = 0x00112023;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sw);
        assert!(instr.writes_mem);
    }

    // ==================== Immediate Arithmetic ====================

    #[test]
    fn test_decode_addi() {
        // ADDI x1, x2, 42
        let opcode = 0x02A10093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Addi);
        assert_eq!(instr.rd, 1);
        assert_eq!(instr.rs1, 2);
        assert_eq!(instr.imm, 42);
    }

    #[test]
    fn test_decode_addi_negative() {
        // ADDI x1, x2, -1
        let opcode = 0xFFF10093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Addi);
        assert_eq!(instr.imm, -1);
    }

    #[test]
    fn test_decode_slti() {
        // SLTI x1, x2, 10
        let opcode = 0x00A12093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Slti);
    }

    #[test]
    fn test_decode_sltiu() {
        // SLTIU x1, x2, 10
        let opcode = 0x00A13093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sltiu);
    }

    #[test]
    fn test_decode_xori() {
        // XORI x1, x2, 0xFF
        let opcode = 0x0FF14093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Xori);
    }

    #[test]
    fn test_decode_ori() {
        // ORI x1, x2, 0xFF
        let opcode = 0x0FF16093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Ori);
    }

    #[test]
    fn test_decode_andi() {
        // ANDI x1, x2, 0xFF
        let opcode = 0x0FF17093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Andi);
    }

    #[test]
    fn test_decode_slli() {
        // SLLI x1, x2, 5
        let opcode = 0x00511093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Slli);
        assert_eq!(instr.imm, 5);
    }

    #[test]
    fn test_decode_srli() {
        // SRLI x1, x2, 5
        let opcode = 0x00515093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Srli);
    }

    #[test]
    fn test_decode_srai() {
        // SRAI x1, x2, 5
        let opcode = 0x40515093;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Srai);
    }

    // ==================== Register Arithmetic ====================

    #[test]
    fn test_decode_add() {
        // ADD x1, x2, x3
        let opcode = 0x003100B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Add);
        assert_eq!(instr.rd, 1);
        assert_eq!(instr.rs1, 2);
        assert_eq!(instr.rs2, 3);
    }

    #[test]
    fn test_decode_sub() {
        // SUB x1, x2, x3
        let opcode = 0x403100B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sub);
        assert_eq!(instr.rd, 1);
        assert_eq!(instr.rs1, 2);
        assert_eq!(instr.rs2, 3);
    }

    #[test]
    fn test_decode_sll() {
        // SLL x1, x2, x3
        let opcode = 0x003110B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sll);
    }

    #[test]
    fn test_decode_slt() {
        // SLT x1, x2, x3
        let opcode = 0x003120B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Slt);
    }

    #[test]
    fn test_decode_sltu() {
        // SLTU x1, x2, x3
        let opcode = 0x003130B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sltu);
    }

    #[test]
    fn test_decode_xor() {
        // XOR x1, x2, x3
        let opcode = 0x003140B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Xor);
    }

    #[test]
    fn test_decode_srl() {
        // SRL x1, x2, x3
        let opcode = 0x003150B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Srl);
    }

    #[test]
    fn test_decode_sra() {
        // SRA x1, x2, x3
        let opcode = 0x403150B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Sra);
    }

    #[test]
    fn test_decode_or() {
        // OR x1, x2, x3
        let opcode = 0x003160B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Or);
    }

    #[test]
    fn test_decode_and() {
        // AND x1, x2, x3
        let opcode = 0x003170B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::And);
    }

    // ==================== M Extension ====================

    #[test]
    fn test_decode_mul() {
        // MUL x1, x2, x3
        let opcode = 0x023100B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Mul);
    }

    #[test]
    fn test_decode_mulh() {
        // MULH x1, x2, x3
        let opcode = 0x023110B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Mulh);
    }

    #[test]
    fn test_decode_mulhsu() {
        // MULHSU x1, x2, x3
        let opcode = 0x023120B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Mulhsu);
    }

    #[test]
    fn test_decode_mulhu() {
        // MULHU x1, x2, x3
        let opcode = 0x023130B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Mulhu);
    }

    #[test]
    fn test_decode_div() {
        // DIV x1, x2, x3
        let opcode = 0x023140B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Div);
    }

    #[test]
    fn test_decode_divu() {
        // DIVU x1, x2, x3
        let opcode = 0x023150B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Divu);
    }

    #[test]
    fn test_decode_rem() {
        // REM x1, x2, x3
        let opcode = 0x023160B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Rem);
    }

    #[test]
    fn test_decode_remu() {
        // REMU x1, x2, x3
        let opcode = 0x023170B3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Remu);
    }

    // ==================== System ====================

    #[test]
    fn test_decode_ecall() {
        // ECALL
        let opcode = 0x00000073;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Ecall);
    }

    #[test]
    fn test_decode_ebreak() {
        // EBREAK
        let opcode = 0x00100073;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Ebreak);
    }

    #[test]
    fn test_decode_csrrw() {
        // CSRRW x1, 0x300, x2
        let opcode = 0x302110F3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Csrrw);
    }

    #[test]
    fn test_decode_csrrs() {
        // CSRRS x1, 0x300, x2
        let opcode = 0x302121F3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Csrrs);
    }

    #[test]
    fn test_decode_csrrc() {
        // CSRRC x1, 0x300, x2
        let opcode = 0x302131F3;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Csrrc);
    }

    // ==================== Fence ====================

    #[test]
    fn test_decode_fence() {
        // FENCE
        let opcode = 0x0000000F;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::Fence);
    }

    #[test]
    fn test_decode_fencei() {
        // FENCE.I
        let opcode = 0x0000100F;
        let instr = rv32_decode(opcode);
        assert_eq!(instr.kind, Rv32Kind::FenceI);
    }

    // ==================== Disassembly ====================

    #[test]
    fn test_disassemble_lui() {
        let opcode = 0x123455B7;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("lui"));
    }

    #[test]
    fn test_disassemble_auipc() {
        let opcode = 0x12345097;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("auipc"));
    }

    #[test]
    fn test_disassemble_addi() {
        let opcode = 0x02A10093;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("addi"));
    }

    #[test]
    fn test_disassemble_add() {
        let opcode = 0x003100B3;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("add"));
    }

    #[test]
    fn test_disassemble_sub() {
        let opcode = 0x403100B3;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("sub"));
    }

    #[test]
    fn test_disassemble_lw() {
        let opcode = 0x00012083;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("lw"));
    }

    #[test]
    fn test_disassemble_sw() {
        let opcode = 0x00112023;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("sw"));
    }

    #[test]
    fn test_disassemble_beq() {
        let opcode = 0x00208663;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("beq"));
    }

    #[test]
    fn test_disassemble_jal() {
        let opcode = 0x000000EF;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("jal"));
    }

    #[test]
    fn test_disassemble_jalr() {
        let opcode = 0x00008067;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("jalr"));
    }

    #[test]
    fn test_disassemble_mul() {
        let opcode = 0x023100B3;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("mul"));
    }

    #[test]
    fn test_disassemble_div() {
        let opcode = 0x023140B3;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("div"));
    }

    #[test]
    fn test_disassemble_ecall() {
        let opcode = 0x00000073;
        let result = rv32_disassemble(opcode, 0);
        assert!(result.contains("ecall"));
    }
}

#[cfg(test)]
mod gpio_tests {
    use rp2350sim_devices::gpio::{Gpio, GpioFunction, GPIO_BASE};

    #[test]
    fn test_gpio_creation() {
        let gpio = Gpio::new();
        assert_eq!(gpio.pin_count(), 48);
    }

    #[test]
    fn test_gpio_direction() {
        let mut gpio = Gpio::new();
        gpio.set_dir(0, true);
        assert!(gpio.get_pin(0).unwrap().direction);
    }

    #[test]
    fn test_gpio_output() {
        let mut gpio = Gpio::new();
        gpio.set_dir(0, true);
        gpio.set_output(0, true);
        assert!(gpio.get_value(0));
    }

    #[test]
    fn test_gpio_input() {
        let mut gpio = Gpio::new();
        gpio.set_input(0, true);
        assert!(gpio.get_value(0));
    }

    #[test]
    fn test_gpio_function() {
        let mut gpio = Gpio::new();
        gpio.get_pin_mut(0).unwrap().function = GpioFunction::Uart;
        assert_eq!(gpio.get_pin(0).unwrap().function, GpioFunction::Uart);
    }
}

#[cfg(test)]
mod uart_tests {
    use rp2350sim_devices::uart::{Uart, UART0_BASE, cr};

    #[test]
    fn test_uart_creation() {
        let uart = Uart::uart0();
        assert_eq!(uart.base(), UART0_BASE);
    }

    #[test]
    fn test_uart_enable() {
        let mut uart = Uart::uart0();
        assert!(!uart.is_enabled());
        uart.enable();
        assert!(uart.is_enabled());
    }

    #[test]
    fn test_uart_tx() {
        let mut uart = Uart::uart0();
        uart.enable_tx();
        uart.write_byte(0x41);
        // TX FIFO should have the byte
    }

    #[test]
    fn test_uart_rx() {
        let mut uart = Uart::uart0();
        uart.enable_rx();
        uart.push_rx(0x42);
        let byte = uart.read_byte();
        assert_eq!(byte, Some(0x42));
    }
}

#[cfg(test)]
mod timer_tests {
    use rp2350sim_devices::timer::Timer;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new();
        assert_eq!(timer.get_time(), 0);
    }

    #[test]
    fn test_timer_tick() {
        let mut timer = Timer::new();
        timer.tick();
        assert_eq!(timer.get_time(), 1);
    }

    #[test]
    fn test_timer_advance() {
        let mut timer = Timer::new();
        timer.advance(100);
        assert_eq!(timer.get_time(), 100);
    }

    #[test]
    fn test_timer_alarm() {
        let mut timer = Timer::new();
        timer.set_alarm(0, 100);
        assert!(timer.is_armed(0));
        timer.advance(100);
        assert!(!timer.is_armed(0));
        assert!(timer.has_interrupt());
    }
}

// ==================== ARM Thumb Execution Tests ====================

#[cfg(test)]
mod thumb_exec_tests {
    use rp2350sim_soc::Soc;
    use rp2350sim_core::CpuArch;

    /// Create a minimal SoC with ARM Cortex-M33 core
    fn create_arm_soc() -> Soc {
        Soc::new(CpuArch::Arm)
    }

    /// Load a simple program into SRAM and run for a few cycles
    #[test]
    fn test_thumb_exec_mov_imm() {
        let mut soc = create_arm_soc();
        
        // Load a simple program:
        // MOVS R0, #0x42    @ 0x20000000: 0x2042
        // B .               @ 0x20000002: 0xE7FE (infinite loop)
        let program: Vec<u16> = vec![0x2042, 0xE7FE];
        
        // Load program into SRAM at 0x20000000
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        // Set PC to start of program, SP to top of SRAM
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        // Run for a few cycles
        for _ in 0..10 {
            soc.step();
        }
        
        // Verify R0 = 0x42
        assert_eq!(soc.read_reg(0), 0x42, "R0 should be 0x42 after MOVS R0, #0x42");
    }

    #[test]
    fn test_thumb_exec_add_reg() {
        let mut soc = create_arm_soc();
        
        // ADDS R2, R0, R1  @ 0x20000000: 0x1842
        // B .              @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x1842, 0xE7FE];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        // Set initial register values
        soc.write_reg(0, 10);  // R0 = 10
        soc.write_reg(1, 20);  // R1 = 20
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        // R2 = R0 + R1 = 30
        assert_eq!(soc.read_reg(2), 30, "R2 should be 30 after ADDS R2, R0, R1");
    }

    #[test]
    fn test_thumb_exec_sub_reg() {
        let mut soc = create_arm_soc();
        
        // SUBS R2, R0, R1  @ 0x20000000: 0x1A42
        // B .              @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x1A42, 0xE7FE];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        soc.write_reg(0, 30);  // R0 = 30
        soc.write_reg(1, 10);  // R1 = 10
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        // R2 = R0 - R1 = 20
        assert_eq!(soc.read_reg(2), 20, "R2 should be 20 after SUBS R2, R0, R1");
    }

    #[test]
    fn test_thumb_exec_and() {
        let mut soc = create_arm_soc();
        
        // ANDS R0, R1  @ 0x20000000: 0x4008
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x4008, 0xE7FE];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        soc.write_reg(0, 0xFF);  // R0 = 0xFF
        soc.write_reg(1, 0x0F);  // R1 = 0x0F
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        // R0 = R0 & R1 = 0x0F
        assert_eq!(soc.read_reg(0), 0x0F, "R0 should be 0x0F after ANDS R0, R1");
    }

    #[test]
    fn test_thumb_exec_orr() {
        let mut soc = create_arm_soc();
        
        // ORRS R0, R1  @ 0x20000000: 0x4308
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x4308, 0xE7FE];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        soc.write_reg(0, 0xF0);  // R0 = 0xF0
        soc.write_reg(1, 0x0F);  // R1 = 0x0F
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        // R0 = R0 | R1 = 0xFF
        assert_eq!(soc.read_reg(0), 0xFF, "R0 should be 0xFF after ORRS R0, R1");
    }

    #[test]
    fn test_thumb_exec_eor() {
        let mut soc = create_arm_soc();
        
        // EORS R0, R1  @ 0x20000000: 0x4048
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x4048, 0xE7FE];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        soc.write_reg(0, 0xFF);  // R0 = 0xFF
        soc.write_reg(1, 0x0F);  // R1 = 0x0F
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        // R0 = R0 ^ R1 = 0xF0
        assert_eq!(soc.read_reg(0), 0xF0, "R0 should be 0xF0 after EORS R0, R1");
    }

    #[test]
    fn test_thumb_exec_lsl() {
        let mut soc = create_arm_soc();
        
        // LSLS R0, R0, #4  @ 0x20000000: 0x0100
        // B .              @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x0100, 0xE7FE];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        soc.write_reg(0, 0x01);  // R0 = 1
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        // R0 = R0 << 4 = 0x10
        assert_eq!(soc.read_reg(0), 0x10, "R0 should be 0x10 after LSLS R0, R0, #4");
    }

    #[test]
    fn test_thumb_exec_lsr() {
        let mut soc = create_arm_soc();
        
        // LSRS R0, R0, #4  @ 0x20000000: 0x0900
        // B .              @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x0900, 0xE7FE];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        soc.write_reg(0, 0x10);  // R0 = 0x10
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        // R0 = R0 >> 4 = 0x01
        assert_eq!(soc.read_reg(0), 0x01, "R0 should be 0x01 after LSRS R0, R0, #4");
    }

    #[test]
    fn test_thumb_exec_str_ldr() {
        let mut soc = create_arm_soc();
        
        // STR R0, [R1]    @ 0x20000000: 0x6008
        // LDR R2, [R1]    @ 0x20000002: 0x680A
        // B .             @ 0x20000004: 0xE7FE
        let program: Vec<u16> = vec![0x6008, 0x680A, 0xE7FE];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }
        
        soc.write_reg(0, 0xDEADBEEF);  // R0 = 0xDEADBEEF
        soc.write_reg(1, 0x20000100);  // R1 = address to store
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..15 {
            soc.step();
        }
        
        // R2 should contain the value stored
        assert_eq!(soc.read_reg(2), 0xDEADBEEF, "R2 should be 0xDEADBEEF after LDR");
    }

    #[test]
    fn test_thumb_exec_lsl_reg() {
        let mut soc = create_arm_soc();

        // LSLS R0, R1  @ 0x20000000: 0x4088
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x4088, 0xE7FE];

        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }

        soc.write_reg(0, 0x01);  // R0 = 0x01
        soc.write_reg(1, 4);     // R1 = 4 (shift amount)
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);

        for _ in 0..10 {
            soc.step();
        }

        // R0 = R0 << R1 = 0x10
        assert_eq!(soc.read_reg(0), 0x10, "R0 should be 0x10 after LSLS R0, R1");
    }

    #[test]
    fn test_thumb_exec_lsr_reg() {
        let mut soc = create_arm_soc();

        // LSRS R0, R1  @ 0x20000000: 0x40C8
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x40C8, 0xE7FE];

        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }

        soc.write_reg(0, 0x100);  // R0 = 0x100
        soc.write_reg(1, 4);      // R1 = 4 (shift amount)
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);

        for _ in 0..10 {
            soc.step();
        }

        // R0 = R0 >> R1 = 0x10
        assert_eq!(soc.read_reg(0), 0x10, "R0 should be 0x10 after LSRS R0, R1");
    }

    #[test]
    fn test_thumb_exec_asr_reg() {
        let mut soc = create_arm_soc();

        // ASRS R0, R1  @ 0x20000000: 0x4108
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x4108, 0xE7FE];

        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }

        soc.write_reg(0, 0x80000000);  // R0 = negative value
        soc.write_reg(1, 4);           // R1 = 4 (shift amount)
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);

        for _ in 0..10 {
            soc.step();
        }

        // R0 = R0 >> R1 (arithmetic) = 0xF8000000
        assert_eq!(soc.read_reg(0), 0xF8000000, "R0 should be 0xF8000000 after ASRS R0, R1");
    }

    #[test]
    fn test_thumb_exec_mul() {
        let mut soc = create_arm_soc();

        // MULS R0, R1  @ 0x20000000: 0x4348
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x4348, 0xE7FE];

        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }

        soc.write_reg(0, 6);   // R0 = 6
        soc.write_reg(1, 7);   // R1 = 7
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);

        for _ in 0..10 {
            soc.step();
        }

        // R0 = R0 * R1 = 42
        assert_eq!(soc.read_reg(0), 42, "R0 should be 42 after MULS R0, R1");
    }

    #[test]
    fn test_thumb_exec_neg() {
        let mut soc = create_arm_soc();

        // NEGS R0, R1  @ 0x20000000: 0x4248
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x4248, 0xE7FE];

        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }

        soc.write_reg(1, 42);  // R1 = 42
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);

        for _ in 0..10 {
            soc.step();
        }

        // R0 = -R1 = -42 (as unsigned: 0xFFFFFFD6)
        assert_eq!(soc.read_reg(0), 0xFFFFFFD6, "R0 should be -42 after NEGS R0, R1");
    }

    #[test]
    fn test_thumb_exec_bic() {
        let mut soc = create_arm_soc();

        // BICS R0, R1  @ 0x20000000: 0x4388
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x4388, 0xE7FE];

        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }

        soc.write_reg(0, 0xFF);  // R0 = 0xFF
        soc.write_reg(1, 0x0F);  // R1 = 0x0F
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);

        for _ in 0..10 {
            soc.step();
        }

        // R0 = R0 & ~R1 = 0xF0
        assert_eq!(soc.read_reg(0), 0xF0, "R0 should be 0xF0 after BICS R0, R1");
    }

    #[test]
    fn test_thumb_exec_mvn() {
        let mut soc = create_arm_soc();

        // MVNS R0, R1  @ 0x20000000: 0x43C8
        // B .          @ 0x20000002: 0xE7FE
        let program: Vec<u16> = vec![0x43C8, 0xE7FE];

        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 2) as u32;
            soc.write_mem_16(addr, instr);
        }

        soc.write_reg(1, 0x0000FFFF);  // R1 = 0x0000FFFF
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);

        for _ in 0..10 {
            soc.step();
        }

        // R0 = ~R1 = 0xFFFF0000
        assert_eq!(soc.read_reg(0), 0xFFFF0000, "R0 should be 0xFFFF0000 after MVNS R0, R1");
    }
}

// ==================== Thumb-2 (32-bit) Tests ====================

#[cfg(test)]
mod thumb2_tests {
    use rp2350sim_cpu_arm::thumb::decode::{decode_32bit, ThumbKind};

    #[test]
    fn test_decode_thumb2_mov_reg32() {
        // MOV.W R0, R1
        // Encoding: EA4F 0001
        let hw1 = 0xEA4F;
        let hw2 = 0x0001;
        let instr = decode_32bit(hw1, hw2);
        assert_eq!(instr.kind, ThumbKind::MovReg32);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
    }

    #[test]
    fn test_decode_thumb2_mvn_reg32() {
        // MVN.W R0, R1
        // Encoding: EA6F 0001
        let hw1 = 0xEA6F;
        let hw2 = 0x0001;
        let instr = decode_32bit(hw1, hw2);
        assert_eq!(instr.kind, ThumbKind::MvnReg32);
    }

    #[test]
    fn test_decode_thumb2_mov_imm32() {
        // MOV.W R0, #0x1234
        // Encoding: F44F 7034 (MOVS.W Rd, #imm12)
        // hw1 = 0xF44F, hw2 = 0x7034
        let hw1 = 0xF44F;
        let hw2 = 0x7034;
        let instr = decode_32bit(hw1, hw2);
        assert_eq!(instr.kind, ThumbKind::MovImm32);
        assert_eq!(instr.rd, 0);
    }

    #[test]
    fn test_decode_thumb2_bl32() {
        // BL <label> - branch with link
        // Encoding: F000 F800 (BL +0)
        let hw1 = 0xF000;
        let hw2 = 0xF800;
        let instr = decode_32bit(hw1, hw2);
        assert!(instr.is_branch);
        assert!(instr.is_call);
    }
}

// ==================== RV32 Execution Tests ====================

#[cfg(test)]
mod rv32_exec_tests {
    use rp2350sim_soc::Soc;
    use rp2350sim_core::CpuArch;

    /// Create a minimal SoC with RISC-V Hazard3 core
    fn create_rv32_soc() -> Soc {
        Soc::new(CpuArch::Hazard3)
    }

    #[test]
    fn test_rv32_exec_lui() {
        let mut soc = create_rv32_soc();
        
        // LUI x1, 0x12345    @ 0x20000000
        // JAL x0, 0          @ 0x20000004 (infinite loop)
        let program: Vec<u32> = vec![
            0x123450B7,  // LUI x1, 0x12345
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        // x1 should be 0x12345000
        assert_eq!(soc.read_reg(1), 0x12345000, "x1 should be 0x12345000 after LUI");
    }

    #[test]
    fn test_rv32_exec_addi() {
        let mut soc = create_rv32_soc();
        
        // ADDI x1, x0, 42    @ 0x20000000
        // JAL x0, 0          @ 0x20000004
        let program: Vec<u32> = vec![
            0x02A00093,  // ADDI x1, x0, 42
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..10 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(1), 42, "x1 should be 42 after ADDI");
    }

    #[test]
    fn test_rv32_exec_add() {
        let mut soc = create_rv32_soc();
        
        // ADDI x1, x0, 10    @ 0x20000000
        // ADDI x2, x0, 20    @ 0x20000004
        // ADD x3, x1, x2     @ 0x20000008
        // JAL x0, 0          @ 0x2000000C
        let program: Vec<u32> = vec![
            0x00A00093,  // ADDI x1, x0, 10
            0x01400113,  // ADDI x2, x0, 20
            0x002081B3,  // ADD x3, x1, x2
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..20 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(3), 30, "x3 should be 30 after ADD");
    }

    #[test]
    fn test_rv32_exec_sub() {
        let mut soc = create_rv32_soc();
        
        // ADDI x1, x0, 30    @ 0x20000000
        // ADDI x2, x0, 10    @ 0x20000004
        // SUB x3, x1, x2     @ 0x20000008
        // JAL x0, 0          @ 0x2000000C
        let program: Vec<u32> = vec![
            0x01E00093,  // ADDI x1, x0, 30
            0x00A00113,  // ADDI x2, x0, 10
            0x402081B3,  // SUB x3, x1, x2
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..20 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(3), 20, "x3 should be 20 after SUB");
    }

    #[test]
    fn test_rv32_exec_and() {
        let mut soc = create_rv32_soc();
        
        // LUI x1, 0xFF       @ 0x20000000
        // ADDI x2, x0, 0x0F  @ 0x20000004
        // AND x3, x1, x2     @ 0x20000008
        // JAL x0, 0          @ 0x2000000C
        let program: Vec<u32> = vec![
            0x000FF0B7,  // LUI x1, 0xFF
            0x00F00113,  // ADDI x2, x0, 0x0F
            0x0020F1B3,  // AND x3, x1, x2
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..20 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(3), 0x00, "x3 should be 0x00 after AND");
    }

    #[test]
    fn test_rv32_exec_or() {
        let mut soc = create_rv32_soc();
        
        // LUI x1, 0xF0       @ 0x20000000
        // ADDI x2, x0, 0x0F  @ 0x20000004
        // OR x3, x1, x2      @ 0x20000008
        // JAL x0, 0          @ 0x2000000C
        let program: Vec<u32> = vec![
            0x000F00B7,  // LUI x1, 0xF0
            0x00F00113,  // ADDI x2, x0, 0x0F
            0x0020E1B3,  // OR x3, x1, x2
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..20 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(3), 0xF000F, "x3 should be 0xF000F after OR");
    }

    #[test]
    fn test_rv32_exec_xor() {
        let mut soc = create_rv32_soc();
        
        // LUI x1, 0xFF       @ 0x20000000
        // ADDI x2, x0, 0x0F  @ 0x20000004
        // XOR x3, x1, x2     @ 0x20000008
        // JAL x0, 0          @ 0x2000000C
        let program: Vec<u32> = vec![
            0x000FF0B7,  // LUI x1, 0xFF
            0x00F00113,  // ADDI x2, x0, 0x0F
            0x0020C1B3,  // XOR x3, x1, x2
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..20 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(3), 0xFF00F, "x3 should be 0xFF00F after XOR");
    }

    #[test]
    fn test_rv32_exec_slli() {
        let mut soc = create_rv32_soc();
        
        // ADDI x1, x0, 1     @ 0x20000000
        // SLLI x2, x1, 4     @ 0x20000004
        // JAL x0, 0          @ 0x20000008
        let program: Vec<u32> = vec![
            0x00100093,  // ADDI x1, x0, 1
            0x00409113,  // SLLI x2, x1, 4
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..15 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(2), 16, "x2 should be 16 after SLLI");
    }

    #[test]
    fn test_rv32_exec_srli() {
        let mut soc = create_rv32_soc();
        
        // ADDI x1, x0, 16    @ 0x20000000
        // SRLI x2, x1, 4     @ 0x20000004
        // JAL x0, 0          @ 0x20000008
        let program: Vec<u32> = vec![
            0x01000093,  // ADDI x1, x0, 16
            0x0040D113,  // SRLI x2, x1, 4
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..15 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(2), 1, "x2 should be 1 after SRLI");
    }

    #[test]
    fn test_rv32_exec_sw_lw() {
        let mut soc = create_rv32_soc();
        
        // LUI x1, 0xDEADB    @ 0x20000000
        // ADDI x1, x1, 0xEEF @ 0x20000004 (x1 = 0xDEADBEEF)
        // LUI x2, 0x20000    @ 0x20000008 (x2 = 0x20000000)
        // SW x1, 0(x2)       @ 0x2000000C
        // LW x3, 0(x2)       @ 0x20000010
        // JAL x0, 0          @ 0x20000014
        let program: Vec<u32> = vec![
            0xDEADC0B7,  // LUI x1, 0xDEADC
            0xEEF08093,  // ADDI x1, x1, 0xEEF
            0x20000137,  // LUI x2, 0x20000
            0x00112023,  // SW x1, 0(x2)
            0x00012183,  // LW x3, 0(x2)
            0x0000006F,  // JAL x0, 0
        ];
        
        for (i, &instr) in program.iter().enumerate() {
            let addr = 0x20000000 + (i * 4) as u32;
            soc.write_mem_32(addr, instr);
        }
        
        soc.set_pc(0x20000000);
        soc.set_sp(0x20040000);
        
        for _ in 0..30 {
            soc.step();
        }
        
        assert_eq!(soc.read_reg(3), 0xDEADBEEF, "x3 should be 0xDEADBEEF after LW");
    }
}
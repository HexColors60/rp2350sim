#![allow(dead_code)]

//! ARM Thumb instruction decoder.
//!
//! Supports decoding of Thumb-1 and Thumb-2 instruction sets.


/// Decoded Thumb instruction.
#[derive(Debug, Clone)]
pub struct ThumbInstruction {
    /// Raw opcode (first halfword for 32-bit instructions)
    pub opcode: u16,
    /// Second halfword for 32-bit Thumb-2 instructions
    pub opcode2: u16,
    /// Instruction length (2 or 4 bytes)
    pub length: u8,
    /// Instruction kind
    pub kind: ThumbKind,
    /// Destination register
    pub rd: u8,
    /// Source register 1
    pub rn: u8,
    /// Source register 2
    pub rm: u8,
    /// Immediate value
    pub imm: u32,
    /// Shift amount
    pub shift: u8,
    /// Shift type
    pub shift_type: ShiftType,
    /// Condition code (for conditional instructions)
    pub cond: u8,
    /// Whether the instruction writes to PC
    pub writes_pc: bool,
    /// Whether the instruction reads memory
    pub reads_mem: bool,
    /// Whether the instruction writes memory
    pub writes_mem: bool,
    /// Whether this is a branch
    pub is_branch: bool,
    /// Whether this is a call (BL)
    pub is_call: bool,
    /// Whether this is a 32-bit Thumb-2 instruction
    pub is_thumb2: bool,
}

impl Default for ThumbInstruction {
    fn default() -> Self {
        Self {
            opcode: 0,
            opcode2: 0,
            length: 2,
            kind: ThumbKind::Nop,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: 0,
            shift: 0,
            shift_type: ShiftType::Lsl,
            cond: 0xE, // Always
            writes_pc: false,
            reads_mem: false,
            writes_mem: false,
            is_branch: false,
            is_call: false,
            is_thumb2: false,
        }
    }
}

/// Thumb instruction kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbKind {
    // Shift, add, subtract, move, and compare
    Lsl,
    Lsr,
    Asr,
    AddReg,
    SubReg,
    AddImm3,
    SubImm3,
    MovImm,
    CmpImm,
    AddImm8,
    SubImm8,

    // Data processing
    And,
    Eor,
    LslReg,
    LsrReg,
    AsrReg,
    Adc,
    Sbc,
    Ror,
    Tst,
    Neg,
    Cmp,
    Cmn,
    Orr,
    Orn,
    Mul,
    Bic,
    Mvn,

    // Special data processing and branch/exchange
    AddHigh,
    CmpHigh,
    MovHigh,
    Bx,
    Blx,

    // Load from literal pool
    LdrLit,

    // Load/store register offset
    LdrReg,
    LdrhReg,
    LdrbReg,
    LdrshReg,
    LdrsbReg,
    StrReg,
    StrhReg,
    StrbReg,

    // Load/store word immediate
    LdrImm5,
    StrImm5,

    // Load/store byte immediate
    LdrbImm5,
    StrbImm5,

    // Load/store halfword immediate
    LdrhImm5,
    StrhImm5,

    // Load/store SP-relative
    LdrSp,
    StrSp,

    // Load address
    Adr,

    // Add to SP or PC
    AddSpImm,
    AddSpReg,

    // Miscellaneous
    Push,
    Pop,
    Setend,
    Cps,

    // Extend and add
    Sxth,
    Sxtb,
    Uxth,
    Uxtb,
    Cbz,
    Cbnz,

    // Multiply
    Smlad,
    Smlsd,
    Smuad,
    Smusd,
    Smlald,
    Smlsld,
    Smmla,
    Smmls,
    Smmul,

    // Long multiply
    Umull,
    Umlal,
    Smull,
    Smlal,

    // Branch
    B,
    BCond,
    Bl,
    BlxReg,

    // IT block (If-Then)
    It,

    // Unconditional
    Nop,
    Wfi,
    Wfe,
    Sev,
    Yield,
    Dmb,
    Dsb,
    Isb,

    // 32-bit Thumb-2 instructions
    // Data processing (32-bit)
    AddImm32,
    SubImm32,
    AndImm32,
    OrrImm32,
    EorImm32,
    BicImm32,
    MovImm32,
    MvnImm32,
    AddReg32,
    SubReg32,
    AndReg32,
    OrrReg32,
    EorReg32,
    BicReg32,
    MovReg32,
    MvnReg32,
    LslImm32,
    LsrImm32,
    AsrImm32,
    RorImm32,

    // Load/store (32-bit)
    LdrImm32,
    LdrbImm32,
    LdrhImm32,
    LdrsbImm32,
    LdrshImm32,
    StrImm32,
    StrbImm32,
    StrhImm32,
    LdrReg32,
    StrReg32,
    Ldm32,
    Stm32,
    Push32,
    Pop32,

    // Branch (32-bit)
    Bl32,
    Blx32,
    Beq32,
    Bne32,
    Bge32,
    Blt32,
    Bgt32,
    Ble32,

    // Multiply (32-bit)
    Mul32,
    Mla32,
    Umlal32,
    Smlal32,

    // Division (ARMv7-M)
    Sdiv,
    Udiv,

    // Bit field operations
    Bfi,
    Bfc,
    Ubfx,
    Sbfx,

    // Count leading zeros
    Clz,

    // Reverse operations
    Rev,
    Rev16,
    Revsh,
    Rbit,

    // 32-bit Thumb-2 (generic)
    Thumb2,

    // Unknown
    Unknown,
}

/// Shift type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShiftType {
    #[default]
    Lsl,
    Lsr,
    Asr,
    Ror,
}

/// Condition codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    Eq = 0x0,
    Ne = 0x1,
    Cs = 0x2,
    Cc = 0x3,
    Mi = 0x4,
    Pl = 0x5,
    Vs = 0x6,
    Vc = 0x7,
    Hi = 0x8,
    Ls = 0x9,
    Ge = 0xA,
    Lt = 0xB,
    Gt = 0xC,
    Le = 0xD,
    Al = 0xE,
    Nv = 0xF,
}

impl Condition {
    pub fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Eq,
            0x1 => Self::Ne,
            0x2 => Self::Cs,
            0x3 => Self::Cc,
            0x4 => Self::Mi,
            0x5 => Self::Pl,
            0x6 => Self::Vs,
            0x7 => Self::Vc,
            0x8 => Self::Hi,
            0x9 => Self::Ls,
            0xA => Self::Ge,
            0xB => Self::Lt,
            0xC => Self::Gt,
            0xD => Self::Le,
            0xE => Self::Al,
            _ => Self::Nv,
        }
    }
}

/// Decode a 16-bit Thumb instruction.
pub fn decode(opcode: u16) -> ThumbInstruction {
    let op1 = (opcode >> 11) & 0x1F;

    match op1 {
        // Shift, add, subtract, move, and compare
        0b00000 => decode_shift_add_sub(opcode, true),  // LSL immediate
        0b00001 => decode_shift_add_sub(opcode, true),  // LSR immediate
        0b00010 => decode_shift_add_sub(opcode, true),  // ASR immediate
        0b00011 => decode_shift_add_sub(opcode, false), // Add/sub register
        0b00100 => decode_mov_cmp_add_sub_imm(opcode, 0), // MOV immediate
        0b00101 => decode_mov_cmp_add_sub_imm(opcode, 1), // CMP immediate
        0b00110 => decode_mov_cmp_add_sub_imm(opcode, 2), // ADD immediate (8-bit)
        0b00111 => decode_mov_cmp_add_sub_imm(opcode, 3), // SUB immediate (8-bit)

        // Data processing
        0b01000 => decode_data_processing(opcode),

        // Load from literal pool (PC-relative)
        0b01001 => decode_ldr_lit(opcode),

        // Load/store register offset: 0101 xxx Rm Rn Rt
        // This includes STR, STRH, STRB, LDRSB, LDR, LDRH, LDRB, LDRSH
        0b01010 | 0b01011 => decode_load_store_reg(opcode),

        // Load/store word immediate: 0110 B imm5 Rn Rt
        0b01100 | 0b01101 => decode_load_store_word_imm(opcode),

        // Load/store byte/halfword immediate: 0111 S H imm5 Rn Rt
        0b01110 | 0b01111 => decode_load_store_bh_imm(opcode),

        // STR SP-relative: 10010 Rt imm8
        0b10010 => decode_ldr_str_sp(opcode, false),

        // LDR SP-relative: 10011 Rt imm8
        0b10011 => decode_ldr_str_sp(opcode, true),

        // Misc - ADD Rd, PC/SP, #imm
        0b10100 => decode_misc_1(opcode),

        // Misc - PUSH/POP, CBZ/CBNZ, extend
        0b10101 | 0b10110 | 0b10111 => decode_misc_group(opcode),

        // Load/store multiple
        0b11000 => decode_ldm_stm(opcode, false), // STM
        0b11001 => decode_ldm_stm(opcode, true),  // LDM

        // Conditional branch (cond 0-7) and CBZ/CBNZ
        0b11010 => decode_b_cond_or_cbz(opcode),

        // Conditional branch (cond 8-15) and misc
        0b11011 => decode_b_cond_misc(opcode),

        // Unconditional branch
        0b11100 => decode_b(opcode),

        // BL/BLX prefix - 32-bit instructions
        0b11101 | 0b11110 | 0b11111 => {
            // This is the first halfword of a 32-bit Thumb-2 instruction
            // Return a placeholder that indicates we need the second halfword
            ThumbInstruction {
                opcode,
                kind: ThumbKind::Thumb2,
                length: 4,
                is_thumb2: true,
                ..Default::default()
            }
        }

        _ => ThumbInstruction {
            opcode,
            kind: ThumbKind::Unknown,
            ..Default::default()
        },
    }
}

/// Decode a 32-bit Thumb-2 instruction.
/// Takes both halfwords (hw1 is the first halfword at lower address).
pub fn decode_32bit(hw1: u16, hw2: u16) -> ThumbInstruction {
    // Check the prefix bits in hw1 (bits [15:11])
    let op1 = (hw1 >> 11) & 0x1F;

    match op1 {
        // 11101: Data processing (register, immediate, etc.)
        0b11101 => decode_thumb2_data_processing(hw1, hw2),

        // 11110: BL, BLX, and data processing (immediate)
        0b11110 => {
            // Check if this is BL/BLX or data processing
            // BL/BLX: hw1[9] = 0 (BLX) or 1 (BL), hw1[14:12] = imm10
            // Data processing: hw1[10:8] = op
            let is_bl_blx = (hw1 & 0x0600) == 0x0000 || (hw1 & 0x0600) == 0x0200;
            if is_bl_blx {
                decode_thumb2_bl_blx(hw1, hw2)
            } else {
                decode_thumb2_data_processing(hw1, hw2)
            }
        }

        // 11111: Load/store and branches
        0b11111 => {
            let op = (hw2 >> 15) & 0x1;
            if op == 0 {
                decode_thumb2_load_store(hw1, hw2)
            } else {
                decode_thumb2_branch(hw1, hw2)
            }
        }
        _ => ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind: ThumbKind::Unknown,
            length: 4,
            is_thumb2: true,
            ..Default::default()
        },
    }
}

/// Decode BL/BLX instructions (32-bit)
fn decode_thumb2_bl_blx(hw1: u16, hw2: u16) -> ThumbInstruction {
    let s = ((hw1 >> 10) & 0x1) as u32;
    let imm10 = (hw1 & 0x3FF) as u32;
    let j1 = ((hw2 >> 13) & 0x1) as u32;
    let j2 = ((hw2 >> 11) & 0x1) as u32;
    let imm11 = (hw2 & 0x7FF) as u32;
    let is_blx = ((hw1 >> 9) & 0x1) == 0;

    // Calculate target address
    let i1 = !(j1 ^ s);
    let i2 = !(j2 ^ s);
    let imm32 = (s << 24) | (i1 << 23) | (i2 << 22) | (imm10 << 12) | (imm11 << 1);

    let offset = if s != 0 {
        (imm32 | 0xFE000000) as i32
    } else {
        imm32 as i32
    };

    ThumbInstruction {
        opcode: hw1,
        opcode2: hw2,
        kind: if is_blx { ThumbKind::Blx32 } else { ThumbKind::Bl32 },
        length: 4,
        imm: offset as u32,
        is_branch: true,
        is_call: true,
        is_thumb2: true,
        ..Default::default()
    }
}

/// Decode 32-bit data processing instructions
fn decode_thumb2_data_processing(hw1: u16, hw2: u16) -> ThumbInstruction {
    // Data processing (register): 1110 101x xxxx xxxx 0xxx xxxx xxxx xxxx
    // Two groups:
    // - 0xEAxx: AND(000), BIC(001), ORR(010), ORN(011), EOR(100), RBIT(101), MOV(110), MVN(111)
    // - 0xEBxx: ADD(000), ADC(001), SBC(010), SUB(011), RSB(100), ...
    // hw1[15:11] = 11101 (prefix)
    // hw1[10:8] = op
    // hw1[7:4] = Rn (4 bits)
    // hw1[3:0] = part of encoding (shift type, etc.)
    // hw2[15:12] = Rd
    // hw2[3:0] = Rm
    if (hw1 & 0xFC00) == 0xEA00 && (hw2 & 0x8000) == 0 {
        // Group 1: AND, BIC, ORR, ORN, EOR, etc.
        let op = ((hw1 >> 8) & 0x7) as u8;  // bits 10:8
        let rn = ((hw1 >> 4) & 0xF) as u8;  // bits 7:4
        let rd = ((hw2 >> 12) & 0xF) as u8; // bits 15:12 of hw2
        let rm = (hw2 & 0xF) as u8;         // bits 3:0 of hw2

        let kind = match op {
            0b000 => ThumbKind::AndReg32,
            0b001 => ThumbKind::BicReg32,
            0b010 => ThumbKind::OrrReg32,
            0b011 => ThumbKind::Orn,  // ORN
            0b100 => ThumbKind::EorReg32,
            0b110 => ThumbKind::MovReg32,  // MOV (when Rn=15)
            0b111 => ThumbKind::MvnReg32,  // MVN (when Rn=15)
            _ => ThumbKind::Unknown,
        };

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind,
            length: 4,
            rd,
            rn,
            rm,
            writes_pc: rd == 15,
            is_thumb2: true,
            ..Default::default()
        };
    }

    if (hw1 & 0xFC00) == 0xEB00 && (hw2 & 0x8000) == 0 {
        // Group 2: ADD, ADC, SBC, SUB, RSB
        let op = ((hw1 >> 8) & 0x7) as u8;  // bits 10:8
        let rn = ((hw1 >> 4) & 0xF) as u8;  // bits 7:4
        let rd = ((hw2 >> 12) & 0xF) as u8; // bits 15:12 of hw2
        let rm = (hw2 & 0xF) as u8;         // bits 3:0 of hw2

        let kind = match op {
            0b000 => ThumbKind::AddReg32,
            0b001 => ThumbKind::Adc,  // ADC
            0b010 => ThumbKind::Sbc,  // SBC
            0b011 => ThumbKind::SubReg32,
            _ => ThumbKind::Unknown,
        };

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind,
            length: 4,
            rd,
            rn,
            rm,
            writes_pc: rd == 15,
            is_thumb2: true,
            ..Default::default()
        };
    }

    // MOV/MVN register: 1110 1010 001S 1111 0000 xxxx xxxx xxxx
    // hw1 = 0xEA4F (MOV) or 0xEA6F (MVN)
    if hw1 == 0xEA4F || hw1 == 0xEA6F {
        let is_mvn = hw1 == 0xEA6F;
        let rd = ((hw2 >> 12) & 0xF) as u8;
        let rm = (hw2 & 0xF) as u8;

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind: if is_mvn { ThumbKind::MvnReg32 } else { ThumbKind::MovReg32 },
            length: 4,
            rd,
            rm,
            is_thumb2: true,
            ..Default::default()
        };
    }

    // MOV immediate: 1111 0x00 0100 xxxx 0imm3 Rd imm8
    // hw1 = 0xF04F or 0xF24F or 0xF44F or 0xF64F
    if (hw1 & 0xFBFF) == 0xF04F {
        let rd = ((hw2 >> 8) & 0xF) as u8;
        let imm4 = (hw1 & 0xF) as u32;
        let i = ((hw1 >> 10) & 0x1) as u32;
        let imm3 = ((hw2 >> 12) & 0x7) as u32;
        let imm8 = (hw2 & 0xFF) as u32;
        let imm12 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind: ThumbKind::MovImm32,
            length: 4,
            rd,
            imm: imm12,
            is_thumb2: true,
            ..Default::default()
        };
    }

    // ADD/SUB immediate: 1111 0xi0 1000 xxxx 0imm3 Rd imm8
    // hw1 = 0xF20x (ADD) or 0xF60x (SUB)
    if (hw1 & 0xFBE0) == 0xF200 {
        let is_sub = (hw1 >> 9) & 0x1 != 0;
        let rn = (hw1 & 0xF) as u8;
        let rd = ((hw2 >> 8) & 0xF) as u8;
        let imm4 = (hw1 & 0xF) as u32;
        let i = ((hw1 >> 10) & 0x1) as u32;
        let imm3 = ((hw2 >> 12) & 0x7) as u32;
        let imm8 = (hw2 & 0xFF) as u32;
        let imm12 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind: if is_sub { ThumbKind::SubImm32 } else { ThumbKind::AddImm32 },
            length: 4,
            rd,
            rn,
            imm: imm12,
            is_thumb2: true,
            ..Default::default()
        };
    }

    // CLZ: 1110 1010 1011 1111 xxxx 1111 xxxx xxxx
    // hw1 = 0xFABF
    if hw1 == 0xFABF {
        let rd = ((hw2 >> 8) & 0xF) as u8;
        let rm = (hw2 & 0xF) as u8;

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind: ThumbKind::Clz,
            length: 4,
            rd,
            rm,
            is_thumb2: true,
            ..Default::default()
        };
    }

    // SDIV/UDIV: 1110 1011 100x xxxx 1111 xxxx xxxx xxxx
    // hw1 = 0xFB9x (SDIV) or 0xFBBx (UDIV)
    if (hw1 & 0xFFF0) == 0xFB90 || (hw1 & 0xFFF0) == 0xFBB0 {
        let is_signed = (hw1 >> 4) & 0x1 != 0;
        let rn = (hw1 & 0xF) as u8;
        let rd = ((hw2 >> 8) & 0xF) as u8;
        let rm = (hw2 & 0xF) as u8;

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind: if is_signed { ThumbKind::Sdiv } else { ThumbKind::Udiv },
            length: 4,
            rd,
            rn,
            rm,
            is_thumb2: true,
            ..Default::default()
        };
    }

    ThumbInstruction {
        opcode: hw1,
        opcode2: hw2,
        kind: ThumbKind::Thumb2,
        length: 4,
        is_thumb2: true,
        ..Default::default()
    }
}

/// Decode 32-bit load/store instructions
fn decode_thumb2_load_store(hw1: u16, hw2: u16) -> ThumbInstruction {
    // LDR/STR immediate: 1111 100x xxxx xxxx xxxx xxxx xxxx xxxx
    // hw1[15:11] = 11111, hw1[10:9] = 00, hw1[8] = L, hw1[7:6] = size
    // hw2[15:12] = Rn, hw2[11:0] = imm12
    if (hw1 & 0xFE00) == 0xF800 {
        let is_load = (hw1 >> 8) & 0x1 != 0;
        let size = ((hw1 >> 6) & 0x3) as u8;
        let _rn = ((hw2 >> 12) & 0xF) as u8;
        let _rt = (hw2 & 0xFFF) as u8;  // Actually need to extract Rt differently

        // Let me re-decode based on the actual encoding:
        // LDR.W Rt, [Rn, #imm12]: F8Dx xxxx
        // hw1 = 0xF8Dx, hw2 = Rn:Rt:imm12
        // hw1[15:11] = 11111
        // hw1[10:9] = 00
        // hw1[8] = 1 (load)
        // hw1[7:6] = 10 (word)
        // hw1[5:0] = part of encoding
        // hw2[15:12] = Rn
        // hw2[11:0] = imm12 (but Rt is somewhere in there)

        // Actually for LDR.W Rt, [Rn, #imm12]:
        // Encoding: 1111 1000 1101 Rnnn tttt iiii iiii iiii
        // hw1 = 1111 1000 1101 Rnnn = 0xF8Dn
        // hw2 = tttt iiii iiii iiii
        // Rt = hw2[15:12], imm12 = hw2[11:0]

        let rn = ((hw1 >> 0) & 0xF) as u8;
        let rt = ((hw2 >> 12) & 0xF) as u8;
        let imm12 = (hw2 & 0xFFF) as u32;

        let kind = match (size, is_load) {
            (0b10, true) => ThumbKind::LdrImm32,
            (0b10, false) => ThumbKind::StrImm32,
            (0b00, true) => ThumbKind::LdrbImm32,
            (0b00, false) => ThumbKind::StrbImm32,
            (0b01, true) => ThumbKind::LdrhImm32,
            (0b01, false) => ThumbKind::StrhImm32,
            _ => ThumbKind::Unknown,
        };

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind,
            length: 4,
            rd: rt,
            rn,
            imm: imm12,
            reads_mem: is_load,
            writes_mem: !is_load,
            is_thumb2: true,
            ..Default::default()
        };
    }

    ThumbInstruction {
        opcode: hw1,
        opcode2: hw2,
        kind: ThumbKind::Thumb2,
        length: 4,
        is_thumb2: true,
        ..Default::default()
    }
}

/// Decode 32-bit branch instructions
fn decode_thumb2_branch(hw1: u16, hw2: u16) -> ThumbInstruction {
    // Conditional branch: 1111 0S cond imm6 10 J1 J2 imm11
    // hw1 = 0xF0xx, hw2 = 10J1J2xxxxxxxx
    if (hw1 & 0xF800) == 0xF000 && (hw2 & 0xD000) == 0x8000 {
        let cond = ((hw1 >> 6) & 0xF) as u8;
        let s = ((hw1 >> 10) & 0x1) as u32;
        let imm6 = ((hw1 >> 0) & 0x3F) as u32;
        let j1 = ((hw2 >> 13) & 0x1) as u32;
        let j2 = ((hw2 >> 11) & 0x1) as u32;
        let imm11 = (hw2 & 0x7FF) as u32;

        let imm32 = (s << 20) | (j1 << 19) | (j2 << 18) | (imm6 << 12) | (imm11 << 1);
        let offset = if s != 0 {
            (imm32 | 0xFFF00000) as i32
        } else {
            imm32 as i32
        };

        let kind = match cond {
            0x0 => ThumbKind::Beq32,
            0x1 => ThumbKind::Bne32,
            0xA => ThumbKind::Bge32,
            0xB => ThumbKind::Blt32,
            0xC => ThumbKind::Bgt32,
            0xD => ThumbKind::Ble32,
            _ => ThumbKind::Thumb2,
        };

        return ThumbInstruction {
            opcode: hw1,
            opcode2: hw2,
            kind,
            length: 4,
            cond,
            imm: offset as u32,
            is_branch: true,
            is_thumb2: true,
            ..Default::default()
        };
    }

    ThumbInstruction {
        opcode: hw1,
        opcode2: hw2,
        kind: ThumbKind::Thumb2,
        length: 4,
        is_thumb2: true,
        ..Default::default()
    }
}

fn decode_shift_add_sub(opcode: u16, is_shift: bool) -> ThumbInstruction {
    let op = (opcode >> 11) & 0x3;
    let imm5 = ((opcode >> 6) & 0x1F) as u8;

    if is_shift {
        let rm = ((opcode >> 3) & 0x7) as u8;
        let rdn = (opcode & 0x7) as u8;
        let kind = match op {
            0b00 => ThumbKind::Lsl,
            0b01 => ThumbKind::Lsr,
            0b10 => ThumbKind::Asr,
            _ => ThumbKind::Unknown,
        };
        let shift_type = match op {
            0b00 => ShiftType::Lsl,
            0b01 => ShiftType::Lsr,
            0b10 => ShiftType::Asr,
            _ => ShiftType::Lsl,
        };
        ThumbInstruction {
            opcode,
            kind,
            rd: rdn,
            rn: rdn,
            rm,
            shift: imm5,
            shift_type,
            writes_pc: rdn == 15,
            ..Default::default()
        }
    } else {
        // Add/subtract register/immediate format:
        // Bits [15:9] = 0001100
        // Bit [10] = 0 for register, 1 for immediate (imm3)
        // Bit [9] = 0 for ADD, 1 for SUB
        // Bits [8:6] = Rm (register) or imm3 (immediate)
        // Bits [5:3] = Rn
        // Bits [2:0] = Rd
        let op_bit = (opcode >> 9) & 0x1;
        let is_imm = ((opcode >> 10) & 0x1) != 0;
        let rm_or_imm3 = ((opcode >> 6) & 0x7) as u8; // Bits [8:6]
        let rn = ((opcode >> 3) & 0x7) as u8;         // Bits [5:3]
        let rd = (opcode & 0x7) as u8;                // Bits [2:0]

        let kind = if op_bit == 0 {
            if is_imm { ThumbKind::AddImm3 } else { ThumbKind::AddReg }
        } else {
            if is_imm { ThumbKind::SubImm3 } else { ThumbKind::SubReg }
        };

        ThumbInstruction {
            opcode,
            kind,
            rd,
            rn,
            rm: rm_or_imm3,
            imm: rm_or_imm3 as u32,
            writes_pc: rd == 15,
            ..Default::default()
        }
    }
}

fn decode_mov_cmp_add_sub_imm(opcode: u16, op: u8) -> ThumbInstruction {
    let rdn = ((opcode >> 8) & 0x7) as u8;
    let imm8 = (opcode & 0xFF) as u32;

    let kind = match op {
        0 => ThumbKind::MovImm,
        1 => ThumbKind::CmpImm,
        2 => ThumbKind::AddImm8,
        3 => ThumbKind::SubImm8,
        _ => ThumbKind::Unknown,
    };

    ThumbInstruction {
        opcode,
        kind,
        rd: rdn,
        rn: rdn,
        imm: imm8,
        writes_pc: rdn == 15 && op != 1,
        ..Default::default()
    }
}

fn decode_data_processing(opcode: u16) -> ThumbInstruction {
    let op = ((opcode >> 6) & 0xF) as u8;
    let rm = ((opcode >> 3) & 0x7) as u8;
    let rdn = (opcode & 0x7) as u8;

    // Check for BX/BLX instructions (special encoding in this space)
    // BX Rm:  0100 0110 0 mmmm m000 (bits 15-7 = 0x46, Rm in bits 6-3)
    // BLX Rm: 0100 0111 0 mmmm m000 (bits 15-7 = 0x47, Rm in bits 6-3)
    let prefix = (opcode >> 7) & 0x1FF; // bits 15-7
    if prefix == 0b010001100 || prefix == 0b010001101 {
        // BX or BLX
        let is_blx = (opcode >> 7) & 1 == 1; // bit 7
        let rm_full = ((opcode >> 3) & 0xF) as u8; // Rm in bits 6-3
        
        return ThumbInstruction {
            opcode,
            kind: if is_blx { ThumbKind::Blx } else { ThumbKind::Bx },
            rm: rm_full,
            is_branch: true,
            ..Default::default()
        };
    }

    let kind = match op {
        0b0000 => ThumbKind::And,
        0b0001 => ThumbKind::Eor,
        0b0010 => ThumbKind::LslReg,
        0b0011 => ThumbKind::LsrReg,
        0b0100 => ThumbKind::AsrReg,
        0b0101 => ThumbKind::Adc,
        0b0110 => ThumbKind::Sbc,
        0b0111 => ThumbKind::Ror,
        0b1000 => ThumbKind::Tst,
        0b1001 => ThumbKind::Neg,
        0b1010 => ThumbKind::Cmp,
        0b1011 => ThumbKind::Cmn,
        0b1100 => ThumbKind::Orr,
        0b1101 => ThumbKind::Mul,
        0b1110 => ThumbKind::Bic,
        0b1111 => ThumbKind::Mvn,
        _ => ThumbKind::Unknown,
    };

    ThumbInstruction {
        opcode,
        kind,
        rd: rdn,
        rn: rdn,
        rm,
        writes_pc: rdn == 15,
        ..Default::default()
    }
}

fn decode_ldr_lit(opcode: u16) -> ThumbInstruction {
    let rt = ((opcode >> 8) & 0x7) as u8;
    let imm8 = ((opcode & 0xFF) as u32) << 2;

    ThumbInstruction {
        opcode,
        kind: ThumbKind::LdrLit,
        rd: rt,
        imm: imm8,
        reads_mem: true,
        ..Default::default()
    }
}

fn decode_ldr_str_sp(opcode: u16, is_load: bool) -> ThumbInstruction {
    let rt = ((opcode >> 8) & 0x7) as u8;
    let imm8 = ((opcode & 0xFF) as u32) << 2;

    ThumbInstruction {
        opcode,
        kind: if is_load { ThumbKind::LdrSp } else { ThumbKind::StrSp },
        rd: rt,
        imm: imm8,
        reads_mem: is_load,
        writes_mem: !is_load,
        ..Default::default()
    }
}

fn decode_misc_1(opcode: u16) -> ThumbInstruction {
    let _l = (opcode >> 11) & 0x1;
    let rd = ((opcode >> 8) & 0x7) as u8;
    let imm8 = ((opcode & 0xFF) as u32) << 2;

    // ADD Rd, PC, #imm8*4 or ADD Rd, SP, #imm8*4
    ThumbInstruction {
        opcode,
        kind: ThumbKind::Adr,
        rd,
        imm: imm8,
        ..Default::default()
    }
}

fn decode_misc_2(opcode: u16) -> ThumbInstruction {
    let pcl = (opcode >> 11) & 0x1;
    let m = (opcode >> 8) & 0x1;
    let register_list = (opcode & 0xFF) as u32;

    if pcl == 0 {
        // PUSH
        ThumbInstruction {
            opcode,
            kind: ThumbKind::Push,
            imm: register_list | if m != 0 { 1 << 14 } else { 0 },
            writes_mem: true,
            ..Default::default()
        }
    } else {
        // POP
        ThumbInstruction {
            opcode,
            kind: ThumbKind::Pop,
            imm: register_list | if m != 0 { 1 << 15 } else { 0 },
            reads_mem: true,
            writes_pc: m != 0,
            ..Default::default()
        }
    }
}

fn decode_push_pop(opcode: u16) -> ThumbInstruction {
    // PUSH: 1011 0100 M register_list
    // POP:  1011 1101 P register_list
    let op = (opcode >> 11) & 0x1; // 0 = PUSH, 1 = POP
    let m_or_p = (opcode >> 8) & 0x1;
    let register_list = (opcode & 0xFF) as u32;

    if op == 0 {
        // PUSH
        ThumbInstruction {
            opcode,
            kind: ThumbKind::Push,
            imm: register_list | if m_or_p != 0 { 1 << 14 } else { 0 }, // LR
            writes_mem: true,
            ..Default::default()
        }
    } else {
        // POP
        ThumbInstruction {
            opcode,
            kind: ThumbKind::Pop,
            imm: register_list | if m_or_p != 0 { 1 << 15 } else { 0 }, // PC
            reads_mem: true,
            writes_pc: m_or_p != 0,
            ..Default::default()
        }
    }
}

fn decode_ldm_stm(opcode: u16, is_load: bool) -> ThumbInstruction {
    let rn = ((opcode >> 8) & 0x7) as u8;
    let register_list = (opcode & 0xFF) as u32;

    ThumbInstruction {
        opcode,
        kind: if is_load { ThumbKind::LdrImm5 } else { ThumbKind::StrImm5 }, // Simplified
        rn,
        imm: register_list,
        reads_mem: is_load,
        writes_mem: !is_load,
        ..Default::default()
    }
}

fn decode_b_cond(opcode: u16) -> ThumbInstruction {
    let cond = ((opcode >> 8) & 0xF) as u8;
    let imm8 = (opcode & 0xFF) as u8;

    // Sign-extend 8-bit offset
    let offset = if imm8 & 0x80 != 0 {
        ((imm8 as u32) | 0xFFFFFF00) << 1
    } else {
        (imm8 as u32) << 1
    };

    ThumbInstruction {
        opcode,
        kind: ThumbKind::BCond,
        cond,
        imm: offset,
        is_branch: true,
        ..Default::default()
    }
}

fn decode_b(opcode: u16) -> ThumbInstruction {
    let imm11 = (opcode & 0x7FF) as u16;

    // Sign-extend 11-bit offset
    let offset = if imm11 & 0x400 != 0 {
        ((imm11 as u32) | 0xFFFFF800) << 1
    } else {
        (imm11 as u32) << 1
    };

    ThumbInstruction {
        opcode,
        kind: ThumbKind::B,
        imm: offset,
        is_branch: true,
        ..Default::default()
    }
}

fn decode_bl_prefix(opcode: u16) -> ThumbInstruction {
    let imm11 = (opcode & 0x7FF) as u16;

    // Sign-extend 11-bit offset (high part)
    let offset = if imm11 & 0x400 != 0 {
        ((imm11 as u32) | 0xFFFFF800) << 12
    } else {
        (imm11 as u32) << 12
    };

    ThumbInstruction {
        opcode,
        kind: ThumbKind::Bl,
        imm: offset,
        length: 4,
        is_branch: true,
        is_call: true,
        ..Default::default()
    }
}

fn decode_32bit_prefix(opcode: u16) -> ThumbInstruction {
    // This is the first halfword of a 32-bit Thumb-2 instruction
    ThumbInstruction {
        opcode,
        kind: ThumbKind::Thumb2,
        length: 4,
        ..Default::default()
    }
}

fn decode_misc_3(opcode: u16) -> ThumbInstruction {
    let _op1 = (opcode >> 12) & 0x1;
    let op2 = (opcode >> 8) & 0xF;

    match op2 {
        0b0000..=0b0011 => {
            // CBZ/CBNZ
            let rn = ((opcode >> 3) & 0x7) as u8;
            let imm5 = ((opcode >> 3) & 0x1F) as u32;
            let non_zero = (opcode >> 11) & 0x1;

            ThumbInstruction {
                opcode,
                kind: if non_zero != 0 { ThumbKind::Cbnz } else { ThumbKind::Cbz },
                rn,
                imm: imm5 << 1,
                is_branch: true,
                ..Default::default()
            }
        }
        _ => ThumbInstruction {
            opcode,
            kind: ThumbKind::Unknown,
            ..Default::default()
        },
    }
}

/// Decode load/store with register offset (bits 15:9 = 0101 xxx)
fn decode_load_store_reg(opcode: u16) -> ThumbInstruction {
    // Format: 0101 xxx Rm Rn Rt
    // Bits 15:9 = 0101 xxx where xxx determines the operation
    // 000 = STR, 001 = STRH, 010 = STRB, 011 = LDRSB
    // 100 = LDR, 101 = LDRH, 110 = LDRB, 111 = LDRSH
    let op_a = (opcode >> 9) & 0x7;
    let rm = ((opcode >> 6) & 0x7) as u8;
    let rn = ((opcode >> 3) & 0x7) as u8;
    let rt = (opcode & 0x7) as u8;

    let (kind, reads_mem, writes_mem) = match op_a {
        0b000 => (ThumbKind::StrReg, false, true),   // STR Rt, [Rn, Rm]
        0b001 => (ThumbKind::StrhReg, false, true),  // STRH Rt, [Rn, Rm]
        0b010 => (ThumbKind::StrbReg, false, true),  // STRB Rt, [Rn, Rm]
        0b011 => (ThumbKind::LdrsbReg, true, false), // LDRSB Rt, [Rn, Rm]
        0b100 => (ThumbKind::LdrReg, true, false),   // LDR Rt, [Rn, Rm]
        0b101 => (ThumbKind::LdrhReg, true, false),  // LDRH Rt, [Rn, Rm]
        0b110 => (ThumbKind::LdrbReg, true, false),  // LDRB Rt, [Rn, Rm]
        0b111 => (ThumbKind::LdrshReg, true, false), // LDRSH Rt, [Rn, Rm]
        _ => (ThumbKind::Unknown, false, false),
    };

    ThumbInstruction {
        opcode,
        kind,
        rd: rt,
        rn,
        rm,
        reads_mem,
        writes_mem,
        ..Default::default()
    }
}

/// Decode load/store word immediate (bits 15:12 = 0110)
fn decode_load_store_word_imm(opcode: u16) -> ThumbInstruction {
    // Format: 0110 B imm5 Rn Rt
    // B = 0: STR, B = 1: LDR
    let b = (opcode >> 11) & 0x1;
    let imm5 = ((opcode >> 6) & 0x1F) as u32 * 4; // Word-aligned offset
    let rn = ((opcode >> 3) & 0x7) as u8;
    let rt = (opcode & 0x7) as u8;

    let (kind, reads_mem, writes_mem) = if b == 0 {
        (ThumbKind::StrImm5, false, true)
    } else {
        (ThumbKind::LdrImm5, true, false)
    };

    ThumbInstruction {
        opcode,
        kind,
        rd: rt,
        rn,
        imm: imm5,
        reads_mem,
        writes_mem,
        ..Default::default()
    }
}

/// Decode load/store byte/halfword immediate (bits 15:12 = 0111)
fn decode_load_store_bh_imm(opcode: u16) -> ThumbInstruction {
    // Format: 0111 S H imm5 Rn Rt
    // S = 0: STR, S = 1: LDR
    // H = 0: Byte, H = 1: Halfword
    let s = (opcode >> 11) & 0x1;
    let h = (opcode >> 10) & 0x1;
    let imm5 = ((opcode >> 6) & 0x1F) as u32;
    let rn = ((opcode >> 3) & 0x7) as u8;
    let rt = (opcode & 0x7) as u8;

    let (kind, reads_mem, writes_mem) = match (s, h) {
        (0, 0) => (ThumbKind::StrbImm5, false, true),   // STRB
        (0, 1) => (ThumbKind::StrhImm5, false, true),   // STRH
        (1, 0) => (ThumbKind::LdrbImm5, true, false),   // LDRB
        (1, 1) => (ThumbKind::LdrhImm5, true, false),   // LDRH
        _ => (ThumbKind::Unknown, false, false),
    };

    // For halfword, offset is imm5 * 2
    let offset = if h == 1 { imm5 * 2 } else { imm5 };

    ThumbInstruction {
        opcode,
        kind,
        rd: rt,
        rn,
        imm: offset,
        reads_mem,
        writes_mem,
        ..Default::default()
    }
}

/// Decode misc group: PUSH/POP, CBZ/CBNZ, extend instructions
fn decode_misc_group(opcode: u16) -> ThumbInstruction {
    // Look at bits 15:8 to distinguish between instructions
    let op = (opcode >> 8) & 0xFF;

    match op {
        // PUSH: 1011 0100 M register_list
        0xB4 | 0xB5 => {
            let m = (opcode >> 8) & 0x1;
            let register_list = (opcode & 0xFF) as u32;
            ThumbInstruction {
                opcode,
                kind: ThumbKind::Push,
                imm: register_list | if m != 0 { 1 << 14 } else { 0 },
                writes_mem: true,
                ..Default::default()
            }
        }
        // POP: 1011 1101 P register_list (but 0xBD is POP with PC, 0xBC is POP without PC)
        0xBC | 0xBD => {
            let p = (opcode >> 8) & 0x1;
            let register_list = (opcode & 0xFF) as u32;
            ThumbInstruction {
                opcode,
                kind: ThumbKind::Pop,
                imm: register_list | if p != 0 { 1 << 15 } else { 0 },
                reads_mem: true,
                writes_pc: p != 0,
                ..Default::default()
            }
        }
        // SXTH/SXTB/UXTH/UXTB: 1011 0010 op Rm Rd
        0xB2 => {
            let op_bits = (opcode >> 6) & 0x3;
            let rm = ((opcode >> 3) & 0x7) as u8;
            let rd = (opcode & 0x7) as u8;
            let kind = match op_bits {
                0b00 => ThumbKind::Uxth,
                0b01 => ThumbKind::Sxth,
                0b10 => ThumbKind::Uxtb,
                0b11 => ThumbKind::Sxtb,
                _ => ThumbKind::Unknown,
            };
            ThumbInstruction {
                opcode,
                kind,
                rd,
                rm,
                ..Default::default()
            }
        }
        // CBZ: 1011 0001 imm5 Rn
        0xB1 => {
            let rn = (opcode & 0x7) as u8;
            let imm5 = (((opcode >> 3) & 0x1F) as u32) << 1;
            ThumbInstruction {
                opcode,
                kind: ThumbKind::Cbz,
                rn,
                imm: imm5,
                is_branch: true,
                ..Default::default()
            }
        }
        // CBNZ: 1011 1001 imm5 Rn
        0xB9 => {
            let rn = (opcode & 0x7) as u8;
            let imm5 = (((opcode >> 3) & 0x1F) as u32) << 1;
            ThumbInstruction {
                opcode,
                kind: ThumbKind::Cbnz,
                rn,
                imm: imm5,
                is_branch: true,
                ..Default::default()
            }
        }
        // IT: 1011 1111 cond firstcond mask
        // IT{x{y{z}}} <cond>
        0xBF => {
            let firstcond = ((opcode >> 4) & 0xF) as u8;
            let mask = (opcode & 0xF) as u8;
            ThumbInstruction {
                opcode,
                kind: ThumbKind::It,
                cond: firstcond,
                imm: mask as u32, // Store mask in imm field
                ..Default::default()
            }
        }
        _ => ThumbInstruction {
            opcode,
            kind: ThumbKind::Unknown,
            ..Default::default()
        },
    }
}

/// Decode conditional branch (cond 0-7) or CBZ/CBNZ
fn decode_b_cond_or_cbz(opcode: u16) -> ThumbInstruction {
    // Bits 15:11 = 11010
    // This is always conditional branch with cond 0-7
    // CBZ/CBNZ has bits 15:11 = 10110, not 11010
    decode_b_cond(opcode)
}

/// Decode conditional branch (cond 8-15) or misc
fn decode_b_cond_misc(opcode: u16) -> ThumbInstruction {
    // Bits 15:11 = 11011
    // This is always conditional branch with cond 8-15
    decode_b_cond(opcode)
}

/// Disassemble a Thumb instruction to a string.
pub fn disassemble(opcode: u16, pc: u32) -> String {
    let instr = decode(opcode);

    match instr.kind {
        ThumbKind::Lsl => format!("lsls r{}, r{}, #{}", instr.rd, instr.rn, instr.shift),
        ThumbKind::Lsr => format!("lsrs r{}, r{}, #{}", instr.rd, instr.rn, instr.shift),
        ThumbKind::Asr => format!("asrs r{}, r{}, #{}", instr.rd, instr.rn, instr.shift),
        ThumbKind::AddReg => format!("adds r{}, r{}, r{}", instr.rd, instr.rn, instr.rm),
        ThumbKind::SubReg => format!("subs r{}, r{}, r{}", instr.rd, instr.rn, instr.rm),
        ThumbKind::AddImm3 => format!("adds r{}, r{}, #{}", instr.rd, instr.rn, instr.imm),
        ThumbKind::SubImm3 => format!("subs r{}, r{}, #{}", instr.rd, instr.rn, instr.imm),
        ThumbKind::MovImm => format!("movs r{}, #{}", instr.rd, instr.imm),
        ThumbKind::CmpImm => format!("cmp r{}, #{}", instr.rd, instr.imm),
        ThumbKind::AddImm8 => format!("adds r{}, #{}", instr.rd, instr.imm),
        ThumbKind::SubImm8 => format!("subs r{}, #{}", instr.rd, instr.imm),
        ThumbKind::And => format!("ands r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Eor => format!("eors r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Orr => format!("orrs r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Bic => format!("bics r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Mvn => format!("mvns r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Tst => format!("tst r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Cmp => format!("cmp r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Cmn => format!("cmn r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Adc => format!("adcs r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Sbc => format!("sbcs r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Ror => format!("rors r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Neg => format!("negs r{}, r{}", instr.rd, instr.rm),
        ThumbKind::Mul => format!("muls r{}, r{}", instr.rd, instr.rm),
        ThumbKind::LdrLit => format!("ldr r{}, [pc, #{}]", instr.rd, instr.imm),
        ThumbKind::LdrSp => format!("ldr r{}, [sp, #{}]", instr.rd, instr.imm),
        ThumbKind::StrSp => format!("str r{}, [sp, #{}]", instr.rd, instr.imm),
        ThumbKind::Push => format!("push {{...}}"),
        ThumbKind::Pop => format!("pop {{...}}"),
        ThumbKind::B => {
            let target = (pc as i32 + 4 + (instr.imm as i32)) as u32;
            format!("b 0x{:08x}", target)
        }
        ThumbKind::BCond => {
            let cond_name = match Condition::from_bits(instr.cond) {
                Condition::Eq => "eq",
                Condition::Ne => "ne",
                Condition::Cs => "cs",
                Condition::Cc => "cc",
                Condition::Mi => "mi",
                Condition::Pl => "pl",
                Condition::Vs => "vs",
                Condition::Vc => "vc",
                Condition::Hi => "hi",
                Condition::Ls => "ls",
                Condition::Ge => "ge",
                Condition::Lt => "lt",
                Condition::Gt => "gt",
                Condition::Le => "le",
                _ => "",
            };
            let target = (pc as i32 + 4 + (instr.imm as i32)) as u32;
            format!("b{} 0x{:08x}", cond_name, target)
        }
        ThumbKind::Bl => format!("bl ..."),
        ThumbKind::Cbz => format!("cbz r{}, ...", instr.rn),
        ThumbKind::Cbnz => format!("cbnz r{}, ...", instr.rn),
        ThumbKind::It => {
            let cond_name = match Condition::from_bits(instr.cond) {
                Condition::Eq => "eq",
                Condition::Ne => "ne",
                Condition::Cs => "cs",
                Condition::Cc => "cc",
                Condition::Mi => "mi",
                Condition::Pl => "pl",
                Condition::Vs => "vs",
                Condition::Vc => "vc",
                Condition::Hi => "hi",
                Condition::Ls => "ls",
                Condition::Ge => "ge",
                Condition::Lt => "lt",
                Condition::Gt => "gt",
                Condition::Le => "le",
                _ => "al",
            };
            let mask = instr.imm as u8;
            // Determine IT block size from mask
            let size = if mask == 0x8 { 1 }
                       else if mask == 0x4 || mask == 0xC { 2 }
                       else if mask == 0x2 || mask == 0x6 || mask == 0xA || mask == 0xE { 3 }
                       else { 4 };
            let suffix = match size {
                1 => "".to_string(),
                2 => "t".to_string(),
                3 => "tt".to_string(),
                4 => "ttt".to_string(),
                _ => "".to_string(),
            };
            format!("it{} {}", suffix, cond_name)
        }
        ThumbKind::Nop => "nop".to_string(),
        ThumbKind::Thumb2 => format!(".word 0x{:04x}...", opcode),
        _ => format!(".word 0x{:04x}", opcode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_and() {
        // AND Rdn, Rm: 0100 00 0000 Rm Rdn
        // AND R0, R1: op=0000, Rm=001, Rdn=000
        // Binary: 0100 0000 0000 1000 = 0x4008
        let opcode = 0x4008u16;
        let instr = decode(opcode);
        
        assert_eq!(instr.kind, ThumbKind::And);
        assert_eq!(instr.rd, 0);  // Rdn = R0
        assert_eq!(instr.rm, 1);  // Rm = R1
        assert_eq!(instr.length, 2);
    }

    #[test]
    fn test_decode_add_reg() {
        // ADD Rd, Rn, Rm: 0001 10 0 0 Rm Rn Rd
        // ADD R2, R3, R1: Rm=001, Rn=011, Rd=010
        // Binary: 0001 1000 0101 1010 = 0x185A
        let opcode = 0x185A;
        let instr = decode(opcode);
        
        assert_eq!(instr.kind, ThumbKind::AddReg);
        assert_eq!(instr.rd, 2);  // Rd = R2
        assert_eq!(instr.rn, 3);  // Rn = R3
        assert_eq!(instr.rm, 1);  // Rm = R1
    }

    #[test]
    fn test_decode_mov_imm() {
        // MOVS Rd, #imm8: 0010 0 Rd imm8
        // MOVS R5, #100: Rd=101, imm8=01100100
        // Binary: 0010 0 101 0110 0100 = 0x2564
        let opcode = 0x2564;
        let instr = decode(opcode);
        
        assert_eq!(instr.kind, ThumbKind::MovImm);
        assert_eq!(instr.rd, 5);
        assert_eq!(instr.imm, 100);
    }

    #[test]
    fn test_decode_cmp_imm() {
        // CMP Rn, #imm8: 0010 1 Rn imm8
        // CMP R3, #42: Rn=011, imm8=00101010
        // Binary: 0010 1 011 0010 1010 = 0x2B2A
        let opcode = 0x2B2A;
        let instr = decode(opcode);
        
        assert_eq!(instr.kind, ThumbKind::CmpImm);
        assert_eq!(instr.rd, 3);  // CMP uses rd field for Rn
        assert_eq!(instr.imm, 42);
    }

    #[test]
    fn test_decode_mul() {
        // MUL Rdn, Rm: 0100 00 1101 Rm Rdn
        // MUL R4, R2: op=1101, Rm=010, Rdn=100
        // Binary: 0100 0011 0101 0100 = 0x4354
        let opcode = 0x4354;
        let instr = decode(opcode);
        
        assert_eq!(instr.kind, ThumbKind::Mul);
        assert_eq!(instr.rd, 4);  // Rdn = R4
        assert_eq!(instr.rm, 2);  // Rm = R2
    }

    #[test]
    fn test_decode_branch() {
        // B #offset: 1110 0 offset11
        // B #24: offset11 = 12 (24/2)
        // Binary: 1110 0000 0000 1100 = 0xE00C
        let opcode = 0xE00C;
        let instr = decode(opcode);
        
        assert_eq!(instr.kind, ThumbKind::B);
        assert!(instr.is_branch);
        assert_eq!(instr.imm, 24);
    }

    #[test]
    fn test_decode_lsl_imm() {
        // LSLS Rd, Rm, #imm5: 0000 0 imm5 Rm Rd
        // LSLS R2, R3, #4: imm5=00100, Rm=011, Rd=010
        // Binary: 0000 0001 0001 1010 = 0x011A
        let opcode = 0x011A;
        let instr = decode(opcode);
        
        assert_eq!(instr.kind, ThumbKind::Lsl);
        assert_eq!(instr.rd, 2);
        assert_eq!(instr.rm, 3);
        assert_eq!(instr.shift, 4);
    }

    #[test]
    fn test_decode_orr() {
        // ORR Rdn, Rm: 0100 00 1100 Rm Rdn
        // ORR R1, R5: op=1100, Rm=101, Rdn=001
        // Binary: 0100 0011 0010 1001 = 0x4329
        let opcode = 0x4329;
        let instr = decode(opcode);
        
        assert_eq!(instr.kind, ThumbKind::Orr);
        assert_eq!(instr.rd, 1);
        assert_eq!(instr.rm, 5);
    }
}
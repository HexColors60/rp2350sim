//! RV32 instruction decoder.
//!
//! Supports decoding of RV32IMC instruction set.



/// Decoded RV32 instruction.
#[derive(Debug, Clone)]
pub struct Rv32Instruction {
    /// Raw opcode
    pub opcode: u32,
    /// Instruction length (2 or 4 bytes)
    pub length: u8,
    /// Instruction kind
    pub kind: Rv32Kind,
    /// Destination register
    pub rd: u8,
    /// Source register 1
    pub rs1: u8,
    /// Source register 2
    pub rs2: u8,
    /// Immediate value
    pub imm: i32,
    /// Function 3
    pub funct3: u8,
    /// Function 7
    pub funct7: u8,
    /// Whether the instruction reads memory
    pub reads_mem: bool,
    /// Whether the instruction writes memory
    pub writes_mem: bool,
    /// Whether this is a branch
    pub is_branch: bool,
    /// Whether this is a call
    pub is_call: bool,
    /// Whether this is a return
    pub is_return: bool,
}

impl Default for Rv32Instruction {
    fn default() -> Self {
        Self {
            opcode: 0,
            length: 4,
            kind: Rv32Kind::Unknown,
            rd: 0,
            rs1: 0,
            rs2: 0,
            imm: 0,
            funct3: 0,
            funct7: 0,
            reads_mem: false,
            writes_mem: false,
            is_branch: false,
            is_call: false,
            is_return: false,
        }
    }
}

/// RV32 instruction kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rv32Kind {
    // LUI
    Lui,
    // AUIPC
    Auipc,
    // JAL
    Jal,
    // JALR
    Jalr,

    // Branch instructions
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,

    // Load instructions
    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,

    // Store instructions
    Sb,
    Sh,
    Sw,

    // Immediate arithmetic
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,

    // Register arithmetic
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,

    // Multiply/divide extension
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Rem,
    Remu,

    // Fence instructions
    Fence,
    FenceI,

    // System instructions
    Ecall,
    Ebreak,
    Csrrw,
    Csrrs,
    Csrrc,
    Csrrwi,
    Csrrsi,
    Csrrci,

    // Compressed instructions (RVC)
    CAddi4spn,
    CLw,
    CSw,
    CAddi,
    CLi,
    CAddi16sp,
    CLui,
    CSrli,
    CSrai,
    CAndi,
    CSub,
    CXor,
    COr,
    CAnd,
    CJ,
    CBeqz,
    CBnez,
    CLi2,
    CAddi16sp2,
    CLui2,
    CSlli,
    CLwsp,
    CJr,
    CMv,
    CEbreak,
    CJalr,
    CAdd,
    CSwsp,

    // Unknown
    Unknown,
}

/// Decode a 32-bit RV32 instruction.
pub fn decode(opcode: u32) -> Rv32Instruction {
    let op = opcode & 0x7F;
    let rd = ((opcode >> 7) & 0x1F) as u8;
    let funct3 = ((opcode >> 12) & 0x7) as u8;
    let rs1 = ((opcode >> 15) & 0x1F) as u8;
    let rs2 = ((opcode >> 20) & 0x1F) as u8;
    let funct7 = ((opcode >> 25) & 0x7F) as u8;

    match op {
        // LUI
        0b0110111 => {
            let imm = (opcode & 0xFFFFF000) as i32;
            Rv32Instruction {
                opcode,
                kind: Rv32Kind::Lui,
                rd,
                imm,
                ..Default::default()
            }
        }

        // AUIPC
        0b0010111 => {
            let imm = (opcode & 0xFFFFF000) as i32;
            Rv32Instruction {
                opcode,
                kind: Rv32Kind::Auipc,
                rd,
                imm,
                ..Default::default()
            }
        }

        // JAL
        0b1101111 => {
            let imm = decode_j_imm(opcode);
            Rv32Instruction {
                opcode,
                kind: Rv32Kind::Jal,
                rd,
                imm,
                is_branch: true,
                is_call: rd == 1, // ra
                ..Default::default()
            }
        }

        // JALR
        0b1100111 => {
            let imm = decode_i_imm(opcode);
            Rv32Instruction {
                opcode,
                kind: Rv32Kind::Jalr,
                rd,
                rs1,
                imm,
                is_branch: true,
                is_call: rd == 1,
                is_return: rs1 == 1 && rd == 0,
                ..Default::default()
            }
        }

        // Branch
        0b1100011 => {
            let imm = decode_b_imm(opcode);
            let kind = match funct3 {
                0b000 => Rv32Kind::Beq,
                0b001 => Rv32Kind::Bne,
                0b100 => Rv32Kind::Blt,
                0b101 => Rv32Kind::Bge,
                0b110 => Rv32Kind::Bltu,
                0b111 => Rv32Kind::Bgeu,
                _ => Rv32Kind::Unknown,
            };
            Rv32Instruction {
                opcode,
                kind,
                rs1,
                rs2,
                imm,
                funct3,
                is_branch: true,
                ..Default::default()
            }
        }

        // Load
        0b0000011 => {
            let imm = decode_i_imm(opcode);
            let kind = match funct3 {
                0b000 => Rv32Kind::Lb,
                0b001 => Rv32Kind::Lh,
                0b010 => Rv32Kind::Lw,
                0b100 => Rv32Kind::Lbu,
                0b101 => Rv32Kind::Lhu,
                _ => Rv32Kind::Unknown,
            };
            Rv32Instruction {
                opcode,
                kind,
                rd,
                rs1,
                imm,
                funct3,
                reads_mem: true,
                ..Default::default()
            }
        }

        // Store
        0b0100011 => {
            let imm = decode_s_imm(opcode);
            let kind = match funct3 {
                0b000 => Rv32Kind::Sb,
                0b001 => Rv32Kind::Sh,
                0b010 => Rv32Kind::Sw,
                _ => Rv32Kind::Unknown,
            };
            Rv32Instruction {
                opcode,
                kind,
                rs1,
                rs2,
                imm,
                funct3,
                writes_mem: true,
                ..Default::default()
            }
        }

        // Immediate arithmetic
        0b0010011 => {
            let imm = decode_i_imm(opcode);
            let shamt = (opcode >> 20) & 0x1F;
            let kind = match funct3 {
                0b000 => Rv32Kind::Addi,
                0b010 => Rv32Kind::Slti,
                0b011 => Rv32Kind::Sltiu,
                0b100 => Rv32Kind::Xori,
                0b110 => Rv32Kind::Ori,
                0b111 => Rv32Kind::Andi,
                0b001 => Rv32Kind::Slli,
                0b101 => if funct7 == 0 { Rv32Kind::Srli } else { Rv32Kind::Srai },
                _ => Rv32Kind::Unknown,
            };
            Rv32Instruction {
                opcode,
                kind,
                rd,
                rs1,
                imm: if funct3 == 0b001 || funct3 == 0b101 { shamt as i32 } else { imm },
                funct3,
                funct7,
                ..Default::default()
            }
        }

        // Register arithmetic
        0b0110011 => {
            let kind = match (funct3, funct7) {
                (0b000, 0b0000000) => Rv32Kind::Add,
                (0b000, 0b0100000) => Rv32Kind::Sub,
                (0b001, 0b0000000) => Rv32Kind::Sll,
                (0b010, 0b0000000) => Rv32Kind::Slt,
                (0b011, 0b0000000) => Rv32Kind::Sltu,
                (0b100, 0b0000000) => Rv32Kind::Xor,
                (0b101, 0b0000000) => Rv32Kind::Srl,
                (0b101, 0b0100000) => Rv32Kind::Sra,
                (0b110, 0b0000000) => Rv32Kind::Or,
                (0b111, 0b0000000) => Rv32Kind::And,
                // M extension
                (0b000, 0b0000001) => Rv32Kind::Mul,
                (0b001, 0b0000001) => Rv32Kind::Mulh,
                (0b010, 0b0000001) => Rv32Kind::Mulhsu,
                (0b011, 0b0000001) => Rv32Kind::Mulhu,
                (0b100, 0b0000001) => Rv32Kind::Div,
                (0b101, 0b0000001) => Rv32Kind::Divu,
                (0b110, 0b0000001) => Rv32Kind::Rem,
                (0b111, 0b0000001) => Rv32Kind::Remu,
                _ => Rv32Kind::Unknown,
            };
            Rv32Instruction {
                opcode,
                kind,
                rd,
                rs1,
                rs2,
                funct3,
                funct7,
                ..Default::default()
            }
        }

        // Fence
        0b0001111 => {
            let kind = if funct3 == 0b000 { Rv32Kind::Fence } else if funct3 == 0b001 { Rv32Kind::FenceI } else { Rv32Kind::Unknown };
            Rv32Instruction {
                opcode,
                kind,
                funct3,
                ..Default::default()
            }
        }

        // System
        0b1110011 => {
            let imm = decode_i_imm(opcode);
            let kind = match funct3 {
                0b000 => if imm == 0 { Rv32Kind::Ecall } else if imm == 1 { Rv32Kind::Ebreak } else { Rv32Kind::Unknown },
                0b001 => Rv32Kind::Csrrw,
                0b010 => Rv32Kind::Csrrs,
                0b011 => Rv32Kind::Csrrc,
                0b101 => Rv32Kind::Csrrwi,
                0b110 => Rv32Kind::Csrrsi,
                0b111 => Rv32Kind::Csrrci,
                _ => Rv32Kind::Unknown,
            };
            Rv32Instruction {
                opcode,
                kind,
                rd,
                rs1,
                imm,
                funct3,
                ..Default::default()
            }
        }

        _ => Rv32Instruction {
            opcode,
            kind: Rv32Kind::Unknown,
            ..Default::default()
        },
    }
}

/// Decode a 16-bit compressed instruction.
pub fn decode_compressed(opcode: u16) -> Rv32Instruction {
    let op = opcode & 0x3;
    let funct3 = ((opcode >> 13) & 0x7) as u8;

    match op {
        0b00 => {
            match funct3 {
                0b000 => {
                    // C.ADDI4SPN
                    let rd = ((opcode >> 2) & 0x7) as u8 + 8;
                    let imm = (((opcode >> 7) & 0x1F) as u32) << 2;
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CAddi4spn,
                        rd,
                        imm: imm as i32,
                        ..Default::default()
                    }
                }
                0b010 => {
                    // C.LW
                    let rd = ((opcode >> 2) & 0x7) as u8 + 8;
                    let rs1 = ((opcode >> 7) & 0x7) as u8 + 8;
                    let imm = (((opcode >> 10) & 0x7) << 2 | ((opcode >> 6) & 0x1) << 6) as i32;
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CLw,
                        rd,
                        rs1,
                        imm,
                        reads_mem: true,
                        ..Default::default()
                    }
                }
                0b110 => {
                    // C.SW
                    let rs2 = ((opcode >> 2) & 0x7) as u8 + 8;
                    let rs1 = ((opcode >> 7) & 0x7) as u8 + 8;
                    let imm = (((opcode >> 10) & 0x7) << 2 | ((opcode >> 6) & 0x1) << 6) as i32;
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CSw,
                        rs1,
                        rs2,
                        imm,
                        writes_mem: true,
                        ..Default::default()
                    }
                }
                _ => Rv32Instruction {
                    opcode: opcode as u32,
                    length: 2,
                    kind: Rv32Kind::Unknown,
                    ..Default::default()
                },
            }
        }
        0b01 => {
            match funct3 {
                0b000 => {
                    // C.ADDI / C.NOP
                    let rd = ((opcode >> 7) & 0x1F) as u8;
                    let imm = sign_extend_6((((opcode >> 2) & 0x1F) | ((opcode >> 12) << 5)) as u32);
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CAddi,
                        rd,
                        imm,
                        ..Default::default()
                    }
                }
                0b001 => {
                    // C.JAL
                    let imm = decode_cj_imm(opcode);
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CJ,
                        rd: 1, // ra
                        imm,
                        is_branch: true,
                        is_call: true,
                        ..Default::default()
                    }
                }
                0b010 => {
                    // C.LI
                    let rd = ((opcode >> 7) & 0x1F) as u8;
                    let imm = sign_extend_6((((opcode >> 2) & 0x1F) | ((opcode >> 12) << 5)) as u32);
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CLi,
                        rd,
                        imm,
                        ..Default::default()
                    }
                }
                0b011 => {
                    // C.ADDI16SP / C.LUI
                    let rd = ((opcode >> 7) & 0x1F) as u8;
                    if rd == 2 {
                        // C.ADDI16SP
                        let imm = sign_extend_10(
                            (((opcode >> 2) & 0x1) << 4 |
                            ((opcode >> 3) & 0x3) << 6 |
                            ((opcode >> 5) & 0x1) << 5 |
                            ((opcode >> 6) & 0x1) << 8 |
                            ((opcode >> 12) & 0x1) << 9) as u32
                        );
                        Rv32Instruction {
                            opcode: opcode as u32,
                            length: 2,
                            kind: Rv32Kind::CAddi16sp,
                            rd,
                            imm,
                            ..Default::default()
                        }
                    } else {
                        // C.LUI
                        let imm = sign_extend_6((((opcode >> 2) & 0x1F) | ((opcode >> 12) << 5)) as u32) << 12;
                        Rv32Instruction {
                            opcode: opcode as u32,
                            length: 2,
                            kind: Rv32Kind::CLui,
                            rd,
                            imm,
                            ..Default::default()
                        }
                    }
                }
                0b100 => {
                    // C.SRLI, C.SRAI, C.ANDI, C.SUB, C.XOR, C.OR, C.AND
                    let funct2 = ((opcode >> 10) & 0x3) as u8;
                    let rs1 = ((opcode >> 7) & 0x7) as u8 + 8;
                    match funct2 {
                        0b00 => Rv32Instruction { opcode: opcode as u32, length: 2, kind: Rv32Kind::CSrli, rs1, ..Default::default() },
                        0b01 => Rv32Instruction { opcode: opcode as u32, length: 2, kind: Rv32Kind::CSrai, rs1, ..Default::default() },
                        0b10 => {
                            let imm = sign_extend_6((((opcode >> 2) & 0x1F) | ((opcode >> 12) << 5)) as u32);
                            Rv32Instruction { opcode: opcode as u32, length: 2, kind: Rv32Kind::CAndi, rs1, imm, ..Default::default() }
                        }
                        0b11 => {
                            let funct2b = ((opcode >> 5) & 0x3) as u8;
                            let rs2 = ((opcode >> 2) & 0x7) as u8 + 8;
                            let kind = match funct2b {
                                0b00 => Rv32Kind::CSub,
                                0b01 => Rv32Kind::CXor,
                                0b10 => Rv32Kind::COr,
                                0b11 => Rv32Kind::CAnd,
                                _ => Rv32Kind::Unknown,
                            };
                            Rv32Instruction { opcode: opcode as u32, length: 2, kind, rs1, rs2, ..Default::default() }
                        }
                        _ => Rv32Instruction { opcode: opcode as u32, length: 2, kind: Rv32Kind::Unknown, ..Default::default() },
                    }
                }
                0b101 => {
                    // C.J
                    let imm = decode_cj_imm(opcode);
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CJ,
                        imm,
                        is_branch: true,
                        ..Default::default()
                    }
                }
                0b110 => {
                    // C.BEQZ
                    let rs1 = ((opcode >> 7) & 0x7) as u8 + 8;
                    let imm = decode_cb_imm(opcode);
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CBeqz,
                        rs1,
                        imm,
                        is_branch: true,
                        ..Default::default()
                    }
                }
                0b111 => {
                    // C.BNEZ
                    let rs1 = ((opcode >> 7) & 0x7) as u8 + 8;
                    let imm = decode_cb_imm(opcode);
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CBnez,
                        rs1,
                        imm,
                        is_branch: true,
                        ..Default::default()
                    }
                }
                _ => Rv32Instruction {
                    opcode: opcode as u32,
                    length: 2,
                    kind: Rv32Kind::Unknown,
                    ..Default::default()
                },
            }
        }
        0b10 => {
            match funct3 {
                0b000 => {
                    // C.SLLI
                    let rd = ((opcode >> 7) & 0x1F) as u8;
                    let shamt = (((opcode >> 2) & 0x1F) | ((opcode >> 12) << 5)) as u8;
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CSlli,
                        rd,
                        imm: shamt as i32,
                        ..Default::default()
                    }
                }
                0b010 => {
                    // C.LWSP
                    let rd = ((opcode >> 7) & 0x1F) as u8;
                    let imm = (((opcode >> 4) & 0x7) << 6 | ((opcode >> 12) & 0x1) << 5 | ((opcode >> 2) & 0x1) << 4) as i32;
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CLwsp,
                        rd,
                        rs1: 2, // sp
                        imm,
                        reads_mem: true,
                        ..Default::default()
                    }
                }
                0b100 => {
                    // C.JR, C.MV, C.EBREAK, C.JALR, C.ADD
                    let rd = ((opcode >> 7) & 0x1F) as u8;
                    let rs2 = ((opcode >> 2) & 0x1F) as u8;
                    let funct4 = ((opcode >> 12) & 0x1) as u8;
                    if funct4 == 0 {
                        if rs2 == 0 {
                            // C.JR
                            Rv32Instruction {
                                opcode: opcode as u32,
                                length: 2,
                                kind: Rv32Kind::CJr,
                                rs1: rd,
                                is_branch: true,
                                is_return: rd == 1,
                                ..Default::default()
                            }
                        } else {
                            // C.MV
                            Rv32Instruction {
                                opcode: opcode as u32,
                                length: 2,
                                kind: Rv32Kind::CMv,
                                rd,
                                rs2,
                                ..Default::default()
                            }
                        }
                    } else {
                        if rd == 0 && rs2 == 0 {
                            // C.EBREAK
                            Rv32Instruction {
                                opcode: opcode as u32,
                                length: 2,
                                kind: Rv32Kind::CEbreak,
                                ..Default::default()
                            }
                        } else if rs2 == 0 {
                            // C.JALR
                            Rv32Instruction {
                                opcode: opcode as u32,
                                length: 2,
                                kind: Rv32Kind::CJalr,
                                rd: 1,
                                rs1: rd,
                                is_branch: true,
                                is_call: true,
                                ..Default::default()
                            }
                        } else {
                            // C.ADD
                            Rv32Instruction {
                                opcode: opcode as u32,
                                length: 2,
                                kind: Rv32Kind::CAdd,
                                rd,
                                rs2,
                                ..Default::default()
                            }
                        }
                    }
                }
                0b110 => {
                    // C.SWSP
                    let rs2 = ((opcode >> 2) & 0x1F) as u8;
                    let imm = (((opcode >> 9) & 0x3F) << 2 | ((opcode >> 7) & 0x3)) as i32;
                    Rv32Instruction {
                        opcode: opcode as u32,
                        length: 2,
                        kind: Rv32Kind::CSwsp,
                        rs1: 2, // sp
                        rs2,
                        imm,
                        writes_mem: true,
                        ..Default::default()
                    }
                }
                _ => Rv32Instruction {
                    opcode: opcode as u32,
                    length: 2,
                    kind: Rv32Kind::Unknown,
                    ..Default::default()
                },
            }
        }
        _ => Rv32Instruction {
            opcode: opcode as u32,
            length: 2,
            kind: Rv32Kind::Unknown,
            ..Default::default()
        },
    }
}

// Immediate decoding helpers

fn decode_i_imm(opcode: u32) -> i32 {
    let imm = (opcode >> 20) as u32;
    sign_extend_12(imm)
}

fn decode_s_imm(opcode: u32) -> i32 {
    let imm = ((opcode >> 7) & 0x1F) | ((opcode >> 20) & 0xFE0);
    sign_extend_12(imm)
}

fn decode_b_imm(opcode: u32) -> i32 {
    let imm = ((opcode >> 7) & 0x1E) | ((opcode >> 20) & 0x7E0) | ((opcode << 4) & 0x800) | ((opcode >> 19) & 0x1000);
    sign_extend_13(imm)
}

fn decode_j_imm(opcode: u32) -> i32 {
    let imm = ((opcode >> 20) & 0x7FE) | ((opcode >> 9) & 0x800) | (opcode & 0xFF000) | ((opcode >> 11) & 0x100000);
    sign_extend_21(imm)
}

fn decode_cj_imm(opcode: u16) -> i32 {
    let imm = (((opcode >> 1) & 0x4) << 4) |
              (((opcode >> 6) & 0x1) << 9) |
              (((opcode >> 7) & 0x1) << 8) |
              (((opcode >> 2) & 0x1) << 6) |
              (((opcode >> 10) & 0x1) << 7) |
              (((opcode >> 8) & 0x3) << 1) |
              (((opcode >> 11) & 0x1) << 5);
    sign_extend_12(imm as u32)
}

fn decode_cb_imm(opcode: u16) -> i32 {
    let imm = (((opcode >> 3) & 0x3) << 1) |
              (((opcode >> 10) & 0x3) << 4) |
              (((opcode >> 2) & 0x1) << 6) |
              (((opcode >> 5) & 0x3) << 3) |
              (((opcode >> 12) & 0x1) << 8);
    sign_extend_9(imm as u32)
}

fn sign_extend_6(value: u32) -> i32 {
    if value & 0x20 != 0 {
        (value | 0xFFFFFFC0) as i32
    } else {
        value as i32
    }
}

fn sign_extend_9(value: u32) -> i32 {
    if value & 0x100 != 0 {
        (value | 0xFFFFFE00) as i32
    } else {
        value as i32
    }
}

fn sign_extend_10(value: u32) -> i32 {
    if value & 0x200 != 0 {
        (value | 0xFFFFFC00) as i32
    } else {
        value as i32
    }
}

fn sign_extend_12(value: u32) -> i32 {
    if value & 0x800 != 0 {
        (value | 0xFFFFF000) as i32
    } else {
        value as i32
    }
}

fn sign_extend_13(value: u32) -> i32 {
    if value & 0x1000 != 0 {
        (value | 0xFFFFE000) as i32
    } else {
        value as i32
    }
}

fn sign_extend_21(value: u32) -> i32 {
    if value & 0x100000 != 0 {
        (value | 0xFFE00000) as i32
    } else {
        value as i32
    }
}

/// Disassemble an RV32 instruction.
pub fn disassemble(opcode: u32, pc: u32) -> String {
    let instr = decode(opcode);

    match instr.kind {
        Rv32Kind::Lui => format!("lui x{}, 0x{:05x}", instr.rd, (instr.imm >> 12) as u32 & 0xFFFFF),
        Rv32Kind::Auipc => format!("auipc x{}, 0x{:05x}", instr.rd, (instr.imm >> 12) as u32 & 0xFFFFF),
        Rv32Kind::Jal => {
            let target = (pc as i32 + instr.imm) as u32;
            format!("jal x{}, 0x{:08x}", instr.rd, target)
        }
        Rv32Kind::Jalr => format!("jalr x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Beq => format!("beq x{}, x{}, {:08x}", instr.rs1, instr.rs2, (pc as i32 + instr.imm) as u32),
        Rv32Kind::Bne => format!("bne x{}, x{}, {:08x}", instr.rs1, instr.rs2, (pc as i32 + instr.imm) as u32),
        Rv32Kind::Blt => format!("blt x{}, x{}, {:08x}", instr.rs1, instr.rs2, (pc as i32 + instr.imm) as u32),
        Rv32Kind::Bge => format!("bge x{}, x{}, {:08x}", instr.rs1, instr.rs2, (pc as i32 + instr.imm) as u32),
        Rv32Kind::Bltu => format!("bltu x{}, x{}, {:08x}", instr.rs1, instr.rs2, (pc as i32 + instr.imm) as u32),
        Rv32Kind::Bgeu => format!("bgeu x{}, x{}, {:08x}", instr.rs1, instr.rs2, (pc as i32 + instr.imm) as u32),
        Rv32Kind::Lb => format!("lb x{}, {}(x{})", instr.rd, instr.imm, instr.rs1),
        Rv32Kind::Lh => format!("lh x{}, {}(x{})", instr.rd, instr.imm, instr.rs1),
        Rv32Kind::Lw => format!("lw x{}, {}(x{})", instr.rd, instr.imm, instr.rs1),
        Rv32Kind::Lbu => format!("lbu x{}, {}(x{})", instr.rd, instr.imm, instr.rs1),
        Rv32Kind::Lhu => format!("lhu x{}, {}(x{})", instr.rd, instr.imm, instr.rs1),
        Rv32Kind::Sb => format!("sb x{}, {}(x{})", instr.rs2, instr.imm, instr.rs1),
        Rv32Kind::Sh => format!("sh x{}, {}(x{})", instr.rs2, instr.imm, instr.rs1),
        Rv32Kind::Sw => format!("sw x{}, {}(x{})", instr.rs2, instr.imm, instr.rs1),
        Rv32Kind::Addi => format!("addi x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Slti => format!("slti x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Sltiu => format!("sltiu x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Xori => format!("xori x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Ori => format!("ori x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Andi => format!("andi x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Slli => format!("slli x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Srli => format!("srli x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Srai => format!("srai x{}, x{}, {}", instr.rd, instr.rs1, instr.imm),
        Rv32Kind::Add => format!("add x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Sub => format!("sub x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Sll => format!("sll x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Slt => format!("slt x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Sltu => format!("sltu x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Xor => format!("xor x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Srl => format!("srl x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Sra => format!("sra x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Or => format!("or x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::And => format!("and x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Mul => format!("mul x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Div => format!("div x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Rem => format!("rem x{}, x{}, x{}", instr.rd, instr.rs1, instr.rs2),
        Rv32Kind::Ecall => "ecall".to_string(),
        Rv32Kind::Ebreak => "ebreak".to_string(),
        _ => format!(".word 0x{:08x}", opcode),
    }
}
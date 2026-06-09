//! Compressed (RVC) instructions.

use rp2350sim_cpu_common::DecodeResult;

/// Decode a compressed instruction.
pub fn decode_compressed(opcode: u16) -> DecodeResult {
    let op = opcode & 0x3;

    match op {
        0b00 => DecodeResult {
            length: 2,
            name: "c0",
            ..Default::default()
        },
        0b01 => DecodeResult {
            length: 2,
            name: "c1",
            ..Default::default()
        },
        0b10 => DecodeResult {
            length: 2,
            name: "c2",
            ..Default::default()
        },
        _ => DecodeResult {
            length: 2,
            name: "unknown",
            ..Default::default()
        },
    }
}
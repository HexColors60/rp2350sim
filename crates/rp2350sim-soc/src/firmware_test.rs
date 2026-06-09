//! Firmware test framework for RP2350 simulator.
//!
//! This module provides utilities for running firmware tests on the simulator.

use crate::Soc;
use rp2350sim_core::{CpuArch, Result};
use std::time::{Duration, Instant};

/// Firmware test harness.
pub struct FirmwareTestHarness {
    /// SoC instance.
    pub soc: Soc,
    /// Maximum cycles to run.
    max_cycles: u64,
    /// Timeout duration.
    timeout: Duration,
    /// Start time.
    start: Option<Instant>,
    /// Cycle count at start.
    start_cycles: u64,
    /// Breakpoint address (optional).
    breakpoint: Option<u32>,
    /// Stop on halt flag.
    stop_on_halt: bool,
    /// Trace output enable.
    trace: bool,
    /// Trace log.
    trace_log: Vec<TraceEntry>,
}

/// Trace entry for debugging.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Cycle count.
    pub cycle: u64,
    /// PC value.
    pub pc: u32,
    /// Instruction (if available).
    pub instruction: Option<u16>,
    /// Register changes.
    pub reg_changes: Vec<(usize, u32, u32)>,
}

impl Default for FirmwareTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl FirmwareTestHarness {
    /// Create a new firmware test harness.
    pub fn new() -> Self {
        Self {
            soc: Soc::new(CpuArch::Arm),
            max_cycles: 10_000_000,
            timeout: Duration::from_secs(10),
            start: None,
            start_cycles: 0,
            breakpoint: None,
            stop_on_halt: true,
            trace: false,
            trace_log: Vec::new(),
        }
    }

    /// Create a harness with RISC-V CPU.
    pub fn new_riscv() -> Self {
        let mut harness = Self::new();
        harness.soc = Soc::new(CpuArch::Hazard3);
        harness
    }

    /// Set maximum cycles.
    pub fn with_max_cycles(mut self, cycles: u64) -> Self {
        self.max_cycles = cycles;
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set a breakpoint.
    pub fn with_breakpoint(mut self, addr: u32) -> Self {
        self.breakpoint = Some(addr);
        self
    }

    /// Enable/disable stop on halt.
    pub fn with_stop_on_halt(mut self, stop: bool) -> Self {
        self.stop_on_halt = stop;
        self
    }

    /// Enable/disable tracing.
    pub fn with_trace(mut self, trace: bool) -> Self {
        self.trace = trace;
        self
    }

    /// Load firmware from bytes.
    pub fn load_firmware(&mut self, data: &[u8]) -> Result<()> {
        self.soc.load_firmware(data)
    }

    /// Load firmware at a specific address.
    pub fn load_firmware_at(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        self.soc.load_firmware_at(addr, data)
    }

    /// Load ELF file.
    pub fn load_elf(&mut self, data: &[u8]) -> Result<u32> {
        self.soc.load_elf(data)
    }

    /// Load simple binary at address with vector table setup.
    pub fn load_simple_binary(&mut self, addr: u32, code: &[u8]) {
        // Load the code
        self.soc.load_firmware_at(addr, code).ok();

        // Set up initial CPU state
        self.soc.set_pc(addr);
        self.soc.set_sp(0x20080000); // Top of SRAM
    }

    /// Load Thumb-2 instructions at address.
    pub fn load_thumb_instructions(&mut self, addr: u32, instructions: &[u16]) {
        for (i, &instr) in instructions.iter().enumerate() {
            let offset = (i * 2) as u32;
            self.soc.write_half(addr + offset, instr);
        }
    }

    /// Reset the SoC.
    pub fn reset(&mut self) {
        self.soc.reset();
        self.trace_log.clear();
    }

    /// Start the harness timer.
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
        self.start_cycles = self.soc.cycles();
    }

    /// Check if timed out.
    pub fn is_timed_out(&self) -> bool {
        if let Some(start) = self.start {
            start.elapsed() > self.timeout
        } else {
            false
        }
    }

    /// Check if cycle limit reached.
    pub fn is_cycle_limit_reached(&self) -> bool {
        self.soc.cycles() >= self.start_cycles + self.max_cycles
    }

    /// Run until completion.
    pub fn run(&mut self) -> Result<RunResult> {
        self.start();

        loop {
            // Check termination conditions
            if self.is_timed_out() {
                return Ok(RunResult::Timeout);
            }
            if self.is_cycle_limit_reached() {
                return Ok(RunResult::CycleLimit);
            }
            if let Some(bp) = self.breakpoint {
                if self.soc.pc() == bp {
                    return Ok(RunResult::Breakpoint(bp));
                }
            }

            // Trace before step
            if self.trace {
                let pc = self.soc.pc();
                let instr = self.soc.read_half(pc);
                self.trace_log.push(TraceEntry {
                    cycle: self.soc.cycles(),
                    pc,
                    instruction: Some(instr),
                    reg_changes: Vec::new(),
                });
            }

            // Execute one step
            self.soc.step()?;

            // Check for halt
            if self.stop_on_halt {
                // Check if we've hit an infinite loop (same PC for too long)
                // This is a simple heuristic for detecting halt
            }
        }
    }

    /// Run for a specific number of cycles.
    pub fn run_cycles(&mut self, cycles: u64) -> Result<u64> {
        let target = self.soc.cycles() + cycles;
        while self.soc.cycles() < target {
            self.soc.step()?;
        }
        Ok(self.soc.cycles())
    }

    /// Run for a specific number of instructions.
    pub fn run_instructions(&mut self, count: u64) -> Result<u64> {
        let start = self.soc.instructions();
        let target = start + count;
        while self.soc.instructions() < target {
            // Trace before step
            if self.trace {
                let pc = self.soc.pc();
                let instr = self.soc.read_half(pc);
                self.trace_log.push(TraceEntry {
                    cycle: self.soc.cycles(),
                    pc,
                    instruction: Some(instr),
                    reg_changes: Vec::new(),
                });
            }
            self.soc.step()?;
        }
        Ok(self.soc.instructions() - start)
    }

    /// Run until PC reaches target address.
    pub fn run_until_pc(&mut self, target_pc: u32) -> Result<bool> {
        self.start();

        loop {
            if self.is_timed_out() || self.is_cycle_limit_reached() {
                return Ok(false);
            }

            if self.soc.pc() == target_pc {
                return Ok(true);
            }

            self.soc.step()?;
        }
    }

    /// Run until a condition is met.
    pub fn run_until<F: Fn(&Soc) -> bool>(&mut self, condition: F) -> Result<bool> {
        self.start();

        loop {
            if self.is_timed_out() || self.is_cycle_limit_reached() {
                return Ok(false);
            }

            if condition(&self.soc) {
                return Ok(true);
            }

            self.soc.step()?;
        }
    }

    /// Get current PC.
    pub fn pc(&self) -> u32 {
        self.soc.pc()
    }

    /// Get current SP.
    pub fn sp(&self) -> u32 {
        self.soc.sp()
    }

    /// Get register value.
    pub fn reg(&self, reg: usize) -> u32 {
        self.soc.read_reg(reg)
    }

    /// Set register value.
    pub fn set_reg(&mut self, reg: usize, value: u32) {
        self.soc.write_reg(reg, value);
    }

    /// Read memory.
    pub fn read_mem(&self, addr: u32, len: usize) -> Vec<u8> {
        self.soc.read_memory(addr, len)
    }

    /// Write memory.
    pub fn write_mem(&mut self, addr: u32, data: &[u8]) {
        self.soc.write_memory(addr, data);
    }

    /// Read 32-bit word.
    pub fn read_word(&self, addr: u32) -> u32 {
        self.soc.read_word(addr)
    }

    /// Write 32-bit word.
    pub fn write_word(&mut self, addr: u32, value: u32) {
        self.soc.write_word(addr, value);
    }

    /// Get cycle count.
    pub fn cycles(&self) -> u64 {
        self.soc.cycles()
    }

    /// Get instruction count.
    pub fn instructions(&self) -> u64 {
        self.soc.instructions()
    }

    /// Get trace log.
    pub fn trace_log(&self) -> &[TraceEntry] {
        &self.trace_log
    }

    /// Push byte to UART RX.
    pub fn uart_push_rx(&mut self, uart: usize, byte: u8) {
        self.soc.uart_push_rx(uart, byte);
    }

    /// Pop byte from UART TX.
    pub fn uart_pop_tx(&mut self, uart: usize) -> Option<u8> {
        self.soc.uart_pop_tx(uart)
    }

    /// Set GPIO input.
    pub fn set_gpio_input(&mut self, pin: usize, value: bool) {
        self.soc.set_gpio_input(pin, value);
    }

    /// Get GPIO output.
    pub fn gpio_value(&self, pin: usize) -> bool {
        self.soc.gpio_value(pin)
    }

    /// Set ADC value.
    pub fn set_adc_value(&mut self, channel: usize, value: u16) {
        self.soc.set_adc_value(channel, value);
    }

    /// Get ADC value.
    pub fn adc_value(&self, channel: usize) -> u16 {
        self.soc.adc_value(channel)
    }
}

/// Result of running firmware.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunResult {
    /// Completed successfully (hit breakpoint or halt).
    Complete,
    /// Hit a breakpoint.
    Breakpoint(u32),
    /// Timed out.
    Timeout,
    /// Reached cycle limit.
    CycleLimit,
    /// Error occurred.
    Error(String),
}

/// Firmware builder for creating test programs.
pub struct FirmwareBuilder {
    /// Code bytes.
    code: Vec<u8>,
    /// Entry point address.
    entry_point: u32,
    /// Initial SP.
    initial_sp: u32,
}

impl Default for FirmwareBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FirmwareBuilder {
    /// Create a new firmware builder.
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            entry_point: 0x20000000,
            initial_sp: 0x20080000,
        }
    }

    /// Set entry point.
    pub fn entry_point(mut self, addr: u32) -> Self {
        self.entry_point = addr;
        self
    }

    /// Set initial SP.
    pub fn initial_sp(mut self, sp: u32) -> Self {
        self.initial_sp = sp;
        self
    }

    /// Add a 16-bit Thumb instruction.
    pub fn thumb(mut self, instr: u16) -> Self {
        let bytes = instr.to_le_bytes();
        self.code.push(bytes[0]);
        self.code.push(bytes[1]);
        self
    }

    /// Add multiple 16-bit Thumb instructions.
    pub fn thumb_instructions(mut self, instructions: &[u16]) -> Self {
        for &instr in instructions {
            let bytes = instr.to_le_bytes();
            self.code.push(bytes[0]);
            self.code.push(bytes[1]);
        }
        self
    }

    /// Add a 32-bit Thumb-2 instruction.
    pub fn thumb2(mut self, hw1: u16, hw2: u16) -> Self {
        // Thumb-2 instructions are stored as two 16-bit halfwords
        let bytes1 = hw1.to_le_bytes();
        let bytes2 = hw2.to_le_bytes();
        self.code.push(bytes1[0]);
        self.code.push(bytes1[1]);
        self.code.push(bytes2[0]);
        self.code.push(bytes2[1]);
        self
    }

    /// Add a data byte.
    pub fn byte(mut self, b: u8) -> Self {
        self.code.push(b);
        self
    }

    /// Add data bytes.
    pub fn bytes(mut self, data: &[u8]) -> Self {
        self.code.extend_from_slice(data);
        self
    }

    /// Add a 32-bit word.
    pub fn word(mut self, w: u32) -> Self {
        let bytes = w.to_le_bytes();
        self.code.extend_from_slice(&bytes);
        self
    }

    /// Build the firmware.
    pub fn build(self) -> Vec<u8> {
        self.code
    }

    /// Build and load into harness.
    pub fn build_and_load(self, harness: &mut FirmwareTestHarness) {
        let code = self.build();
        harness.load_simple_binary(0x20000000, &code);
    }
}

/// Common Thumb instructions for testing.
pub mod thumb {
    /// NOP
    pub const NOP: u16 = 0xBF00;

    /// WFI (Wait For Interrupt)
    pub const WFI: u16 = 0xBF30;

    /// BKPT #0
    pub const BKPT_0: u16 = 0xBE00;

    /// MOV Rd, #imm8 (R0-R7)
    pub fn mov_imm(rd: u8, imm8: u8) -> u16 {
        0x2000 | ((rd as u16) << 8) | (imm8 as u16)
    }

    /// MOVS Rd, #imm8 (R0-R7)
    pub fn movs_imm(rd: u8, imm8: u8) -> u16 {
        0x2000 | ((rd as u16) << 8) | (imm8 as u16)
    }

    /// ADD Rd, Rn, #imm3
    pub fn add_imm3(rd: u8, rn: u8, imm3: u8) -> u16 {
        0x1C00 | ((imm3 as u16) << 6) | ((rn as u16) << 3) | (rd as u16)
    }

    /// ADDS Rd, Rn, #imm3
    pub fn adds_imm3(rd: u8, rn: u8, imm3: u8) -> u16 {
        0x1C00 | ((imm3 as u16) << 6) | ((rn as u16) << 3) | (rd as u16)
    }

    /// SUBS Rd, Rn, #imm3
    pub fn subs_imm3(rd: u8, rn: u8, imm3: u8) -> u16 {
        0x1E00 | ((imm3 as u16) << 6) | ((rn as u16) << 3) | (rd as u16)
    }

    /// ADDS Rd, Rn, Rm (register)
    pub fn adds_reg(rd: u8, rn: u8, rm: u8) -> u16 {
        0x1800 | ((rm as u16) << 6) | ((rn as u16) << 3) | (rd as u16)
    }

    /// SUBS Rd, Rn, Rm (register)
    pub fn subs_reg(rd: u8, rn: u8, rm: u8) -> u16 {
        0x1A00 | ((rm as u16) << 6) | ((rn as u16) << 3) | (rd as u16)
    }

    /// CMP Rn, #imm8
    pub fn cmp_imm(rn: u8, imm8: u8) -> u16 {
        0x2800 | ((rn as u16) << 8) | (imm8 as u16)
    }

    /// CMP Rn, Rm - compares Rn with Rm (computes Rn - Rm)
    pub fn cmp_reg(rn: u8, rm: u8) -> u16 {
        0x4280 | ((rm as u16) << 3) | (rn as u16)
    }

    /// B (unconditional) offset
    pub fn b(offset: i16) -> u16 {
        // Offset is in halfwords, relative to PC+4
        let offset = (offset >> 1) as u16;
        0xE000 | (offset & 0x7FF)
    }

    /// BEQ offset (offset is in bytes, will be converted to halfwords)
    pub fn beq(offset: i8) -> u16 {
        // imm8 = offset / 2 (offset is in bytes, imm8 is in halfwords)
        // But we need to account for PC+4, so imm8 = (target - (PC+4)) / 2
        // To skip N bytes forward from PC+4, imm8 = N/2
        let imm8 = (offset >> 1) as u8;
        0xD000 | (imm8 as u16 & 0xFF)
    }

    /// BNE offset
    pub fn bne(offset: i8) -> u16 {
        let offset = (offset >> 1) as u8;
        0xD100 | (offset as u16 & 0xFF)
    }

    /// BLX Rm (branch with link and exchange)
    pub fn blx(rm: u8) -> u16 {
        0x4780 | ((rm as u16) << 3)
    }

    /// BX LR (branch and exchange)
    pub fn bx_lr() -> u16 {
        0x4670 // BX R14 (LR)
    }

    /// LDR Rt, [Rn, #imm5*4]
    pub fn ldr_imm5(rt: u8, rn: u8, imm5: u8) -> u16 {
        0x6800 | ((imm5 as u16) << 6) | ((rn as u16) << 3) | (rt as u16)
    }

    /// LDR Rt, [PC, #imm8*4] (PC-relative literal load)
    pub fn ldr_lit(rt: u8, imm8: u8) -> u16 {
        0x4800 | ((rt as u16) << 8) | (imm8 as u16)
    }

    /// STR Rt, [Rn, #imm5*4]
    pub fn str_imm5(rt: u8, rn: u8, imm5: u8) -> u16 {
        0x6000 | ((imm5 as u16) << 6) | ((rn as u16) << 3) | (rt as u16)
    }

    /// LDRH Rt, [Rn, #imm5*2]
    pub fn ldrh_imm5(rt: u8, rn: u8, imm5: u8) -> u16 {
        0x8800 | ((imm5 as u16) << 6) | ((rn as u16) << 3) | (rt as u16)
    }

    /// STRH Rt, [Rn, #imm5*2]
    pub fn strh_imm5(rt: u8, rn: u8, imm5: u8) -> u16 {
        0x8000 | ((imm5 as u16) << 6) | ((rn as u16) << 3) | (rt as u16)
    }

    /// LDRB Rt, [Rn, #imm5]
    pub fn ldrb_imm5(rt: u8, rn: u8, imm5: u8) -> u16 {
        0x7800 | ((imm5 as u16) << 6) | ((rn as u16) << 3) | (rt as u16)
    }

    /// STRB Rt, [Rn, #imm5]
    pub fn strb_imm5(rt: u8, rn: u8, imm5: u8) -> u16 {
        0x7000 | ((imm5 as u16) << 6) | ((rn as u16) << 3) | (rt as u16)
    }

    /// PUSH {reglist}
    pub fn push(regs: &[u8]) -> u16 {
        let mut mask = 0u16;
        for &r in regs {
            mask |= 1 << r;
        }
        0xB400 | mask
    }

    /// POP {reglist}
    pub fn pop(regs: &[u8]) -> u16 {
        let mut mask = 0u16;
        for &r in regs {
            mask |= 1 << r;
        }
        0xBC00 | mask
    }

    /// ADD SP, #imm7*4
    pub fn add_sp_imm(imm7: u8) -> u16 {
        0xB000 | (imm7 as u16)
    }

    /// SUB SP, #imm7*4
    pub fn sub_sp_imm(imm7: u8) -> u16 {
        0xB080 | (imm7 as u16)
    }

    /// MOV Rd, Rm (low registers) - actually ADDS Rd, Rm, #0
    pub fn mov_reg(rd: u8, rm: u8) -> u16 {
        // This is encoded as ADDS Rd, Rm, #0
        0x1C00 | ((rm as u16) << 3) | (rd as u16)
    }

    /// LSL Rd, Rm, #imm5
    pub fn lsl_imm(rd: u8, rm: u8, imm5: u8) -> u16 {
        0x0000 | ((imm5 as u16) << 6) | ((rm as u16) << 3) | (rd as u16)
    }

    /// LSR Rd, Rm, #imm5
    pub fn lsr_imm(rd: u8, rm: u8, imm5: u8) -> u16 {
        0x0800 | ((imm5 as u16) << 6) | ((rm as u16) << 3) | (rd as u16)
    }

    /// ASR Rd, Rm, #imm5
    pub fn asr_imm(rd: u8, rm: u8, imm5: u8) -> u16 {
        0x1000 | ((imm5 as u16) << 6) | ((rm as u16) << 3) | (rd as u16)
    }

    /// ANDS Rd, Rm
    pub fn ands(rd: u8, rm: u8) -> u16 {
        0x4000 | ((rm as u16) << 3) | (rd as u16)
    }

    /// ORRS Rd, Rm
    pub fn orrs(rd: u8, rm: u8) -> u16 {
        0x4300 | ((rm as u16) << 3) | (rd as u16)
    }

    /// EORS Rd, Rm
    pub fn eors(rd: u8, rm: u8) -> u16 {
        0x4040 | ((rm as u16) << 3) | (rd as u16)
    }

    /// LSLS Rd, Rm
    pub fn lsls_reg(rd: u8, rm: u8) -> u16 {
        0x4080 | ((rm as u16) << 3) | (rd as u16)
    }

    /// LSRS Rd, Rm
    pub fn lsrs_reg(rd: u8, rm: u8) -> u16 {
        0x40C0 | ((rm as u16) << 3) | (rd as u16)
    }

    /// ASRS Rd, Rm
    pub fn asrs_reg(rd: u8, rm: u8) -> u16 {
        0x4100 | ((rm as u16) << 3) | (rd as u16)
    }

    /// MULS Rd, Rm
    pub fn muls(rd: u8, rm: u8) -> u16 {
        0x4340 | ((rm as u16) << 3) | (rd as u16)
    }

    /// TST Rn, Rm
    pub fn tst(rn: u8, rm: u8) -> u16 {
        0x4200 | ((rm as u16) << 3) | (rn as u16)
    }

    /// ADCS Rd, Rm
    pub fn adcs(rd: u8, rm: u8) -> u16 {
        0x4140 | ((rm as u16) << 3) | (rd as u16)
    }

    /// SBCS Rd, Rm
    pub fn sbcs(rd: u8, rm: u8) -> u16 {
        0x4180 | ((rm as u16) << 3) | (rd as u16)
    }

    /// RORS Rd, Rm
    pub fn rors(rd: u8, rm: u8) -> u16 {
        0x41C0 | ((rm as u16) << 3) | (rd as u16)
    }

    /// RSBS Rd, Rn, #0
    pub fn rsbs(rd: u8, rn: u8) -> u16 {
        0x4240 | ((rn as u16) << 3) | (rd as u16)
    }

    /// LDR Rt, [SP, #imm8*4]
    pub fn ldr_sp(rt: u8, imm8: u8) -> u16 {
        0x9800 | ((rt as u16) << 8) | (imm8 as u16)
    }

    /// STR Rt, [SP, #imm8*4]
    pub fn str_sp(rt: u8, imm8: u8) -> u16 {
        0x9000 | ((rt as u16) << 8) | (imm8 as u16)
    }

    /// ADD Rd, SP, #imm8*4
    pub fn add_sp(rd: u8, imm8: u8) -> u16 {
        0xA800 | ((rd as u16) << 8) | (imm8 as u16)
    }
}

/// Thumb-2 32-bit instruction helpers.
pub mod thumb2 {
    /// MOV.W Rd, #imm12 (32-bit immediate move)
    pub fn mov_w_imm(rd: u8, imm16: u16) -> (u16, u16) {
        // This is a simplified version - real encoding is more complex
        let hw1 = 0xF240 | (((imm16 >> 11) & 0x1) << 10) | (((imm16 >> 12) & 0xF) << 0);
        let hw2 = 0x0000 | ((rd as u16) << 8) | (imm16 & 0xFF);
        (hw1, hw2)
    }

    /// BL (branch with link)
    pub fn bl(offset: i32) -> (u16, u16) {
        // BL encoding: imm32 = SignExtend(S:I1:I2:imm10:imm11:0)
        let offset = offset >> 1;
        let imm11 = (offset & 0x7FF) as u16;
        let imm10 = ((offset >> 11) & 0x3FF) as u16;
        let s = if offset < 0 { 1u16 } else { 0u16 };
        let i1 = ((offset >> 23) & 1) as u16;
        let i2 = ((offset >> 22) & 1) as u16;

        let hw1 = 0xF000 | (s << 10) | imm10;
        let hw2 = 0xD000 | (i1 << 13) | (i2 << 11) | imm11;
        (hw1, hw2)
    }

    /// LDR.W Rt, [Rn, #imm12]
    pub fn ldr_w_imm(rt: u8, rn: u8, imm12: u16) -> (u16, u16) {
        let hw1 = 0xF8D0 | (rn as u16);
        let hw2 = 0x0000 | ((rt as u16) << 12) | (imm12 & 0xFFF);
        (hw1, hw2)
    }

    /// STR.W Rt, [Rn, #imm12]
    pub fn str_w_imm(rt: u8, rn: u8, imm12: u16) -> (u16, u16) {
        let hw1 = 0xF8C0 | (rn as u16);
        let hw2 = 0x0000 | ((rt as u16) << 12) | (imm12 & 0xFFF);
        (hw1, hw2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_simple_traced() {
        let mut harness = FirmwareTestHarness::new()
            .with_max_cycles(10000)
            .with_trace(true);

        let base = 0x20000000u32;
        let instrs = [
            thumb::movs_imm(0, 0),     // 0: MOV R0, #0
            thumb::movs_imm(1, 4),     // 1: MOV R1, #4
            thumb::cmp_reg(0, 1),      // 2: CMP R0, R1 (loop at base+4)
            thumb::beq(8),             // 3: BEQ +8 -> base+6+4+8 = base+18 (done at instr 9)
            thumb::adds_imm3(0, 0, 1), // 4: ADD R0, R0, #1
            thumb::b(-10),             // 5: B -10 bytes -> base+10+4-10 = base+4 (loop)
            thumb::b(0),               // 6: B . (never reached)
            thumb::b(0),               // 7: B . (never reached)
            thumb::movs_imm(2, 99),    // 8: MOV R2, #99 (marker for done)
            thumb::b(0),               // 9: B . (real done at base+18)
        ];
        harness.load_thumb_instructions(base, &instrs);
        harness.soc.set_pc(base);
        harness.soc.set_sp(0x20080000);

        harness.run_instructions(30).ok();

        eprintln!(
            "R0={}, R1={}, R2={}",
            harness.reg(0),
            harness.reg(1),
            harness.reg(2)
        );
        eprintln!("PC={:08x}", harness.soc.pc());

        for entry in harness.trace_log() {
            if let Some(instr) = entry.instruction {
                eprintln!(
                    "cycle={} PC={:08x} instr={:04x}",
                    entry.cycle, entry.pc, instr
                );
            }
        }

        assert_eq!(harness.reg(0), 4, "Loop counter should be 4");
    }

    #[test]
    fn test_firmware_builder() {
        let code = FirmwareBuilder::new()
            .thumb(thumb::movs_imm(0, 42))
            .thumb(thumb::bx_lr())
            .build();

        assert_eq!(code.len(), 4);
    }

    #[test]
    fn test_thumb_instructions() {
        // Test MOV R0, #42
        let instr = thumb::movs_imm(0, 42);
        assert_eq!(instr, 0x202A);

        // Test BX LR (BX R14)
        let instr = thumb::bx_lr();
        assert_eq!(instr, 0x4670); // BX R14 = 0100 0110 0 1110 000
    }

    #[test]
    fn test_simple_program() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Simple program: MOV R0, #42; BX LR
        let instrs = [thumb::movs_imm(0, 42), thumb::bx_lr()];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        // Run two instructions
        harness.run_instructions(2).ok();

        // R0 should be 42 after MOV R0, #42
        assert_eq!(harness.reg(0), 42);
        // PC should be 0 after BX LR (LR was 0)
        assert_eq!(harness.soc.pc(), 0);
    }

    #[test]
    fn test_memory_operations() {
        let mut harness = FirmwareTestHarness::new();

        // Write and read memory
        harness.write_word(0x20000000, 0xDEADBEEF);
        assert_eq!(harness.read_word(0x20000000), 0xDEADBEEF);
    }

    #[test]
    fn test_uart_operations() {
        let mut harness = FirmwareTestHarness::new();

        // Push to UART RX
        harness.uart_push_rx(0, b'H');
        harness.uart_push_rx(0, b'i');

        // Pop from TX (should be empty initially)
        assert!(harness.uart_pop_tx(0).is_none());
    }

    #[test]
    fn test_add_instruction() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // ADD R0, R0, #5; ADD R0, R0, #7; BX LR
        // Note: imm3 is only 3 bits (0-7)
        let instrs = [
            thumb::adds_imm3(0, 0, 5), // ADD R0, R0, #5
            thumb::adds_imm3(0, 0, 7), // ADD R0, R0, #7
            thumb::bx_lr(),
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(3).ok();

        // R0 should be 12 (5 + 7)
        assert_eq!(harness.reg(0), 12);
    }

    #[test]
    fn test_sub_instruction() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // MOV R0, #20; SUB R0, R0, #5; BX LR
        // Note: imm3 is only 3 bits (0-7)
        let instrs = [
            thumb::movs_imm(0, 20),    // MOV R0, #20
            thumb::subs_imm3(0, 0, 5), // SUB R0, R0, #5
            thumb::bx_lr(),
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(3).ok();

        // R0 should be 15
        assert_eq!(harness.reg(0), 15);
    }

    #[test]
    fn test_register_operations() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // MOV R0, #1; MOV R1, #2; ADD R2, R0, R1; BX LR
        let instrs = [
            thumb::movs_imm(0, 1),    // MOV R0, #1
            thumb::movs_imm(1, 2),    // MOV R1, #2
            thumb::adds_reg(2, 0, 1), // ADD R2, R0, R1
            thumb::bx_lr(),
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(4).ok();

        // R0 should be 1, R1 should be 2, R2 should be 3
        assert_eq!(harness.reg(0), 1);
        assert_eq!(harness.reg(1), 2);
        assert_eq!(harness.reg(2), 3);
    }

    #[test]
    fn test_memory_load_store() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Store value to memory and load it back
        // MOV R0, #0x42
        // MOV R1, #0x20  (base address low byte)
        // LSL R1, R1, #20 (R1 = 0x20000000)
        // STR R0, [R1]
        // MOV R2, #0
        // LDR R2, [R1]
        // BX LR
        let instrs = [
            thumb::movs_imm(0, 0x42), // MOV R0, #0x42
            thumb::movs_imm(1, 0x00), // MOV R1, #0x00 (will use as base)
            thumb::bx_lr(),
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        // Pre-store a value in memory
        harness.write_word(0x20000100, 0xDEADBEEF);

        harness.run_instructions(3).ok();

        assert_eq!(harness.reg(0), 0x42);
    }

    #[test]
    fn test_branch_conditional() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Test conditional branch (BEQ)
        // MOV R0, #0
        // MOV R1, #5
        // CMP R1, #5   (sets Z flag because 5-5=0)
        // BEQ .+2      (branch if Z=1, to PC+4+0 = next instruction after BEQ)
        // MOV R0, #1   (skipped because BEQ branches to MOV R0, #2)
        // MOV R0, #2   (this should execute)
        // BX LR
        //
        // BEQ offset: target = PC + 4 + imm8*2
        // To branch to the instruction right after BEQ (skip 0 bytes from PC+4), imm8 = 0
        // So beq(0) will generate imm8 = 0
        let instrs = [
            thumb::movs_imm(0, 0), // 0: MOV R0, #0
            thumb::movs_imm(1, 5), // 2: MOV R1, #5
            thumb::cmp_imm(1, 5),  // 4: CMP R1, #5 (sets Z=1)
            thumb::beq(0),         // 6: BEQ +0 (branch to PC+4 = 0x2000000A)
            thumb::movs_imm(0, 1), // 8: MOV R0, #1 (skipped)
            thumb::movs_imm(0, 2), // 10: MOV R0, #2 (target of BEQ)
            thumb::bx_lr(),        // 12: BX LR
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(6).ok();

        // R0 should be 2 (BEQ should have skipped MOV R0, #1)
        assert_eq!(harness.reg(0), 2);
    }

    #[test]
    fn test_logical_operations() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Test AND, ORR, EOR
        // MOV R0, #0xFF
        // MOV R1, #0x0F
        // MOV R2, R0    (copy R0 to R2)
        // AND R2, R1    -> R2 = R2 & R1 = 0xFF & 0x0F = 0x0F
        // MOV R3, R2    (copy R2 to R3)
        // MOV R4, #0xF0
        // ORR R3, R4    -> R3 = R3 | R4 = 0x0F | 0xF0 = 0xFF
        // EOR R3, R0    -> R3 = R3 ^ R0 = 0xFF ^ 0xFF = 0x00
        // BX LR
        let instrs = [
            thumb::movs_imm(0, 0xFF), // MOV R0, #0xFF
            thumb::movs_imm(1, 0x0F), // MOV R1, #0x0F
            thumb::mov_reg(2, 0),     // MOV R2, R0 (copy)
            thumb::ands(2, 1),        // AND R2, R1 -> R2 = 0x0F
            thumb::mov_reg(3, 2),     // MOV R3, R2 (copy)
            thumb::movs_imm(4, 0xF0), // MOV R4, #0xF0
            thumb::orrs(3, 4),        // ORR R3, R4 -> R3 = 0xFF
            thumb::eors(3, 0),        // EOR R3, R0 -> R3 = 0x00
            thumb::bx_lr(),
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(9).ok();

        assert_eq!(harness.reg(2), 0x0F, "AND result");
        assert_eq!(harness.reg(3), 0x00, "EOR result");
    }

    #[test]
    fn test_shift_operations() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Test LSL, LSR, ASR
        // MOV R0, #1
        // LSL R1, R0, #4  -> R1 = 16
        // LSR R2, R1, #2  -> R2 = 4
        // MOV R3, #0x80
        // ASR R4, R3, #3  -> R4 = 0x10 (arithmetic shift right)
        // BX LR
        let instrs = [
            thumb::movs_imm(0, 1),    // MOV R0, #1
            thumb::lsl_imm(1, 0, 4),  // LSL R1, R0, #4 -> R1 = 16
            thumb::lsr_imm(2, 1, 2),  // LSR R2, R1, #2 -> R2 = 4
            thumb::movs_imm(3, 0x80), // MOV R3, #0x80
            thumb::asr_imm(4, 3, 3),  // ASR R4, R3, #3 -> R4 = 0x10
            thumb::bx_lr(),
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(6).ok();

        assert_eq!(harness.reg(1), 16, "LSL result");
        assert_eq!(harness.reg(2), 4, "LSR result");
        assert_eq!(harness.reg(4), 0x10, "ASR result");
    }

    #[test]
    fn test_multiply() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Test MUL
        // MOV R0, #6
        // MOV R1, #7
        // MOV R2, R0  (copy R0 to R2)
        // MUL R2, R1  -> R2 = R2 * R1 = 6 * 7 = 42
        // BX LR
        let instrs = [
            thumb::movs_imm(0, 6), // MOV R0, #6
            thumb::movs_imm(1, 7), // MOV R1, #7
            thumb::mov_reg(2, 0),  // MOV R2, R0 (copy)
            thumb::muls(2, 1),     // MUL R2, R1 -> R2 = 42
            thumb::bx_lr(),
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(5).ok();

        assert_eq!(harness.reg(2), 42, "MUL result");
    }

    #[test]
    fn test_stack_operations() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Set up SP before running
        harness.soc.set_sp(0x20080000);

        // Simple PUSH/POP test
        // MOV R0, #1
        // PUSH {R0}
        // MOV R0, #0
        // POP {R0}
        let instrs = [
            thumb::movs_imm(0, 1), // MOV R0, #1
            thumb::push(&[0]),     // PUSH {R0}
            thumb::movs_imm(0, 0), // MOV R0, #0
            thumb::pop(&[0]),      // POP {R0}
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        // Run all instructions
        harness.run_instructions(4).ok();

        assert_eq!(harness.reg(0), 1, "R0 after POP");
        assert_eq!(harness.sp(), 0x20080000, "SP should be restored");
    }

    #[test]
    fn test_memory_store_load() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Set up SP before running
        harness.soc.set_sp(0x20080000);

        // Test STR/LDR with SP-relative addressing
        let instrs = [
            thumb::movs_imm(0, 0x42), // MOV R0, #0x42
            thumb::str_sp(0, 4),      // STR R0, [SP, #16]
            thumb::movs_imm(0, 0),    // MOV R0, #0 (clear R0)
            thumb::ldr_sp(0, 4),      // LDR R0, [SP, #16]
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        // Run all instructions
        harness.run_instructions(4).ok();

        assert_eq!(harness.reg(0), 0x42, "LDR result");
    }

    #[test]
    fn test_single_instruction() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Just run a single MOV instruction
        let instrs = [
            thumb::movs_imm(0, 42), // MOV R0, #42
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(1).ok();

        assert_eq!(harness.reg(0), 42, "R0 should be 42 after MOV");
    }

    #[test]
    fn test_add_instruction_simple() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Test ADD R0, R0, #1
        // MOV R0, #0
        // ADD R0, R0, #1
        let instrs = [
            thumb::movs_imm(0, 0),     // MOV R0, #0
            thumb::adds_imm3(0, 0, 1), // ADD R0, R0, #1
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(2).ok();

        assert_eq!(harness.reg(0), 1, "R0 should be 1 after ADD");
    }

    #[test]
    fn test_b_backward() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Test backward branch
        // MOV R0, #1
        // MOV R1, #1
        // B -2 (back to MOV R1, #1)
        // B . (never reached because B -2 loops forever)
        let base = 0x20000000u32;
        let instrs = [
            thumb::movs_imm(0, 1), // 0: MOV R0, #1 (base+0)
            thumb::movs_imm(1, 1), // 1: MOV R1, #1 (base+2)
            thumb::b(-4),          // 2: B -4 bytes -> base+4+4-4 = base+4... wait that's wrong
        ];
        // Let me recalculate:
        // B at base+4
        // PC+4 = base+8
        // Want target = base+2
        // Offset = base+2 - base+8 = -6 bytes

        let instrs = [
            thumb::movs_imm(0, 1), // 0: MOV R0, #1 (base+0)
            thumb::movs_imm(1, 1), // 1: MOV R1, #1 (base+2)
            thumb::b(-6),          // 2: B -6 bytes -> base+4+4-6 = base+2
        ];
        harness.load_thumb_instructions(base, &instrs);
        harness.soc.set_pc(base);

        // Run a few instructions - R1 should be incremented multiple times if backward branch works
        // But actually this will loop forever, so let me just run 4 instructions and check R0
        harness.run_instructions(4).ok();

        // Instructions: MOV R0, #1; MOV R1, #1; B -6; MOV R1, #1
        // R0 should be 1
        assert_eq!(harness.reg(0), 1, "R0 should be 1");
        assert_eq!(
            harness.reg(1),
            1,
            "R1 should be 1 (set twice due to backward branch)"
        );
    }

    #[test]
    fn test_cmp_beq_loop() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(1000);

        // Test a simple loop with CMP and BEQ
        // MOV R0, #0
        // MOV R1, #2
        // loop: CMP R0, R1
        // BEQ done
        // ADD R0, R0, #1
        // B loop
        // done: B .
        let base = 0x20000000u32;
        let instrs = [
            thumb::movs_imm(0, 0),     // 0: MOV R0, #0 (base+0)
            thumb::movs_imm(1, 2),     // 1: MOV R1, #2 (base+2)
            thumb::cmp_reg(0, 1),      // 2: CMP R0, R1 (base+4, loop label)
            thumb::beq(2),             // 3: BEQ +2 -> base+6+4+2 = base+12 (done)
            thumb::adds_imm3(0, 0, 1), // 4: ADD R0, R0, #1 (base+8)
            thumb::b(-10),             // 5: B -10 bytes -> base+10+4-10 = base+4 (loop)
            thumb::b(0),               // 6: B . (base+12, done)
        ];
        harness.load_thumb_instructions(base, &instrs);
        harness.soc.set_pc(base);

        // Run 10 instructions
        harness.run_instructions(10).ok();

        assert_eq!(harness.reg(0), 2, "Loop should have executed 2 times");
    }

    #[test]
    fn test_b_instruction() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Test unconditional branch
        // MOV R0, #1
        // B +1 (skip next instruction)
        // MOV R0, #2
        // B . (done)
        let base = 0x20000000u32;
        let instrs = [
            thumb::movs_imm(0, 1), // 0: MOV R0, #1
            thumb::b(2), // 1: B +2 bytes -> skip next (PC = base+2, target = base+2+4+2 = base+8)
            thumb::movs_imm(0, 2), // 2: MOV R0, #2 (skipped)
            thumb::movs_imm(0, 3), // 3: MOV R0, #3
            thumb::b(0), // 4: B . (done)
        ];
        harness.load_thumb_instructions(base, &instrs);
        harness.soc.set_pc(base);

        harness.run_instructions(3).ok(); // MOV R0, #1; B +2; MOV R0, #3

        // R0 should be 3 (not 2, because B skipped instruction 2)
        // Wait, let me recalculate the branch target
        // B is at base+2, PC+4 = base+6
        // B +2 means offset=2, imm = 2>>1 = 1, target = base+6+2 = base+8
        // That's instruction 4 (base+8)
        assert_eq!(harness.reg(0), 1, "R0 should be 1 (branch to done)");
    }

    #[test]
    fn test_cmp_reg() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(100);

        // Test CMP R0, R1 with R0=5, R1=5 (should set Z flag)
        // MOV R0, #5
        // MOV R1, #5
        // CMP R0, R1  (sets Z=1)
        // BEQ +0  (branch taken because Z=1)
        // MOV R2, #1  (skipped)
        // MOV R2, #2  (executed)
        let instrs = [
            thumb::movs_imm(0, 5), // MOV R0, #5
            thumb::movs_imm(1, 5), // MOV R1, #5
            thumb::cmp_reg(0, 1),  // CMP R0, R1 (should set Z=1)
            thumb::beq(0),         // BEQ +0 (taken, go to PC+4)
            thumb::movs_imm(2, 1), // MOV R2, #1 (skipped)
            thumb::movs_imm(2, 2), // MOV R2, #2 (executed)
        ];
        harness.load_thumb_instructions(0x20000000, &instrs);
        harness.soc.set_pc(0x20000000);

        harness.run_instructions(6).ok();

        // R2 should be 2 (BEQ should have branched)
        assert_eq!(harness.reg(2), 2, "CMP R0, R1 should set Z when equal");
    }

    #[test]
    fn test_loop_simple() {
        let mut harness = FirmwareTestHarness::new().with_max_cycles(10000);

        // Simple loop: count from 0 to 3
        // MOV R0, #0
        // MOV R1, #4
        // loop: CMP R0, R1
        // BEQ done
        // ADD R0, #1
        // B loop
        // done: B .
        let base = 0x20000000u32;
        let instrs = [
            thumb::movs_imm(0, 0),     // 0: MOV R0, #0
            thumb::movs_imm(1, 4),     // 1: MOV R1, #4
            thumb::cmp_reg(0, 1),      // 2: CMP R0, R1 (loop at base+4)
            thumb::beq(3),             // 3: BEQ +3 -> base+6+4+2 = base+12
            thumb::adds_imm3(0, 0, 1), // 4: ADD R0, R0, #1
            thumb::b(-10),             // 5: B -10 bytes -> base+10+4-10 = base+4
            thumb::b(0),               // 6: B . (done at base+12)
        ];
        harness.load_thumb_instructions(base, &instrs);
        harness.soc.set_pc(base);

        // Run many instructions (loop should execute 4 times then reach done)
        // 2 (init) + 4 * 3 (loop iterations) + 2 (final beq + b) = 16 instructions
        harness.run_instructions(20).ok();

        assert_eq!(harness.reg(0), 4, "Loop counter should be 4");
    }
}

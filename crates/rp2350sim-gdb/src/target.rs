//! GDB target trait for RP2350 simulator.
//!
//! This module defines the interface that the simulator must implement
//! to be debuggable via GDB.

use crate::protocol::{GdbError, GdbResponse};

/// GDB target trait.
///
/// Implement this trait to make your target debuggable via GDB.
pub trait GdbTarget {
    /// Read all registers.
    fn read_registers(&self) -> Result<Vec<u8>, GdbError>;

    /// Write all registers.
    fn write_registers(&mut self, data: &[u8]) -> Result<(), GdbError>;

    /// Read a single register.
    fn read_register(&self, reg: u32) -> Result<Vec<u8>, GdbError>;

    /// Write a single register.
    fn write_register(&mut self, reg: u32, data: &[u8]) -> Result<(), GdbError>;

    /// Read memory.
    fn read_memory(&self, addr: u64, length: u32) -> Result<Vec<u8>, GdbError>;

    /// Write memory.
    fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<(), GdbError>;

    /// Continue execution.
    fn continue_exec(&mut self) -> Result<(), GdbError>;

    /// Single step.
    fn step(&mut self) -> Result<(), GdbError>;

    /// Set a breakpoint.
    fn set_breakpoint(&mut self, addr: u64, kind: BreakpointKind) -> Result<(), GdbError>;

    /// Remove a breakpoint.
    fn remove_breakpoint(&mut self, addr: u64, kind: BreakpointKind) -> Result<(), GdbError>;

    /// Check if target is running.
    fn is_running(&self) -> bool;

    /// Get the current PC.
    fn get_pc(&self) -> u64;

    /// Set the PC.
    fn set_pc(&mut self, pc: u64) -> Result<(), GdbError>;

    /// Get the last signal (for stop reply).
    fn get_last_signal(&self) -> u8;

    /// Reset the target.
    fn reset(&mut self) -> Result<(), GdbError>;

    /// Get target description (for qXfer:features:read).
    fn get_target_description(&self) -> Option<String> {
        None
    }

    /// Get memory map (for qXfer:memory-map:read).
    fn get_memory_map(&self) -> Option<String> {
        None
    }

    /// Handle query.
    fn handle_query(&self, query: &str) -> Option<GdbResponse> {
        match query {
            "Supported" => Some(GdbResponse::Data("qXfer:features:read+;qXfer:memory-map:read+;multiprocess+".to_string())),
            "C" => Some(GdbResponse::Data("QC1.1".to_string())), // Current thread (process 1, thread 1)
            "Attached" => Some(GdbResponse::Data("1".to_string())),
            "fThreadInfo" => Some(GdbResponse::Data("m1.1,1.2".to_string())), // Two threads (cores)
            "sThreadInfo" => Some(GdbResponse::Data("l".to_string())),
            "ThreadExtraInfo,1.1" => Some(GdbResponse::Data("ARM Cortex-M33 Core 0".as_bytes().iter().map(|b| format!("{:02x}", b)).collect())),
            "ThreadExtraInfo,1.2" => Some(GdbResponse::Data("Hazard3 RISC-V Core 1".as_bytes().iter().map(|b| format!("{:02x}", b)).collect())),
            _ => None,
        }
    }

    /// Set the current thread for subsequent operations.
    fn set_thread(&mut self, _thread_id: u64) -> Result<(), GdbError> {
        Ok(())
    }

    /// Get the number of cores/threads.
    fn get_thread_count(&self) -> u32 {
        2 // RP2350 has 2 cores
    }

    /// Get thread name.
    fn get_thread_name(&self, thread_id: u64) -> Option<String> {
        match thread_id {
            1 => Some("ARM Cortex-M33 Core 0".to_string()),
            2 => Some("Hazard3 RISC-V Core 1".to_string()),
            _ => None,
        }
    }

    /// Check if the target supports multi-core debugging.
    fn supports_multicore(&self) -> bool {
        true
    }
}

/// Breakpoint kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointKind {
    /// Software breakpoint
    Software,
    /// Hardware breakpoint
    Hardware,
    /// Write watchpoint
    WriteWatchpoint,
    /// Read watchpoint
    ReadWatchpoint,
    /// Access watchpoint
    AccessWatchpoint,
}

impl BreakpointKind {
    /// Create from GDB breakpoint type number.
    pub fn from_type(type_: u32) -> Option<Self> {
        match type_ {
            0 => Some(Self::Software),
            1 => Some(Self::Hardware),
            2 => Some(Self::WriteWatchpoint),
            3 => Some(Self::ReadWatchpoint),
            4 => Some(Self::AccessWatchpoint),
            _ => None,
        }
    }
}

/// Default target description for ARM Cortex-M33.
pub fn get_arm_target_description() -> String {
    r#"<?xml version="1.0"?>
<!DOCTYPE target SYSTEM "gdb-target.dtd">
<target version="1.0">
    <architecture>arm</architecture>
    <feature name="org.gnu.gdb.arm.m-profile">
        <reg name="r0" bitsize="32" type="uint32"/>
        <reg name="r1" bitsize="32" type="uint32"/>
        <reg name="r2" bitsize="32" type="uint32"/>
        <reg name="r3" bitsize="32" type="uint32"/>
        <reg name="r4" bitsize="32" type="uint32"/>
        <reg name="r5" bitsize="32" type="uint32"/>
        <reg name="r6" bitsize="32" type="uint32"/>
        <reg name="r7" bitsize="32" type="uint32"/>
        <reg name="r8" bitsize="32" type="uint32"/>
        <reg name="r9" bitsize="32" type="uint32"/>
        <reg name="r10" bitsize="32" type="uint32"/>
        <reg name="r11" bitsize="32" type="uint32"/>
        <reg name="r12" bitsize="32" type="uint32"/>
        <reg name="sp" bitsize="32" type="data_ptr"/>
        <reg name="lr" bitsize="32" type="code_ptr"/>
        <reg name="pc" bitsize="32" type="code_ptr"/>
        <reg name="xpsr" bitsize="32" type="uint32"/>
    </feature>
</target>
"#.to_string()
}

/// Default target description for RISC-V RV32.
pub fn get_riscv_target_description() -> String {
    r#"<?xml version="1.0"?>
<!DOCTYPE target SYSTEM "gdb-target.dtd">
<target version="1.0">
    <architecture>riscv:rv32</architecture>
    <feature name="org.gnu.gdb.riscv.cpu">
        <reg name="zero" bitsize="32" type="uint32"/>
        <reg name="ra" bitsize="32" type="code_ptr"/>
        <reg name="sp" bitsize="32" type="data_ptr"/>
        <reg name="gp" bitsize="32" type="data_ptr"/>
        <reg name="tp" bitsize="32" type="data_ptr"/>
        <reg name="t0" bitsize="32" type="uint32"/>
        <reg name="t1" bitsize="32" type="uint32"/>
        <reg name="t2" bitsize="32" type="uint32"/>
        <reg name="fp" bitsize="32" type="data_ptr"/>
        <reg name="s1" bitsize="32" type="uint32"/>
        <reg name="a0" bitsize="32" type="uint32"/>
        <reg name="a1" bitsize="32" type="uint32"/>
        <reg name="a2" bitsize="32" type="uint32"/>
        <reg name="a3" bitsize="32" type="uint32"/>
        <reg name="a4" bitsize="32" type="uint32"/>
        <reg name="a5" bitsize="32" type="uint32"/>
        <reg name="a6" bitsize="32" type="uint32"/>
        <reg name="a7" bitsize="32" type="uint32"/>
        <reg name="s2" bitsize="32" type="uint32"/>
        <reg name="s3" bitsize="32" type="uint32"/>
        <reg name="s4" bitsize="32" type="uint32"/>
        <reg name="s5" bitsize="32" type="uint32"/>
        <reg name="s6" bitsize="32" type="uint32"/>
        <reg name="s7" bitsize="32" type="uint32"/>
        <reg name="s8" bitsize="32" type="uint32"/>
        <reg name="s9" bitsize="32" type="uint32"/>
        <reg name="s10" bitsize="32" type="uint32"/>
        <reg name="s11" bitsize="32" type="uint32"/>
        <reg name="t3" bitsize="32" type="uint32"/>
        <reg name="t4" bitsize="32" type="uint32"/>
        <reg name="t5" bitsize="32" type="uint32"/>
        <reg name="t6" bitsize="32" type="uint32"/>
        <reg name="pc" bitsize="32" type="code_ptr"/>
    </feature>
</target>
"#.to_string()
}

/// Default memory map for RP2350.
pub fn get_rp2350_memory_map() -> String {
    r#"<?xml version="1.0"?>
<!DOCTYPE memory-map SYSTEM "gdb-memory-map.dtd">
<memory-map>
    <!-- ROM (Boot ROM) -->
    <memory type="rom" start="0x00000000" length="0x00004000"/>
    
    <!-- XIP (Flash) -->
    <memory type="flash" start="0x10000000" length="0x01000000">
        <property name="blocksize">0x1000</property>
    </memory>
    
    <!-- SRAM Bank 0 -->
    <memory type="ram" start="0x20000000" length="0x00042000"/>
    
    <!-- SRAM Bank 1 -->
    <memory type="ram" start="0x20042000" length="0x00042000"/>
    
    <!-- SRAM Bank 2 -->
    <memory type="ram" start="0x20084000" length="0x00042000"/>
    
    <!-- SRAM Bank 3 -->
    <memory type="ram" start="0x200c6000" length="0x00042000"/>
    
    <!-- SRAM Bank 4 -->
    <memory type="ram" start="0x20108000" length="0x00004000"/>
    
    <!-- Peripheral registers -->
    <memory type="ram" start="0x40000000" length="0x00040000"/>
    
    <!-- CoreSight debug registers -->
    <memory type="ram" start="0xe0000000" length="0x00100000"/>
</memory-map>
"#.to_string()
}
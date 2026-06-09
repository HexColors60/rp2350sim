//! GDB target implementation for RP2350 SoC.

use std::collections::HashSet;
use rp2350sim_gdb::{GdbError, GdbTarget};
use rp2350sim_gdb::target::BreakpointKind;
use rp2350sim_core::CpuArch;

use crate::Soc;

/// GDB target wrapper for the RP2350 SoC.
pub struct SocGdbTarget {
    soc: Soc,
    last_signal: u8,
    breakpoints: HashSet<u32>,
}

impl SocGdbTarget {
    /// Create a new GDB target wrapper.
    pub fn new(soc: Soc) -> Self {
        Self {
            soc,
            last_signal: 5, // SIGTRAP
            breakpoints: HashSet::new(),
        }
    }

    /// Get a reference to the SoC.
    pub fn soc(&self) -> &Soc {
        &self.soc
    }

    /// Get a mutable reference to the SoC.
    pub fn soc_mut(&mut self) -> &mut Soc {
        &mut self.soc
    }

    /// Consume the wrapper and return the SoC.
    pub fn into_inner(self) -> Soc {
        self.soc
    }

    /// Get the CPU architecture.
    fn get_arch(&self) -> CpuArch {
        if self.soc.cpu_arm.is_some() {
            CpuArch::Arm
        } else {
            CpuArch::Hazard3
        }
    }
}

impl GdbTarget for SocGdbTarget {
    fn read_registers(&self) -> Result<Vec<u8>, GdbError> {
        let mut data = Vec::new();

        if let Some(ref cpu) = self.soc.cpu_arm {
            // ARM Cortex-M33 registers: R0-R15, xPSR
            for i in 0..16 {
                let reg = cpu.read_reg(i);
                data.extend_from_slice(&reg.to_le_bytes());
            }
            // xPSR
            let xpsr = cpu.flags();
            data.extend_from_slice(&xpsr.to_le_bytes());
        } else if let Some(ref cpu) = self.soc.cpu_hazard3 {
            // RISC-V RV32 registers: x0-x31, PC
            for i in 0..32 {
                let reg = cpu.read_reg(i);
                data.extend_from_slice(&reg.to_le_bytes());
            }
            let pc = cpu.pc();
            data.extend_from_slice(&pc.to_le_bytes());
        }

        Ok(data)
    }

    fn write_registers(&mut self, data: &[u8]) -> Result<(), GdbError> {
        if let Some(ref mut cpu) = self.soc.cpu_arm {
            for (i, chunk) in data.chunks(4).enumerate() {
                if i < 16 && chunk.len() == 4 {
                    let value = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    cpu.write_reg(i, value);
                }
            }
        } else if let Some(ref mut cpu) = self.soc.cpu_hazard3 {
            for (i, chunk) in data.chunks(4).enumerate() {
                if i < 32 && chunk.len() == 4 {
                    let value = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    cpu.write_reg(i, value);
                }
            }
        }

        Ok(())
    }

    fn read_register(&self, reg: u32) -> Result<Vec<u8>, GdbError> {
        let value = if let Some(ref cpu) = self.soc.cpu_arm {
            if reg < 16 {
                cpu.read_reg(reg as usize)
            } else if reg == 16 {
                cpu.flags()
            } else {
                return Err(GdbError::InvalidRegister(reg));
            }
        } else if let Some(ref cpu) = self.soc.cpu_hazard3 {
            if reg < 32 {
                cpu.read_reg(reg as usize)
            } else if reg == 32 {
                cpu.pc()
            } else {
                return Err(GdbError::InvalidRegister(reg));
            }
        } else {
            return Err(GdbError::InvalidRegister(reg));
        };

        Ok(value.to_le_bytes().to_vec())
    }

    fn write_register(&mut self, reg: u32, data: &[u8]) -> Result<(), GdbError> {
        if data.len() < 4 {
            return Err(GdbError::InvalidRegister(reg));
        }

        let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        if let Some(ref mut cpu) = self.soc.cpu_arm {
            if reg < 16 {
                cpu.write_reg(reg as usize, value);
            } else if reg == 16 {
                // xPSR is read-only for now
            } else {
                return Err(GdbError::InvalidRegister(reg));
            }
        } else if let Some(ref mut cpu) = self.soc.cpu_hazard3 {
            if reg < 32 {
                cpu.write_reg(reg as usize, value);
            } else if reg == 32 {
                cpu.set_pc(value);
            } else {
                return Err(GdbError::InvalidRegister(reg));
            }
        }

        Ok(())
    }

    fn read_memory(&self, addr: u64, length: u32) -> Result<Vec<u8>, GdbError> {
        let addr = addr as u32;
        let mut data = vec![0u8; length as usize];

        // Read from appropriate memory region
        for i in 0..length as usize {
            let byte_addr = addr.wrapping_add(i as u32);
            data[i] = self.soc.read_byte(byte_addr);
        }

        Ok(data)
    }

    fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<(), GdbError> {
        let addr = addr as u32;

        for (i, &byte) in data.iter().enumerate() {
            let byte_addr = addr.wrapping_add(i as u32);
            self.soc.write_byte(byte_addr, byte);
        }

        Ok(())
    }

    fn continue_exec(&mut self) -> Result<(), GdbError> {
        self.soc.set_running(true);
        Ok(())
    }

    fn step(&mut self) -> Result<(), GdbError> {
        self.soc.set_running(false);
        let _ = self.soc.step();
        self.last_signal = 5; // SIGTRAP
        Ok(())
    }

    fn set_breakpoint(&mut self, addr: u64, _kind: BreakpointKind) -> Result<(), GdbError> {
        self.breakpoints.insert(addr as u32);
        Ok(())
    }

    fn remove_breakpoint(&mut self, addr: u64, _kind: BreakpointKind) -> Result<(), GdbError> {
        self.breakpoints.remove(&(addr as u32));
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.soc.is_running()
    }

    fn get_pc(&self) -> u64 {
        if let Some(ref cpu) = self.soc.cpu_arm {
            cpu.pc() as u64
        } else if let Some(ref cpu) = self.soc.cpu_hazard3 {
            cpu.pc() as u64
        } else {
            0
        }
    }

    fn set_pc(&mut self, pc: u64) -> Result<(), GdbError> {
        if let Some(ref mut cpu) = self.soc.cpu_arm {
            cpu.set_pc(pc as u32);
        } else if let Some(ref mut cpu) = self.soc.cpu_hazard3 {
            cpu.set_pc(pc as u32);
        }
        Ok(())
    }

    fn get_last_signal(&self) -> u8 {
        self.last_signal
    }

    fn reset(&mut self) -> Result<(), GdbError> {
        self.soc.reset();
        self.breakpoints.clear();
        self.last_signal = 5; // SIGTRAP
        Ok(())
    }

    fn get_target_description(&self) -> Option<String> {
        match self.get_arch() {
            CpuArch::Arm => Some(rp2350sim_gdb::target::get_arm_target_description()),
            CpuArch::Hazard3 => Some(rp2350sim_gdb::target::get_riscv_target_description()),
        }
    }

    fn get_memory_map(&self) -> Option<String> {
        Some(r#"<?xml version="1.0"?>
<!DOCTYPE memory-map SYSTEM "gdb-memory-map.dtd">
<memory-map>
    <memory type="flash" start="0x10000000" length="0x1000000">
        <property name="blocksize">0x1000</property>
    </memory>
    <memory type="ram" start="0x20000000" length="0x82000"/>
</memory-map>
"#.to_string())
    }
}
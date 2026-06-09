//! Trace export.
//!
//! This module provides export functionality for trace data in various formats.

use std::io::Write;
use std::path::Path;
use rp2350sim_core::Result;

/// Export trace to file.
pub fn export_trace(trace: &[u8], path: &str) -> Result<()> {
    std::fs::write(path, trace)?;
    Ok(())
}

/// VCD (Value Change Dump) exporter.
/// 
/// VCD is a standard format for digital waveform data that can be viewed
/// with tools like GTKWave.
pub struct VcdExporter {
    /// Output buffer
    buffer: String,
    /// Current simulation time
    current_time: u64,
    /// Variable definitions
    variables: Vec<VcdVariable>,
    /// Module hierarchy
    modules: Vec<String>,
}

/// VCD variable definition.
#[derive(Debug, Clone)]
pub struct VcdVariable {
    /// Variable identifier (single character)
    pub id: char,
    /// Variable name
    pub name: String,
    /// Bit width
    pub width: u32,
    /// Variable type
    pub var_type: VcdVarType,
}

/// VCD variable type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcdVarType {
    /// Wire
    Wire,
    /// Register
    Reg,
    /// Integer
    Integer,
    /// Real
    Real,
    /// Parameter
    Parameter,
}

impl VcdExporter {
    /// Create a new VCD exporter.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            current_time: 0,
            variables: Vec::new(),
            modules: Vec::new(),
        }
    }

    /// Write the VCD header.
    pub fn write_header(&mut self, timescale: &str, comment: Option<&str>) {
        self.buffer.push_str("$timescale ");
        self.buffer.push_str(timescale);
        self.buffer.push_str(" $end\n");
        
        if let Some(c) = comment {
            self.buffer.push_str("$comment ");
            self.buffer.push_str(c);
            self.buffer.push_str(" $end\n");
        }
        
        self.buffer.push_str("$scope module TOP $end\n");
    }

    /// Add a module scope.
    pub fn push_scope(&mut self, name: &str) {
        self.buffer.push_str("$scope module ");
        self.buffer.push_str(name);
        self.buffer.push_str(" $end\n");
        self.modules.push(name.to_string());
    }

    /// Pop a module scope.
    pub fn pop_scope(&mut self) {
        self.buffer.push_str("$upscope $end\n");
        self.modules.pop();
    }

    /// Add a variable.
    pub fn add_variable(&mut self, var: VcdVariable) {
        let type_str = match var.var_type {
            VcdVarType::Wire => "wire",
            VcdVarType::Reg => "reg",
            VcdVarType::Integer => "integer",
            VcdVarType::Real => "real",
            VcdVarType::Parameter => "parameter",
        };
        
        self.buffer.push_str("$var ");
        self.buffer.push_str(type_str);
        self.buffer.push_str(" ");
        self.buffer.push_str(&var.width.to_string());
        self.buffer.push_str(" ");
        self.buffer.push(var.id);
        self.buffer.push_str(" ");
        self.buffer.push_str(&var.name);
        self.buffer.push_str(" $end\n");
        
        self.variables.push(var);
    }

    /// End the header section.
    pub fn end_header(&mut self) {
        self.buffer.push_str("$enddefinitions $end\n");
    }

    /// Set the current simulation time.
    pub fn set_time(&mut self, time: u64) {
        if time != self.current_time {
            self.current_time = time;
            self.buffer.push_str("#");
            self.buffer.push_str(&time.to_string());
            self.buffer.push_str("\n");
        }
    }

    /// Write a binary value change.
    pub fn write_binary(&mut self, id: char, value: u64) {
        if self.variables.iter().any(|v| v.id == id) {
            let var = self.variables.iter().find(|v| v.id == id).unwrap();
            let bits = format_binary(value, var.width);
            self.buffer.push_str(&bits);
            self.buffer.push(id);
            self.buffer.push_str("\n");
        }
    }

    /// Write a single-bit value change.
    pub fn write_bit(&mut self, id: char, value: bool) {
        self.buffer.push_str(if value { "1" } else { "0" });
        self.buffer.push(id);
        self.buffer.push_str("\n");
    }

    /// Write a vector value change.
    pub fn write_vector(&mut self, id: char, value: &[u8]) {
        self.buffer.push_str("b");
        for byte in value.iter().rev() {
            self.buffer.push_str(&format!("{:08b}", byte));
        }
        self.buffer.push_str(" ");
        self.buffer.push(id);
        self.buffer.push_str("\n");
    }

    /// Write a real value change.
    pub fn write_real(&mut self, id: char, value: f64) {
        self.buffer.push_str("r");
        self.buffer.push_str(&value.to_string());
        self.buffer.push_str(" ");
        self.buffer.push(id);
        self.buffer.push_str("\n");
    }

    /// Write a string value change.
    pub fn write_string(&mut self, id: char, value: &str) {
        self.buffer.push_str("s");
        self.buffer.push_str(value);
        self.buffer.push_str(" ");
        self.buffer.push(id);
        self.buffer.push_str("\n");
    }

    /// Get the VCD content.
    pub fn content(&self) -> &str {
        &self.buffer
    }

    /// Export to a file.
    pub fn export<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = std::fs::File::create(path)?;
        file.write_all(self.buffer.as_bytes())?;
        Ok(())
    }
}

impl Default for VcdExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a binary value with the specified width.
fn format_binary(value: u64, width: u32) -> String {
    if width == 1 {
        if value != 0 { "1".to_string() } else { "0".to_string() }
    } else {
        format!("b{:0width$b}", value, width = width as usize)
    }
}

/// GPIO trace VCD exporter.
pub struct GpioVcdExporter {
    /// VCD exporter
    pub exporter: VcdExporter,
    pin_count: usize,
}

impl GpioVcdExporter {
    /// Create a new GPIO VCD exporter.
    pub fn new(pin_count: usize) -> Self {
        let mut exporter = VcdExporter::new();
        exporter.write_header("1ns", Some("RP2350 GPIO Trace"));
        
        // Add GPIO pins
        exporter.push_scope("GPIO");
        for pin in 0..pin_count {
            exporter.add_variable(VcdVariable {
                id: char::from(b'a' + (pin % 26) as u8),
                name: format!("pin_{}", pin),
                width: 1,
                var_type: VcdVarType::Wire,
            });
        }
        exporter.pop_scope();
        exporter.end_header();
        
        Self { exporter, pin_count }
    }

    /// Record GPIO state at a time.
    pub fn record(&mut self, time: u64, states: &[bool]) {
        self.exporter.set_time(time);
        for (pin, &state) in states.iter().enumerate().take(self.pin_count) {
            let id = char::from(b'a' + (pin % 26) as u8);
            self.exporter.write_bit(id, state);
        }
    }

    /// Export to a file.
    pub fn export<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.exporter.export(path)
    }
}

/// CPU trace VCD exporter.
pub struct CpuVcdExporter {
    /// VCD exporter
    pub exporter: VcdExporter,
}

impl CpuVcdExporter {
    /// Create a new CPU VCD exporter.
    pub fn new() -> Self {
        let mut exporter = VcdExporter::new();
        exporter.write_header("1ns", Some("RP2350 CPU Trace"));
        
        // Add CPU signals
        exporter.push_scope("CPU");
        exporter.add_variable(VcdVariable {
            id: 'A',
            name: "PC".to_string(),
            width: 32,
            var_type: VcdVarType::Wire,
        });
        exporter.add_variable(VcdVariable {
            id: 'B',
            name: "SP".to_string(),
            width: 32,
            var_type: VcdVarType::Wire,
        });
        exporter.add_variable(VcdVariable {
            id: 'C',
            name: "LR".to_string(),
            width: 32,
            var_type: VcdVarType::Wire,
        });
        exporter.add_variable(VcdVariable {
            id: 'D',
            name: "Cycles".to_string(),
            width: 64,
            var_type: VcdVarType::Integer,
        });
        exporter.add_variable(VcdVariable {
            id: 'E',
            name: "IRQ".to_string(),
            width: 1,
            var_type: VcdVarType::Wire,
        });
        exporter.pop_scope();
        exporter.end_header();
        
        Self { exporter }
    }

    /// Record CPU state at a time.
    pub fn record(&mut self, time: u64, pc: u32, sp: u32, lr: u32, cycles: u64, irq: bool) {
        self.exporter.set_time(time);
        self.exporter.write_binary('A', pc as u64);
        self.exporter.write_binary('B', sp as u64);
        self.exporter.write_binary('C', lr as u64);
        self.exporter.write_binary('D', cycles);
        self.exporter.write_bit('E', irq);
    }

    /// Export to a file.
    pub fn export<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.exporter.export(path)
    }
}

impl Default for CpuVcdExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory trace VCD exporter.
pub struct MemoryVcdExporter {
    /// VCD exporter
    pub exporter: VcdExporter,
}

impl MemoryVcdExporter {
    /// Create a new memory VCD exporter.
    pub fn new() -> Self {
        let mut exporter = VcdExporter::new();
        exporter.write_header("1ns", Some("RP2350 Memory Trace"));
        
        exporter.push_scope("Memory");
        exporter.add_variable(VcdVariable {
            id: 'A',
            name: "Address".to_string(),
            width: 32,
            var_type: VcdVarType::Wire,
        });
        exporter.add_variable(VcdVariable {
            id: 'B',
            name: "Data".to_string(),
            width: 32,
            var_type: VcdVarType::Wire,
        });
        exporter.add_variable(VcdVariable {
            id: 'C',
            name: "Write".to_string(),
            width: 1,
            var_type: VcdVarType::Wire,
        });
        exporter.add_variable(VcdVariable {
            id: 'D',
            name: "Width".to_string(),
            width: 3,
            var_type: VcdVarType::Wire,
        });
        exporter.pop_scope();
        exporter.end_header();
        
        Self { exporter }
    }

    /// Record a memory access.
    pub fn record(&mut self, time: u64, addr: u32, data: u32, write: bool, width: u8) {
        self.exporter.set_time(time);
        self.exporter.write_binary('A', addr as u64);
        self.exporter.write_binary('B', data as u64);
        self.exporter.write_bit('C', write);
        self.exporter.write_binary('D', width as u64);
    }

    /// Export to a file.
    pub fn export<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.exporter.export(path)
    }
}

impl Default for MemoryVcdExporter {
    fn default() -> Self {
        Self::new()
    }
}
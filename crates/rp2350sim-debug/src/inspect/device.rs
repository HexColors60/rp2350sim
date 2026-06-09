//! Device inspector.
#![allow(dead_code)]

use std::collections::HashMap;

/// Device register info.
#[derive(Debug, Clone)]
pub struct RegisterInfo {
    /// Register name.
    pub name: String,
    /// Register offset.
    pub offset: u32,
    /// Register value.
    pub value: u32,
    /// Register description.
    pub description: String,
    /// Bit field definitions.
    pub fields: Vec<BitField>,
}

/// Bit field definition.
#[derive(Debug, Clone)]
pub struct BitField {
    /// Field name.
    pub name: String,
    /// Bit position (LSB).
    pub bit_pos: u8,
    /// Bit width.
    pub bit_width: u8,
    /// Field description.
    pub description: String,
    /// Enum values (if applicable).
    pub enum_values: Vec<(u32, String)>,
}

impl BitField {
    /// Create a new bit field.
    pub fn new(name: &str, bit_pos: u8, bit_width: u8) -> Self {
        Self {
            name: name.to_string(),
            bit_pos,
            bit_width,
            description: String::new(),
            enum_values: Vec::new(),
        }
    }

    /// Add description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Add enum value.
    pub fn with_enum(mut self, value: u32, name: &str) -> Self {
        self.enum_values.push((value, name.to_string()));
        self
    }

    /// Extract field value from register.
    pub fn extract(&self, reg_value: u32) -> u32 {
        let mask = ((1u32 << self.bit_width) - 1) << self.bit_pos;
        (reg_value & mask) >> self.bit_pos
    }

    /// Format field value as string.
    pub fn format_value(&self, reg_value: u32) -> String {
        let value = self.extract(reg_value);
        
        // Check for enum value
        for (enum_val, enum_name) in &self.enum_values {
            if *enum_val == value {
                return format!("{} ({})", enum_name, value);
            }
        }
        
        // Format as binary or decimal
        if self.bit_width == 1 {
            if value != 0 { "1 (set)".to_string() } else { "0 (clear)".to_string() }
        } else if self.bit_width <= 4 {
            format!("{} (0b{:0width$b})", value, value, width = self.bit_width as usize)
        } else {
            let hex_width = ((self.bit_width as usize) + 3) / 4;
            format!("{} (0x{:0width$X})", value, value, width = hex_width)
        }
    }
}

/// Device inspector.
pub struct DeviceInspector {
    /// Device name.
    pub name: String,
    /// Device base address.
    pub base_address: u32,
    /// Device size.
    pub size: u32,
    /// Register definitions.
    pub registers: Vec<RegisterInfo>,
    /// Register map by offset.
    pub register_map: HashMap<u32, usize>,
    /// Read function.
    read_fn: Option<Box<dyn FnMut(u32) -> u32 + Send + Sync>>,
}

impl std::fmt::Debug for DeviceInspector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceInspector")
            .field("name", &self.name)
            .field("base_address", &self.base_address)
            .field("size", &self.size)
            .field("registers", &self.registers)
            .field("register_map", &self.register_map)
            .field("read_fn", &self.read_fn.as_ref().map(|_| "<function>"))
            .finish()
    }
}

impl DeviceInspector {
    /// Create a new device inspector.
    pub fn new(name: &str, base_address: u32, size: u32) -> Self {
        Self {
            name: name.to_string(),
            base_address,
            size,
            registers: Vec::new(),
            register_map: HashMap::new(),
            read_fn: None,
        }
    }

    /// Set the read function.
    pub fn set_read_fn(&mut self, f: Box<dyn FnMut(u32) -> u32 + Send + Sync>) {
        self.read_fn = Some(f);
    }

    /// Add a register definition.
    pub fn add_register(&mut self, name: &str, offset: u32, description: &str) -> &mut RegisterInfo {
        let idx = self.registers.len();
        self.register_map.insert(offset, idx);
        
        self.registers.push(RegisterInfo {
            name: name.to_string(),
            offset,
            value: 0,
            description: description.to_string(),
            fields: Vec::new(),
        });
        
        self.registers.last_mut().unwrap()
    }

    /// Add a bit field to the last register.
    pub fn add_field(&mut self, name: &str, bit_pos: u8, bit_width: u8) -> &mut Self {
        if let Some(reg) = self.registers.last_mut() {
            reg.fields.push(BitField::new(name, bit_pos, bit_width));
        }
        self
    }

    /// Read a register value.
    pub fn read_register(&mut self, offset: u32) -> u32 {
        if let Some(ref mut read_fn) = self.read_fn {
            let value = read_fn(self.base_address + offset);
            
            // Update cached value
            if let Some(&idx) = self.register_map.get(&offset) {
                if let Some(reg) = self.registers.get_mut(idx) {
                    reg.value = value;
                }
            }
            
            value
        } else {
            0
        }
    }

    /// Refresh all register values.
    pub fn refresh(&mut self) {
        for reg in &mut self.registers {
            if let Some(ref mut read_fn) = self.read_fn {
                reg.value = read_fn(self.base_address + reg.offset);
            }
        }
    }

    /// Get register by offset.
    pub fn get_register(&self, offset: u32) -> Option<&RegisterInfo> {
        self.register_map.get(&offset).and_then(|&idx| self.registers.get(idx))
    }

    /// Get register by name.
    pub fn get_register_by_name(&self, name: &str) -> Option<&RegisterInfo> {
        self.registers.iter().find(|r| r.name == name)
    }

    /// Format register value as string.
    pub fn format_register(&self, offset: u32) -> String {
        if let Some(reg) = self.get_register(offset) {
            let mut result = format!("{} (0x{:02X}): 0x{:08X}\n", reg.name, reg.offset, reg.value);
            
            for field in &reg.fields {
                result.push_str(&format!("  [{}]: {}\n", field.name, field.format_value(reg.value)));
            }
            
            result
        } else {
            format!("Unknown register at offset 0x{:02X}", offset)
        }
    }

    /// Generate a summary of all registers.
    pub fn summary(&self) -> String {
        let mut result = format!("=== {} ===\n", self.name);
        result.push_str(&format!("Base: 0x{:08X}, Size: 0x{:X}\n\n", self.base_address, self.size));
        
        for reg in &self.registers {
            result.push_str(&format!("{} (0x{:02X}): 0x{:08X}\n", reg.name, reg.offset, reg.value));
        }
        
        result
    }
}

/// GPIO inspector.
pub fn create_gpio_inspector() -> DeviceInspector {
    let mut inspector = DeviceInspector::new("GPIO", 0x40014000, 0x1000);
    
    // GPIO status registers
    for pin in 0..30 {
        let offset = pin * 4;
        inspector.add_register(&format!("GPIO{}_STATUS", pin), offset, "GPIO status");
        inspector.add_field("IRQTOPROC", 26, 1);
        inspector.add_field("IRQFROMPAD", 24, 1);
        inspector.add_field("OUTTOPAD", 12, 1);
        inspector.add_field("OUTFROMPERI", 8, 1);
        inspector.add_field("OEFROMPERI", 4, 1);
        inspector.add_field("INFROMPAD", 0, 1);
    }
    
    // GPIO control registers
    for pin in 0..30 {
        let offset = 0x04 + pin * 4;
        inspector.add_register(&format!("GPIO{}_CTRL", pin), offset, "GPIO control");
        inspector.add_field("IRQOVER", 28, 2);
        inspector.add_field("OUTOVER", 12, 2);
        inspector.add_field("OEOVER", 8, 2);
        inspector.add_field("INOVER", 4, 2);
        inspector.add_field("FUNCSEL", 0, 5);
    }
    
    inspector
}

/// UART inspector.
pub fn create_uart_inspector(index: u8) -> DeviceInspector {
    let base = if index == 0 { 0x40034000 } else { 0x40038000 };
    let mut inspector = DeviceInspector::new(&format!("UART{}", index), base, 0x1000);
    
    inspector.add_register("UARTDR", 0x00, "Data Register");
    inspector.add_field("OE", 11, 1);
    inspector.add_field("BE", 10, 1);
    inspector.add_field("PE", 9, 1);
    inspector.add_field("FE", 8, 1);
    inspector.add_field("DATA", 0, 8);
    
    inspector.add_register("UARTRSR", 0x04, "Receive Status Register");
    inspector.add_field("OE", 3, 1);
    inspector.add_field("BE", 2, 1);
    inspector.add_field("PE", 1, 1);
    inspector.add_field("FE", 0, 1);
    
    inspector.add_register("UARTFR", 0x18, "Flag Register");
    inspector.add_field("TXFE", 7, 1);
    inspector.add_field("RXFF", 6, 1);
    inspector.add_field("TXFF", 5, 1);
    inspector.add_field("RXFE", 4, 1);
    inspector.add_field("BUSY", 3, 1);
    
    inspector.add_register("UARTIBRD", 0x24, "Integer Baud Rate");
    inspector.add_field("BAUD_DIVINT", 0, 16);
    
    inspector.add_register("UARTFBRD", 0x28, "Fractional Baud Rate");
    inspector.add_field("BAUD_DIVFRAC", 0, 6);
    
    inspector.add_register("UARTLCR_H", 0x2C, "Line Control");
    inspector.add_field("SPS", 7, 1);
    inspector.add_field("WLEN", 5, 2);
    inspector.add_field("FEN", 4, 1);
    inspector.add_field("STP2", 3, 1);
    inspector.add_field("EPS", 2, 1);
    inspector.add_field("PEN", 1, 1);
    inspector.add_field("BRK", 0, 1);
    
    inspector.add_register("UARTCR", 0x30, "Control Register");
    inspector.add_field("CTSEN", 15, 1);
    inspector.add_field("RTSEN", 14, 1);
    inspector.add_field("OUT2", 13, 1);
    inspector.add_field("OUT1", 12, 1);
    inspector.add_field("RTS", 11, 1);
    inspector.add_field("DTR", 10, 1);
    inspector.add_field("RXE", 9, 1);
    inspector.add_field("TXE", 8, 1);
    inspector.add_field("LBE", 7, 1);
    inspector.add_field("UARTEN", 0, 1);
    
    inspector
}

/// Timer inspector.
pub fn create_timer_inspector() -> DeviceInspector {
    let mut inspector = DeviceInspector::new("Timer", 0x40054000, 0x1000);
    
    inspector.add_register("TIMEHR", 0x00, "Time High");
    inspector.add_field("TIME", 0, 32);
    
    inspector.add_register("TIMELR", 0x04, "Time Low");
    inspector.add_field("TIME", 0, 32);
    
    for i in 0..4 {
        inspector.add_register(&format!("ALARM{}", i), 0x10 + i * 4, &format!("Alarm {}", i));
        inspector.add_field("VALUE", 0, 32);
    }
    
    inspector.add_register("ARMED", 0x20, "Armed Alarms");
    inspector.add_field("ALARM3", 3, 1);
    inspector.add_field("ALARM2", 2, 1);
    inspector.add_field("ALARM1", 1, 1);
    inspector.add_field("ALARM0", 0, 1);
    
    inspector.add_register("TIMERAWH", 0x24, "Time Raw High");
    inspector.add_field("TIME", 0, 32);
    
    inspector.add_register("TIMERAWL", 0x28, "Time Raw Low");
    inspector.add_field("TIME", 0, 32);
    
    inspector
}
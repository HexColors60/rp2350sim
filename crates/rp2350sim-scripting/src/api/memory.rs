//! Memory API for scripting.

use std::cell::RefCell;
use std::rc::Rc;

/// Memory control trait.
pub trait MemoryControl: Send + Sync {
    /// Read a byte from memory.
    fn read_byte(&self, addr: u32) -> u8;
    /// Write a byte to memory.
    fn write_byte(&mut self, addr: u32, value: u8);
    /// Read a halfword from memory.
    fn read_half(&self, addr: u32) -> u16;
    /// Write a halfword to memory.
    fn write_half(&mut self, addr: u32, value: u16);
    /// Read a word from memory.
    fn read_word(&self, addr: u32) -> u32;
    /// Write a word to memory.
    fn write_word(&mut self, addr: u32, value: u32);
    /// Read a block of memory.
    fn read_block(&self, addr: u32, size: u32) -> Vec<u8>;
    /// Write a block of memory.
    fn write_block(&mut self, addr: u32, data: &[u8]);
}

/// Memory API for Rhai scripting.
pub struct MemoryApi {
    control: Option<Rc<RefCell<dyn MemoryControl>>>,
}

impl MemoryApi {
    /// Create a new memory API.
    pub fn new() -> Self {
        Self { control: None }
    }

    /// Create with a control reference.
    pub fn with_control(control: Rc<RefCell<dyn MemoryControl>>) -> Self {
        Self { control: Some(control) }
    }

    /// Set the control reference.
    pub fn set_control(&mut self, control: Rc<RefCell<dyn MemoryControl>>) {
        self.control = Some(control);
    }

    /// Read a byte from memory.
    pub fn read_byte(&self, addr: i64) -> i64 {
        if let Some(ref control) = self.control {
            control.borrow().read_byte(addr as u32) as i64
        } else {
            0
        }
    }

    /// Write a byte to memory.
    pub fn write_byte(&mut self, addr: i64, value: i64) {
        if let Some(ref control) = self.control {
            control.borrow_mut().write_byte(addr as u32, value as u8);
        }
    }

    /// Read a halfword from memory.
    pub fn read_half(&self, addr: i64) -> i64 {
        if let Some(ref control) = self.control {
            control.borrow().read_half(addr as u32) as i64
        } else {
            0
        }
    }

    /// Write a halfword to memory.
    pub fn write_half(&mut self, addr: i64, value: i64) {
        if let Some(ref control) = self.control {
            control.borrow_mut().write_half(addr as u32, value as u16);
        }
    }

    /// Read a word from memory.
    pub fn read_word(&self, addr: i64) -> i64 {
        if let Some(ref control) = self.control {
            control.borrow().read_word(addr as u32) as i64
        } else {
            0
        }
    }

    /// Write a word to memory.
    pub fn write_word(&mut self, addr: i64, value: i64) {
        if let Some(ref control) = self.control {
            control.borrow_mut().write_word(addr as u32, value as u32);
        }
    }

    /// Read a block of memory.
    pub fn read_block(&self, addr: i64, size: i64) -> Vec<i64> {
        if let Some(ref control) = self.control {
            control.borrow().read_block(addr as u32, size as u32)
                .into_iter()
                .map(|b| b as i64)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Write a block of memory.
    pub fn write_block(&mut self, addr: i64, data: Vec<i64>) {
        if let Some(ref control) = self.control {
            let bytes: Vec<u8> = data.into_iter().map(|b| b as u8).collect();
            control.borrow_mut().write_block(addr as u32, &bytes);
        }
    }
}

impl Default for MemoryApi {
    fn default() -> Self {
        Self::new()
    }
}
//! Memory abstraction.

use crate::faults::MemoryFault;
use crate::perms::Permissions;
use crate::watch::{WatchKind, Watchpoint};
use parking_lot::RwLock;
use rp2350sim_core::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Memory region trait.
pub trait MemoryRegion: Send + Sync {
    fn read(&self, offset: u64, data: &mut [u8]) -> Result<()>;
    fn write(&mut self, offset: u64, data: &[u8]) -> Result<()>;
    fn size(&self) -> u64;
    fn permissions(&self) -> Permissions;
}

/// Generic memory block.
#[derive(Debug)]
pub struct MemoryBlock {
    data: Vec<u8>,
    permissions: Permissions,
    name: String,
}

impl MemoryBlock {
    pub fn new(name: impl Into<String>, size: usize) -> Self {
        Self {
            data: vec![0; size],
            permissions: Permissions::read_write(),
            name: name.into(),
        }
    }

    pub fn with_permissions(mut self, perms: Permissions) -> Self {
        self.permissions = perms;
        self
    }

    pub fn readonly(mut self) -> Self {
        self.permissions.write = false;
        self
    }

    pub fn writeonly(mut self) -> Self {
        self.permissions.read = false;
        self
    }

    pub fn executable(mut self) -> Self {
        self.permissions.execute = true;
        self
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn load(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        let end = offset + data.len();
        if end > self.data.len() {
            return Err(rp2350sim_core::Error::InvalidState("Load out of bounds".into()));
        }
        self.data[offset..end].copy_from_slice(data);
        Ok(())
    }

    pub fn read_byte(&self, offset: usize) -> u8 {
        self.data.get(offset).copied().unwrap_or(0)
    }

    pub fn write_byte(&mut self, offset: usize, value: u8) {
        if offset < self.data.len() {
            self.data[offset] = value;
        }
    }

    pub fn read_half(&self, offset: usize) -> u16 {
        let lo = self.read_byte(offset);
        let hi = self.read_byte(offset + 1);
        u16::from_le_bytes([lo, hi])
    }

    pub fn write_half(&mut self, offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.write_byte(offset, bytes[0]);
        self.write_byte(offset + 1, bytes[1]);
    }

    pub fn read_word(&self, offset: usize) -> u32 {
        let lo = self.read_half(offset);
        let hi = self.read_half(offset + 2);
        u32::from_le_bytes([lo as u8, (lo >> 8) as u8, hi as u8, (hi >> 8) as u8])
    }

    pub fn write_word(&mut self, offset: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.write_byte(offset, bytes[0]);
        self.write_byte(offset + 1, bytes[1]);
        self.write_byte(offset + 2, bytes[2]);
        self.write_byte(offset + 3, bytes[3]);
    }
}

impl MemoryRegion for MemoryBlock {
    fn read(&self, offset: u64, data: &mut [u8]) -> Result<()> {
        let offset = offset as usize;
        let end = offset + data.len();
        if end > self.data.len() {
            return Err(rp2350sim_core::Error::MemoryAccess(offset as u32));
        }
        data.copy_from_slice(&self.data[offset..end]);
        Ok(())
    }

    fn write(&mut self, offset: u64, data: &[u8]) -> Result<()> {
        let offset = offset as usize;
        let end = offset + data.len();
        if end > self.data.len() {
            return Err(rp2350sim_core::Error::MemoryAccess(offset as u32));
        }
        self.data[offset..end].copy_from_slice(data);
        Ok(())
    }

    fn size(&self) -> u64 {
        self.data.len() as u64
    }

    fn permissions(&self) -> Permissions {
        self.permissions
    }
}

/// Main memory system.
pub struct Memory {
    regions: HashMap<String, Arc<RwLock<MemoryBlock>>>,
    watchpoints: Vec<Watchpoint>,
    fault_handler: Option<Box<dyn Fn(MemoryFault) + Send + Sync>>,
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memory")
            .field("regions", &self.regions)
            .field("watchpoints", &self.watchpoints)
            .field("fault_handler", &self.fault_handler.as_ref().map(|_| "..."))
            .finish()
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            watchpoints: Vec::new(),
            fault_handler: None,
        }
    }

    pub fn add_region(&mut self, name: impl Into<String>, size: usize) -> Arc<RwLock<MemoryBlock>> {
        let name = name.into();
        let block = Arc::new(RwLock::new(MemoryBlock::new(&name, size)));
        self.regions.insert(name, block.clone());
        block
    }

    pub fn get_region(&self, name: &str) -> Option<Arc<RwLock<MemoryBlock>>> {
        self.regions.get(name).cloned()
    }

    pub fn remove_region(&mut self, name: &str) -> Option<Arc<RwLock<MemoryBlock>>> {
        self.regions.remove(name)
    }

    pub fn add_watchpoint(&mut self, watchpoint: Watchpoint) {
        self.watchpoints.push(watchpoint);
    }

    pub fn remove_watchpoint(&mut self, addr: u64) {
        self.watchpoints.retain(|w| w.addr != addr);
    }

    pub fn check_watchpoints(&self, addr: u64, is_write: bool) -> Option<&Watchpoint> {
        self.watchpoints.iter().find(|w| {
            w.enabled && addr >= w.addr && addr < w.addr + w.size as u64 &&
            ((is_write && w.kind == WatchKind::Write) || (!is_write && w.kind == WatchKind::Read))
        })
    }

    pub fn set_fault_handler<F: Fn(MemoryFault) + Send + Sync + 'static>(&mut self, handler: F) {
        self.fault_handler = Some(Box::new(handler));
    }

    pub fn regions(&self) -> impl Iterator<Item = (&String, &Arc<RwLock<MemoryBlock>>)> {
        self.regions.iter()
    }

    pub fn clear(&mut self) {
        for block in self.regions.values() {
            block.write().clear();
        }
    }
}
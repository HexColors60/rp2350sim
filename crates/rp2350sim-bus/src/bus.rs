//! Bus implementation.

use crate::{BusError, MemoryMap, Result};
use parking_lot::RwLock;
use rp2350sim_core::AccessWidth;
use std::collections::HashMap;
use std::sync::Arc;

/// Device access trait.
pub trait DeviceAccess: Send + Sync {
    fn read(&self, offset: u32, width: AccessWidth) -> Result<u64>;
    fn write(&mut self, offset: u32, value: u64, width: AccessWidth) -> Result<()>;
}

/// Device entry in the bus.
struct DeviceEntry {
    base: u32,
    size: u32,
    device: RwLock<Box<dyn DeviceAccess>>,
}

/// Wrapper for Arc<RwLock<D>> to implement DeviceAccess.
struct ArcDeviceWrapper<D: DeviceAccess> {
    device: Arc<RwLock<D>>,
}

impl<D: DeviceAccess + 'static> DeviceAccess for ArcDeviceWrapper<D> {
    fn read(&self, offset: u32, width: AccessWidth) -> Result<u64> {
        self.device.read().read(offset, width)
    }

    fn write(&mut self, offset: u32, value: u64, width: AccessWidth) -> Result<()> {
        self.device.write().write(offset, value, width)
    }
}

/// The main bus implementation.
pub struct Bus {
    memory_map: MemoryMap,
    devices: HashMap<u32, DeviceEntry>,
    hooks: Vec<Box<dyn BusHook>>,
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory_map: MemoryMap::rp2350_default(),
            devices: HashMap::new(),
            hooks: Vec::new(),
        }
    }

    /// Attach a device to the bus.
    pub fn attach_device<D: DeviceAccess + 'static>(&mut self, base: u32, size: u32, device: D) {
        let entry = DeviceEntry {
            base,
            size,
            device: RwLock::new(Box::new(device)),
        };
        self.devices.insert(base, entry);
    }

    /// Attach a device using Arc for shared access.
    pub fn attach_device_arc<D: DeviceAccess + 'static>(&mut self, base: u32, size: u32, device: Arc<RwLock<D>>) {
        let wrapper = ArcDeviceWrapper { device };
        let entry = DeviceEntry {
            base,
            size,
            device: RwLock::new(Box::new(wrapper)),
        };
        self.devices.insert(base, entry);
    }

    /// Add a bus hook.
    pub fn add_hook<H: BusHook + 'static>(&mut self, hook: H) {
        self.hooks.push(Box::new(hook));
    }

    /// Read from the bus.
    pub fn read(&mut self, addr: u32, width: AccessWidth) -> Result<u64> {
        // Check alignment
        let align = width.bytes();
        if addr % align as u32 != 0 {
            return Err(BusError::MisalignedAccess(addr));
        }

        // Find the device
        for (_, entry) in &self.devices {
            if addr >= entry.base && addr < entry.base + entry.size {
                let offset = addr - entry.base;

                // Call hooks
                let mut value = 0u64;
                let mut handled = false;
                for hook in &self.hooks {
                    if hook.on_read(addr, width, &mut value) {
                        handled = true;
                        break;
                    }
                }

                if !handled {
                    let device = entry.device.write();
                    value = device.read(offset, width)?;
                }

                return Ok(value);
            }
        }

        // Check if address is in memory map
        if let Some(region) = self.memory_map.find_region(addr) {
            if !region.readable {
                return Err(BusError::WriteOnlyRead(addr));
            }
            // Memory access would be handled here
            return Ok(0);
        }

        Err(BusError::UnmappedAddress(addr))
    }

    /// Write to the bus.
    pub fn write(&mut self, addr: u32, value: u64, width: AccessWidth) -> Result<()> {
        // Check alignment
        let align = width.bytes();
        if addr % align as u32 != 0 {
            return Err(BusError::MisalignedAccess(addr));
        }

        // Find the device
        for (_, entry) in &self.devices {
            if addr >= entry.base && addr < entry.base + entry.size {
                let offset = addr - entry.base;

                // Call hooks
                let mut handled = false;
                for hook in &self.hooks {
                    if hook.on_write(addr, width, value) {
                        handled = true;
                        break;
                    }
                }

                if !handled {
                    let mut device = entry.device.write();
                    device.write(offset, value, width)?;
                }

                return Ok(());
            }
        }

        // Check if address is in memory map
        if let Some(region) = self.memory_map.find_region(addr) {
            if !region.writable {
                return Err(BusError::ReadOnlyWrite(addr));
            }
            // Memory access would be handled here
            return Ok(());
        }

        Err(BusError::UnmappedAddress(addr))
    }

    /// Get the memory map.
    pub fn memory_map(&self) -> &MemoryMap {
        &self.memory_map
    }

    /// Get the memory map (mutable).
    pub fn memory_map_mut(&mut self) -> &mut MemoryMap {
        &mut self.memory_map
    }

    /// Reset the bus.
    pub fn reset(&mut self) {
        // Reset all devices
        for (_, entry) in &self.devices {
            // Devices would have a reset method
            let _ = entry.device.read();
        }
    }
}

/// Bus hook trait.
pub trait BusHook: Send + Sync {
    fn on_read(&self, addr: u32, width: AccessWidth, value: &mut u64) -> bool;
    fn on_write(&self, addr: u32, width: AccessWidth, value: u64) -> bool;
}

/// Read hook.
pub struct ReadHook<F>(pub F)
where
    F: Fn(u32, AccessWidth) -> Option<u64> + Send + Sync;

impl<F> BusHook for ReadHook<F>
where
    F: Fn(u32, AccessWidth) -> Option<u64> + Send + Sync,
{
    fn on_read(&self, addr: u32, width: AccessWidth, value: &mut u64) -> bool {
        if let Some(v) = (self.0)(addr, width) {
            *value = v;
            true
        } else {
            false
        }
    }

    fn on_write(&self, _addr: u32, _width: AccessWidth, _value: u64) -> bool {
        false
    }
}

/// Write hook.
pub struct WriteHook<F>(pub F)
where
    F: Fn(u32, AccessWidth, u64) -> bool + Send + Sync;

impl<F> BusHook for WriteHook<F>
where
    F: Fn(u32, AccessWidth, u64) -> bool + Send + Sync,
{
    fn on_read(&self, _addr: u32, _width: AccessWidth, _value: &mut u64) -> bool {
        false
    }

    fn on_write(&self, addr: u32, width: AccessWidth, value: u64) -> bool {
        (self.0)(addr, width, value)
    }
}
//! USB device controller for RP2350.
//!
//! This module implements the USB device controller with:
//! - Full register model for control, status, and endpoints
//! - DPSRAM (Data Packet RAM) for endpoint buffers
//! - SETUP packet handling for control transfers
//! - IN/OUT transfer simulation

use rp2350sim_core::{Device, DeviceId, Result};

/// USB base address.
pub const USB_BASE: u32 = 0x5010_0000;

/// USB DPSRAM base address (4KB buffer RAM).
pub const DPSRAM_BASE: u32 = 0x5010_8000;

/// Size of DPSRAM in bytes (4KB).
const DPSRAM_SIZE: usize = 4096;

/// Maximum endpoint buffer size (64 bytes for full-speed USB).
const EP_MAX_BUFFER_SIZE: usize = 64;

/// Maximum number of endpoints.
const NUM_ENDPOINTS: usize = 16;

/// USB register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const STATUS: u32 = 0x004;
    pub const ADDR_ENDP: u32 = 0x008;
    pub const INTR: u32 = 0x00C;
    pub const INTE: u32 = 0x010;
    pub const INTF: u32 = 0x014;
    pub const INTS: u32 = 0x018;
    pub const SIE_STATUS: u32 = 0x01C;
    pub const BUFF_STATUS: u32 = 0x020;
    pub const BUFF_CPU_SHOULD_HANDLE: u32 = 0x024;
    pub const EP_CTRL: u32 = 0x028;
    pub const EP_STATUS: u32 = 0x02C;
    pub const EP_ABORT: u32 = 0x030;
    pub const MAIN_CTRL: u32 = 0x034;
    pub const SOF_WRITER: u32 = 0x038;
}

/// CTRL register bits.
pub mod ctrl {
    pub const ENABLE: u32 = 1 << 0;
    pub const DEVICE_MODE: u32 = 1 << 1;
    pub const HOST_MODE: u32 = 1 << 2;
    pub const VBUS_DETECT: u32 = 1 << 3;
}

/// STATUS register bits.
pub mod status {
    pub const ENABLED: u32 = 1 << 0;
    pub const DEVICE_MODE: u32 = 1 << 1;
    pub const HOST_MODE: u32 = 1 << 2;
    pub const VBUS_PRESENT: u32 = 1 << 3;
    pub const BUS_RESET: u32 = 1 << 4;
    pub const SUSPENDED: u32 = 1 << 5;
}

/// SIE_STATUS register bits.
pub mod sie_status {
    pub const DATA_SEQ_ERROR: u32 = 1 << 0;
    pub const ACK_REC: u32 = 1 << 1;
    pub const STALL_REC: u32 = 1 << 2;
    pub const NAK_REC: u32 = 1 << 3;
    pub const RX_TIMEOUT: u32 = 1 << 4;
    pub const RX_OVERFLOW: u32 = 1 << 5;
    pub const BIT_STUFF_ERROR: u32 = 1 << 6;
    pub const CRC_ERROR: u32 = 1 << 7;
    pub const BUS_RESET: u32 = 1 << 8;
    pub const TRANS_COMPLETE: u32 = 1 << 9;
    pub const SETUP_REC: u32 = 1 << 10;
    pub const CONNECTED: u32 = 1 << 16;
    pub const RESUME: u32 = 1 << 17;
    pub const VBUS_DETECTED: u32 = 1 << 18;
    pub const SUSPENDED: u32 = 1 << 19;
}

/// EP_CTRL register bits.
pub mod ep_ctrl_bits {
    pub const ENABLE: u32 = 1 << 0;
    pub const DOUBLE_BUFFERED: u32 = 1 << 1;
    pub const INTERRUPT_PER_BUFF: u32 = 1 << 2;
    pub const INTERRUPT_ON_NAK: u32 = 1 << 3;
    pub const TYPE_CONTROL: u32 = 0 << 4;
    pub const TYPE_ISOCHRONOUS: u32 = 1 << 4;
    pub const TYPE_BULK: u32 = 2 << 4;
    pub const TYPE_INTERRUPT: u32 = 3 << 4;
    pub const TYPE_MASK: u32 = 3 << 4;
    pub const INTERRUPT_ON_STALL: u32 = 1 << 6;
    pub const BUFFER_ADDRESS_SHIFT: u32 = 16;
    pub const BUFFER_ADDRESS_MASK: u32 = 0xFFFF_0000;
}

/// Buffer control register bits.
pub mod buf_ctrl {
    /// Buffer length (bits 0-9).
    pub const LENGTH_MASK: u32 = 0x3FF;
    /// Last buffer in transaction.
    pub const LAST: u32 = 1 << 15;
    /// Full buffer (for OUT endpoints).
    pub const FULL: u32 = 1 << 15;
    /// Stall handshake.
    pub const STALL: u32 = 1 << 16;
    /// Available (buffer ready for use).
    pub const AVAILABLE: u32 = 1 << 10;
    /// Data PID toggle.
    pub const DATA_PID: u32 = 1 << 18;
}

/// Endpoint buffer information.
#[derive(Debug, Clone)]
pub struct EpBuffer {
    /// Buffer control register.
    pub control: u32,
    /// Data in the buffer.
    pub data: [u8; EP_MAX_BUFFER_SIZE],
    /// Actual data length.
    pub len: usize,
}

impl Default for EpBuffer {
    fn default() -> Self {
        Self {
            control: 0,
            data: [0; EP_MAX_BUFFER_SIZE],
            len: 0,
        }
    }
}

impl EpBuffer {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self {
            control: 0,
            data: [0; EP_MAX_BUFFER_SIZE],
            len: 0,
        }
    }
}

/// SETUP packet structure (8 bytes).
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SetupPacket {
    pub bmRequestType: u8,
    pub bRequest: u8,
    pub wValue: u16,
    pub wIndex: u16,
    pub wLength: u16,
}

impl SetupPacket {
    /// Parse from bytes.
    pub fn from_bytes(data: &[u8; 8]) -> Self {
        Self {
            bmRequestType: data[0],
            bRequest: data[1],
            wValue: u16::from_le_bytes([data[2], data[3]]),
            wIndex: u16::from_le_bytes([data[4], data[5]]),
            wLength: u16::from_le_bytes([data[6], data[7]]),
        }
    }

    /// Convert to bytes.
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut data = [0u8; 8];
        data[0] = self.bmRequestType;
        data[1] = self.bRequest;
        data[2..4].copy_from_slice(&self.wValue.to_le_bytes());
        data[4..6].copy_from_slice(&self.wIndex.to_le_bytes());
        data[6..8].copy_from_slice(&self.wLength.to_le_bytes());
        data
    }

    /// Check if this is a standard request.
    pub fn is_standard(&self) -> bool {
        (self.bmRequestType & 0x60) == 0x00
    }

    /// Check if this is a class request.
    pub fn is_class(&self) -> bool {
        (self.bmRequestType & 0x60) == 0x20
    }

    /// Check if this is a vendor request.
    pub fn is_vendor(&self) -> bool {
        (self.bmRequestType & 0x60) == 0x40
    }

    /// Check if direction is device-to-host (IN).
    pub fn is_in(&self) -> bool {
        (self.bmRequestType & 0x80) != 0
    }

    /// Get recipient.
    pub fn recipient(&self) -> u8 {
        self.bmRequestType & 0x1F
    }
}

/// Standard USB requests.
pub mod std_requests {
    pub const GET_STATUS: u8 = 0;
    pub const CLEAR_FEATURE: u8 = 1;
    pub const SET_FEATURE: u8 = 3;
    pub const SET_ADDRESS: u8 = 5;
    pub const GET_DESCRIPTOR: u8 = 6;
    pub const SET_DESCRIPTOR: u8 = 7;
    pub const GET_CONFIGURATION: u8 = 8;
    pub const SET_CONFIGURATION: u8 = 9;
    pub const GET_INTERFACE: u8 = 10;
    pub const SET_INTERFACE: u8 = 11;
    pub const SYNCH_FRAME: u8 = 12;
}

/// USB device controller.
#[derive(Debug)]
pub struct Usb {
    // Main registers
    ctrl: u32,
    status: u32,
    addr_endp: u32,
    intr: u32,
    inte: u32,
    intf: u32,
    ints: u32,
    sie_status: u32,
    buff_status: u32,
    buff_cpu_should_handle: u32,
    main_ctrl: u32,
    sof_writer: u32,

    // Endpoint registers
    ep_ctrl: [u32; NUM_ENDPOINTS],
    ep_status: [u32; NUM_ENDPOINTS],
    ep_abort: [u32; NUM_ENDPOINTS],

    // DPSRAM (Data Packet RAM) - 4KB
    dpsram: [u8; DPSRAM_SIZE],

    // Endpoint buffers (IN and OUT for each endpoint)
    ep_in_buffers: [EpBuffer; NUM_ENDPOINTS],
    ep_out_buffers: [EpBuffer; NUM_ENDPOINTS],

    // Last received SETUP packet
    last_setup: SetupPacket,

    // State flags
    enabled: bool,
    device_mode: bool,
    vbus_detected: bool,
    connected: bool,
    suspended: bool,

    // Device configuration
    configuration: u8,
    alternate_settings: [u8; 4],
}

impl Default for Usb {
    fn default() -> Self {
        Self::new()
    }
}

impl Usb {
    /// Create a new USB controller.
    pub fn new() -> Self {
        Self {
            ctrl: 0,
            status: 0,
            addr_endp: 0,
            intr: 0,
            inte: 0,
            intf: 0,
            ints: 0,
            sie_status: 0,
            buff_status: 0,
            buff_cpu_should_handle: 0,
            main_ctrl: 0,
            sof_writer: 0,
            ep_ctrl: [0; NUM_ENDPOINTS],
            ep_status: [0; NUM_ENDPOINTS],
            ep_abort: [0; NUM_ENDPOINTS],
            dpsram: [0; DPSRAM_SIZE],
            ep_in_buffers: std::array::from_fn(|_| EpBuffer::new()),
            ep_out_buffers: std::array::from_fn(|_| EpBuffer::new()),
            last_setup: SetupPacket::default(),
            enabled: false,
            device_mode: true,
            vbus_detected: false,
            connected: false,
            suspended: false,
            configuration: 0,
            alternate_settings: [0; 4],
        }
    }

    // State query methods

    /// Check if USB is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if in device mode.
    pub fn is_device_mode(&self) -> bool {
        self.device_mode
    }

    /// Check if VBUS is detected.
    pub fn is_vbus_detected(&self) -> bool {
        self.vbus_detected
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Check if suspended.
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    /// Get device address.
    pub fn get_address(&self) -> u8 {
        (self.addr_endp & 0x7F) as u8
    }

    /// Set device address.
    pub fn set_address(&mut self, addr: u8) {
        self.addr_endp = (self.addr_endp & !0x7F) | (addr as u32 & 0x7F);
    }

    /// Get current configuration.
    pub fn get_configuration(&self) -> u8 {
        self.configuration
    }

    /// Get last SETUP packet.
    pub fn get_last_setup(&self) -> &SetupPacket {
        &self.last_setup
    }

    // Internal methods

    fn enable(&mut self) {
        self.enabled = true;
        self.status |= status::ENABLED;
        if self.device_mode {
            self.status |= status::DEVICE_MODE;
        } else {
            self.status |= status::HOST_MODE;
        }
    }

    fn disable(&mut self) {
        self.enabled = false;
        self.connected = false;
        self.suspended = false;
        self.status = 0;
        self.sie_status = 0;
    }

    fn clear_sie_status(&mut self, bits: u32) {
        self.sie_status &= !bits;
    }

    // Bus simulation methods

    /// Simulate VBUS connection.
    pub fn connect_vbus(&mut self) {
        self.vbus_detected = true;
        self.sie_status |= sie_status::VBUS_DETECTED;
        self.status |= status::VBUS_PRESENT;
    }

    /// Simulate VBUS disconnection.
    pub fn disconnect_vbus(&mut self) {
        self.vbus_detected = false;
        self.sie_status &= !sie_status::VBUS_DETECTED;
        self.status &= !status::VBUS_PRESENT;
        self.connected = false;
        self.sie_status &= !sie_status::CONNECTED;
    }

    /// Simulate bus reset.
    pub fn bus_reset(&mut self) {
        self.sie_status |= sie_status::BUS_RESET;
        self.addr_endp = 0; // Reset address to 0
        self.connected = true;
        self.sie_status |= sie_status::CONNECTED;
        self.suspended = false;
        self.sie_status &= !sie_status::SUSPENDED;
        self.configuration = 0;
    }

    /// Simulate suspend.
    pub fn suspend(&mut self) {
        self.suspended = true;
        self.sie_status |= sie_status::SUSPENDED;
        self.status |= status::SUSPENDED;
    }

    /// Simulate resume.
    pub fn resume(&mut self) {
        self.suspended = false;
        self.sie_status &= !sie_status::SUSPENDED;
        self.sie_status |= sie_status::RESUME;
        self.status &= !status::SUSPENDED;
    }

    /// Check for pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        self.ints != 0
    }

    // DPSRAM access methods

    /// Read from DPSRAM.
    pub fn read_dpsram(&self, offset: u32) -> u8 {
        if offset < DPSRAM_SIZE as u32 {
            self.dpsram[offset as usize]
        } else {
            0
        }
    }

    /// Write to DPSRAM.
    pub fn write_dpsram(&mut self, offset: u32, value: u8) {
        if offset < DPSRAM_SIZE as u32 {
            self.dpsram[offset as usize] = value;
        }
    }

    /// Read DPSRAM block.
    pub fn read_dpsram_block(&self, offset: u32, len: usize) -> Vec<u8> {
        let end = ((offset as usize) + len).min(DPSRAM_SIZE);
        self.dpsram[offset as usize..end].to_vec()
    }

    /// Write DPSRAM block.
    pub fn write_dpsram_block(&mut self, offset: u32, data: &[u8]) {
        let start = offset as usize;
        let end = (start + data.len()).min(DPSRAM_SIZE);
        let copy_len = end - start;
        self.dpsram[start..end].copy_from_slice(&data[..copy_len]);
    }

    // Endpoint methods

    /// Check if endpoint is enabled.
    pub fn is_endpoint_enabled(&self, ep: usize) -> bool {
        if ep < NUM_ENDPOINTS {
            (self.ep_ctrl[ep] & ep_ctrl_bits::ENABLE) != 0
        } else {
            false
        }
    }

    /// Get endpoint type.
    pub fn get_endpoint_type(&self, ep: usize) -> u32 {
        if ep < NUM_ENDPOINTS {
            self.ep_ctrl[ep] & ep_ctrl_bits::TYPE_MASK
        } else {
            0
        }
    }

    /// Get endpoint buffer address from EP_CTRL.
    pub fn get_endpoint_buffer_address(&self, ep: usize) -> u32 {
        if ep < NUM_ENDPOINTS {
            (self.ep_ctrl[ep] & ep_ctrl_bits::BUFFER_ADDRESS_MASK) >> ep_ctrl_bits::BUFFER_ADDRESS_SHIFT
        } else {
            0
        }
    }

    /// Get buffer length from buffer control.
    pub fn get_buffer_length(control: u32) -> usize {
        (control & buf_ctrl::LENGTH_MASK) as usize
    }

    /// Check if buffer is available.
    pub fn is_buffer_available(control: u32) -> bool {
        (control & buf_ctrl::AVAILABLE) != 0
    }

    /// Check if buffer is full (for OUT endpoints).
    pub fn is_buffer_full(control: u32) -> bool {
        (control & buf_ctrl::FULL) != 0
    }

    // Transfer simulation methods

    /// Simulate receiving a SETUP packet.
    pub fn receive_setup(&mut self, packet: &SetupPacket) {
        self.last_setup = *packet;

        // Store in EP0 OUT buffer
        let bytes = packet.to_bytes();
        self.ep_out_buffers[0].data[..8].copy_from_slice(&bytes);
        self.ep_out_buffers[0].len = 8;
        self.ep_out_buffers[0].control = 8 | buf_ctrl::FULL;

        // Also store in DPSRAM at offset 0x100 (EP0 OUT buffer)
        self.write_dpsram_block(0x100, &bytes);

        // Set SETUP_REC bit
        self.sie_status |= sie_status::SETUP_REC;

        // Update buffer status
        self.buff_status |= 1 << 0; // EP0 OUT buffer ready
    }

    /// Simulate receiving OUT data on an endpoint.
    pub fn receive_out(&mut self, ep: usize, data: &[u8]) -> Result<()> {
        if ep >= NUM_ENDPOINTS {
            return Ok(());
        }

        let len = data.len().min(EP_MAX_BUFFER_SIZE);
        self.ep_out_buffers[ep].data[..len].copy_from_slice(&data[..len]);
        self.ep_out_buffers[ep].len = len;
        self.ep_out_buffers[ep].control = (len as u32) | buf_ctrl::FULL;

        // Write to DPSRAM at endpoint buffer offset
        let buf_addr = self.get_endpoint_buffer_address(ep);
        if buf_addr > 0 {
            self.write_dpsram_block(buf_addr, &data[..len]);
        }

        // Update buffer status (bit for each endpoint OUT)
        self.buff_status |= 1 << (ep * 2);

        Ok(())
    }

    /// Simulate sending IN data on an endpoint.
    pub fn send_in(&mut self, ep: usize, data: &[u8]) -> Result<()> {
        if ep >= NUM_ENDPOINTS {
            return Ok(());
        }

        let len = data.len().min(EP_MAX_BUFFER_SIZE);
        self.ep_in_buffers[ep].data[..len].copy_from_slice(&data[..len]);
        self.ep_in_buffers[ep].len = len;
        self.ep_in_buffers[ep].control = (len as u32) | buf_ctrl::AVAILABLE;

        // Write to DPSRAM
        let buf_addr = self.get_endpoint_buffer_address(ep);
        if buf_addr > 0 {
            self.write_dpsram_block(buf_addr, &data[..len]);
        }

        // Mark buffer as available for hardware to send
        self.buff_status |= 1 << (ep * 2 + 1); // IN buffers are odd bits

        // Simulate ACK received
        self.sie_status |= sie_status::ACK_REC;

        Ok(())
    }

    /// Get data from IN buffer (for testing).
    pub fn get_in_data(&self, ep: usize) -> Option<&[u8]> {
        if ep < NUM_ENDPOINTS {
            Some(&self.ep_in_buffers[ep].data[..self.ep_in_buffers[ep].len])
        } else {
            None
        }
    }

    /// Get data from OUT buffer (for testing).
    pub fn get_out_data(&self, ep: usize) -> Option<&[u8]> {
        if ep < NUM_ENDPOINTS {
            Some(&self.ep_out_buffers[ep].data[..self.ep_out_buffers[ep].len])
        } else {
            None
        }
    }

    /// Stall an endpoint.
    pub fn stall_endpoint(&mut self, ep: usize, in_dir: bool) {
        if ep >= NUM_ENDPOINTS {
            return;
        }

        if in_dir {
            self.ep_in_buffers[ep].control |= buf_ctrl::STALL;
        } else {
            self.ep_out_buffers[ep].control |= buf_ctrl::STALL;
        }

        self.sie_status |= sie_status::STALL_REC;
    }

    /// Clear endpoint stall.
    pub fn clear_stall(&mut self, ep: usize, in_dir: bool) {
        if ep >= NUM_ENDPOINTS {
            return;
        }

        if in_dir {
            self.ep_in_buffers[ep].control &= !buf_ctrl::STALL;
        } else {
            self.ep_out_buffers[ep].control &= !buf_ctrl::STALL;
        }
    }

    /// Check if endpoint is stalled.
    pub fn is_endpoint_stalled(&self, ep: usize, in_dir: bool) -> bool {
        if ep >= NUM_ENDPOINTS {
            return false;
        }

        if in_dir {
            (self.ep_in_buffers[ep].control & buf_ctrl::STALL) != 0
        } else {
            (self.ep_out_buffers[ep].control & buf_ctrl::STALL) != 0
        }
    }

    // Standard request handlers

    /// Handle SET_ADDRESS request.
    pub fn handle_set_address(&mut self, addr: u8) {
        self.set_address(addr);
    }

    /// Handle SET_CONFIGURATION request.
    pub fn handle_set_configuration(&mut self, config: u8) {
        self.configuration = config;
    }

    /// Handle SET_INTERFACE request.
    pub fn handle_set_interface(&mut self, interface: u8, alternate: u8) {
        if (interface as usize) < self.alternate_settings.len() {
            self.alternate_settings[interface as usize] = alternate;
        }
    }

    /// Process a standard request (for simulation).
    pub fn process_standard_request(&mut self, setup: &SetupPacket) -> Result<Vec<u8>> {
        match setup.bRequest {
            std_requests::GET_STATUS => {
                // Return 2 bytes: self-powered and remote-wakeup status
                Ok(vec![0x01, 0x00]) // Self-powered
            }
            std_requests::SET_ADDRESS => {
                self.set_address((setup.wValue & 0x7F) as u8);
                Ok(vec![])
            }
            std_requests::GET_DESCRIPTOR => {
                // Return empty - real implementation would return descriptor
                Ok(vec![])
            }
            std_requests::SET_CONFIGURATION => {
                self.configuration = (setup.wValue & 0xFF) as u8;
                Ok(vec![])
            }
            std_requests::SET_INTERFACE => {
                let interface = (setup.wIndex & 0xFF) as u8;
                let alternate = (setup.wValue & 0xFF) as u8;
                self.handle_set_interface(interface, alternate);
                Ok(vec![])
            }
            std_requests::CLEAR_FEATURE => {
                // Handle endpoint halt
                if setup.recipient() == 0x02 {
                    // Endpoint
                    let ep = (setup.wIndex & 0x0F) as usize;
                    let in_dir = (setup.wIndex & 0x80) != 0;
                    self.clear_stall(ep, in_dir);
                }
                Ok(vec![])
            }
            std_requests::SET_FEATURE => {
                // Handle endpoint halt
                if setup.recipient() == 0x02 && setup.wValue == 0 {
                    // ENDPOINT_HALT
                    let ep = (setup.wIndex & 0x0F) as usize;
                    let in_dir = (setup.wIndex & 0x80) != 0;
                    self.stall_endpoint(ep, in_dir);
                }
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }
}

impl Device for Usb {
    fn id(&self) -> DeviceId {
        DeviceId::USB
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - USB_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::STATUS => Ok(self.status),
            regs::ADDR_ENDP => Ok(self.addr_endp),
            regs::INTR => Ok(self.intr),
            regs::INTE => Ok(self.inte),
            regs::INTF => Ok(self.intf),
            regs::INTS => Ok(self.ints),
            regs::SIE_STATUS => Ok(self.sie_status),
            regs::BUFF_STATUS => Ok(self.buff_status),
            regs::BUFF_CPU_SHOULD_HANDLE => Ok(self.buff_cpu_should_handle),
            regs::MAIN_CTRL => Ok(self.main_ctrl),
            regs::SOF_WRITER => Ok(self.sof_writer),
            offset if offset >= regs::EP_CTRL && offset < regs::EP_CTRL + NUM_ENDPOINTS as u32 * 4 => {
                let idx = ((offset - regs::EP_CTRL) / 4) as usize;
                if idx < NUM_ENDPOINTS {
                    Ok(self.ep_ctrl[idx])
                } else {
                    Ok(0)
                }
            }
            offset if offset >= regs::EP_STATUS && offset < regs::EP_STATUS + NUM_ENDPOINTS as u32 * 4 => {
                let idx = ((offset - regs::EP_STATUS) / 4) as usize;
                if idx < NUM_ENDPOINTS {
                    Ok(self.ep_status[idx])
                } else {
                    Ok(0)
                }
            }
            offset if offset >= regs::EP_ABORT && offset < regs::EP_ABORT + NUM_ENDPOINTS as u32 * 4 => {
                let idx = ((offset - regs::EP_ABORT) / 4) as usize;
                if idx < NUM_ENDPOINTS {
                    Ok(self.ep_abort[idx])
                } else {
                    Ok(0)
                }
            }
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - USB_BASE;

        match offset {
            regs::CTRL => {
                self.ctrl = value;
                if (value & ctrl::ENABLE) != 0 {
                    self.enable();
                } else {
                    self.disable();
                }
                self.device_mode = (value & ctrl::HOST_MODE) == 0;
            }
            regs::ADDR_ENDP => {
                self.addr_endp = value;
            }
            regs::INTE => {
                self.inte = value;
                self.ints = self.intr | self.intf & self.inte;
            }
            regs::INTF => {
                self.intf = value;
                self.ints = self.intr | self.intf & self.inte;
            }
            regs::SIE_STATUS => {
                // Write 1 to clear
                self.clear_sie_status(value);
            }
            regs::BUFF_STATUS => {
                // Write 1 to clear
                self.buff_status &= !value;
            }
            regs::BUFF_CPU_SHOULD_HANDLE => {
                self.buff_cpu_should_handle = value;
            }
            regs::MAIN_CTRL => {
                self.main_ctrl = value;
            }
            regs::SOF_WRITER => {
                self.sof_writer = value;
            }
            offset if offset >= regs::EP_CTRL && offset < regs::EP_CTRL + NUM_ENDPOINTS as u32 * 4 => {
                let idx = ((offset - regs::EP_CTRL) / 4) as usize;
                if idx < NUM_ENDPOINTS {
                    self.ep_ctrl[idx] = value;
                }
            }
            offset if offset >= regs::EP_STATUS && offset < regs::EP_STATUS + NUM_ENDPOINTS as u32 * 4 => {
                let idx = ((offset - regs::EP_STATUS) / 4) as usize;
                if idx < NUM_ENDPOINTS {
                    // Write 1 to clear status bits
                    self.ep_status[idx] &= !value;
                }
            }
            offset if offset >= regs::EP_ABORT && offset < regs::EP_ABORT + NUM_ENDPOINTS as u32 * 4 => {
                let idx = ((offset - regs::EP_ABORT) / 4) as usize;
                if idx < NUM_ENDPOINTS {
                    self.ep_abort[idx] = value;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USB_BASE: u32 = super::USB_BASE;

    #[test]
    fn test_usb_creation() {
        let usb = Usb::new();
        assert!(!usb.is_enabled());
        assert!(usb.is_device_mode());
        assert!(!usb.is_vbus_detected());
        assert!(!usb.is_connected());
    }

    #[test]
    fn test_usb_enable() {
        let mut usb = Usb::new();

        usb.write(USB_BASE + regs::CTRL, ctrl::ENABLE).unwrap();
        assert!(usb.is_enabled());
        assert!(usb.is_device_mode());
    }

    #[test]
    fn test_usb_disable() {
        let mut usb = Usb::new();

        usb.write(USB_BASE + regs::CTRL, ctrl::ENABLE).unwrap();
        assert!(usb.is_enabled());

        usb.write(USB_BASE + regs::CTRL, 0).unwrap();
        assert!(!usb.is_enabled());
    }

    #[test]
    fn test_usb_host_mode() {
        let mut usb = Usb::new();

        usb.write(USB_BASE + regs::CTRL, ctrl::ENABLE | ctrl::HOST_MODE).unwrap();
        assert!(usb.is_enabled());
        assert!(!usb.is_device_mode());
    }

    #[test]
    fn test_usb_address() {
        let mut usb = Usb::new();

        usb.set_address(0x42);
        assert_eq!(usb.get_address(), 0x42);

        assert_eq!(usb.read(USB_BASE + regs::ADDR_ENDP).unwrap() & 0x7F, 0x42);
    }

    #[test]
    fn test_usb_vbus_detect() {
        let mut usb = Usb::new();

        usb.connect_vbus();
        assert!(usb.is_vbus_detected());

        let status = usb.read(USB_BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::VBUS_PRESENT, status::VBUS_PRESENT);

        let sie_status = usb.read(USB_BASE + regs::SIE_STATUS).unwrap();
        assert_eq!(sie_status & sie_status::VBUS_DETECTED, sie_status::VBUS_DETECTED);
    }

    #[test]
    fn test_usb_vbus_disconnect() {
        let mut usb = Usb::new();

        usb.connect_vbus();
        assert!(usb.is_vbus_detected());

        usb.disconnect_vbus();
        assert!(!usb.is_vbus_detected());
    }

    #[test]
    fn test_usb_bus_reset() {
        let mut usb = Usb::new();

        usb.set_address(0x42);
        usb.bus_reset();

        assert_eq!(usb.get_address(), 0);
        assert!(usb.is_connected());
    }

    #[test]
    fn test_usb_suspend_resume() {
        let mut usb = Usb::new();

        usb.suspend();
        assert!(usb.is_suspended());

        usb.resume();
        assert!(!usb.is_suspended());
    }

    #[test]
    fn test_usb_interrupt_enable() {
        let mut usb = Usb::new();

        usb.write(USB_BASE + regs::INTE, 0xFF).unwrap();
        let inte = usb.read(USB_BASE + regs::INTE).unwrap();
        assert_eq!(inte, 0xFF);
    }

    #[test]
    fn test_usb_sie_status_clear() {
        let mut usb = Usb::new();

        usb.sie_status = sie_status::BUS_RESET | sie_status::SETUP_REC;

        usb.write(USB_BASE + regs::SIE_STATUS, sie_status::BUS_RESET).unwrap();

        let sie_status = usb.read(USB_BASE + regs::SIE_STATUS).unwrap();
        assert_eq!(sie_status & super::sie_status::BUS_RESET, 0);
        assert_eq!(sie_status & super::sie_status::SETUP_REC, super::sie_status::SETUP_REC);
    }

    #[test]
    fn test_usb_endpoint_control() {
        let mut usb = Usb::new();

        usb.write(USB_BASE + regs::EP_CTRL, 0x12345678).unwrap();
        let ep0_ctrl = usb.read(USB_BASE + regs::EP_CTRL).unwrap();
        assert_eq!(ep0_ctrl, 0x12345678);

        usb.write(USB_BASE + regs::EP_CTRL + 4, 0xDEADBEEF).unwrap();
        let ep1_ctrl = usb.read(USB_BASE + regs::EP_CTRL + 4).unwrap();
        assert_eq!(ep1_ctrl, 0xDEADBEEF);
    }

    #[test]
    fn test_usb_endpoint_status() {
        let mut usb = Usb::new();

        usb.write(USB_BASE + regs::EP_STATUS, 0x1).unwrap();
        let ep0_status = usb.read(USB_BASE + regs::EP_STATUS).unwrap();
        assert_eq!(ep0_status, 0x1);
    }

    #[test]
    fn test_usb_buffer_status() {
        let mut usb = Usb::new();

        usb.buff_status = 0xFF;

        usb.write(USB_BASE + regs::BUFF_STATUS, 0x0F).unwrap();

        let buff_status = usb.read(USB_BASE + regs::BUFF_STATUS).unwrap();
        assert_eq!(buff_status, 0xF0);
    }

    #[test]
    fn test_usb_device_reset() {
        let mut usb = Usb::new();

        usb.write(USB_BASE + regs::CTRL, ctrl::ENABLE).unwrap();
        usb.set_address(0x42);
        usb.connect_vbus();

        usb.reset();

        assert!(!usb.is_enabled());
        assert!(!usb.is_vbus_detected());
        assert_eq!(usb.get_address(), 0);
    }

    #[test]
    fn test_usb_status_register() {
        let mut usb = Usb::new();

        usb.write(USB_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        let status = usb.read(USB_BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::ENABLED, status::ENABLED);
        assert_eq!(status & status::DEVICE_MODE, status::DEVICE_MODE);
    }

    // New tests for expanded functionality

    #[test]
    fn test_dpsram_read_write() {
        let mut usb = Usb::new();

        usb.write_dpsram(0, 0x42);
        assert_eq!(usb.read_dpsram(0), 0x42);

        usb.write_dpsram_block(0x100, &[1, 2, 3, 4, 5]);
        let data = usb.read_dpsram_block(0x100, 5);
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_setup_packet_parsing() {
        let data = [0x80, 0x06, 0x00, 0x01, 0x00, 0x00, 0x12, 0x00];
        let setup = SetupPacket::from_bytes(&data);

        assert!(setup.is_standard());
        assert!(setup.is_in());
        assert_eq!(setup.bRequest, std_requests::GET_DESCRIPTOR);
        assert_eq!(setup.wValue, 0x0100);
        assert_eq!(setup.wIndex, 0x0000);
        assert_eq!(setup.wLength, 0x0012);
    }

    #[test]
    fn test_setup_packet_to_bytes() {
        let setup = SetupPacket {
            bmRequestType: 0x00,
            bRequest: std_requests::SET_ADDRESS,
            wValue: 0x007F,
            wIndex: 0x0000,
            wLength: 0x0000,
        };

        let bytes = setup.to_bytes();
        assert_eq!(bytes[0], 0x00);
        assert_eq!(bytes[1], std_requests::SET_ADDRESS);
        assert_eq!(u16::from_le_bytes([bytes[2], bytes[3]]), 0x007F);
    }

    #[test]
    fn test_receive_setup() {
        let mut usb = Usb::new();

        let setup = SetupPacket {
            bmRequestType: 0x00,
            bRequest: std_requests::SET_ADDRESS,
            wValue: 0x0042,
            wIndex: 0x0000,
            wLength: 0x0000,
        };

        usb.receive_setup(&setup);

        // Check SETUP_REC bit is set
        let sie_status = usb.read(USB_BASE + regs::SIE_STATUS).unwrap();
        assert_eq!(sie_status & sie_status::SETUP_REC, sie_status::SETUP_REC);

        // Check buffer status
        let buff_status = usb.read(USB_BASE + regs::BUFF_STATUS).unwrap();
        assert_ne!(buff_status & 0x1, 0);

        // Check last setup packet
        assert_eq!(usb.get_last_setup().bRequest, std_requests::SET_ADDRESS);
    }

    #[test]
    fn test_receive_out() {
        let mut usb = Usb::new();

        // Configure endpoint 1
        usb.write(USB_BASE + regs::EP_CTRL + 4, ep_ctrl_bits::ENABLE | (0x200 << 16)).unwrap();

        let data = [1, 2, 3, 4, 5, 6, 7, 8];
        usb.receive_out(1, &data).unwrap();

        // Check OUT data
        let out_data = usb.get_out_data(1).unwrap();
        assert_eq!(out_data, &data);

        // Check buffer status
        let buff_status = usb.read(USB_BASE + regs::BUFF_STATUS).unwrap();
        assert_ne!(buff_status & (1 << 2), 0); // EP1 OUT is bit 2
    }

    #[test]
    fn test_send_in() {
        let mut usb = Usb::new();

        // Configure endpoint 2
        usb.write(USB_BASE + regs::EP_CTRL + 8, ep_ctrl_bits::ENABLE | (0x300 << 16)).unwrap();

        let data = [0xA0, 0xA1, 0xA2, 0xA3];
        usb.send_in(2, &data).unwrap();

        // Check IN data
        let in_data = usb.get_in_data(2).unwrap();
        assert_eq!(in_data, &data);

        // Check buffer status
        let buff_status = usb.read(USB_BASE + regs::BUFF_STATUS).unwrap();
        assert_ne!(buff_status & (1 << 5), 0); // EP2 IN is bit 5
    }

    #[test]
    fn test_endpoint_stall() {
        let mut usb = Usb::new();

        usb.stall_endpoint(0, true);
        assert!(usb.is_endpoint_stalled(0, true));

        usb.clear_stall(0, true);
        assert!(!usb.is_endpoint_stalled(0, true));
    }

    #[test]
    fn test_standard_request_set_address() {
        let mut usb = Usb::new();

        let setup = SetupPacket {
            bmRequestType: 0x00,
            bRequest: std_requests::SET_ADDRESS,
            wValue: 0x007F,
            wIndex: 0x0000,
            wLength: 0x0000,
        };

        usb.process_standard_request(&setup).unwrap();
        assert_eq!(usb.get_address(), 0x7F);
    }

    #[test]
    fn test_standard_request_set_configuration() {
        let mut usb = Usb::new();

        let setup = SetupPacket {
            bmRequestType: 0x00,
            bRequest: std_requests::SET_CONFIGURATION,
            wValue: 0x0001,
            wIndex: 0x0000,
            wLength: 0x0000,
        };

        usb.process_standard_request(&setup).unwrap();
        assert_eq!(usb.get_configuration(), 1);
    }

    #[test]
    fn test_standard_request_get_status() {
        let mut usb = Usb::new();

        let setup = SetupPacket {
            bmRequestType: 0x80, // Device-to-host, standard, device
            bRequest: std_requests::GET_STATUS,
            wValue: 0x0000,
            wIndex: 0x0000,
            wLength: 0x0002,
        };

        let response = usb.process_standard_request(&setup).unwrap();
        assert_eq!(response.len(), 2);
        assert_eq!(response[0] & 0x01, 0x01); // Self-powered
    }

    #[test]
    fn test_endpoint_buffer_address() {
        let mut usb = Usb::new();

        // Set buffer address in EP_CTRL
        usb.write(USB_BASE + regs::EP_CTRL + 4, 0x01000000).unwrap();

        let addr = usb.get_endpoint_buffer_address(1);
        assert_eq!(addr, 0x100);
    }

    #[test]
    fn test_buffer_control() {
        let control = 0x0040 | buf_ctrl::AVAILABLE | buf_ctrl::LAST;

        assert_eq!(Usb::get_buffer_length(control), 64);
        assert!(Usb::is_buffer_available(control));
    }
}
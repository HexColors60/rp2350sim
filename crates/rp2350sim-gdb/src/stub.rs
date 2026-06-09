//! GDB Remote Serial Protocol stub implementation.
//!
//! This module provides the main GDB stub that handles communication
//! with GDB and delegates operations to the target.

use crate::protocol::{calculate_checksum, parse_command, bytes_to_hex, GdbCommand, GdbResponse};
use crate::target::{BreakpointKind, GdbTarget};
use log::{debug, error, info, trace};

/// GDB stub state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StubState {
    /// Not connected
    Disconnected,
    /// Connected and waiting for commands
    Connected,
    /// Target is running
    Running,
}

/// GDB stub for RP2350 simulator.
pub struct GdbStub<T: GdbTarget> {
    /// Target being debugged
    target: T,
    /// Current state
    state: StubState,
    #[allow(dead_code)]
    /// Pending output
    output_buffer: String,
    /// No-ack mode enabled
    no_ack_mode: bool,
}

impl<T: GdbTarget> GdbStub<T> {
    /// Create a new GDB stub.
    pub fn new(target: T) -> Self {
        Self {
            target,
            state: StubState::Disconnected,
            output_buffer: String::new(),
            no_ack_mode: false,
        }
    }

    /// Get a reference to the target.
    pub fn target(&self) -> &T {
        &self.target
    }

    /// Get a mutable reference to the target.
    pub fn target_mut(&mut self) -> &mut T {
        &mut self.target
    }

    /// Get the current state.
    pub fn state(&self) -> StubState {
        self.state
    }

    /// Process incoming data from GDB.
    /// Returns data to send back to GDB.
    pub fn process(&mut self, input: &str) -> String {
        let mut output = String::new();

        for packet in Self::extract_packets(input) {
            let response = self.handle_packet(&packet);
            let response_str = self.format_response(&response);
            output.push_str(&response_str);
        }

        output
    }

    /// Extract packets from input data.
    fn extract_packets(input: &str) -> Vec<String> {
        let mut packets = Vec::new();
        let mut current = String::new();
        let mut in_packet = false;

        for c in input.chars() {
            if c == '$' {
                in_packet = true;
                current.clear();
            } else if c == '#' && in_packet {
                in_packet = false;
                packets.push(current.clone());
                current.clear();
            } else if in_packet {
                current.push(c);
            }
        }

        packets
    }

    /// Handle a single packet.
    fn handle_packet(&mut self, packet: &str) -> GdbResponse {
        // Split packet into data and checksum
        let parts: Vec<&str> = packet.splitn(2, ':').collect();
        let (data, checksum_str) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            // No checksum, just data
            return self.handle_data(packet);
        };

        // Verify checksum
        let expected_checksum = calculate_checksum(data);
        if let Ok(received_checksum) = u8::from_str_radix(checksum_str, 16) {
            if expected_checksum != received_checksum {
                error!("Checksum mismatch: expected {:02x}, got {:02x}", expected_checksum, received_checksum);
                return GdbResponse::Error(0x01);
            }
        }

        // Handle the data
        self.handle_data(data)
    }

    /// Handle packet data.
    fn handle_data(&mut self, data: &str) -> GdbResponse {
        trace!("Handling GDB command: {}", data);

        if data.is_empty() {
            return GdbResponse::Empty;
        }

        match parse_command(data) {
            Ok(cmd) => self.handle_command(cmd),
            Err(e) => {
                error!("Failed to parse command: {}", e);
                GdbResponse::Error(0x01)
            }
        }
    }

    /// Handle a parsed command.
    fn handle_command(&mut self, cmd: GdbCommand) -> GdbResponse {
        match cmd {
            GdbCommand::Continue => {
                self.state = StubState::Running;
                if let Err(e) = self.target.continue_exec() {
                    error!("Continue failed: {}", e);
                    return GdbResponse::Error(0x01);
                }
                GdbResponse::Ok
            }
            GdbCommand::Step => {
                if let Err(e) = self.target.step() {
                    error!("Step failed: {}", e);
                    return GdbResponse::Error(0x01);
                }
                GdbResponse::Signal(self.target.get_last_signal())
            }
            GdbCommand::ReadRegisters => {
                match self.target.read_registers() {
                    Ok(data) => GdbResponse::Data(bytes_to_hex(&data)),
                    Err(e) => {
                        error!("Read registers failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::WriteRegisters(data) => {
                match self.target.write_registers(&data) {
                    Ok(()) => GdbResponse::Ok,
                    Err(e) => {
                        error!("Write registers failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::ReadMemory { addr, length } => {
                match self.target.read_memory(addr, length) {
                    Ok(data) => GdbResponse::Data(bytes_to_hex(&data)),
                    Err(e) => {
                        error!("Read memory failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::WriteMemory { addr, data } => {
                match self.target.write_memory(addr, &data) {
                    Ok(()) => GdbResponse::Ok,
                    Err(e) => {
                        error!("Write memory failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::ReadRegister(reg) => {
                match self.target.read_register(reg) {
                    Ok(data) => GdbResponse::Data(bytes_to_hex(&data)),
                    Err(e) => {
                        error!("Read register failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::WriteRegister { reg, value } => {
                match self.target.write_register(reg, &value) {
                    Ok(()) => GdbResponse::Ok,
                    Err(e) => {
                        error!("Write register failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::SetBreakpoint { type_, addr, kind: _ } => {
                let bp_kind = match BreakpointKind::from_type(type_) {
                    Some(k) => k,
                    None => return GdbResponse::Error(0x01),
                };
                match self.target.set_breakpoint(addr, bp_kind) {
                    Ok(()) => GdbResponse::Ok,
                    Err(e) => {
                        error!("Set breakpoint failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::RemoveBreakpoint { type_, addr, kind: _ } => {
                let bp_kind = match BreakpointKind::from_type(type_) {
                    Some(k) => k,
                    None => return GdbResponse::Error(0x01),
                };
                match self.target.remove_breakpoint(addr, bp_kind) {
                    Ok(()) => GdbResponse::Ok,
                    Err(e) => {
                        error!("Remove breakpoint failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::Query(query) => {
                // Handle special queries
                if query.starts_with("Xfer:features:read:target.xml:") {
                    if let Some(desc) = self.target.get_target_description() {
                        let offset_len: Vec<&str> = query.split(':').nth(4).unwrap_or("").split(',').collect();
                        if offset_len.len() == 2 {
                            if let (Ok(offset), Ok(len)) = (usize::from_str_radix(offset_len[0], 16), usize::from_str_radix(offset_len[1], 16)) {
                                let desc_bytes = desc.as_bytes();
                                let end = (offset + len).min(desc_bytes.len());
                                let data: String = desc_bytes[offset..end].iter().map(|b| format!("{:02x}", b)).collect();
                                let more = end < desc_bytes.len();
                                return GdbResponse::Data(format!("{}{}", if more { "m" } else { "l" }, data));
                            }
                        }
                    }
                    return GdbResponse::Unsupported;
                }
                
                if query.starts_with("Xfer:memory-map:read::") {
                    if let Some(map) = self.target.get_memory_map() {
                        let offset_len: Vec<&str> = query.split(':').nth(4).unwrap_or("").split(',').collect();
                        if offset_len.len() == 2 {
                            if let (Ok(offset), Ok(len)) = (usize::from_str_radix(offset_len[0], 16), usize::from_str_radix(offset_len[1], 16)) {
                                let map_bytes = map.as_bytes();
                                let end = (offset + len).min(map_bytes.len());
                                let data: String = map_bytes[offset..end].iter().map(|b| format!("{:02x}", b)).collect();
                                let more = end < map_bytes.len();
                                return GdbResponse::Data(format!("{}{}", if more { "m" } else { "l" }, data));
                            }
                        }
                    }
                    return GdbResponse::Unsupported;
                }

                self.target.handle_query(&query).unwrap_or(GdbResponse::Unsupported)
            }
            GdbCommand::Set(name, _value) => {
                match name.as_str() {
                    "StartNoAckMode" => {
                        self.no_ack_mode = true;
                        GdbResponse::Ok
                    }
                    _ => GdbResponse::Unsupported,
                }
            }
            GdbCommand::Kill => {
                info!("GDB requested kill");
                GdbResponse::Ok
            }
            GdbCommand::Detach => {
                info!("GDB detached");
                self.state = StubState::Disconnected;
                GdbResponse::Ok
            }
            GdbCommand::Reset => {
                match self.target.reset() {
                    Ok(()) => GdbResponse::Ok,
                    Err(e) => {
                        error!("Reset failed: {}", e);
                        GdbResponse::Error(0x01)
                    }
                }
            }
            GdbCommand::ThreadInfo => {
                GdbResponse::Signal(self.target.get_last_signal())
            }
            GdbCommand::EnableExtendedMode => {
                GdbResponse::Ok
            }
            GdbCommand::Unknown(cmd) => {
                debug!("Unknown GDB command: {}", cmd);
                GdbResponse::Unsupported
            }
        }
    }

    /// Format a response for sending to GDB.
    fn format_response(&self, response: &GdbResponse) -> String {
        let content = response.to_string();
        let checksum = calculate_checksum(&content);
        format!("${}#{:02x}", content, checksum)
    }

    /// Check if the target has stopped (for polling).
    pub fn is_stopped(&self) -> bool {
        self.state != StubState::Running || !self.target.is_running()
    }

    /// Notify that the target has stopped.
    pub fn notify_stop(&mut self, signal: u8) -> String {
        self.state = StubState::Connected;
        let response = GdbResponse::Signal(signal);
        self.format_response(&response)
    }
}
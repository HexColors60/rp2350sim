//! GDB Remote Serial Protocol definitions.
//!
//! This module defines the GDB RSP commands, responses, and error types.

use std::fmt;

/// GDB RSP error type.
#[derive(Debug, Clone)]
pub enum GdbError {
    /// Invalid command format
    InvalidCommand(String),
    /// Checksum mismatch
    ChecksumMismatch,
    /// Unknown command
    UnknownCommand(String),
    /// Invalid register number
    InvalidRegister(u32),
    /// Invalid memory address
    InvalidAddress(u64),
    /// Target not running
    TargetNotRunning,
    /// IO error
    IoError(String),
}

impl std::fmt::Display for GdbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCommand(cmd) => write!(f, "Invalid command: {}", cmd),
            Self::ChecksumMismatch => write!(f, "Checksum mismatch"),
            Self::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            Self::InvalidRegister(reg) => write!(f, "Invalid register: {}", reg),
            Self::InvalidAddress(addr) => write!(f, "Invalid address: 0x{:08x}", addr),
            Self::TargetNotRunning => write!(f, "Target not running"),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for GdbError {}

/// GDB RSP command.
#[derive(Debug, Clone)]
pub enum GdbCommand {
    /// Continue execution
    Continue,
    /// Single step
    Step,
    /// Read registers (g)
    ReadRegisters,
    /// Write registers (G)
    WriteRegisters(Vec<u8>),
    /// Read memory (m addr,length)
    ReadMemory { addr: u64, length: u32 },
    /// Write memory (M addr,length:data)
    WriteMemory { addr: u64, data: Vec<u8> },
    /// Read single register (p n)
    ReadRegister(u32),
    /// Write single register (P n=r)
    WriteRegister { reg: u32, value: Vec<u8> },
    /// Set breakpoint (Z type,addr,kind)
    SetBreakpoint { type_: u32, addr: u64, kind: u32 },
    /// Remove breakpoint (z type,addr,kind)
    RemoveBreakpoint { type_: u32, addr: u64, kind: u32 },
    /// Query (q name)
    Query(String),
    /// Set (Q name:value)
    Set(String, String),
    /// Kill request
    Kill,
    /// Detach
    Detach,
    /// Reset
    Reset,
    /// Thread info
    ThreadInfo,
    /// Extended mode
    EnableExtendedMode,
    /// Unknown command
    Unknown(String),
}

/// GDB RSP response.
#[derive(Debug, Clone)]
pub enum GdbResponse {
    /// OK response
    Ok,
    /// Error response (E nn)
    Error(u8),
    /// Data response (hex string)
    Data(String),
    /// Signal response (S nn)
    Signal(u8),
    /// Empty response
    Empty,
    /// Unsupported
    Unsupported,
}

impl fmt::Display for GdbResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::Error(code) => write!(f, "E{:02x}", code),
            Self::Data(data) => write!(f, "{}", data),
            Self::Signal(sig) => write!(f, "S{:02x}", sig),
            Self::Empty => write!(f, ""),
            Self::Unsupported => write!(f, ""),
        }
    }
}

/// Parse a GDB command from a string.
pub fn parse_command(input: &str) -> Result<GdbCommand, GdbError> {
    if input.is_empty() {
        return Ok(GdbCommand::Unknown(String::new()));
    }

    let cmd = input.chars().next().unwrap();
    let args = &input[1..];

    match cmd {
        '?' => Ok(GdbCommand::ThreadInfo),
        'c' => Ok(GdbCommand::Continue),
        'C' => Ok(GdbCommand::Continue), // Continue with signal
        's' => Ok(GdbCommand::Step),
        'S' => Ok(GdbCommand::Step), // Step with signal
        'g' => Ok(GdbCommand::ReadRegisters),
        'G' => {
            let data = parse_hex_bytes(args)?;
            Ok(GdbCommand::WriteRegisters(data))
        }
        'm' => {
            let parts: Vec<&str> = args.split(',').collect();
            if parts.len() != 2 {
                return Err(GdbError::InvalidCommand(input.to_string()));
            }
            let addr = parse_hex_u64(parts[0])?;
            let length = parse_hex_u32(parts[1])?;
            Ok(GdbCommand::ReadMemory { addr, length })
        }
        'M' => {
            let parts: Vec<&str> = args.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(GdbError::InvalidCommand(input.to_string()));
            }
            let addr_len: Vec<&str> = parts[0].split(',').collect();
            if addr_len.len() != 2 {
                return Err(GdbError::InvalidCommand(input.to_string()));
            }
            let addr = parse_hex_u64(addr_len[0])?;
            let data = parse_hex_bytes(parts[1])?;
            Ok(GdbCommand::WriteMemory { addr, data })
        }
        'p' => {
            let reg = parse_hex_u32(args)?;
            Ok(GdbCommand::ReadRegister(reg))
        }
        'P' => {
            let parts: Vec<&str> = args.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(GdbError::InvalidCommand(input.to_string()));
            }
            let reg = parse_hex_u32(parts[0])?;
            let value = parse_hex_bytes(parts[1])?;
            Ok(GdbCommand::WriteRegister { reg, value })
        }
        'Z' => {
            let parts: Vec<&str> = args.split(',').collect();
            if parts.len() != 3 {
                return Err(GdbError::InvalidCommand(input.to_string()));
            }
            let type_ = parse_hex_u32(parts[0])?;
            let addr = parse_hex_u64(parts[1])?;
            let kind = parse_hex_u32(parts[2])?;
            Ok(GdbCommand::SetBreakpoint { type_, addr, kind })
        }
        'z' => {
            let parts: Vec<&str> = args.split(',').collect();
            if parts.len() != 3 {
                return Err(GdbError::InvalidCommand(input.to_string()));
            }
            let type_ = parse_hex_u32(parts[0])?;
            let addr = parse_hex_u64(parts[1])?;
            let kind = parse_hex_u32(parts[2])?;
            Ok(GdbCommand::RemoveBreakpoint { type_, addr, kind })
        }
        'q' => Ok(GdbCommand::Query(args.to_string())),
        'Q' => {
            let parts: Vec<&str> = args.splitn(2, ':').collect();
            if parts.len() == 2 {
                Ok(GdbCommand::Set(parts[0].to_string(), parts[1].to_string()))
            } else {
                Ok(GdbCommand::Set(parts[0].to_string(), String::new()))
            }
        }
        'k' => Ok(GdbCommand::Kill),
        'D' => Ok(GdbCommand::Detach),
        'R' => Ok(GdbCommand::Reset),
        '!' => Ok(GdbCommand::EnableExtendedMode),
        _ => Ok(GdbCommand::Unknown(input.to_string())),
    }
}

/// Parse hex bytes from a string.
fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, GdbError> {
    let mut bytes = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    for i in (0..chars.len()).step_by(2) {
        if i + 1 < chars.len() {
            let hex: String = chars[i..=i + 1].iter().collect();
            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                bytes.push(b);
            } else {
                return Err(GdbError::InvalidCommand(s.to_string()));
            }
        }
    }
    Ok(bytes)
}

/// Parse a hex u32 from a string.
fn parse_hex_u32(s: &str) -> Result<u32, GdbError> {
    u32::from_str_radix(s, 16).map_err(|_| GdbError::InvalidCommand(s.to_string()))
}

/// Parse a hex u64 from a string.
fn parse_hex_u64(s: &str) -> Result<u64, GdbError> {
    u64::from_str_radix(s, 16).map_err(|_| GdbError::InvalidCommand(s.to_string()))
}

/// Convert bytes to hex string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Convert u32 to hex bytes (little-endian).
pub fn u32_to_hex_bytes(value: u32) -> String {
    format!("{:02x}{:02x}{:02x}{:02x}",
        value & 0xFF,
        (value >> 8) & 0xFF,
        (value >> 16) & 0xFF,
        (value >> 24) & 0xFF)
}

/// Calculate checksum for a packet.
pub fn calculate_checksum(data: &str) -> u8 {
    data.bytes().fold(0u8, |acc, b| acc.wrapping_add(b))
}
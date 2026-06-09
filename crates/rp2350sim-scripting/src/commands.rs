//! Scripting commands.

/// Available scripting commands.
#[derive(Debug, Clone)]
pub enum Command {
    /// Reset the simulator.
    Reset,
    /// Step the simulator.
    Step,
    /// Run the simulator.
    Run,
    /// Stop the simulator.
    Stop,
    /// Load firmware.
    LoadFirmware(String),
    /// Set a breakpoint.
    SetBreakpoint(u32),
    /// Remove a breakpoint.
    RemoveBreakpoint(u32),
}

impl Command {
    /// Parse a command from a string.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "reset" => Some(Self::Reset),
            "step" => Some(Self::Step),
            "run" => Some(Self::Run),
            "stop" => Some(Self::Stop),
            "load" => parts.get(1).map(|p| Self::LoadFirmware(p.to_string())),
            "break" => parts.get(1).and_then(|p| p.parse().ok()).map(Self::SetBreakpoint),
            "unbreak" => parts.get(1).and_then(|p| p.parse().ok()).map(Self::RemoveBreakpoint),
            _ => None,
        }
    }
}
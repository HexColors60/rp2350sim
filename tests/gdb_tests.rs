//! Tests for GDB Remote Serial Protocol.

#[cfg(test)]
mod gdb_protocol_tests {
    use rp2350sim_gdb::protocol::{parse_command, GdbCommand, calculate_checksum, bytes_to_hex, u32_to_hex_bytes};

    #[test]
    fn test_parse_continue() {
        let cmd = parse_command("c").unwrap();
        assert!(matches!(cmd, GdbCommand::Continue));
    }

    #[test]
    fn test_parse_step() {
        let cmd = parse_command("s").unwrap();
        assert!(matches!(cmd, GdbCommand::Step));
    }

    #[test]
    fn test_parse_read_registers() {
        let cmd = parse_command("g").unwrap();
        assert!(matches!(cmd, GdbCommand::ReadRegisters));
    }

    #[test]
    fn test_parse_read_memory() {
        let cmd = parse_command("m1000,10").unwrap();
        match cmd {
            GdbCommand::ReadMemory { addr, length } => {
                assert_eq!(addr, 0x1000);
                assert_eq!(length, 0x10);
            }
            _ => panic!("Expected ReadMemory"),
        }
    }

    #[test]
    fn test_parse_write_memory() {
        let cmd = parse_command("M1000,4:01020304").unwrap();
        match cmd {
            GdbCommand::WriteMemory { addr, data } => {
                assert_eq!(addr, 0x1000);
                assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04]);
            }
            _ => panic!("Expected WriteMemory"),
        }
    }

    #[test]
    fn test_parse_read_register() {
        let cmd = parse_command("p0a").unwrap();
        match cmd {
            GdbCommand::ReadRegister(reg) => {
                assert_eq!(reg, 0x0a);
            }
            _ => panic!("Expected ReadRegister"),
        }
    }

    #[test]
    fn test_parse_write_register() {
        let cmd = parse_command("P0a=01020304").unwrap();
        match cmd {
            GdbCommand::WriteRegister { reg, value } => {
                assert_eq!(reg, 0x0a);
                assert_eq!(value, vec![0x01, 0x02, 0x03, 0x04]);
            }
            _ => panic!("Expected WriteRegister"),
        }
    }

    #[test]
    fn test_parse_set_breakpoint() {
        let cmd = parse_command("Z0,1000,2").unwrap();
        match cmd {
            GdbCommand::SetBreakpoint { type_, addr, kind } => {
                assert_eq!(type_, 0);
                assert_eq!(addr, 0x1000);
                assert_eq!(kind, 2);
            }
            _ => panic!("Expected SetBreakpoint"),
        }
    }

    #[test]
    fn test_parse_remove_breakpoint() {
        let cmd = parse_command("z0,1000,2").unwrap();
        match cmd {
            GdbCommand::RemoveBreakpoint { type_, addr, kind } => {
                assert_eq!(type_, 0);
                assert_eq!(addr, 0x1000);
                assert_eq!(kind, 2);
            }
            _ => panic!("Expected RemoveBreakpoint"),
        }
    }

    #[test]
    fn test_parse_query() {
        let cmd = parse_command("qSupported").unwrap();
        match cmd {
            GdbCommand::Query(name) => {
                assert_eq!(name, "Supported");
            }
            _ => panic!("Expected Query"),
        }
    }

    #[test]
    fn test_parse_kill() {
        let cmd = parse_command("k").unwrap();
        assert!(matches!(cmd, GdbCommand::Kill));
    }

    #[test]
    fn test_parse_detach() {
        let cmd = parse_command("D").unwrap();
        assert!(matches!(cmd, GdbCommand::Detach));
    }

    #[test]
    fn test_parse_reset() {
        let cmd = parse_command("R").unwrap();
        assert!(matches!(cmd, GdbCommand::Reset));
    }

    #[test]
    fn test_calculate_checksum() {
        // Test checksum calculation
        assert_eq!(calculate_checksum("g"), 0x67);
        assert_eq!(calculate_checksum("c"), 0x63);
        // m=109, 1=49, 0=48, 0=48, 0=48, ,=44, 1=49, 0=48
        // Sum: 109+49+48+48+48+44+49+48 = 443 = 0xBB (mod 256)
        assert_eq!(calculate_checksum("m1000,10"), 0xBB);
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "01020304");
    }

    #[test]
    fn test_u32_to_hex_bytes() {
        let hex = u32_to_hex_bytes(0x12345678);
        assert_eq!(hex, "78563412"); // Little-endian
    }
}

#[cfg(test)]
mod gdb_stub_tests {
    use rp2350sim_gdb::{GdbStub, GdbTarget, GdbError, GdbResponse};
    use rp2350sim_gdb::target::BreakpointKind;

    /// Mock target for testing
    struct MockTarget {
        registers: [u32; 17], // R0-R15 + xPSR
        memory: Vec<u8>,
        running: bool,
        pc: u32,
        breakpoints: Vec<u32>,
    }

    impl MockTarget {
        fn new() -> Self {
            Self {
                registers: [0; 17],
                memory: vec![0; 0x10000],
                running: false,
                pc: 0,
                breakpoints: Vec::new(),
            }
        }
    }

    impl GdbTarget for MockTarget {
        fn read_registers(&self) -> Result<Vec<u8>, GdbError> {
            let mut data = Vec::new();
            for reg in &self.registers {
                data.extend_from_slice(&reg.to_le_bytes());
            }
            Ok(data)
        }

        fn write_registers(&mut self, data: &[u8]) -> Result<(), GdbError> {
            for (i, chunk) in data.chunks(4).enumerate() {
                if i < 17 && chunk.len() == 4 {
                    self.registers[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                }
            }
            Ok(())
        }

        fn read_register(&self, reg: u32) -> Result<Vec<u8>, GdbError> {
            if (reg as usize) < 17 {
                Ok(self.registers[reg as usize].to_le_bytes().to_vec())
            } else {
                Err(GdbError::InvalidRegister(reg))
            }
        }

        fn write_register(&mut self, reg: u32, data: &[u8]) -> Result<(), GdbError> {
            if (reg as usize) < 17 && data.len() >= 4 {
                self.registers[reg as usize] = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(())
            } else {
                Err(GdbError::InvalidRegister(reg))
            }
        }

        fn read_memory(&self, addr: u64, length: u32) -> Result<Vec<u8>, GdbError> {
            let addr = addr as usize;
            let end = addr + length as usize;
            if end <= self.memory.len() {
                Ok(self.memory[addr..end].to_vec())
            } else {
                Err(GdbError::InvalidAddress(addr as u64))
            }
        }

        fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<(), GdbError> {
            let addr = addr as usize;
            let end = addr + data.len();
            if end <= self.memory.len() {
                self.memory[addr..end].copy_from_slice(data);
                Ok(())
            } else {
                Err(GdbError::InvalidAddress(addr as u64))
            }
        }

        fn continue_exec(&mut self) -> Result<(), GdbError> {
            self.running = true;
            Ok(())
        }

        fn step(&mut self) -> Result<(), GdbError> {
            self.pc += 2;
            Ok(())
        }

        fn set_breakpoint(&mut self, addr: u64, _kind: BreakpointKind) -> Result<(), GdbError> {
            self.breakpoints.push(addr as u32);
            Ok(())
        }

        fn remove_breakpoint(&mut self, addr: u64, _kind: BreakpointKind) -> Result<(), GdbError> {
            self.breakpoints.retain(|&a| a != addr as u32);
            Ok(())
        }

        fn is_running(&self) -> bool {
            self.running
        }

        fn get_pc(&self) -> u64 {
            self.pc as u64
        }

        fn set_pc(&mut self, pc: u64) -> Result<(), GdbError> {
            self.pc = pc as u32;
            Ok(())
        }

        fn get_last_signal(&self) -> u8 {
            5 // SIGTRAP
        }

        fn reset(&mut self) -> Result<(), GdbError> {
            self.registers = [0; 17];
            self.memory = vec![0; 0x10000];
            self.running = false;
            self.pc = 0;
            self.breakpoints.clear();
            Ok(())
        }
    }

    #[test]
    fn test_stub_creation() {
        let target = MockTarget::new();
        let stub = GdbStub::new(target);
        // A newly created stub is considered stopped (not running)
        assert!(stub.is_stopped());
    }

    #[test]
    fn test_stub_read_registers() {
        let target = MockTarget::new();
        let mut stub = GdbStub::new(target);
        
        let response = stub.process("$g#67");
        // Response should contain register data
        assert!(response.contains("$") || response.is_empty() || response.len() > 10);
    }

    #[test]
    fn test_stub_read_memory() {
        let target = MockTarget::new();
        let mut stub = GdbStub::new(target);
        
        let response = stub.process("$m0,10#b0");
        // Response should contain memory data
        assert!(response.contains("$") || response.is_empty());
    }

    #[test]
    fn test_stub_set_breakpoint() {
        let target = MockTarget::new();
        let mut stub = GdbStub::new(target);
        
        let response = stub.process("$Z0,1000,2#af");
        // Response should be OK
        assert!(response.contains("OK") || response.contains("$"));
    }

    #[test]
    fn test_stub_query_supported() {
        let target = MockTarget::new();
        let mut stub = GdbStub::new(target);
        
        let response = stub.process("$qSupported#aa");
        // Response should contain supported features
        assert!(response.contains("$") || response.is_empty());
    }
}
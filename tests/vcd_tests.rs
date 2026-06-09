//! Tests for VCD waveform export.

#[cfg(test)]
mod vcd_export_tests {
    use rp2350sim_trace::export::{VcdExporter, VcdVariable, VcdVarType, CpuVcdExporter, GpioVcdExporter, MemoryVcdExporter};
    use std::io::Read;

    #[test]
    fn test_vcd_exporter_creation() {
        let exporter = VcdExporter::new();
        assert!(exporter.content().is_empty());
    }

    #[test]
    fn test_vcd_header() {
        let mut exporter = VcdExporter::new();
        exporter.write_header("1ns", Some("Test VCD"));
        
        let content = exporter.content();
        assert!(content.contains("$timescale 1ns $end"));
        assert!(content.contains("$comment Test VCD $end"));
        assert!(content.contains("$scope module TOP $end"));
    }

    #[test]
    fn test_vcd_add_variable() {
        let mut exporter = VcdExporter::new();
        exporter.write_header("1ns", None);
        exporter.push_scope("CPU");
        exporter.add_variable(VcdVariable {
            id: 'A',
            name: "PC".to_string(),
            width: 32,
            var_type: VcdVarType::Wire,
        });
        exporter.pop_scope();
        
        let content = exporter.content();
        assert!(content.contains("$scope module CPU $end"));
        assert!(content.contains("$var wire 32 A PC $end"));
        assert!(content.contains("$upscope $end"));
    }

    #[test]
    fn test_vcd_time_change() {
        let mut exporter = VcdExporter::new();
        exporter.write_header("1ns", None);
        exporter.add_variable(VcdVariable {
            id: 'A',
            name: "value".to_string(),
            width: 8,
            var_type: VcdVarType::Wire,
        });
        exporter.end_header();
        
        // Note: set_time(0) won't write #0 since initial time is 0
        exporter.write_binary('A', 0x42);
        exporter.set_time(100);
        exporter.write_binary('A', 0x43);
        
        let content = exporter.content();
        assert!(content.contains("#100"));
        // Note: VCD format is "b<binary><id>" without space
        assert!(content.contains("b01000010A"));
        assert!(content.contains("b01000011A"));
    }

    #[test]
    fn test_vcd_bit_change() {
        let mut exporter = VcdExporter::new();
        exporter.write_header("1ns", None);
        exporter.add_variable(VcdVariable {
            id: 'B',
            name: "flag".to_string(),
            width: 1,
            var_type: VcdVarType::Wire,
        });
        exporter.end_header();
        
        exporter.set_time(0);
        exporter.write_bit('B', false);
        exporter.set_time(50);
        exporter.write_bit('B', true);
        
        let content = exporter.content();
        assert!(content.contains("0B"));
        assert!(content.contains("1B"));
    }

    #[test]
    fn test_cpu_vcd_exporter() {
        let mut exporter = CpuVcdExporter::new();
        
        exporter.record(0, 0x1000, 0x20000000, 0x08000000, 0, false);
        exporter.record(100, 0x1002, 0x20000000, 0x08000000, 1, false);
        exporter.record(200, 0x1004, 0x20000000, 0x08000000, 2, true);
        
        let content = exporter.exporter.content();
        assert!(content.contains("CPU"));
        assert!(content.contains("PC"));
        assert!(content.contains("SP"));
        assert!(content.contains("LR"));
        assert!(content.contains("Cycles"));
        assert!(content.contains("IRQ"));
    }

    #[test]
    fn test_gpio_vcd_exporter() {
        let mut exporter = GpioVcdExporter::new(8);
        
        exporter.record(0, &[false, true, false, true, false, true, false, true]);
        exporter.record(100, &[true, false, true, false, true, false, true, false]);
        
        let content = exporter.exporter.content();
        assert!(content.contains("GPIO"));
        assert!(content.contains("pin_0"));
        assert!(content.contains("pin_7"));
    }

    #[test]
    fn test_memory_vcd_exporter() {
        let mut exporter = MemoryVcdExporter::new();
        
        exporter.record(0, 0x1000, 0xDEADBEEF, true, 4);
        exporter.record(100, 0x2000, 0x12345678, false, 4);
        
        let content = exporter.exporter.content();
        assert!(content.contains("Memory"));
        assert!(content.contains("Address"));
        assert!(content.contains("Data"));
        assert!(content.contains("Write"));
        assert!(content.contains("Width"));
    }

    #[test]
    fn test_vcd_export_to_file() {
        let mut exporter = VcdExporter::new();
        exporter.write_header("1ns", Some("Test Export"));
        exporter.add_variable(VcdVariable {
            id: 'A',
            name: "test".to_string(),
            width: 8,
            var_type: VcdVarType::Wire,
        });
        exporter.end_header();
        exporter.write_binary('A', 0x42);
        
        // Export to temp file
        let temp_path = std::env::temp_dir().join("test_vcd_export.vcd");
        let result = exporter.export(&temp_path);
        assert!(result.is_ok());
        
        // Verify file exists and has content
        assert!(temp_path.exists());
        
        // Read and verify content
        let mut file = std::fs::File::open(&temp_path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert!(content.contains("$timescale"));
        // Note: VCD format is "b<binary><id>" without space
        assert!(content.contains("b01000010A"));
        
        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }
}
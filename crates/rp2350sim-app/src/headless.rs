#![allow(dead_code)]

//! Headless mode CLI for RP2350 Simulator.

use std::path::PathBuf;
use std::time::Instant;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::net::TcpListener;
use std::io::{Read, Write};

use clap::Parser;
use rp2350sim_core::{CpuArch, Result};
use rp2350sim_soc::Soc;
use rp2350sim_soc::gdb::SocGdbTarget;
use rp2350sim_debug::DebugController;
use rp2350sim_trace::Trace;
use rp2350sim_save;
use rp2350sim_gdb::GdbStub;

/// Headless simulator CLI.
#[derive(Parser, Debug)]
#[command(name = "rp2350sim-headless")]
#[command(about = "RP2350 Simulator - Headless Mode")]
pub struct HeadlessArgs {
    /// Firmware file to load (ELF, BIN, UF2, HEX)
    #[arg(short, long)]
    pub firmware: Option<PathBuf>,

    /// CPU architecture (arm or hazard3)
    #[arg(short, long, default_value = "arm")]
    pub cpu: String,

    /// Number of instructions to run (0 = unlimited)
    #[arg(short = 'n', long, default_value = "0")]
    pub count: u64,

    /// Enable tracing
    #[arg(short, long)]
    pub trace: bool,

    /// Trace output file
    #[arg(long)]
    pub trace_file: Option<PathBuf>,

    /// Enable MMIO tracing
    #[arg(long)]
    pub trace_mmio: bool,

    /// Enable GPIO tracing
    #[arg(long)]
    pub trace_gpio: bool,

    /// Breakpoint address (hex)
    #[arg(short = 'b', long)]
    pub breakpoint: Option<String>,

    /// Save state file on exit
    #[arg(short, long)]
    pub save: Option<PathBuf>,

    /// Load state file
    #[arg(short, long)]
    pub load: Option<PathBuf>,

    /// UART output file
    #[arg(long)]
    pub uart_output: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(short, long)]
    pub quiet: bool,

    /// Max cycles before timeout (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub max_cycles: u64,

    /// Export VCD waveform
    #[arg(long)]
    pub vcd: Option<PathBuf>,

    /// Enable GDB server for remote debugging
    #[arg(long)]
    pub gdb: bool,

    /// GDB server port (default: 3333)
    #[arg(long, default_value = "3333")]
    pub gdb_port: u16,

    /// GDB server host (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    pub gdb_host: String,
}

/// Headless simulator runner.
pub struct HeadlessRunner {
    args: HeadlessArgs,
    soc: Option<Soc>,
    debugger: DebugController,
    tracer: Trace,
    start_time: Instant,
    instructions_executed: u64,
    verbose: bool,
    gdb_stub: Option<GdbStub<SocGdbTarget>>,
    running: Arc<AtomicBool>,
}

impl HeadlessRunner {
    /// Create a new headless runner.
    pub fn new(args: HeadlessArgs) -> Self {
        let verbose = args.verbose;
        Self {
            args,
            soc: None,
            debugger: DebugController::new(),
            tracer: Trace::new(),
            start_time: Instant::now(),
            instructions_executed: 0,
            verbose,
            gdb_stub: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Run the simulation.
    pub fn run(&mut self) -> Result<()> {
        if !self.args.quiet {
            println!("RP2350 Simulator - Headless Mode");
            println!("=================================");
        }

        // Parse CPU architecture
        let cpu_arch = match self.args.cpu.to_lowercase().as_str() {
            "arm" | "cortex-m33" => CpuArch::Arm,
            "hazard3" | "risc-v" | "riscv" => CpuArch::Hazard3,
            _ => {
                eprintln!("Unknown CPU architecture: {}", self.args.cpu);
                eprintln!("Valid options: arm, hazard3");
                std::process::exit(1);
            }
        };

        if self.verbose {
            println!("CPU Architecture: {:?}", cpu_arch);
        }

        // Initialize SoC
        self.soc = Some(Soc::new(cpu_arch));

        // Load state if specified
        if let Some(ref path) = self.args.load {
            let path = path.clone();
            self.load_state(&path)?;
        }

        // Load firmware if specified
        if let Some(ref path) = self.args.firmware {
            let path = path.clone();
            self.load_firmware(&path)?;
        }

        // Parse breakpoints
        if let Some(ref bp_str) = self.args.breakpoint {
            if let Ok(addr) = u32::from_str_radix(bp_str.trim_start_matches("0x"), 16) {
                self.debugger.add_breakpoint(addr);
                if self.verbose {
                    println!("Breakpoint set at 0x{:08X}", addr);
                }
            }
        }

        // Start GDB server if requested
        if self.args.gdb {
            self.start_gdb_server()?;
        }

        // Run simulation
        self.start_time = Instant::now();
        self.run_simulation()?;

        // Save state if requested
        if let Some(ref path) = self.args.save {
            self.save_state(path)?;
        }

        // Export VCD if requested
        if let Some(ref path) = self.args.vcd {
            self.export_vcd(path)?;
        }

        // Print statistics
        if !self.args.quiet {
            self.print_stats();
        }

        Ok(())
    }

    /// Load firmware from file.
    fn load_firmware(&mut self, path: &PathBuf) -> Result<()> {
        if self.verbose {
            println!("Loading firmware: {}", path.display());
        }

        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "elf" => {
                if self.verbose {
                    println!("Format: ELF");
                }
                // ELF loading would be done through boot module
            }
            "bin" => {
                if self.verbose {
                    println!("Format: Binary");
                }
            }
            "uf2" => {
                if self.verbose {
                    println!("Format: UF2");
                }
            }
            "hex" => {
                if self.verbose {
                    println!("Format: Intel HEX");
                }
            }
            _ => {
                eprintln!("Unknown firmware format: {}", extension);
            }
        }

        Ok(())
    }

    /// Load state from file.
    fn load_state(&mut self, path: &PathBuf) -> Result<()> {
        if self.verbose {
            println!("Loading state: {}", path.display());
        }

        let data = std::fs::read(path)
            .map_err(|e| rp2350sim_core::Error::Io(e))?;

        if let Some(ref mut soc) = self.soc {
            rp2350sim_save::load_state(soc, &data)?;
        }

        Ok(())
    }

    /// Save state to file.
    fn save_state(&self, path: &PathBuf) -> Result<()> {
        if self.verbose {
            println!("Saving state: {}", path.display());
        }

        if let Some(ref soc) = self.soc {
            let data = rp2350sim_save::save_state(soc)?;
            std::fs::write(path, data)
                .map_err(|e| rp2350sim_core::Error::Io(e))?;
        }

        Ok(())
    }

    /// Export VCD waveform.
    fn export_vcd(&self, path: &PathBuf) -> Result<()> {
        if self.verbose {
            println!("Exporting VCD: {}", path.display());
        }

        use rp2350sim_trace::{CpuVcdExporter, GpioVcdExporter};

        // Create a combined VCD exporter
        let mut cpu_exporter = CpuVcdExporter::new();
        let mut gpio_exporter = GpioVcdExporter::new(48);  // RP2350 has 48 GPIO pins

        // Export CPU trace
        if let Some(ref soc) = self.soc {
            // Record final CPU state
            cpu_exporter.record(
                soc.cycles(),
                soc.pc(),
                soc.sp(),
                soc.lr(),
                soc.cycles(),
                false,  // IRQ state
            );

            // Record GPIO state
            let mut gpio_states = [false; 48];
            for pin in 0..48 {
                gpio_states[pin] = soc.gpio_value(pin);
            }
            gpio_exporter.record(soc.cycles(), &gpio_states);
        }

        // Combine exports into a single file
        let cpu_content = cpu_exporter.exporter.content();
        let gpio_content = gpio_exporter.exporter.content();

        // Write combined VCD
        let mut content = String::new();
        content.push_str("$timescale 1ns $end\n");
        content.push_str("$scope module TOP $end\n");
        content.push_str(cpu_content);
        content.push_str(gpio_content);
        content.push_str("$enddefinitions $end\n");

        std::fs::write(path, content)
            .map_err(|e| rp2350sim_core::Error::Io(e))?;

        if !self.args.quiet {
            println!("VCD exported to: {}", path.display());
        }

        Ok(())
    }

    /// Start the GDB server.
    fn start_gdb_server(&mut self) -> Result<()> {
        if self.soc.is_none() {
            eprintln!("Cannot start GDB server without SoC");
            return Ok(());
        }

        let port = self.args.gdb_port;
        let host = self.args.gdb_host.clone();
        let addr = format!("{}:{}", host, port);

        if !self.args.quiet {
            println!("Starting GDB server on {}", addr);
            println!("Connect with: arm-none-eabi-gdb -ex 'target remote {}'", addr);
        }

        // Bind to the address
        let listener = TcpListener::bind(&addr)
            .map_err(|e| rp2350sim_core::Error::Io(e))?;

        if !self.args.quiet {
            println!("Waiting for GDB connection...");
        }

        // Accept one connection (blocking)
        let (mut stream, client_addr) = listener.accept()
            .map_err(|e| rp2350sim_core::Error::Io(e))?;

        if !self.args.quiet {
            println!("GDB connected from {}", client_addr);
        }

        // Take ownership of SoC and wrap it in GDB target
        let soc = self.soc.take().unwrap();
        let gdb_target = SocGdbTarget::new(soc);

        // Create GDB stub
        let mut stub = GdbStub::new(gdb_target);

        // Mark as running
        self.running.store(true, Ordering::SeqCst);

        // GDB session loop
        let mut buffer = [0u8; 4096];
        let mut input_buffer = Vec::new();

        while self.running.load(Ordering::SeqCst) {
            // Read data from GDB
            match stream.read(&mut buffer) {
                Ok(0) => {
                    if !self.args.quiet {
                        println!("GDB disconnected");
                    }
                    break;
                }
                Ok(n) => {
                    input_buffer.extend_from_slice(&buffer[..n]);

                    // Process complete packets
                    while let Some(packet_end) = find_packet_end(&input_buffer) {
                        let packet_data: Vec<u8> = input_buffer.drain(..=packet_end).collect();
                        let packet_str = String::from_utf8_lossy(&packet_data);

                        // Process the packet
                        let response = stub.process(&packet_str);

                        // Send response
                        if !response.is_empty() {
                            if let Err(e) = stream.write_all(response.as_bytes()) {
                                eprintln!("GDB write error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("GDB read error: {}", e);
                    break;
                }
            }
        }

        // Restore SoC from stub
        // Note: The stub owns the SoC now, so we can't restore it here
        // In a full implementation, we'd use Arc<Mutex<Soc>> for sharing

        Ok(())
    }

    /// Run the main simulation loop.
    fn run_simulation(&mut self) -> Result<()> {
        let max_instructions = self.args.count;
        let max_cycles = self.args.max_cycles;

        if self.verbose {
            if max_instructions > 0 {
                println!("Running {} instructions...", max_instructions);
            } else {
                println!("Running simulation (press Ctrl+C to stop)...");
            }
        }

        loop {
            // Check instruction limit
            if max_instructions > 0 && self.instructions_executed >= max_instructions {
                if self.verbose {
                    println!("Instruction limit reached: {}", max_instructions);
                }
                break;
            }

            // Check cycle limit
            if max_cycles > 0 {
                if let Some(ref soc) = self.soc {
                    if soc.cycles() >= max_cycles {
                        if self.verbose {
                            println!("Cycle limit reached: {}", max_cycles);
                        }
                        break;
                    }
                }
            }

            // Step simulation
            if let Some(ref mut soc) = self.soc {
                // Check breakpoints
                if self.debugger.has_breakpoint(soc.pc()) {
                    if !self.args.quiet {
                        println!("\nBreakpoint hit at 0x{:08X}", soc.pc());
                    }
                    break;
                }

                // Execute one instruction
                soc.step()?;
                self.instructions_executed += 1;

                // Trace if enabled
                if self.args.trace {
                    self.trace_instruction();
                }
            }

            // Progress indicator
            if self.verbose && self.instructions_executed % 100000 == 0 {
                print!(".");
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }

        if self.verbose && self.instructions_executed >= 100000 {
            println!();
        }

        Ok(())
    }

    /// Trace current instruction.
    fn trace_instruction(&mut self) {
        if let Some(ref soc) = self.soc {
            let pc = soc.pc();
            let sp = soc.sp();
            let lr = soc.lr();

            // Log to trace file if specified
            if let Some(ref path) = self.args.trace_file {
                let entry = format!("{:016}: PC=0x{:08X} SP=0x{:08X} LR=0x{:08X}\n",
                    self.instructions_executed, pc, sp, lr);
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .and_then(|mut f| std::io::Write::write_all(&mut f, entry.as_bytes()));
            }
        }
    }

    /// Print simulation statistics.
    fn print_stats(&self) {
        let elapsed = self.start_time.elapsed();
        let ips = if elapsed.as_secs_f64() > 0.0 {
            self.instructions_executed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        println!("\nSimulation Statistics");
        println!("---------------------");
        println!("Instructions executed: {}", self.instructions_executed);
        println!("Time elapsed: {:.2?}", elapsed);
        println!("Instructions/second: {:.0}", ips);

        if let Some(ref soc) = self.soc {
            println!("Cycles: {}", soc.cycles());
            println!("Final PC: 0x{:08X}", soc.pc());
            println!("Final SP: 0x{:08X}", soc.sp());
        }
    }
}

/// Run headless mode with arguments.
pub fn run_headless(args: HeadlessArgs) -> Result<()> {
    let mut runner = HeadlessRunner::new(args);
    runner.run()
}

/// Find the end of a GDB packet (after checksum).
fn find_packet_end(data: &[u8]) -> Option<usize> {
    // GDB packet format: $data#checksum
    for i in 0..data.len() {
        if data[i] == b'$' {
            for j in (i + 1)..data.len() {
                if data[j] == b'#' {
                    if j + 2 < data.len() {
                        return Some(j + 2);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_args_parsing() {
        let args = HeadlessArgs::parse_from(["test", "--cpu", "arm", "--count", "100"]);
        assert_eq!(args.cpu, "arm");
        assert_eq!(args.count, 100);
    }

    #[test]
    fn test_headless_runner_creation() {
        let args = HeadlessArgs::parse_from(["test"]);
        let runner = HeadlessRunner::new(args);
        assert!(!runner.verbose);
    }
}
//! Main entry point for RP2350 Simulator.
//!
//! This application supports multiple GUI backends selected at compile time:
//! - `gui-macroquad`: Default macroquad + egui backend
//! - `gui-bevy`: Bevy game engine with egui integration
//! - `gui-winapi`: Native Windows API with wgpu/egui
//! - `headless`: No GUI, command-line only

use clap::Parser;
use rp2350sim_app::{App, Config};
use crate::args::Commands;

mod args;

/// Main entry point.
fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse arguments
    let args = args::Args::parse();

    // Create configuration
    let mut config = Config {
        headless: args.headless,
        ..Default::default()
    };

    // Print backend info
    #[cfg(feature = "gui-macroquad")]
    tracing::info!("GUI Backend: macroquad + egui");

    #[cfg(all(feature = "gui-bevy", not(feature = "gui-macroquad")))]
    tracing::info!("GUI Backend: Bevy + egui");

    #[cfg(all(feature = "gui-winapi", not(any(feature = "gui-macroquad", feature = "gui-bevy"))))]
    tracing::info!("GUI Backend: WinAPI + wgpu + egui");

    #[cfg(all(feature = "headless", not(any(feature = "gui-macroquad", feature = "gui-bevy", feature = "gui-winapi"))))]
    tracing::info!("Running in headless mode");

    // Handle subcommands
    match args.command {
        Some(Commands::Gui { project }) => {
            if let Some(proj) = project {
                config.project = Some(proj);
            }
            run_gui(config)?;
        }
        Some(Commands::Run { firmware, cycles, verbose, trace }) => {
            run_headless(config, &firmware, cycles, verbose, trace)?;
        }
        Some(Commands::Disasm { firmware, format }) => {
            disasm_firmware(&firmware, &format)?;
        }
        Some(Commands::Info { firmware }) => {
            show_firmware_info(&firmware)?;
        }
        Some(Commands::Test { test }) => {
            run_tests(test.as_deref())?;
        }
        None => {
            // Default: launch GUI
            run_gui(config)?;
        }
    }

    Ok(())
}

/// Run with GUI (feature-gated).
#[cfg(any(feature = "gui-macroquad", feature = "gui-bevy", feature = "gui-winapi"))]
fn run_gui(config: Config) -> anyhow::Result<()> {
    use rp2350sim_app::gui::{create_backend, current_backend_name, GuiBackend};

    tracing::info!("Starting GUI with backend: {}", current_backend_name());

    // Create application
    let mut app = App::new(config.clone());

    // Create and run GUI backend
    let mut backend = create_backend(&config)?;
    backend.run(&mut app)?;

    Ok(())
}

/// Run with GUI (headless - no GUI features enabled).
#[cfg(not(any(feature = "gui-macroquad", feature = "gui-bevy", feature = "gui-winapi")))]
fn run_gui(config: Config) -> anyhow::Result<()> {
    tracing::warn!("No GUI backend enabled. Running in headless mode.");
    let mut app = App::new(config);
    app.run()
}

/// Run firmware in headless mode.
fn run_headless(config: Config, firmware: &str, cycles: Option<u64>, verbose: bool, trace: bool) -> anyhow::Result<()> {
    tracing::info!("Running firmware in headless mode: {}", firmware);
    
    let mut app = App::new(Config {
        headless: true,
        ..config
    });
    
    // Load firmware
    app.load_firmware(firmware)?;
    
    // Show initial state if verbose
    if verbose {
        println!("\n=== Initial State ===");
        app.dump_registers();
    }
    
    // Run for specified cycles (default: run until exit)
    let max_cycles = cycles.unwrap_or(10000);
    
    if trace {
        app.run_headless_with_trace(max_cycles)?;
    } else {
        app.run_headless(max_cycles)?;
    }
    
    // Show final state if verbose
    if verbose {
        println!("\n=== Final State ===");
        app.dump_registers();
        app.dump_stats();
    }
    
    Ok(())
}

/// Disassemble firmware file.
fn disasm_firmware(firmware: &str, format: &str) -> anyhow::Result<()> {
    use rp2350sim_debug::disasm::{disasm_arm_thumb, disasm_rv32};
    
    tracing::info!("Disassembling firmware: {}", firmware);
    
    // Load the firmware
    let path = std::path::Path::new(firmware);
    let data = std::fs::read(path)?;
    
    // Determine architecture from the file or default to ARM
    let is_arm = !firmware.contains("riscv") && !firmware.contains("rv32");
    
    // Disassemble
    println!("Disassembly of {}:", firmware);
    println!("================{}", "=".repeat(firmware.len()));
    
    if is_arm {
        // ARM Thumb: 16-bit instructions
        for (i, chunk) in data.chunks(2).enumerate() {
            if chunk.len() == 2 {
                let opcode = u16::from_le_bytes([chunk[0], chunk[1]]);
                let addr = 0x10000000 + (i * 2) as u32;
                let inst = disasm_arm_thumb(opcode, addr);
                match format {
                    "json" => println!("{{\"addr\": \"0x{:08X}\", \"inst\": \"{}\"}}", addr, inst),
                    _ => println!("0x{:08X}: {}", addr, inst),
                }
            }
        }
    } else {
        // RISC-V: 32-bit instructions
        for (i, chunk) in data.chunks(4).enumerate() {
            if chunk.len() == 4 {
                let opcode = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let addr = 0x10000000 + (i * 4) as u32;
                let inst = disasm_rv32(opcode, addr);
                match format {
                    "json" => println!("{{\"addr\": \"0x{:08X}\", \"inst\": \"{}\"}}", addr, inst),
                    _ => println!("0x{:08X}: {}", addr, inst),
                }
            }
        }
    }
    
    Ok(())
}

/// Show firmware info.
fn show_firmware_info(firmware: &str) -> anyhow::Result<()> {
    use rp2350sim_mem::loader::ElfLoader;
    
    tracing::info!("Showing firmware info: {}", firmware);
    
    let path = std::path::Path::new(firmware);
    let metadata = std::fs::metadata(path)?;
    
    println!("Firmware: {}", firmware);
    println!("=========={}", "=".repeat(firmware.len()));
    println!("File Size:    {} bytes", metadata.len());
    
    // Check extension for more info
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        "elf" => {
            println!("Format:       ELF executable");
            
            // Parse ELF and show details
            match ElfLoader::load_from_path(path) {
                Ok(info) => {
                    println!("\n=== ELF Information ===");
                    println!("Architecture: {}", info.architecture);
                    println!("Endianness:   {}", if info.is_little_endian { "Little" } else { "Big" });
                    println!("Entry Point:  0x{:08X}", info.entry_point);
                    
                    if !info.sections.is_empty() {
                        println!("\n=== Sections ({}) ===", info.sections.len());
                        println!("{:<20} {:<10} {:<12} {:<10}", "Name", "Type", "Address", "Size");
                        println!("{}", "-".repeat(52));
                        for section in &info.sections {
                            println!("{:<20} {:<10} 0x{:08X}   {:>8} B", 
                                section.name, 
                                section.section_type,
                                section.address,
                                section.size
                            );
                        }
                    }
                    
                    // Count symbols by type
                    let funcs = info.symbols.iter().filter(|s| s.kind == rp2350sim_mem::loader::SymbolKind::Function).count();
                    let vars = info.symbols.iter().filter(|s| s.kind == rp2350sim_mem::loader::SymbolKind::Variable).count();
                    let others = info.symbols.len() - funcs - vars;
                    
                    if !info.symbols.is_empty() {
                        println!("\n=== Symbols ({} total) ===", info.symbols.len());
                        println!("Functions:    {}", funcs);
                        println!("Variables:    {}", vars);
                        println!("Other:        {}", others);
                        
                        // Show first 20 symbols
                        println!("\n{:<10} {:<8} {:<8} {}", "Address", "Type", "Size", "Name");
                        println!("{}", "-".repeat(60));
                        for sym in info.symbols.iter().take(20) {
                            println!("0x{:08X} {:<8} {:>6} B   {}", 
                                sym.address, 
                                sym.kind,
                                sym.size,
                                sym.name
                            );
                        }
                        if info.symbols.len() > 20 {
                            println!("... and {} more symbols", info.symbols.len() - 20);
                        }
                    }
                }
                Err(e) => {
                    println!("\nWarning: Could not parse ELF: {}", e);
                }
            }
        }
        "bin" => {
            println!("Format:       Raw binary");
            println!("Load Address: 0x10000000 (XIP Flash)");
        }
        "hex" => println!("Format:       Intel HEX"),
        "uf2" => println!("Format:       UF2 (USB Flashing Format)"),
        _ => println!("Format:       Unknown ({})", extension),
    }
    
    Ok(())
}

/// Run tests.
fn run_tests(test: Option<&str>) -> anyhow::Result<()> {
    tracing::info!("Running tests: {:?}", test);
    
    // For now, just run cargo test
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("test");
    
    if let Some(test_name) = test {
        cmd.arg(test_name);
    }
    
    let status = cmd.status()?;
    if status.success() {
        println!("Tests passed!");
    } else {
        println!("Some tests failed.");
    }
    
    Ok(())
}
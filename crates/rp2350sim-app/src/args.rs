#![allow(dead_code)]

//! Command-line arguments.

use clap::{Parser, Subcommand};
use rp2350sim_core::CpuArch;

#[derive(Parser)]
#[command(name = "rp2350sim")]
#[command(about = "RP2350 Pico 2 W Full-System Simulator")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// CPU architecture (arm or hazard3)
    #[arg(long, default_value = "arm")]
    pub cpu: String,

    /// Run in headless mode (no GUI)
    #[arg(long)]
    pub headless: bool,

    /// Enable instruction tracing
    #[arg(long)]
    pub trace: bool,

    /// Firmware file to load
    #[arg(short, long)]
    pub firmware: Option<String>,

    /// Project directory
    #[arg(short, long)]
    pub project: Option<String>,

    /// Configuration file
    #[arg(short, long)]
    pub config: Option<String>,
}

impl Args {
    pub fn cpu_arch(&self) -> CpuArch {
        match self.cpu.to_lowercase().as_str() {
            "arm" => CpuArch::Arm,
            "hazard3" | "riscv" => CpuArch::Hazard3,
            _ => CpuArch::Arm,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the GUI simulator
    Gui {
        /// Project directory
        project: Option<String>,
    },

    /// Run firmware in headless mode
    Run {
        /// Firmware file
        firmware: String,

        /// Number of cycles to run
        #[arg(short, long)]
        cycles: Option<u64>,

        /// Show verbose output (registers, flags)
        #[arg(short, long)]
        verbose: bool,

        /// Show instruction trace
        #[arg(short, long)]
        trace: bool,
    },

    /// Run tests
    Test {
        /// Test file or directory
        test: Option<String>,
    },

    /// Disassemble a firmware file
    Disasm {
        /// Firmware file
        firmware: String,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show firmware info
    Info {
        /// Firmware file
        firmware: String,
    },
}
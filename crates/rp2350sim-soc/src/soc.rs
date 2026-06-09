//! SoC implementation.

use rp2350sim_bus::Bus;
use rp2350sim_clocks::Clocks;
use rp2350sim_core::{CpuArch, Device, Result, SimulatorConfig};
use rp2350sim_core::{
    IRQ_TIMER_0, IRQ_PWM_WRAP, IRQ_SPI0, IRQ_SPI1, IRQ_I2C0, IRQ_I2C1,
    IRQ_UART0, IRQ_UART1, IRQ_ADC_FIFO, IRQ_IO_BANK0,
};
use rp2350sim_cpu_arm::thumb::execute::MemoryAccess as ArmMemoryAccess;
use rp2350sim_cpu_arm::ArmBackend;
use rp2350sim_cpu_common::CpuBackend;
use rp2350sim_cpu_hazard3::rv32::execute::MemoryAccess as Rv32MemoryAccess;
use rp2350sim_cpu_hazard3::Hazard3Backend;
use rp2350sim_devices::{
    adc::Adc, gpio::Gpio, i2c::I2c, pio::Pio, pwm::Pwm, spi::Spi, timer::Timer, uart::Uart,
    watchdog::Watchdog,
    dma::Dma, xip::Xip, i2s::I2s, rtc::Rtc, pll::{Pll, PllType}, sha256::Sha256, trng::Trng, usb::Usb,
    nvic::Nvic, systick::Systick, plic::Plic, powman::PowerManager, sysinfo::Sysinfo, otp::Otp,
    mpu::Mpu, coresight::Coresight, hstx::Hstx, interp::Interp, bootram::Bootram, busctrl::BusControl,
    reset::Reset,
};
use rp2350sim_irq::IrqLines;
use rp2350sim_mem::{bootrom::BootRomImage, flash::FlashImage, sram::SramState};
use rp2350sim_debug::symbols::{DwarfDebugInfo, SymbolTable};

/// SoC state.
pub struct Soc {
    pub config: SimulatorConfig,
    pub bus: Bus,
    pub clocks: Clocks,
    pub irq_lines: IrqLines,
    pub sram: SramState,
    pub flash: FlashImage,
    pub bootrom: BootRomImage,
    pub gpio: Gpio,
    pub uart0: Uart,
    pub uart1: Uart,
    pub spi0: Spi,
    pub spi1: Spi,
    pub i2c0: I2c,
    pub i2c1: I2c,
    pub pwm: Pwm,
    pub adc: Adc,
    pub pio0: Pio,
    pub pio1: Pio,
    pub timer: Timer,
    pub watchdog: Watchdog,
    // Additional peripherals
    pub dma: Dma,
    pub xip: Xip,
    pub i2s0: I2s,
    pub i2s1: I2s,
    pub rtc: Rtc,
    pub pll_sys: Pll,
    pub pll_usb: Pll,
    pub sha256: Sha256,
    pub trng: Trng,
    pub usb: Usb,
    // System peripherals
    pub nvic: Nvic,
    pub systick: Systick,
    pub plic: Plic,
    pub powman: PowerManager,
    pub sysinfo: Sysinfo,
    pub otp: Otp,
    // Core peripherals
    pub mpu: Mpu,
    pub coresight: Coresight,
    pub hstx: Hstx,
    pub interp: Interp,
    pub bootram: Bootram,
    pub busctrl: BusControl,
    pub reset: Reset,
    /// Symbol table for debugging.
    pub symbols: SymbolTable,
    /// DWARF debug info for source-level debugging.
    pub dwarf: DwarfDebugInfo,
    // CPU backend (ARM or Hazard3)
    pub cpu_arm: Option<ArmBackend>,
    pub cpu_hazard3: Option<Hazard3Backend>,
    // Simulation state
    cycles: u64,
    instructions: u64,
    running: bool,
}

impl Soc {
    /// Create a new SoC.
    pub fn new(cpu_arch: CpuArch) -> Self {
        Self {
            cpu_arm: if cpu_arch == CpuArch::Arm {
                Some(ArmBackend::new())
            } else {
                None
            },
            cpu_hazard3: if cpu_arch == CpuArch::Hazard3 {
                Some(Hazard3Backend::new())
            } else {
                None
            },
            config: SimulatorConfig::default(),
            bus: Bus::new(),
            clocks: Clocks::new(),
            irq_lines: IrqLines::new(),
            sram: SramState::new(),
            flash: FlashImage::new(16 * 1024 * 1024),
            bootrom: BootRomImage::new(),
            gpio: Gpio::new(),
            uart0: Uart::uart0(),
            uart1: Uart::uart1(),
            spi0: Spi::spi0(),
            spi1: Spi::spi1(),
            i2c0: I2c::i2c0(),
            i2c1: I2c::i2c1(),
            pwm: Pwm::new(),
            adc: Adc::new(),
            pio0: Pio::new(0),
            pio1: Pio::new(1),
            timer: Timer::new(),
            watchdog: Watchdog::new(),
            // Additional peripherals
            dma: Dma::new(),
            xip: Xip::new(),
            i2s0: I2s::new(0),
            i2s1: I2s::new(1),
            rtc: Rtc::new(),
            pll_sys: Pll::new(PllType::Sys),
            pll_usb: Pll::new(PllType::Usb),
            sha256: Sha256::new(),
            trng: Trng::new(),
            usb: Usb::new(),
            // System peripherals
            nvic: Nvic::new(),
            systick: Systick::new(),
            plic: Plic::new(),
            powman: PowerManager::new(),
            sysinfo: Sysinfo::new(),
            otp: Otp::new(),
            // Core peripherals
            mpu: Mpu::new(),
            coresight: Coresight::new(),
            hstx: Hstx::new(),
            interp: Interp::new(0),
            bootram: Bootram::new(),
            busctrl: BusControl::new(),
            reset: Reset::new(),
            // Debug support
            symbols: SymbolTable::new(),
            dwarf: DwarfDebugInfo::new(),
            // Simulation state
            cycles: 0,
            instructions: 0,
            running: false,
        }
    }
    /// Reset the SoC.
    pub fn reset(&mut self) {
        if let Some(ref mut cpu) = self.cpu_arm {
            cpu.reset();
        }
        if let Some(ref mut cpu) = self.cpu_hazard3 {
            cpu.reset();
        }
        self.clocks.reset();
        self.sram.clear();
        self.gpio.reset();
        self.uart0.reset();
        self.uart1.reset();
        self.spi0.reset();
        self.spi1.reset();
        self.i2c0.reset();
        self.i2c1.reset();
        self.pwm.reset();
        self.adc.reset();
        self.pio0.reset();
        self.pio1.reset();
        self.timer.reset();
        self.watchdog.reset();
        // Reset additional peripherals
        self.dma.reset();
        self.xip.reset();
        self.i2s0.reset();
        self.i2s1.reset();
        self.rtc.reset();
        self.pll_sys.reset();
        self.pll_usb.reset();
        self.sha256.reset();
        self.trng.reset();
        self.usb.reset();
        // Reset system peripherals
        self.nvic.reset();
        self.systick.reset();
        self.plic.reset();
        self.powman.reset();
        self.sysinfo.reset();
        self.otp.reset();
        // Reset core peripherals
        self.mpu.reset();
        self.coresight.reset();
        self.hstx.reset();
        self.interp.reset();
        self.bootram.reset();
        self.busctrl.reset();
        self.reset.reset();
        self.cycles = 0;
        self.instructions = 0;
    }
    /// Execute one simulation step.
    pub fn step(&mut self) -> Result<()> {
        // Tick clock system
        self.clocks.tick();
        
        // Tick timers
        self.timer.tick();
        let _ = self.watchdog.tick();
        
        // Tick peripherals
        self.pwm.tick();
        self.adc.tick();
        self.pll_sys.tick();
        self.pll_usb.tick();
        self.pio0.tick();
        self.pio1.tick();
        self.i2s0.tick();
        self.i2s1.tick();
        self.xip.tick();
        
        // Update peripheral interrupts
        self.update_interrupts();

        // Execute CPU instruction with memory access
        // We need to work around the borrow conflict by temporarily moving the CPU
        if let Some(mut cpu) = self.cpu_arm.take() {
            let result = cpu.step_with_memory(self)?;
            self.cpu_arm = Some(cpu);
            self.cycles += result.cycles;
            self.instructions += 1;
        }
        if let Some(mut cpu) = self.cpu_hazard3.take() {
            let result = cpu.step_with_memory(self)?;
            self.cpu_hazard3 = Some(cpu);
            self.cycles += result.cycles;
            self.instructions += 1;
        }

        Ok(())
    }

    /// Update peripheral interrupts in NVIC/PLIC.
    /// This polls peripherals for pending interrupts and updates the interrupt controllers.
    fn update_interrupts(&mut self) {
        // Collect all pending interrupt sources first (avoid borrow conflicts)
        let timer_irq = self.timer.has_interrupt();
        let pwm_irq = self.pwm.has_interrupt();
        let spi0_irq = self.spi0.has_interrupt();
        let spi1_irq = self.spi1.has_interrupt();
        let i2c0_irq = self.i2c0.has_interrupt();
        let i2c1_irq = self.i2c1.has_interrupt();
        let uart0_irq = self.uart0.has_interrupt();
        let uart1_irq = self.uart1.has_interrupt();
        let adc_irq = self.adc.has_interrupt();
        
        // Check GPIO interrupts (any pin)
        let gpio_irq = (0..48).any(|pin| self.gpio.has_interrupt(pin));
        
        // Now set IRQs on CPU
        if let Some(ref mut cpu) = self.cpu_arm {
            if timer_irq { cpu.set_irq(IRQ_TIMER_0 as usize, true); }
            if pwm_irq { cpu.set_irq(IRQ_PWM_WRAP as usize, true); }
            if spi0_irq { cpu.set_irq(IRQ_SPI0 as usize, true); }
            if spi1_irq { cpu.set_irq(IRQ_SPI1 as usize, true); }
            if i2c0_irq { cpu.set_irq(IRQ_I2C0 as usize, true); }
            if i2c1_irq { cpu.set_irq(IRQ_I2C1 as usize, true); }
            if uart0_irq { cpu.set_irq(IRQ_UART0 as usize, true); }
            if uart1_irq { cpu.set_irq(IRQ_UART1 as usize, true); }
            if adc_irq { cpu.set_irq(IRQ_ADC_FIFO as usize, true); }
            if gpio_irq { cpu.set_irq(IRQ_IO_BANK0 as usize, true); }
        }
    }

    /// Get the current PC.
    pub fn pc(&self) -> u32 {
        if let Some(ref cpu) = self.cpu_arm {
            cpu.pc()
        } else if let Some(ref cpu) = self.cpu_hazard3 {
            cpu.pc()
        } else {
            0
        }
    }

    /// Get the stack pointer.
    pub fn sp(&self) -> u32 {
        self.read_reg(13)
    }

    /// Get the link register.
    pub fn lr(&self) -> u32 {
        self.read_reg(14)
    }

    /// Read a register.
    pub fn read_reg(&self, reg: usize) -> u32 {
        if let Some(ref cpu) = self.cpu_arm {
            cpu.read_reg(reg)
        } else if let Some(ref cpu) = self.cpu_hazard3 {
            cpu.read_reg(reg)
        } else {
            0
        }
    }

    /// Write a register.
    pub fn write_reg(&mut self, reg: usize, value: u32) {
        if let Some(ref mut cpu) = self.cpu_arm {
            cpu.write_reg(reg, value);
        }
        if let Some(ref mut cpu) = self.cpu_hazard3 {
            cpu.write_reg(reg, value);
        }
    }

    /// Set the program counter.
    pub fn set_pc(&mut self, pc: u32) {
        if let Some(ref mut cpu) = self.cpu_arm {
            cpu.set_pc(pc);
        }
        if let Some(ref mut cpu) = self.cpu_hazard3 {
            cpu.set_pc(pc);
        }
    }

    /// Set the stack pointer.
    pub fn set_sp(&mut self, sp: u32) {
        self.write_reg(13, sp); // SP is R13/x13
    }

    /// Write a 16-bit value to memory.
    pub fn write_mem_16(&mut self, addr: u32, value: u16) {
        let bytes = value.to_le_bytes();
        self.write_memory(addr, &bytes);
    }

    /// Write a 32-bit value to memory.
    pub fn write_mem_32(&mut self, addr: u32, value: u32) {
        let bytes = value.to_le_bytes();
        self.write_memory(addr, &bytes);
    }

    /// Get the flags/PSR.
    pub fn flags(&self) -> u32 {
        if let Some(ref cpu) = self.cpu_arm {
            cpu.flags()
        } else if let Some(ref cpu) = self.cpu_hazard3 {
            cpu.flags()
        } else {
            0
        }
    }

    /// Get cycle count.
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    /// Get instruction count.
    pub fn instructions(&self) -> u64 {
        self.instructions
    }

    /// Get GPIO value.
    pub fn gpio_value(&self, pin: usize) -> bool {
        self.gpio.get_value(pin)
    }

    /// Get GPIO direction.
    pub fn gpio_direction(&self, pin: usize) -> bool {
        if let Some(p) = self.gpio.get_pin(pin) {
            p.direction
        } else {
            false
        }
    }

    /// Toggle GPIO pin.
    pub fn toggle_gpio(&mut self, pin: usize) {
        if pin < 30 {
            let current = self.gpio.get_value(pin);
            self.gpio.set_input(pin, !current);
        }
    }

    /// Set GPIO input value.
    pub fn set_gpio_input(&mut self, pin: usize, value: bool) {
        self.gpio.set_input(pin, value);
    }

    /// Check if running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Set running state.
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    /// Load firmware into flash.
    pub fn load_firmware(&mut self, data: &[u8]) -> Result<()> {
        self.flash.load(data)
    }

    /// Load firmware at address.
    pub fn load_firmware_at(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        // Determine which memory region
        if addr >= 0x20000000 && addr < 0x20000000 + 520 * 1024 {
            // SRAM
            let offset = (addr - 0x20000000) as usize;
            self.sram.write(offset, data);
        } else {
            // Flash
            self.flash.load_at(addr, data)?;
        }
        Ok(())
    }

    /// Load ELF file into flash/SRAM and extract symbols.
    pub fn load_elf(&mut self, data: &[u8]) -> Result<u32> {
        use rp2350sim_mem::loader::ElfLoader;
        use std::io::Cursor;

        let mut cursor = Cursor::new(data);
        let flash_data = self.flash.data_mut();

        // Create a temporary buffer for SRAM since it's banked
        let sram_size = self.sram.total_size();
        let mut sram_temp = vec![0u8; sram_size];

        let info = ElfLoader::load(&mut cursor, flash_data, &mut sram_temp)?;

        // Copy SRAM data from temp buffer
        self.sram.write(0, &sram_temp);

        // Load symbols into symbol table
        self.symbols.load_from_info(&info.symbols);

        Ok(info.entry_point)
    }

    /// Load only symbols from an ELF file.
    pub fn load_symbols(&mut self, data: &[u8]) -> Result<()> {
        use rp2350sim_mem::loader::ElfLoader;
        use std::io::Cursor;

        let mut cursor = Cursor::new(data);
        let symbols = ElfLoader::load_symbols(&mut cursor)?;

        self.symbols.load_from_info(&symbols);

        Ok(())
    }

    /// Load DWARF debug info from an ELF file.
    pub fn load_dwarf(&mut self, data: &[u8]) -> Result<()> {
        self.dwarf
            .load_from_bytes(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(())
    }

    /// Find source location for an address.
    pub fn find_source_location(&self, addr: u64) -> Option<String> {
        if let Some(loc) = self.dwarf.find_location(addr) {
            return Some(loc.format());
        }
        // Fall back to symbol table
        if let Some((sym, _offset)) = self.symbols.find_function(addr as u32) {
            return Some(format!("{} @ 0x{:08x}", sym.name, sym.address));
        }
        None
    }

    /// Read memory.
    pub fn read_memory(&self, addr: u32, len: usize) -> Vec<u8> {
        let mut data = vec![0u8; len];

        // Determine which memory region
        if addr >= 0x20000000 && addr < 0x20000000 + 520 * 1024 {
            // SRAM
            let offset = (addr - 0x20000000) as usize;
            data.copy_from_slice(&self.sram.read(offset, len));
        } else if addr >= 0x10000000 && addr < 0x11000000 {
            // Flash XIP
            let offset = (addr - 0x10000000) as usize;
            data = self.flash.read_slice(offset, len);
        }

        data
    }

    /// Write memory.
    pub fn write_memory(&mut self, addr: u32, data: &[u8]) {
        if addr >= 0x20000000 && addr < 0x20000000 + 520 * 1024 {
            // SRAM
            let offset = (addr - 0x20000000) as usize;
            self.sram.write(offset, data);
        }
    }

    /// Read a single byte from memory.
    pub fn read_byte(&self, addr: u32) -> u8 {
        if addr >= 0x20000000 && addr < 0x20000000 + 520 * 1024 {
            // SRAM
            self.sram.read_byte(addr)
        } else if addr >= 0x10000000 && addr < 0x11000000 {
            // Flash XIP
            let offset = (addr - 0x10000000) as usize;
            self.flash.data().get(offset).copied().unwrap_or(0)
        } else {
            0
        }
    }

    /// Write a single byte to memory.
    pub fn write_byte(&mut self, addr: u32, byte: u8) {
        if addr >= 0x20000000 && addr < 0x20000000 + 520 * 1024 {
            // SRAM
            self.sram.write_byte(addr, byte);
        }
    }

    /// Push a byte to UART RX FIFO.
    pub fn uart_push_rx(&mut self, uart_num: usize, byte: u8) {
        match uart_num {
            0 => self.uart0.push_rx(byte),
            1 => self.uart1.push_rx(byte),
            _ => {}
        }
    }

    /// Pop a byte from UART TX FIFO.
    pub fn uart_pop_tx(&mut self, uart_num: usize) -> Option<u8> {
        match uart_num {
            0 => self.uart0.pop_tx(),
            1 => self.uart1.pop_tx(),
            _ => None,
        }
    }

    /// Check if UART TX FIFO has data.
    pub fn uart_tx_has_data(&self, uart_num: usize) -> bool {
        match uart_num {
            0 => self.uart0.has_tx_data(),
            1 => self.uart1.has_tx_data(),
            _ => false,
        }
    }

    /// Get UART TX FIFO length.
    pub fn uart_tx_len(&self, uart_num: usize) -> usize {
        match uart_num {
            0 => self.uart0.tx_len(),
            1 => self.uart1.tx_len(),
            _ => 0,
        }
    }

    /// Get UART RX FIFO length.
    pub fn uart_rx_len(&self, _uart_num: usize) -> usize {
        // RX FIFO length not tracked separately
        0
    }

    /// Check if UART is enabled.
    pub fn uart_enabled(&self, uart_num: usize) -> bool {
        match uart_num {
            0 => self.uart0.is_enabled(),
            1 => self.uart1.is_enabled(),
            _ => false,
        }
    }

    /// Get UART baud rate.
    pub fn uart_baud_rate(&self, _uart_num: usize) -> u32 {
        // Default baud rate
        115200
    }

    // Timer methods

    /// Get timer value.
    pub fn timer_value(&self) -> u64 {
        self.cycles
    }

    /// Check if timer is running.
    pub fn timer_running(&self) -> bool {
        self.running
    }

    // ADC methods

    /// Get ADC value.
    pub fn adc_value(&self, channel: usize) -> u16 {
        self.adc.get_value(channel)
    }

    /// Set ADC value (for simulation).
    pub fn set_adc_value(&mut self, channel: usize, value: u16) {
        self.adc.set_value(channel, value);
    }

    /// Check if ADC is enabled.
    pub fn adc_enabled(&self) -> bool {
        self.adc.is_enabled()
    }

    /// Start ADC conversion.
    pub fn adc_start_once(&mut self) {
        self.adc.start_once();
    }

    /// Check if ADC is ready.
    pub fn adc_ready(&self) -> bool {
        self.adc.is_ready()
    }

    // PWM methods

    /// Get PWM duty cycle.
    pub fn pwm_duty(&self, channel: usize) -> u16 {
        self.pwm.get_duty(channel)
    }

    /// Set PWM duty cycle.
    pub fn set_pwm_duty(&mut self, channel: usize, duty: u16) {
        self.pwm.set_duty(channel, duty);
    }

    /// Check if PWM slice is enabled.
    pub fn pwm_slice_enabled(&self, slice: usize) -> bool {
        self.pwm.is_slice_enabled(slice)
    }

    /// Enable/disable PWM slice.
    pub fn pwm_set_slice_enabled(&mut self, slice: usize, enabled: bool) {
        self.pwm.set_slice_enabled(slice, enabled);
    }

    // IRQ methods

    /// Set an IRQ line level.
    pub fn set_irq(&mut self, line: usize, level: bool) {
        self.irq_lines.set(line as u8, level);
        if let Some(ref mut cpu) = self.cpu_arm {
            cpu.set_irq(line, level);
        }
        if let Some(ref mut cpu) = self.cpu_hazard3 {
            cpu.set_irq(line, level);
        }
    }

    /// Get IRQ line level.
    pub fn get_irq(&self, line: usize) -> bool {
        self.irq_lines.get(line as u8)
    }

    /// Get active IRQ lines.
    pub fn active_irqs(&self) -> Vec<u8> {
        self.irq_lines.active_lines()
    }

    /// Get PWM top value.
    pub fn pwm_top(&self, slice: usize) -> u16 {
        self.pwm.get_top(slice)
    }

    /// Set PWM top value.
    pub fn pwm_set_top(&mut self, slice: usize, top: u16) {
        self.pwm.set_top(slice, top);
    }

    /// Get PWM output value.
    pub fn pwm_output(&self, channel: usize) -> bool {
        self.pwm.get_output(channel)
    }

    // SPI methods

    /// Check if SPI is enabled.
    pub fn spi_enabled(&self, spi_num: usize) -> bool {
        match spi_num {
            0 => self.spi0.is_enabled(),
            1 => self.spi1.is_enabled(),
            _ => false,
        }
    }

    /// Get SPI clock rate.
    pub fn spi_clock_rate(&self, _spi_num: usize) -> u32 {
        // Default clock rate
        1_000_000
    }

    /// Get SPI CPOL.
    pub fn spi_cpol(&self, _spi_num: usize) -> bool {
        false
    }

    /// Get SPI CPHA.
    pub fn spi_cpha(&self, _spi_num: usize) -> bool {
        false
    }

    /// Perform SPI transfer.
    pub fn spi_transfer(&mut self, spi_num: usize, byte: u8) -> u8 {
        match spi_num {
            0 => self
                .spi0
                .transfer(byte as u16)
                .map(|v| v as u8)
                .unwrap_or(0),
            1 => self
                .spi1
                .transfer(byte as u16)
                .map(|v| v as u8)
                .unwrap_or(0),
            _ => 0,
        }
    }

    // I2C methods

    /// Check if I2C is enabled.
    pub fn i2c_enabled(&self, i2c_num: usize) -> bool {
        match i2c_num {
            0 => self.i2c0.is_enabled(),
            1 => self.i2c1.is_enabled(),
            _ => false,
        }
    }

    /// Get I2C clock rate.
    pub fn i2c_clock_rate(&self, _i2c_num: usize) -> u32 {
        // Default clock rate
        100_000
    }

    /// I2C read (stub).
    pub fn i2c_read(&mut self, _i2c_num: usize, _addr: u8, _len: u8) -> Vec<u8> {
        // Not fully implemented
        Vec::new()
    }

    /// I2C write (stub).
    pub fn i2c_write(&mut self, _i2c_num: usize, _addr: u8, _data: &[u8]) -> bool {
        // Not fully implemented
        false
    }

    // PIO methods

    /// Check if PIO is enabled (any state machine running).
    pub fn pio_enabled(&self, pio_num: usize) -> bool {
        match pio_num {
            0 => self.pio0.any_sm_enabled(),
            1 => self.pio1.any_sm_enabled(),
            _ => false,
        }
    }

    /// Check if PIO state machine is enabled.
    pub fn pio_sm_enabled(&self, pio_num: usize, sm: usize) -> bool {
        match pio_num {
            0 => self.pio0.sm_enabled(sm),
            1 => self.pio1.sm_enabled(sm),
            _ => false,
        }
    }

    /// Get PIO state machine PC.
    pub fn pio_sm_pc(&self, pio_num: usize, sm: usize) -> u8 {
        match pio_num {
            0 => self.pio0.sm_pc(sm),
            1 => self.pio1.sm_pc(sm),
            _ => 0,
        }
    }

    /// Load PIO program.
    pub fn pio_load_program(&mut self, pio_num: usize, program: &[u16]) {
        match pio_num {
            0 => self.pio0.load_program(0, program),
            1 => self.pio1.load_program(0, program),
            _ => {}
        }
    }

    /// Start PIO state machine.
    pub fn pio_start_sm(&mut self, pio_num: usize, sm: usize) {
        match pio_num {
            0 => self.pio0.start_sm(sm),
            1 => self.pio1.start_sm(sm),
            _ => {}
        }
    }

    /// Stop PIO state machine.
    pub fn pio_stop_sm(&mut self, pio_num: usize, sm: usize) {
        match pio_num {
            0 => self.pio0.stop_sm(sm),
            1 => self.pio1.stop_sm(sm),
            _ => {}
        }
    }

    /// Push data to PIO TX FIFO.
    pub fn pio_push_tx(&mut self, pio_num: usize, sm: usize, value: u32) -> bool {
        match pio_num {
            0 => self.pio0.push_tx(sm, value),
            1 => self.pio1.push_tx(sm, value),
            _ => false,
        }
    }

    /// Pop data from PIO RX FIFO.
    pub fn pio_pop_rx(&mut self, pio_num: usize, sm: usize) -> Option<u32> {
        match pio_num {
            0 => self.pio0.pop_rx(sm),
            1 => self.pio1.pop_rx(sm),
            _ => None,
        }
    }

    /// Tick PIO (execute one cycle).
    pub fn pio_tick(&mut self) {
        self.pio0.tick();
        self.pio1.tick();
    }

    // USB methods

    /// Check if USB is connected.
    pub fn usb_connected(&self) -> bool {
        // USB device not fully implemented yet
        false
    }

    /// Check if USB is in device mode.
    pub fn usb_device_mode(&self) -> bool {
        // USB device not fully implemented yet
        true
    }

    /// Read a 16-bit halfword from memory.
    pub fn read_half(&self, addr: u32) -> u16 {
        let lo = self.read_byte(addr);
        let hi = self.read_byte(addr + 1);
        u16::from_le_bytes([lo, hi])
    }

    /// Read a 32-bit word from memory.
    pub fn read_word(&self, addr: u32) -> u32 {
        let b0 = self.read_byte(addr);
        let b1 = self.read_byte(addr + 1);
        let b2 = self.read_byte(addr + 2);
        let b3 = self.read_byte(addr + 3);
        u32::from_le_bytes([b0, b1, b2, b3])
    }

    /// Write a 16-bit halfword to memory.
    pub fn write_half(&mut self, addr: u32, value: u16) {
        let bytes = value.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr + 1, bytes[1]);
    }

    /// Write a 32-bit word to memory.
    pub fn write_word(&mut self, addr: u32, value: u32) {
        let bytes = value.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr + 1, bytes[1]);
        self.write_byte(addr + 2, bytes[2]);
        self.write_byte(addr + 3, bytes[3]);
    }
}

// Implement MemoryAccess trait for ARM backend
impl ArmMemoryAccess for Soc {
    fn read_byte(&self, addr: u32) -> Result<u8> {
        Ok(self.read_byte(addr))
    }

    fn read_half(&self, addr: u32) -> Result<u16> {
        Ok(self.read_half(addr))
    }

    fn read_word(&self, addr: u32) -> Result<u32> {
        Ok(self.read_word(addr))
    }

    fn write_byte(&mut self, addr: u32, value: u8) -> Result<()> {
        self.write_byte(addr, value);
        Ok(())
    }

    fn write_half(&mut self, addr: u32, value: u16) -> Result<()> {
        self.write_half(addr, value);
        Ok(())
    }

    fn write_word(&mut self, addr: u32, value: u32) -> Result<()> {
        self.write_word(addr, value);
        Ok(())
    }
}

// Implement MemoryAccess trait for Hazard3 backend
impl Rv32MemoryAccess for Soc {
    fn read_byte(&self, addr: u32) -> Result<u8> {
        Ok(self.read_byte(addr))
    }

    fn read_half(&self, addr: u32) -> Result<u16> {
        Ok(self.read_half(addr))
    }

    fn read_word(&self, addr: u32) -> Result<u32> {
        Ok(self.read_word(addr))
    }

    fn write_byte(&mut self, addr: u32, value: u8) -> Result<()> {
        self.write_byte(addr, value);
        Ok(())
    }

    fn write_half(&mut self, addr: u32, value: u16) -> Result<()> {
        self.write_half(addr, value);
        Ok(())
    }

    fn write_word(&mut self, addr: u32, value: u32) -> Result<()> {
        self.write_word(addr, value);
        Ok(())
    }
}

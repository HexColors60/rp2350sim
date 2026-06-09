//! SPI display emulation.

use super::framebuffer::Framebuffer;

/// SPI display commands (common ST7735/ILI9341 style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum DisplayCommand {
    /// No operation.
    Nop = 0x00,
    /// Software reset.
    SwReset = 0x01,
    /// Read display ID.
    RdDid = 0x04,
    /// Read display status.
    RdStatus = 0x09,
    /// Read display power mode.
    RdPwrMode = 0x0A,
    /// Read display MADCTL.
    RdMadCtl = 0x0B,
    /// Read display pixel format.
    RdPixFmt = 0x0C,
    /// Read display image format.
    RdImgFmt = 0x0D,
    /// Read display signal mode.
    RdSigMode = 0x0E,
    /// Read display self-diagnostic.
    RdSelfDiag = 0x0F,
    /// Enter sleep mode.
    SlpIn = 0x10,
    /// Exit sleep mode.
    SlpOut = 0x11,
    /// Partial mode on.
    PtlOn = 0x12,
    /// Partial mode off.
    PtlOff = 0x13,
    /// Normal display mode on.
    NorOn = 0x14,
    /// Display inversion off.
    InvOff = 0x20,
    /// Display inversion on.
    InvOn = 0x21,
    /// Gamma set.
    GamSet = 0x26,
    /// Display off.
    DisOff = 0x28,
    /// Display on.
    DisOn = 0x29,
    /// Column address set.
    Caset = 0x2A,
    /// Row address set.
    Raset = 0x2B,
    /// Memory write.
    RamWr = 0x2C,
    /// Memory read.
    RamRd = 0x2E,
    /// Partial area.
    Ptlar = 0x30,
    /// Vertical scrolling definition.
    VscrDef = 0x33,
    /// Tearing effect line off.
    TefOff = 0x34,
    /// Tearing effect line on.
    TefOn = 0x35,
    /// Memory access control.
    MadCtl = 0x36,
    /// Vertical scrolling start address.
    VscSAd = 0x37,
    /// Idle mode off.
    IdmOff = 0x38,
    /// Idle mode on.
    IdmOn = 0x39,
    /// Pixel format set.
    PixFmt = 0x3A,
    /// Write memory continue.
    WrMemC = 0x3C,
    /// Read memory continue.
    RdMemC = 0x3E,
    /// Set tear scanline.
    SetTearScanline = 0x44,
    /// Get scanline.
    GetScanline = 0x45,
    /// Write display brightness.
    WrDisBr = 0x51,
    /// Read display brightness.
    RdDisBr = 0x52,
    /// Write CTRL display.
    WrCtrlDis = 0x53,
    /// Read CTRL display.
    RdCtrlDis = 0x54,
    /// Write content adaptive brightness control.
    WrCabc = 0x55,
    /// Read content adaptive brightness control.
    RdCabc = 0x56,
    /// Write CABC minimum brightness.
    WrCabcMb = 0x5E,
    /// Read CABC minimum brightness.
    RdCabcMb = 0x5F,
    /// RGB interface signal control.
    RgbIntf = 0xB0,
    /// Frame rate control (normal mode).
    FrmCtr1 = 0xB1,
    /// Frame rate control (idle mode).
    FrmCtr2 = 0xB2,
    /// Frame rate control (partial mode).
    FrmCtr3 = 0xB3,
    /// Display inversion control.
    InvCtr = 0xB4,
    /// Power control 1.
    PwrCtr1 = 0xC0,
    /// Power control 2.
    PwrCtr2 = 0xC1,
    /// Power control 3.
    PwrCtr3 = 0xC2,
    /// Power control 4.
    PwrCtr4 = 0xC3,
    /// Power control 5.
    PwrCtr5 = 0xC4,
    /// VCOM control 1.
    VmCtr1 = 0xC5,
    /// VCOM control 2.
    VmCtr2 = 0xC7,
    /// NV memory write.
    NvWr = 0xD0,
    /// NV memory protection key.
    NvKey = 0xD1,
    /// NV memory status read.
    NvSt = 0xD2,
    /// Read ID1.
    RdId1 = 0xDA,
    /// Read ID2.
    RdId2 = 0xDB,
    /// Read ID3.
    RdId3 = 0xDC,
    /// Positive gamma correction.
    Pvgam = 0xE0,
    /// Negative gamma correction.
    Nvgam = 0xE1,
    /// Digital gamma control 1.
    DigGam1 = 0xE2,
    /// Digital gamma control 2.
    DigGam2 = 0xE3,
    /// Interface control.
    IntfCtr = 0xF6,
    /// Unknown command.
    Unknown = 0xFF,
}

impl From<u8> for DisplayCommand {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Nop,
            0x01 => Self::SwReset,
            0x04 => Self::RdDid,
            0x09 => Self::RdStatus,
            0x0A => Self::RdPwrMode,
            0x0B => Self::RdMadCtl,
            0x0C => Self::RdPixFmt,
            0x0D => Self::RdImgFmt,
            0x0E => Self::RdSigMode,
            0x0F => Self::RdSelfDiag,
            0x10 => Self::SlpIn,
            0x11 => Self::SlpOut,
            0x12 => Self::PtlOn,
            0x13 => Self::PtlOff,
            0x20 => Self::InvOff,
            0x21 => Self::InvOn,
            0x26 => Self::GamSet,
            0x28 => Self::DisOff,
            0x29 => Self::DisOn,
            0x2A => Self::Caset,
            0x2B => Self::Raset,
            0x2C => Self::RamWr,
            0x2E => Self::RamRd,
            0x30 => Self::Ptlar,
            0x33 => Self::VscrDef,
            0x34 => Self::TefOff,
            0x35 => Self::TefOn,
            0x36 => Self::MadCtl,
            0x37 => Self::VscSAd,
            0x38 => Self::IdmOff,
            0x39 => Self::IdmOn,
            0x3A => Self::PixFmt,
            0x3C => Self::WrMemC,
            0x3E => Self::RdMemC,
            0x44 => Self::SetTearScanline,
            0x45 => Self::GetScanline,
            0x51 => Self::WrDisBr,
            0x52 => Self::RdDisBr,
            0x53 => Self::WrCtrlDis,
            0x54 => Self::RdCtrlDis,
            0x55 => Self::WrCabc,
            0x56 => Self::RdCabc,
            0x5E => Self::WrCabcMb,
            0x5F => Self::RdCabcMb,
            0xB0 => Self::RgbIntf,
            0xB1 => Self::FrmCtr1,
            0xB2 => Self::FrmCtr2,
            0xB3 => Self::FrmCtr3,
            0xB4 => Self::InvCtr,
            0xC0 => Self::PwrCtr1,
            0xC1 => Self::PwrCtr2,
            0xC2 => Self::PwrCtr3,
            0xC3 => Self::PwrCtr4,
            0xC4 => Self::PwrCtr5,
            0xC5 => Self::VmCtr1,
            0xC7 => Self::VmCtr2,
            0xD0 => Self::NvWr,
            0xD1 => Self::NvKey,
            0xD2 => Self::NvSt,
            0xDA => Self::RdId1,
            0xDB => Self::RdId2,
            0xDC => Self::RdId3,
            0xE0 => Self::Pvgam,
            0xE1 => Self::Nvgam,
            0xE2 => Self::DigGam1,
            0xE3 => Self::DigGam2,
            0xF6 => Self::IntfCtr,
            _ => Self::Unknown,
        }
    }
}

/// SPI display device.
#[derive(Debug)]
pub struct SpiDisplay {
    /// Display width.
    width: u32,
    /// Display height.
    height: u32,
    /// Framebuffer.
    framebuffer: Framebuffer,
    /// Command/data mode (D/CX pin state).
    is_data: bool,
    /// Current command.
    current_cmd: DisplayCommand,
    /// Command data buffer.
    cmd_data: [u8; 16],
    /// Byte counter for current command.
    byte_count: usize,
    /// Column start address.
    col_start: u16,
    /// Column end address.
    col_end: u16,
    /// Row start address.
    row_start: u16,
    /// Row end address.
    row_end: u16,
    /// Current X position for drawing.
    draw_x: u16,
    /// Current Y position for drawing.
    draw_y: u16,
    /// Memory access control register.
    madctl: u8,
    /// Pixel format.
    pixfmt: u8,
    /// Display is on.
    display_on: bool,
    /// Sleep mode.
    sleep_mode: bool,
    /// Inversion mode.
    inversion: bool,
}

impl SpiDisplay {
    /// Create a new SPI display.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            framebuffer: Framebuffer::new(width, height),
            is_data: false,
            current_cmd: DisplayCommand::Nop,
            cmd_data: [0; 16],
            byte_count: 0,
            col_start: 0,
            col_end: width as u16 - 1,
            row_start: 0,
            row_end: height as u16 - 1,
            draw_x: 0,
            draw_y: 0,
            madctl: 0,
            pixfmt: 0x55, // 16-bit/pixel
            display_on: false,
            sleep_mode: true,
            inversion: false,
        }
    }

    /// Get the framebuffer.
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    /// Get mutable framebuffer.
    pub fn framebuffer_mut(&mut self) -> &mut Framebuffer {
        &mut self.framebuffer
    }

    /// Get the display dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Set D/CX pin state (command/data mode).
    pub fn set_data_mode(&mut self, is_data: bool) {
        self.is_data = is_data;
    }

    /// Handle a command byte.
    fn handle_command(&mut self, cmd: u8) {
        self.current_cmd = DisplayCommand::from(cmd);
        self.byte_count = 0;
        self.cmd_data = [0; 16];

        // Handle immediate commands
        match self.current_cmd {
            DisplayCommand::SwReset => {
                self.reset();
            }
            DisplayCommand::DisOff => {
                self.display_on = false;
            }
            DisplayCommand::DisOn => {
                self.display_on = true;
            }
            DisplayCommand::SlpIn => {
                self.sleep_mode = true;
            }
            DisplayCommand::SlpOut => {
                self.sleep_mode = false;
            }
            DisplayCommand::InvOff => {
                self.inversion = false;
            }
            DisplayCommand::InvOn => {
                self.inversion = true;
            }
            _ => {}
        }
    }

    /// Handle a data byte.
    fn handle_data(&mut self, data: u8) {
        if self.byte_count < 16 {
            self.cmd_data[self.byte_count] = data;
        }
        self.byte_count += 1;

        match self.current_cmd {
            DisplayCommand::Caset => {
                // Column address set (4 bytes: SC[15:8], SC[7:0], EC[15:8], EC[7:0])
                match self.byte_count {
                    1 => self.col_start = (data as u16) << 8,
                    2 => self.col_start |= data as u16,
                    3 => self.col_end = (data as u16) << 8,
                    4 => self.col_end |= data as u16,
                    _ => {}
                }
            }
            DisplayCommand::Raset => {
                // Row address set (4 bytes: SP[15:8], SP[7:0], EP[15:8], EP[7:0])
                match self.byte_count {
                    1 => self.row_start = (data as u16) << 8,
                    2 => self.row_start |= data as u16,
                    3 => self.row_end = (data as u16) << 8,
                    4 => self.row_end |= data as u16,
                    _ => {}
                }
            }
            DisplayCommand::RamWr => {
                // Memory write - pixel data
                self.write_pixel_data(data);
            }
            DisplayCommand::MadCtl => {
                // Memory access control
                if self.byte_count == 1 {
                    self.madctl = data;
                }
            }
            DisplayCommand::PixFmt => {
                // Pixel format set
                if self.byte_count == 1 {
                    self.pixfmt = data;
                }
            }
            _ => {}
        }
    }

    /// Write pixel data to framebuffer.
    fn write_pixel_data(&mut self, data: u8) {
        // For 16-bit color (RGB565), we need 2 bytes per pixel
        static mut PIXEL_BUFFER: [u8; 2] = [0; 2];
        static mut BUFFER_INDEX: usize = 0;

        // SAFETY: This is a simple static buffer for pixel data
        // In a real implementation, this would be part of the struct
        unsafe {
            PIXEL_BUFFER[BUFFER_INDEX] = data;
            BUFFER_INDEX += 1;

            if BUFFER_INDEX >= 2 {
                // We have a complete pixel - RGB565 format
                let color = ((PIXEL_BUFFER[0] as u16) << 8) | (PIXEL_BUFFER[1] as u16);

                // Write to framebuffer
                if self.draw_x >= self.col_start && self.draw_x <= self.col_end &&
                   self.draw_y >= self.row_start && self.draw_y <= self.row_end {
                    let x = self.draw_x.min(self.width as u16 - 1) as u32;
                    let y = self.draw_y.min(self.height as u16 - 1) as u32;
                    self.framebuffer.set_pixel(x, y, color);
                }

                // Advance position
                self.draw_x += 1;
                if self.draw_x > self.col_end {
                    self.draw_x = self.col_start;
                    self.draw_y += 1;
                    if self.draw_y > self.row_end {
                        self.draw_y = self.row_start;
                    }
                }

                BUFFER_INDEX = 0;
            }
        }
    }

    /// Reset the display.
    pub fn reset(&mut self) {
        self.col_start = 0;
        self.col_end = self.width as u16 - 1;
        self.row_start = 0;
        self.row_end = self.height as u16 - 1;
        self.draw_x = 0;
        self.draw_y = 0;
        self.madctl = 0;
        self.pixfmt = 0x55;
        self.display_on = false;
        self.sleep_mode = true;
        self.inversion = false;
        self.framebuffer.clear(0);
    }

    /// Write a byte to the display.
    pub fn write_byte(&mut self, data: u8) {
        if self.is_data {
            self.handle_data(data);
        } else {
            self.handle_command(data);
        }
    }

    /// Check if display is on.
    pub fn is_display_on(&self) -> bool {
        self.display_on
    }

    /// Check if in sleep mode.
    pub fn is_sleeping(&self) -> bool {
        self.sleep_mode
    }
}

impl rp2350sim_devices::spi::SpiSlave for SpiDisplay {
    fn transfer(&mut self, tx_data: u16) -> u16 {
        let byte = tx_data as u8;

        if self.is_data {
            self.handle_data(byte);
        } else {
            self.handle_command(byte);
        }

        // Return dummy data for reads
        0xFF00 | (byte as u16)
    }
}
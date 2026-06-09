//! SysTick timer.


/// SysTick timer.
#[derive(Debug, Clone, Default)]
pub struct SysTick {
    /// Control and Status Register
    csr: u32,
    /// Reload Value Register
    rvr: u32,
    /// Current Value Register
    cvr: u32,
    /// Calibration Value Register
    #[allow(dead_code)]
    calib: u32,
}

impl SysTick {
    pub fn new() -> Self {
        Self {
            csr: 0,
            rvr: 0,
            cvr: 0,
            calib: 0x00FFFFFF, // 24-bit counter
        }
    }

    /// Check if SysTick is enabled.
    pub fn is_enabled(&self) -> bool {
        (self.csr & 1) != 0
    }

    /// Check if interrupt is enabled.
    pub fn is_interrupt_enabled(&self) -> bool {
        (self.csr & 2) != 0
    }

    /// Check if using external clock.
    pub fn is_external_clock(&self) -> bool {
        (self.csr & 4) != 0
    }

    /// Check if COUNTFLAG is set.
    pub fn count_flag(&self) -> bool {
        (self.csr & (1 << 16)) != 0
    }

    /// Get the reload value.
    pub fn reload_value(&self) -> u32 {
        self.rvr
    }

    /// Get the current value.
    pub fn current_value(&self) -> u32 {
        self.cvr
    }

    /// Tick the timer.
    pub fn tick(&mut self) -> bool {
        if !self.is_enabled() {
            return false;
        }

        if self.cvr == 0 {
            self.cvr = self.rvr;
            self.csr |= 1 << 16; // Set COUNTFLAG
            return self.is_interrupt_enabled();
        } else {
            self.cvr -= 1;
        }

        false
    }

    /// Read CSR.
    pub fn read_csr(&mut self) -> u32 {
        let value = self.csr;
        self.csr &= !(1 << 16); // Clear COUNTFLAG on read
        value
    }

    /// Write CSR.
    pub fn write_csr(&mut self, value: u32) {
        self.csr = (self.csr & 0x00010000) | (value & 0x00010007);
        if !self.is_enabled() {
            self.cvr = 0;
        }
    }

    /// Read RVR.
    pub fn read_rvr(&self) -> u32 {
        self.rvr
    }

    /// Write RVR.
    pub fn write_rvr(&mut self, value: u32) {
        self.rvr = value & 0x00FFFFFF;
    }

    /// Read CVR.
    pub fn read_cvr(&self) -> u32 {
        self.cvr
    }

    /// Write CVR.
    pub fn write_cvr(&mut self, _value: u32) {
        self.cvr = 0; // Any write clears CVR
        self.csr &= !(1 << 16); // Clear COUNTFLAG
    }

    /// Reset the timer.
    pub fn reset(&mut self) {
        self.csr = 0;
        self.rvr = 0;
        self.cvr = 0;
    }
}
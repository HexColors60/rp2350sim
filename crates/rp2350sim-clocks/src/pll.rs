//! PLL (Phase-Locked Loop) implementation for RP2350.
//!
//! Implements the PLL peripherals (PLL_SYS and PLL_USB) for clock generation.

/// PLL reference divider range.
pub const REFDIV_MIN: u32 = 1;
pub const REFDIV_MAX: u32 = 63;

/// PLL feedback divider range.
pub const FBDIV_MIN: u32 = 16;
pub const FBDIV_MAX: u32 = 320;

/// PLL post divider range.
pub const POSTDIV_MIN: u32 = 1;
pub const POSTDIV_MAX: u32 = 7;

/// PLL VCO frequency range (MHz).
pub const VCO_MIN_MHZ: u64 = 400;
pub const VCO_MAX_MHZ: u64 = 1600;

/// PLL state.
#[derive(Debug, Clone)]
pub struct Pll {
    /// PLL enabled.
    pub enabled: bool,
    /// Reference clock divider (1-63).
    pub refdiv: u32,
    /// Feedback divider (16-320).
    pub fbdiv: u32,
    /// VCO frequency in Hz.
    pub vco_freq: u64,
    /// Post divider 1 (1-7).
    pub postdiv1: u32,
    /// Post divider 2 (1-7).
    pub postdiv2: u32,
    /// Reference clock frequency in Hz.
    pub ref_freq: u64,
    /// PLL locked status.
    pub locked: bool,
}

impl Default for Pll {
    fn default() -> Self {
        Self {
            enabled: false,
            refdiv: 1,
            fbdiv: 16,
            vco_freq: 0,
            postdiv1: 1,
            postdiv2: 1,
            ref_freq: 12_000_000, // Default 12 MHz reference
            locked: false,
        }
    }
}

impl Pll {
    /// Create a new PLL with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new PLL with a specific reference frequency.
    pub fn with_ref_freq(ref_freq: u64) -> Self {
        Self {
            ref_freq,
            ..Self::default()
        }
    }

    /// Calculate the output frequency.
    pub fn output_freq(&self) -> u64 {
        if !self.enabled || !self.locked {
            return 0;
        }
        if self.postdiv1 == 0 || self.postdiv2 == 0 {
            return 0;
        }
        self.vco_freq / (self.postdiv1 as u64 * self.postdiv2 as u64)
    }

    /// Calculate VCO frequency from dividers.
    pub fn calculate_vco_freq(&self) -> u64 {
        if self.refdiv == 0 {
            return 0;
        }
        self.ref_freq * self.fbdiv as u64 / self.refdiv as u64
    }

    /// Configure the PLL with the given parameters.
    /// Returns true if configuration is valid.
    pub fn configure(&mut self, refdiv: u32, fbdiv: u32, postdiv1: u32, postdiv2: u32) -> bool {
        // Validate parameters
        if refdiv < REFDIV_MIN || refdiv > REFDIV_MAX {
            return false;
        }
        if fbdiv < FBDIV_MIN || fbdiv > FBDIV_MAX {
            return false;
        }
        if postdiv1 < POSTDIV_MIN || postdiv1 > POSTDIV_MAX {
            return false;
        }
        if postdiv2 < POSTDIV_MIN || postdiv2 > POSTDIV_MAX {
            return false;
        }

        self.refdiv = refdiv;
        self.fbdiv = fbdiv;
        self.postdiv1 = postdiv1;
        self.postdiv2 = postdiv2;

        // Calculate VCO frequency
        self.vco_freq = self.calculate_vco_freq();

        // Validate VCO frequency
        let vco_mhz = self.vco_freq / 1_000_000;
        if vco_mhz < VCO_MIN_MHZ || vco_mhz > VCO_MAX_MHZ {
            return false;
        }

        true
    }

    /// Enable the PLL.
    pub fn enable(&mut self) {
        self.enabled = true;
        // Simulate lock time (in real hardware this takes time)
        self.locked = true;
    }

    /// Disable the PLL.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.locked = false;
    }

    /// Check if the PLL is locked.
    pub fn is_locked(&self) -> bool {
        self.enabled && self.locked
    }

    /// Reset the PLL to default state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get the reference frequency after refdiv.
    pub fn ref_freq_divided(&self) -> u64 {
        if self.refdiv == 0 {
            return 0;
        }
        self.ref_freq / self.refdiv as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pll_creation() {
        let pll = Pll::new();
        assert!(!pll.enabled);
        assert_eq!(pll.refdiv, 1);
        assert_eq!(pll.fbdiv, 16);
        assert_eq!(pll.postdiv1, 1);
        assert_eq!(pll.postdiv2, 1);
    }

    #[test]
    fn test_pll_output_disabled() {
        let pll = Pll::new();
        assert_eq!(pll.output_freq(), 0);
    }

    #[test]
    fn test_pll_enable() {
        let mut pll = Pll::new();
        pll.vco_freq = 1_000_000_000; // 1 GHz VCO
        pll.enable();
        assert!(pll.enabled);
        assert!(pll.locked);
    }

    #[test]
    fn test_pll_output_enabled() {
        let mut pll = Pll::new();
        pll.vco_freq = 1_000_000_000; // 1 GHz VCO
        pll.postdiv1 = 5;
        pll.postdiv2 = 2;
        pll.enable();

        // Output = VCO / (postdiv1 * postdiv2) = 1GHz / 10 = 100 MHz
        assert_eq!(pll.output_freq(), 100_000_000);
    }

    #[test]
    fn test_pll_configure() {
        let mut pll = Pll::with_ref_freq(12_000_000); // 12 MHz reference

        // Configure for 125 MHz output (RP2040/RP2350 typical)
        // refdiv=1, fbdiv=125, postdiv1=6, postdiv2=2
        // VCO = 12MHz * 125 / 1 = 1500 MHz
        // Output = 1500MHz / (6 * 2) = 125 MHz
        let result = pll.configure(1, 125, 6, 2);
        assert!(result);
        assert_eq!(pll.vco_freq, 1_500_000_000);
    }

    #[test]
    fn test_pll_configure_invalid_refdiv() {
        let mut pll = Pll::new();
        let result = pll.configure(0, 100, 2, 2); // refdiv too low
        assert!(!result);

        let result = pll.configure(64, 100, 2, 2); // refdiv too high
        assert!(!result);
    }

    #[test]
    fn test_pll_configure_invalid_fbdiv() {
        let mut pll = Pll::new();
        let result = pll.configure(1, 15, 2, 2); // fbdiv too low
        assert!(!result);

        let result = pll.configure(1, 321, 2, 2); // fbdiv too high
        assert!(!result);
    }

    #[test]
    fn test_pll_configure_invalid_postdiv() {
        let mut pll = Pll::new();
        let result = pll.configure(1, 100, 0, 2); // postdiv1 too low
        assert!(!result);

        let result = pll.configure(1, 100, 8, 2); // postdiv1 too high
        assert!(!result);
    }

    #[test]
    fn test_pll_vco_out_of_range() {
        let mut pll = Pll::with_ref_freq(12_000_000);

        // Try to configure VCO below 400 MHz
        // VCO = 12MHz * 20 / 1 = 240 MHz (too low)
        let result = pll.configure(1, 20, 1, 1);
        assert!(!result);

        // Try to configure VCO above 1600 MHz
        // VCO = 12MHz * 200 / 1 = 2400 MHz (too high)
        let result = pll.configure(1, 200, 1, 1);
        assert!(!result);
    }

    #[test]
    fn test_pll_usb_configuration() {
        // Typical USB PLL configuration: 48 MHz output
        // refdiv=1, fbdiv=40, postdiv1=5, postdiv2=2
        // VCO = 12MHz * 40 = 480 MHz
        // Output = 480MHz / (5 * 2) = 48 MHz
        let mut pll = Pll::with_ref_freq(12_000_000);
        let result = pll.configure(1, 40, 5, 2);
        assert!(result);
        assert_eq!(pll.vco_freq, 480_000_000);

        pll.enable();
        assert_eq!(pll.output_freq(), 48_000_000);
    }

    #[test]
    fn test_pll_sys_configuration() {
        // Typical SYS PLL configuration: 150 MHz output for RP2350
        // refdiv=1, fbdiv=100, postdiv1=4, postdiv2=2
        // VCO = 12MHz * 100 = 1200 MHz
        // Output = 1200MHz / (4 * 2) = 150 MHz
        let mut pll = Pll::with_ref_freq(12_000_000);
        let result = pll.configure(1, 100, 4, 2);
        assert!(result);
        assert_eq!(pll.vco_freq, 1_200_000_000);

        pll.enable();
        assert_eq!(pll.output_freq(), 150_000_000);
    }

    #[test]
    fn test_pll_disable() {
        let mut pll = Pll::new();
        pll.vco_freq = 1_000_000_000;
        pll.enable();
        assert!(pll.enabled);
        assert!(pll.locked);

        pll.disable();
        assert!(!pll.enabled);
        assert!(!pll.locked);
        assert_eq!(pll.output_freq(), 0);
    }

    #[test]
    fn test_pll_reset() {
        let mut pll = Pll::new();
        pll.configure(1, 100, 4, 2);
        pll.enable();

        pll.reset();
        assert!(!pll.enabled);
        assert_eq!(pll.refdiv, 1);
        assert_eq!(pll.fbdiv, 16);
        assert_eq!(pll.vco_freq, 0);
    }

    #[test]
    fn test_pll_ref_freq_divided() {
        let mut pll = Pll::with_ref_freq(12_000_000);
        pll.refdiv = 2;
        assert_eq!(pll.ref_freq_divided(), 6_000_000);

        pll.refdiv = 4;
        assert_eq!(pll.ref_freq_divided(), 3_000_000);
    }

    #[test]
    fn test_pll_zero_postdiv() {
        let mut pll = Pll::new();
        pll.vco_freq = 1_000_000_000;
        pll.postdiv1 = 0;
        pll.enable();
        assert_eq!(pll.output_freq(), 0);

        pll.postdiv1 = 2;
        pll.postdiv2 = 0;
        assert_eq!(pll.output_freq(), 0);
    }

    #[test]
    fn test_pll_not_locked() {
        let mut pll = Pll::new();
        pll.vco_freq = 1_000_000_000;
        pll.enabled = true;
        pll.locked = false; // Not locked
        assert_eq!(pll.output_freq(), 0);
    }
}
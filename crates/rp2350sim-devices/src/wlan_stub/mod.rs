//! WLAN (CYW43439) controller stub for RP2350 Pico 2 W.
//!
//! This module simulates the Infineon CYW43439 WiFi/Bluetooth chip
//! found on the Raspberry Pi Pico 2 W. It provides:
//! - SPI interface simulation
//! - Basic WiFi state tracking
//! - IOCTL/IOVAR command simulation
//! - Connection state management
//!
//! Note: This is a stub implementation for simulation purposes.
//! Real WiFi network operations are not performed.

use rp2350sim_core::{Device, DeviceId, Result};

/// CYW43 base address (not memory-mapped, accessed via SPI).
/// This is a virtual address for simulation purposes.
pub const CYW43_BASE: u32 = 0x0000_0000;

/// SPI function codes for CYW43.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiFunction {
    /// Bus function (function 0).
    Bus = 0,
    /// Backplane function (function 1).
    Backplane = 1,
    /// WLAN function (function 2).
    Wlan = 2,
    /// Bluetooth function (function 3).
    Bluetooth = 3,
}

/// WiFi link state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkState {
    /// Not connected.
    #[default]
    Down,
    /// Connecting to AP.
    Connecting,
    /// Connected to AP.
    Up,
    /// Connection failed.
    Failed,
}

/// WiFi authentication mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMode {
    /// Open (no authentication).
    #[default]
    Open,
    /// WPA/WPA2 Personal.
    WpaPsk,
    /// WPA3 Personal.
    Wpa3Psk,
}

/// WiFi channel information.
#[derive(Debug, Clone, Default)]
pub struct ChannelInfo {
    /// Channel number (1-14 for 2.4GHz).
    pub number: u8,
    /// Frequency in MHz.
    pub frequency: u16,
}

/// WiFi access point information.
#[derive(Debug, Clone, Default)]
pub struct AccessPoint {
    /// SSID.
    pub ssid: String,
    /// BSSID (MAC address of AP).
    pub bssid: [u8; 6],
    /// Channel info.
    pub channel: ChannelInfo,
    /// Signal strength (RSSI in dBm).
    pub rssi: i8,
    /// Authentication mode.
    pub auth_mode: AuthMode,
}

/// WiFi statistics.
#[derive(Debug, Clone, Default)]
pub struct WifiStats {
    /// Packets transmitted.
    pub tx_packets: u64,
    /// Packets received.
    pub rx_packets: u64,
    /// Transmit errors.
    pub tx_errors: u64,
    /// Receive errors.
    pub rx_errors: u64,
    /// Bytes transmitted.
    pub tx_bytes: u64,
    /// Bytes received.
    pub rx_bytes: u64,
}

/// CYW43 register definitions.
pub mod regs {
    // SPI Bus registers (Function 0)
    pub const SPI_BUS_CONTROL: u32 = 0x000;
    pub const SPI_INTERRUPT_REGISTER: u32 = 0x004;
    pub const SPI_INTERRUPT_ENABLE_REGISTER: u32 = 0x006;
    pub const SPI_STATUS_REGISTER: u32 = 0x008;
    pub const SPI_RESPONSE_DELAY: u32 = 0x00C;

    // Backplane registers (Function 1)
    pub const SDIO_CHIP_CLOCK_CSR: u32 = 0x10000;
    pub const SDIO_PULL_UP: u32 = 0x10004;
    pub const SDIO_INT_HOST_MASK: u32 = 0x10008;

    // Core addresses
    pub const CORE_WLAN_ARM: u32 = 0x1800_0000;
    pub const CORE_SOCSRAM: u32 = 0x1800_1000;
    pub const SOCSRAM_BASE_ADDRESS: u32 = 0x1800_4000;
    pub const SDIO_BASE_ADDRESS: u32 = 0x1800_7000;

    // Clock status bits
    pub const SBSDIO_ALP_AVAIL: u8 = 0x04;
    pub const SBSDIO_HT_AVAIL: u8 = 0x80;
}

/// IOCTL command codes.
pub mod ioctl {
    /// Get SSID.
    pub const GET_SSID: u32 = 2;
    /// Set SSID.
    pub const SET_SSID: u32 = 3;
    /// Get channel.
    pub const GET_CHANNEL: u32 = 29;
    /// Set channel.
    pub const SET_CHANNEL: u32 = 30;
    /// Get RSSI.
    pub const GET_RSSI: u32 = 127;
    /// Scan results.
    pub const SCAN_RESULTS: u32 = 250;
}

/// Async event types.
pub mod event {
    /// SET_SSID event.
    pub const SET_SSID: u32 = 0;
    /// JOIN event.
    pub const JOIN: u32 = 1;
    /// AUTH event.
    pub const AUTH: u32 = 3;
    /// DEAUTH event.
    pub const DEAUTH: u32 = 5;
    /// LINK event.
    pub const LINK: u32 = 16;
    /// SCAN_COMPLETE event.
    pub const SCAN_COMPLETE: u32 = 26;
    /// PSK_SUP event (key exchange).
    pub const PSK_SUP: u32 = 46;
}

/// CYW43439 WiFi/Bluetooth controller stub.
#[derive(Debug)]
pub struct WlanStub {
    // Power and enable state
    enabled: bool,
    power_on: bool,

    // WiFi state
    link_state: LinkState,
    current_ap: AccessPoint,
    mac_address: [u8; 6],

    // Network credentials (stored when set)
    ssid: String,
    password: String,

    // Connection state
    auth_ok: bool,
    join_ok: bool,
    key_exchange_ok: bool,

    // Statistics
    stats: WifiStats,

    // SPI state
    spi_status: u16,
    spi_interrupts: u16,
    spi_int_enable: u16,

    // Backplane state
    #[allow(dead_code)]
    backplane_window: u32,
    clock_csr: u8,

    // Buffer for IOCTL responses
    #[allow(dead_code)]
    response_buffer: Vec<u8>,
}

impl Default for WlanStub {
    fn default() -> Self {
        Self::new()
    }
}

impl WlanStub {
    /// Create a new WLAN stub.
    pub fn new() -> Self {
        Self {
            enabled: false,
            power_on: false,
            link_state: LinkState::Down,
            current_ap: AccessPoint::default(),
            mac_address: [0x02, 0x00, 0x00, 0x00, 0x00, 0x00], // Locally administered
            ssid: String::new(),
            password: String::new(),
            auth_ok: false,
            join_ok: false,
            key_exchange_ok: false,
            stats: WifiStats::default(),
            spi_status: 0,
            spi_interrupts: 0,
            spi_int_enable: 0,
            backplane_window: 0,
            clock_csr: 0,
            response_buffer: Vec::new(),
        }
    }

    // Public state query methods

    /// Check if WLAN is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if power is on.
    pub fn is_power_on(&self) -> bool {
        self.power_on
    }

    /// Get current link state.
    pub fn link_state(&self) -> LinkState {
        self.link_state
    }

    /// Check if connected to WiFi.
    pub fn is_connected(&self) -> bool {
        self.link_state == LinkState::Up
    }

    /// Get current SSID.
    pub fn ssid(&self) -> &str {
        &self.ssid
    }

    /// Get MAC address.
    pub fn mac_address(&self) -> &[u8; 6] {
        &self.mac_address
    }

    /// Get current access point info.
    pub fn current_ap(&self) -> &AccessPoint {
        &self.current_ap
    }

    /// Get WiFi statistics.
    pub fn stats(&self) -> &WifiStats {
        &self.stats
    }

    // Power management

    /// Power on the CYW43.
    pub fn power_on(&mut self) {
        self.power_on = true;
    }

    /// Power off the CYW43.
    pub fn power_off(&mut self) {
        self.power_on = false;
        self.enabled = false;
        self.link_state = LinkState::Down;
        self.ssid.clear();
        self.password.clear();
    }

    // WiFi operations

    /// Enable WiFi.
    pub fn enable(&mut self) {
        if self.power_on {
            self.enabled = true;
        }
    }

    /// Disable WiFi.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.link_state = LinkState::Down;
    }

    /// Set SSID (for connection).
    pub fn set_ssid(&mut self, ssid: &str) {
        self.ssid = ssid.to_string();
        self.current_ap.ssid = ssid.to_string();
    }

    /// Set password (for connection).
    pub fn set_password(&mut self, password: &str) {
        self.password = password.to_string();
    }

    /// Simulate connecting to WiFi.
    pub fn connect(&mut self) {
        if !self.enabled || self.ssid.is_empty() {
            self.link_state = LinkState::Failed;
            return;
        }

        self.link_state = LinkState::Connecting;
        self.auth_ok = false;
        self.join_ok = false;
        self.key_exchange_ok = false;

        // Simulate connection process
        // In real hardware, this would involve multiple steps:
        // 1. Authentication (AUTH)
        // 2. Association (JOIN)
        // 3. Key exchange for WPA (PSK_SUP)

        // For simulation, we'll assume success
        self.auth_ok = true;
        self.join_ok = true;
        self.key_exchange_ok = true;

        // Generate simulated BSSID
        self.current_ap.bssid = [
            0x02,
            0x00,
            0x00,
            0x00,
            0x00,
            self.ssid.len() as u8,
        ];
        self.current_ap.channel = ChannelInfo {
            number: 6,
            frequency: 2437,
        };
        self.current_ap.rssi = -45; // Good signal
        self.current_ap.auth_mode = if self.password.is_empty() {
            AuthMode::Open
        } else {
            AuthMode::WpaPsk
        };

        self.link_state = LinkState::Up;
    }

    /// Simulate disconnecting from WiFi.
    pub fn disconnect(&mut self) {
        self.link_state = LinkState::Down;
        self.auth_ok = false;
        self.join_ok = false;
        self.key_exchange_ok = false;
        self.ssid.clear();
        self.password.clear();
    }

    /// Simulate scanning for networks.
    pub fn scan(&mut self) -> Vec<AccessPoint> {
        // Return simulated scan results
        vec![
            AccessPoint {
                ssid: "TestNetwork".to_string(),
                bssid: [0x02, 0x00, 0x00, 0x00, 0x00, 0x01],
                channel: ChannelInfo { number: 1, frequency: 2412 },
                rssi: -50,
                auth_mode: AuthMode::WpaPsk,
            },
            AccessPoint {
                ssid: "OpenNetwork".to_string(),
                bssid: [0x02, 0x00, 0x00, 0x00, 0x00, 0x02],
                channel: ChannelInfo { number: 6, frequency: 2437 },
                rssi: -60,
                auth_mode: AuthMode::Open,
            },
            AccessPoint {
                ssid: "WPA3Network".to_string(),
                bssid: [0x02, 0x00, 0x00, 0x00, 0x00, 0x03],
                channel: ChannelInfo { number: 11, frequency: 2462 },
                rssi: -70,
                auth_mode: AuthMode::Wpa3Psk,
            },
        ]
    }

    // Data transfer simulation

    /// Simulate sending data over WiFi.
    pub fn send_data(&mut self, data: &[u8]) -> Result<()> {
        if self.link_state != LinkState::Up {
            self.stats.tx_errors += 1;
            return Ok(());
        }

        self.stats.tx_packets += 1;
        self.stats.tx_bytes += data.len() as u64;
        Ok(())
    }

    /// Simulate receiving data over WiFi.
    pub fn receive_data(&mut self) -> Option<Vec<u8>> {
        if self.link_state != LinkState::Up {
            return None;
        }

        // No data in stub
        None
    }

    // SPI interface simulation

    /// Read SPI status.
    pub fn spi_status(&self) -> u16 {
        self.spi_status
    }

    /// Check if F2 (WLAN) is ready.
    pub fn f2_ready(&self) -> bool {
        self.enabled && self.power_on
    }

    /// Set MAC address.
    pub fn set_mac_address(&mut self, mac: [u8; 6]) {
        self.mac_address = mac;
    }
}

impl Device for WlanStub {
    fn id(&self) -> DeviceId {
        DeviceId::WLAN
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        // Simulate register reads
        match addr {
            // SPI status register
            regs::SPI_STATUS_REGISTER => {
                let mut status = 0u16;
                if self.f2_ready() {
                    status |= 0x0100; // F2 ready
                }
                Ok(status as u32)
            }
            // SPI interrupt register
            regs::SPI_INTERRUPT_REGISTER => {
                Ok(self.spi_interrupts as u32)
            }
            // Clock control
            regs::SDIO_CHIP_CLOCK_CSR => {
                Ok(self.clock_csr as u32)
            }
            // Backplane window
            0x1000_0000..=0x1FFF_FFFF => {
                // Simulate backplane read
                Ok(0)
            }
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        match addr {
            // SPI interrupt enable
            regs::SPI_INTERRUPT_ENABLE_REGISTER => {
                self.spi_int_enable = value as u16;
            }
            // Clock control
            regs::SDIO_CHIP_CLOCK_CSR => {
                self.clock_csr = value as u8;
                if (value as u8) & regs::SBSDIO_HT_AVAIL != 0 {
                    // HT clock requested
                }
            }
            // Backplane window
            0x1000_0000..=0x1FFF_FFFF => {
                // Simulate backplane write
            }
            _ => {}
        }
        Ok(())
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wlan_creation() {
        let wlan = WlanStub::new();
        assert!(!wlan.is_enabled());
        assert!(!wlan.is_power_on());
        assert!(!wlan.is_connected());
        assert_eq!(wlan.link_state(), LinkState::Down);
    }

    #[test]
    fn test_power_on_off() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        assert!(wlan.is_power_on());
        assert!(!wlan.is_enabled()); // Not enabled until explicitly enabled

        wlan.power_off();
        assert!(!wlan.is_power_on());
        assert!(!wlan.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let mut wlan = WlanStub::new();

        // Can't enable without power
        wlan.enable();
        assert!(!wlan.is_enabled());

        // Enable after power on
        wlan.power_on();
        wlan.enable();
        assert!(wlan.is_enabled());

        // Disable
        wlan.disable();
        assert!(!wlan.is_enabled());
    }

    #[test]
    fn test_wifi_connect_no_power() {
        let mut wlan = WlanStub::new();

        wlan.set_ssid("TestNetwork");
        wlan.set_password("password");
        wlan.connect();

        // Should fail - not enabled
        assert_eq!(wlan.link_state(), LinkState::Failed);
    }

    #[test]
    fn test_wifi_connect_no_ssid() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.connect();

        // Should fail - no SSID
        assert_eq!(wlan.link_state(), LinkState::Failed);
    }

    #[test]
    fn test_wifi_connect_success() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.set_ssid("TestNetwork");
        wlan.set_password("password123");
        wlan.connect();

        assert_eq!(wlan.link_state(), LinkState::Up);
        assert!(wlan.is_connected());
        assert_eq!(wlan.ssid(), "TestNetwork");
        assert_eq!(wlan.current_ap().auth_mode, AuthMode::WpaPsk);
    }

    #[test]
    fn test_wifi_connect_open() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.set_ssid("OpenNetwork");
        // No password
        wlan.connect();

        assert_eq!(wlan.link_state(), LinkState::Up);
        assert!(wlan.is_connected());
        assert_eq!(wlan.current_ap().auth_mode, AuthMode::Open);
    }

    #[test]
    fn test_wifi_disconnect() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.set_ssid("TestNetwork");
        wlan.connect();
        assert!(wlan.is_connected());

        wlan.disconnect();
        assert!(!wlan.is_connected());
        assert_eq!(wlan.link_state(), LinkState::Down);
        assert!(wlan.ssid().is_empty());
    }

    #[test]
    fn test_wifi_scan() {
        let mut wlan = WlanStub::new();

        let networks = wlan.scan();
        assert!(!networks.is_empty());
        assert!(networks.iter().any(|n| n.ssid == "TestNetwork"));
    }

    #[test]
    fn test_wifi_stats() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.set_ssid("TestNetwork");
        wlan.connect();

        // Send some data
        wlan.send_data(&[1, 2, 3, 4, 5]).unwrap();
        wlan.send_data(&[6, 7, 8, 9, 10]).unwrap();

        assert_eq!(wlan.stats().tx_packets, 2);
        assert_eq!(wlan.stats().tx_bytes, 10);
    }

    #[test]
    fn test_wifi_stats_no_connection() {
        let mut wlan = WlanStub::new();

        // Try to send without connection
        wlan.send_data(&[1, 2, 3]).unwrap();

        assert_eq!(wlan.stats().tx_packets, 0);
        assert_eq!(wlan.stats().tx_errors, 1);
    }

    #[test]
    fn test_mac_address() {
        let mut wlan = WlanStub::new();

        let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        wlan.set_mac_address(mac);

        assert_eq!(wlan.mac_address(), &mac);
    }

    #[test]
    fn test_spi_status() {
        let mut wlan = WlanStub::new();

        // Not ready without power
        assert!(!wlan.f2_ready());

        wlan.power_on();
        wlan.enable();
        assert!(wlan.f2_ready());
    }

    #[test]
    fn test_device_reset() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.set_ssid("TestNetwork");
        wlan.connect();
        assert!(wlan.is_connected());

        wlan.reset();

        assert!(!wlan.is_enabled());
        assert!(!wlan.is_power_on());
        assert!(!wlan.is_connected());
    }

    #[test]
    fn test_power_off_clears_state() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.set_ssid("TestNetwork");
        wlan.connect();
        assert!(wlan.is_connected());

        wlan.power_off();

        assert!(!wlan.is_enabled());
        assert!(!wlan.is_connected());
        assert!(wlan.ssid().is_empty());
    }

    #[test]
    fn test_rssi_signal_strength() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.set_ssid("TestNetwork");
        wlan.connect();

        // Should have a valid RSSI
        assert!(wlan.current_ap().rssi < 0);
        assert!(wlan.current_ap().rssi > -100);
    }

    #[test]
    fn test_channel_info() {
        let mut wlan = WlanStub::new();

        wlan.power_on();
        wlan.enable();
        wlan.set_ssid("TestNetwork");
        wlan.connect();

        let channel = &wlan.current_ap().channel;
        assert!(channel.number >= 1 && channel.number <= 14);
        // 2.4GHz band: channels 1-14
        assert!(channel.frequency >= 2412 && channel.frequency <= 2484);
    }
}
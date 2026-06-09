//! Clock domains.

use rp2350sim_core::ClockDomain;

/// Clock domains for RP2350.
#[derive(Debug)]
pub struct ClockDomains {
    pub system: ClockDomain,
    pub peripheral: ClockDomain,
    pub usb: ClockDomain,
    pub adc: ClockDomain,
    pub rtc: ClockDomain,
}

impl Default for ClockDomains {
    fn default() -> Self {
        Self::new()
    }
}

impl ClockDomains {
    pub fn new() -> Self {
        Self {
            system: ClockDomain::new(rp2350sim_core::ClockDomainId::SYSTEM, 150_000_000),
            peripheral: ClockDomain::new(rp2350sim_core::ClockDomainId::PERIPHERAL, 150_000_000),
            usb: ClockDomain::new(rp2350sim_core::ClockDomainId::USB, 48_000_000),
            adc: ClockDomain::new(rp2350sim_core::ClockDomainId::ADC, 48_000_000),
            rtc: ClockDomain::new(rp2350sim_core::ClockDomainId::RTC, 32_768),
        }
    }

    pub fn reset(&mut self) {
        self.system.reset();
        self.peripheral.reset();
        self.usb.reset();
        self.adc.reset();
        self.rtc.reset();
    }
}
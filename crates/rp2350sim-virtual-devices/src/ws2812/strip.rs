//! WS2812 LED strip emulation.

/// A single LED color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }

    pub fn red() -> Self {
        Self::new(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::new(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self::new(0, 0, 255)
    }
}

/// WS2812 LED strip.
#[derive(Debug)]
pub struct Ws2812Strip {
    /// Number of LEDs.
    count: usize,
    /// LED colors.
    leds: Vec<Rgb>,
    /// Receive buffer.
    buffer: Vec<u8>,
    /// Bit position in current byte.
    #[allow(dead_code)]
    bit_pos: usize,
    /// Current LED index being received.
    led_index: usize,
    /// Current color byte index (0=R, 1=G, 2=B).
    color_index: usize,
}

impl Ws2812Strip {
    /// Create a new LED strip.
    pub fn new(count: usize) -> Self {
        Self {
            count,
            leds: vec![Rgb::black(); count],
            buffer: Vec::new(),
            bit_pos: 0,
            led_index: 0,
            color_index: 0,
        }
    }

    /// Get the number of LEDs.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the strip is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get all LED colors.
    pub fn leds(&self) -> &[Rgb] {
        &self.leds
    }

    /// Get a single LED color.
    pub fn get(&self, index: usize) -> Option<Rgb> {
        self.leds.get(index).copied()
    }

    /// Set a single LED color.
    pub fn set(&mut self, index: usize, color: Rgb) {
        if index < self.count {
            self.leds[index] = color;
        }
    }

    /// Clear all LEDs.
    pub fn clear(&mut self) {
        for led in &mut self.leds {
            *led = Rgb::black();
        }
    }

    /// Process a bit from the WS2812 protocol.
    /// WS2812 uses a specific timing pattern:
    /// - 0 bit: ~400ns high, ~850ns low
    /// - 1 bit: ~800ns high, ~450ns low
    /// For simulation, we just track the bit value.
    pub fn process_bit(&mut self, bit: bool) {
        // Accumulate bits into bytes
        // WS2812 sends GRB format
        self.buffer.push(if bit { 1 } else { 0 });
        
        if self.buffer.len() >= 8 {
            let byte = self.buffer.iter().fold(0u8, |acc, &b| (acc << 1) | b);
            self.buffer.clear();
            
            // Process the byte
            match self.color_index {
                0 => self.leds[self.led_index].g = byte, // Green first
                1 => self.leds[self.led_index].r = byte, // Red second
                2 => {
                    self.leds[self.led_index].b = byte; // Blue third
                    self.led_index = (self.led_index + 1) % self.count;
                }
                _ => {}
            }
            self.color_index = (self.color_index + 1) % 3;
        }
    }

    /// Reset the receive state (called after a reset period).
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.led_index = 0;
        self.color_index = 0;
    }
}
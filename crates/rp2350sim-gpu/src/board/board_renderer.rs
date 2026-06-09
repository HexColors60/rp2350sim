//! Board renderer.
#![allow(dead_code)]

use crate::board::pins::PinRenderer;
use crate::board::overlays::OverlayRenderer;

/// Renders the board visualization.
#[derive(Debug)]
pub struct BoardRenderer {
    /// Board width in pixels.
    width: f32,
    /// Board height in pixels.
    height: f32,
    /// Pin renderer.
    pin_renderer: PinRenderer,
    /// Overlay renderer.
    overlay_renderer: OverlayRenderer,
    /// Board background color.
    bg_color: [f32; 4],
    /// Pin data (30 pins for Pico 2 W).
    pins: [PinState; 40],
}

/// Pin state for rendering.
#[derive(Debug, Clone, Copy)]
pub struct PinState {
    /// Pin number.
    pub number: u8,
    /// Pin value (high/low).
    pub value: bool,
    /// Pin direction (input/output).
    pub output: bool,
    /// Pin function.
    pub function: u8,
    /// Pin position x.
    pub x: f32,
    /// Pin position y.
    pub y: f32,
}

impl Default for PinState {
    fn default() -> Self {
        Self {
            number: 0,
            value: false,
            output: false,
            function: 0,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl Default for BoardRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardRenderer {
    /// Create a new board renderer.
    pub fn new() -> Self {
        let mut pins = [PinState::default(); 40];
        
        // Initialize pin positions (20 pins on each side)
        let board_width = 200.0;
        let board_height = 400.0;
        let pin_spacing = 18.0;
        let pin_offset_x = 10.0;
        let pin_offset_y = 30.0;
        
        // Left side pins (0-19)
        for i in 0..20 {
            pins[i] = PinState {
                number: i as u8,
                value: false,
                output: false,
                function: 0,
                x: pin_offset_x,
                y: pin_offset_y + i as f32 * pin_spacing,
            };
        }
        
        // Right side pins (20-39)
        for i in 0..20 {
            pins[20 + i] = PinState {
                number: (20 + i) as u8,
                value: false,
                output: false,
                function: 0,
                x: board_width - pin_offset_x,
                y: pin_offset_y + i as f32 * pin_spacing,
            };
        }
        
        Self {
            width: board_width,
            height: board_height,
            pin_renderer: PinRenderer::new(),
            overlay_renderer: OverlayRenderer::new(),
            bg_color: [0.1, 0.15, 0.2, 1.0], // Dark blue-green
            pins,
        }
    }

    /// Render the board.
    pub fn render(&self) {
        self.render_background();
        self.render_outline();
        
        // Render pins
        for pin in &self.pins {
            self.pin_renderer.render_pin(pin);
        }
        
        // Render overlays (LEDs, buttons, etc.)
        self.overlay_renderer.render();
    }

    /// Render board background.
    fn render_background(&self) {
        // Draw the PCB-like background
        let x = 0.0;
        let y = 0.0;
        let w = self.width;
        let h = self.height;
        
        // Use macroquad's drawing functions if available
        #[cfg(feature = "macroquad")]
        {
            use macroquad::prelude::*;
            draw_rectangle(x, y, w, h, Color::from(self.bg_color));
            
            // Draw some PCB traces pattern
            let trace_color = Color::from_rgba(40, 60, 80, 255);
            for i in 0..10 {
                let trace_y = 50.0 + i as f32 * 35.0;
                draw_line(20.0, trace_y, w - 20.0, trace_y, 1.0, trace_color);
            }
        }
        
        // Placeholder for non-macroquad builds
        let _ = (x, y, w, h);
    }

    /// Render board outline.
    fn render_outline(&self) {
        #[cfg(feature = "macroquad")]
        {
            use macroquad::prelude::*;
            let outline_color = Color::from_rgba(60, 90, 120, 255);
            
            // Draw rounded rectangle outline
            draw_rectangle_lines(0.0, 0.0, self.width, self.height, 2.0, outline_color);
            
            // Draw USB connector
            draw_rectangle(self.width / 2.0 - 15.0, -10.0, 30.0, 15.0, 
                Color::from_rgba(80, 80, 80, 255));
            
            // Draw reset button
            draw_circle(self.width - 20.0, 15.0, 5.0, Color::from_rgba(50, 50, 50, 255));
            
            // Draw BOOTSEL button
            draw_circle(self.width - 20.0, 30.0, 5.0, Color::from_rgba(50, 50, 50, 255));
        }
    }

    /// Get board width.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Get board height.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Resize the board view.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
    
    /// Update pin state.
    pub fn update_pin(&mut self, pin_num: u8, value: bool, output: bool, function: u8) {
        if (pin_num as usize) < self.pins.len() {
            self.pins[pin_num as usize].value = value;
            self.pins[pin_num as usize].output = output;
            self.pins[pin_num as usize].function = function;
        }
    }
    
    /// Get pin states.
    pub fn get_pins(&self) -> &[PinState] {
        &self.pins
    }
}
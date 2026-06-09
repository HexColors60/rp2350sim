//! Overlay renderer for LEDs, buttons, etc.

/// Renders overlays on the board (LEDs, buttons, etc.).
#[derive(Debug)]
pub struct OverlayRenderer {
    /// LED states.
    leds: Vec<LedOverlay>,
    /// Button states.
    buttons: Vec<ButtonOverlay>,
}

/// LED overlay.
#[derive(Debug, Clone)]
pub struct LedOverlay {
    /// LED ID.
    pub id: u8,
    /// Position x.
    pub x: f32,
    /// Position y.
    pub y: f32,
    /// LED state (on/off).
    pub on: bool,
    /// LED color when on.
    pub color: [f32; 4],
}

/// Button overlay.
#[derive(Debug, Clone)]
pub struct ButtonOverlay {
    /// Button ID.
    pub id: u8,
    /// Position x.
    pub x: f32,
    /// Position y.
    pub y: f32,
    /// Button width.
    pub width: f32,
    /// Button height.
    pub height: f32,
    /// Button pressed state.
    pub pressed: bool,
    /// Button label.
    pub label: String,
}

impl Default for OverlayRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayRenderer {
    pub fn new() -> Self {
        Self {
            leds: Vec::new(),
            buttons: Vec::new(),
        }
    }

    /// Add an LED overlay.
    pub fn add_led(&mut self, id: u8, x: f32, y: f32, color: [f32; 4]) {
        self.leds.push(LedOverlay {
            id,
            x,
            y,
            on: false,
            color,
        });
    }

    /// Set LED state.
    pub fn set_led(&mut self, id: u8, on: bool) {
        if let Some(led) = self.leds.iter_mut().find(|l| l.id == id) {
            led.on = on;
        }
    }

    /// Add a button overlay.
    pub fn add_button(&mut self, id: u8, x: f32, y: f32, width: f32, height: f32, label: &str) {
        self.buttons.push(ButtonOverlay {
            id,
            x,
            y,
            width,
            height,
            pressed: false,
            label: label.to_string(),
        });
    }

    /// Set button pressed state.
    pub fn set_button(&mut self, id: u8, pressed: bool) {
        if let Some(button) = self.buttons.iter_mut().find(|b| b.id == id) {
            button.pressed = pressed;
        }
    }

    /// Check if a point is over a button.
    pub fn hit_test_button(&self, x: f32, y: f32) -> Option<u8> {
        for button in &self.buttons {
            if x >= button.x && x <= button.x + button.width &&
               y >= button.y && y <= button.y + button.height {
                return Some(button.id);
            }
        }
        None
    }

    /// Render all overlays.
    pub fn render(&self) {
        // Render LEDs
        for led in &self.leds {
            self.render_led(led);
        }

        // Render buttons
        for button in &self.buttons {
            self.render_button(button);
        }
    }

    /// Render an LED.
    fn render_led(&self, led: &LedOverlay) {
        #[cfg(feature = "macroquad")]
        {
            use macroquad::prelude::*;
            
            let color = if led.on {
                Color::from(led.color)
            } else {
                Color::from_rgba(50, 50, 50, 255) // Dark gray when off
            };
            
            // Draw LED body
            draw_circle(led.x, led.y, 5.0, color);
            
            // Draw glow effect when on
            if led.on {
                let glow_color = Color::from_rgba(
                    (led.color[0] * 255.0) as u8,
                    (led.color[1] * 255.0) as u8,
                    (led.color[2] * 255.0) as u8,
                    50, // Semi-transparent
                );
                draw_circle(led.x, led.y, 8.0, glow_color);
            }
        }
        
        // Placeholder for non-macroquad builds
        let _ = led;
    }

    /// Render a button.
    fn render_button(&self, button: &ButtonOverlay) {
        #[cfg(feature = "macroquad")]
        {
            use macroquad::prelude::*;
            
            let bg_color = if button.pressed {
                Color::from_rgba(100, 100, 120, 255)
            } else {
                Color::from_rgba(70, 70, 85, 255)
            };
            
            // Draw button background
            draw_rectangle(button.x, button.y, button.width, button.height, bg_color);
            
            // Draw border
            let border_color = Color::from_rgba(90, 90, 110, 255);
            draw_rectangle_lines(button.x, button.y, button.width, button.height, 1.0, border_color);
            
            // Draw label
            let text_color = Color::from_rgba(200, 200, 220, 255);
            let font_size = (button.height * 0.6) as u16;
            let text_dims = measure_text(&button.label, None, font_size, 1.0);
            let text_x = button.x + (button.width - text_dims.width) / 2.0;
            let text_y = button.y + (button.height + text_dims.height) / 2.0;
            draw_text(&button.label, text_x, text_y, font_size as f32, text_color);
        }
        
        // Placeholder for non-macroquad builds
        let _ = button;
    }
}
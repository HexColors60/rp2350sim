//! UART terminal emulation.

use std::collections::VecDeque;

/// UART terminal for displaying text output.
#[derive(Debug)]
pub struct UartTerminal {
    /// Terminal width in columns.
    width: u32,
    /// Terminal height in rows.
    height: u32,
    /// Screen buffer (characters).
    screen: Vec<Vec<char>>,
    /// Current cursor position.
    cursor_x: u32,
    cursor_y: u32,
    /// Input buffer.
    input_buffer: VecDeque<u8>,
    /// Output buffer.
    output_buffer: VecDeque<u8>,
}

impl UartTerminal {
    /// Create a new UART terminal.
    pub fn new(width: u32, height: u32) -> Self {
        let screen = vec![vec![' '; width as usize]; height as usize];
        Self {
            width,
            height,
            screen,
            cursor_x: 0,
            cursor_y: 0,
            input_buffer: VecDeque::new(),
            output_buffer: VecDeque::new(),
        }
    }

    /// Create a standard 80x24 terminal.
    pub fn standard() -> Self {
        Self::new(80, 24)
    }

    /// Write a byte to the terminal.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\r' => {
                self.cursor_x = 0;
            }
            b'\n' => {
                self.cursor_y += 1;
                if self.cursor_y >= self.height {
                    self.scroll_up();
                    self.cursor_y = self.height - 1;
                }
            }
            0x08 => {  // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            0x20..=0x7E => {
                if self.cursor_x < self.width && self.cursor_y < self.height {
                    self.screen[self.cursor_y as usize][self.cursor_x as usize] = byte as char;
                    self.cursor_x += 1;
                    if self.cursor_x >= self.width {
                        self.cursor_x = 0;
                        self.cursor_y += 1;
                        if self.cursor_y >= self.height {
                            self.scroll_up();
                            self.cursor_y = self.height - 1;
                        }
                    }
                }
            }
            _ => {}
        }
        self.output_buffer.push_back(byte);
    }

    /// Write a string to the terminal.
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    /// Push a byte to the input buffer.
    pub fn push_input(&mut self, byte: u8) {
        self.input_buffer.push_back(byte);
    }

    /// Read a byte from the input buffer.
    pub fn read_input(&mut self) -> Option<u8> {
        self.input_buffer.pop_front()
    }

    /// Get the screen content.
    pub fn screen(&self) -> &[Vec<char>] {
        &self.screen
    }

    /// Get the screen as a string.
    pub fn screen_string(&self) -> String {
        self.screen.iter().map(|row| row.iter().collect::<String>()).collect::<Vec<_>>().join("\n")
    }

    /// Clear the terminal.
    pub fn clear(&mut self) {
        for row in &mut self.screen {
            for c in row.iter_mut() {
                *c = ' ';
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Scroll the screen up by one line.
    fn scroll_up(&mut self) {
        self.screen.remove(0);
        self.screen.push(vec![' '; self.width as usize]);
    }
}
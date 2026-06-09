//! GPIO API for scripting.

use std::cell::RefCell;
use std::rc::Rc;

/// GPIO control trait.
pub trait GpioControl: Send + Sync {
    /// Set a GPIO pin value.
    fn set_pin(&mut self, pin: u8, value: bool);
    /// Get a GPIO pin value.
    fn get_pin(&self, pin: u8) -> bool;
    /// Set GPIO direction.
    fn set_direction(&mut self, pin: u8, output: bool);
    /// Get GPIO direction.
    fn get_direction(&self, pin: u8) -> bool;
    /// Set GPIO function.
    fn set_function(&mut self, pin: u8, function: u8);
    /// Get GPIO function.
    fn get_function(&self, pin: u8) -> u8;
    /// Toggle a GPIO pin.
    fn toggle_pin(&mut self, pin: u8);
}

/// GPIO API for Rhai scripting.
pub struct GpioApi {
    control: Option<Rc<RefCell<dyn GpioControl>>>,
}

impl GpioApi {
    /// Create a new GPIO API.
    pub fn new() -> Self {
        Self { control: None }
    }

    /// Create with a control reference.
    pub fn with_control(control: Rc<RefCell<dyn GpioControl>>) -> Self {
        Self { control: Some(control) }
    }

    /// Set the control reference.
    pub fn set_control(&mut self, control: Rc<RefCell<dyn GpioControl>>) {
        self.control = Some(control);
    }

    /// Set a GPIO pin.
    pub fn set_pin(&mut self, pin: i64, value: bool) {
        if let Some(ref control) = self.control {
            control.borrow_mut().set_pin(pin as u8, value);
        }
    }

    /// Get a GPIO pin.
    pub fn get_pin(&self, pin: i64) -> bool {
        if let Some(ref control) = self.control {
            control.borrow().get_pin(pin as u8)
        } else {
            false
        }
    }

    /// Set GPIO direction.
    pub fn set_direction(&mut self, pin: i64, output: bool) {
        if let Some(ref control) = self.control {
            control.borrow_mut().set_direction(pin as u8, output);
        }
    }

    /// Get GPIO direction.
    pub fn get_direction(&self, pin: i64) -> bool {
        if let Some(ref control) = self.control {
            control.borrow().get_direction(pin as u8)
        } else {
            false
        }
    }

    /// Set GPIO function.
    pub fn set_function(&mut self, pin: i64, function: i64) {
        if let Some(ref control) = self.control {
            control.borrow_mut().set_function(pin as u8, function as u8);
        }
    }

    /// Get GPIO function.
    pub fn get_function(&self, pin: i64) -> i64 {
        if let Some(ref control) = self.control {
            control.borrow().get_function(pin as u8) as i64
        } else {
            0
        }
    }

    /// Toggle a GPIO pin.
    pub fn toggle_pin(&mut self, pin: i64) {
        if let Some(ref control) = self.control {
            control.borrow_mut().toggle_pin(pin as u8);
        }
    }
}

impl Default for GpioApi {
    fn default() -> Self {
        Self::new()
    }
}
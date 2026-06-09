//! Simulator API for scripting.

use std::cell::RefCell;
use std::rc::Rc;

/// Simulator control trait.
pub trait SimulatorControl: Send + Sync {
    /// Reset the simulator.
    fn reset(&mut self);
    /// Step the simulator one instruction.
    fn step(&mut self);
    /// Run the simulator.
    fn run(&mut self);
    /// Stop the simulator.
    fn stop(&mut self);
    /// Check if the simulator is running.
    fn is_running(&self) -> bool;
    /// Get the current PC.
    fn pc(&self) -> u32;
    /// Set the PC.
    fn set_pc(&mut self, value: u32);
    /// Get cycle count.
    fn cycles(&self) -> u64;
    /// Get instruction count.
    fn instructions(&self) -> u64;
}

/// Simulator API for Rhai scripting.
pub struct SimulatorApi {
    control: Option<Rc<RefCell<dyn SimulatorControl>>>,
}

impl SimulatorApi {
    /// Create a new simulator API.
    pub fn new() -> Self {
        Self { control: None }
    }

    /// Create with a control reference.
    pub fn with_control(control: Rc<RefCell<dyn SimulatorControl>>) -> Self {
        Self { control: Some(control) }
    }

    /// Set the control reference.
    pub fn set_control(&mut self, control: Rc<RefCell<dyn SimulatorControl>>) {
        self.control = Some(control);
    }

    /// Reset the simulator.
    pub fn reset(&mut self) {
        if let Some(ref control) = self.control {
            control.borrow_mut().reset();
        }
    }

    /// Step the simulator.
    pub fn step(&mut self) {
        if let Some(ref control) = self.control {
            control.borrow_mut().step();
        }
    }

    /// Run the simulator.
    pub fn run(&mut self) {
        if let Some(ref control) = self.control {
            control.borrow_mut().run();
        }
    }

    /// Stop the simulator.
    pub fn stop(&mut self) {
        if let Some(ref control) = self.control {
            control.borrow_mut().stop();
        }
    }

    /// Check if running.
    pub fn is_running(&self) -> bool {
        self.control.as_ref().map(|c| c.borrow().is_running()).unwrap_or(false)
    }

    /// Get PC.
    pub fn pc(&self) -> i64 {
        self.control.as_ref().map(|c| c.borrow().pc() as i64).unwrap_or(0)
    }

    /// Set PC.
    pub fn set_pc(&mut self, value: i64) {
        if let Some(ref control) = self.control {
            control.borrow_mut().set_pc(value as u32);
        }
    }

    /// Get cycles.
    pub fn cycles(&self) -> i64 {
        self.control.as_ref().map(|c| c.borrow().cycles() as i64).unwrap_or(0)
    }

    /// Get instruction count.
    pub fn instructions(&self) -> i64 {
        self.control.as_ref().map(|c| c.borrow().instructions() as i64).unwrap_or(0)
    }
}

impl Default for SimulatorApi {
    fn default() -> Self {
        Self::new()
    }
}
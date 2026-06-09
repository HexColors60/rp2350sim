//! Step control.


/// Step mode for CPU execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StepMode {
    /// Run continuously
    #[default]
    Run,
    /// Single step
    Step,
    /// Step over (skip calls)
    StepOver,
    /// Step out (run until return)
    StepOut,
    /// Run until breakpoint
    RunUntilBreak,
    /// Run until address
    RunUntilAddress(u32),
}

/// Step controller.
#[derive(Debug, Clone)]
pub struct StepController {
    mode: StepMode,
    step_count: u64,
    target_address: Option<u32>,
    call_depth: usize,
}

impl Default for StepController {
    fn default() -> Self {
        Self::new()
    }
}

impl StepController {
    pub fn new() -> Self {
        Self {
            mode: StepMode::Run,
            step_count: 0,
            target_address: None,
            call_depth: 0,
        }
    }

    pub fn set_mode(&mut self, mode: StepMode) {
        self.mode = mode;
        self.step_count = 0;
        if let StepMode::RunUntilAddress(addr) = mode {
            self.target_address = Some(addr);
        }
    }

    pub fn mode(&self) -> StepMode {
        self.mode.clone()
    }

    pub fn should_stop(&mut self, pc: u32, is_call: bool, is_return: bool) -> bool {
        match self.mode {
            StepMode::Run => false,
            StepMode::Step => {
                self.step_count += 1;
                self.step_count >= 1
            }
            StepMode::StepOver => {
                if is_call {
                    self.call_depth += 1;
                    false
                } else if is_return && self.call_depth > 0 {
                    self.call_depth -= 1;
                    false
                } else {
                    self.call_depth == 0
                }
            }
            StepMode::StepOut => {
                if is_call {
                    self.call_depth += 1;
                }
                if is_return && self.call_depth > 0 {
                    self.call_depth -= 1;
                }
                is_return && self.call_depth == 0
            }
            StepMode::RunUntilBreak => false,
            StepMode::RunUntilAddress(addr) => pc == addr,
        }
    }

    pub fn reset(&mut self) {
        self.mode = StepMode::Run;
        self.step_count = 0;
        self.target_address = None;
        self.call_depth = 0;
    }
}
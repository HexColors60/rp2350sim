//! Timer implementation.


/// Machine timer.
#[derive(Debug, Clone, Default)]
pub struct MachineTimer {
    /// mtime register
    mtime: u64,
    /// mtimecmp register
    mtimecmp: u64,
}

impl MachineTimer {
    pub fn new() -> Self {
        Self {
            mtime: 0,
            mtimecmp: u64::MAX,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.mtime = self.mtime.wrapping_add(1);
        self.mtime >= self.mtimecmp
    }

    pub fn read_mtime(&self) -> u64 {
        self.mtime
    }

    pub fn write_mtime(&mut self, value: u64) {
        self.mtime = value;
    }

    pub fn read_mtimecmp(&self) -> u64 {
        self.mtimecmp
    }

    pub fn write_mtimecmp(&mut self, value: u64) {
        self.mtimecmp = value;
    }

    pub fn reset(&mut self) {
        self.mtime = 0;
        self.mtimecmp = u64::MAX;
    }
}
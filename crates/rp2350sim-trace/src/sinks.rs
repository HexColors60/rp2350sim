#![allow(dead_code)]
//! Trace sinks.

use rp2350sim_core::Result;

/// Trace sink trait.
pub trait TraceSink: Send + Sync {
    fn write(&mut self, data: &[u8]) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

/// File trace sink.
pub struct FileSink {
    file: std::fs::File,
}

impl FileSink {
    pub fn new(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            file: std::fs::File::create(path)?,
        })
    }
}

impl TraceSink for FileSink {
    fn write(&mut self, data: &[u8]) -> Result<()> {
        use std::io::Write;
        self.file.write_all(data)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        use std::io::Write;
        self.file.flush()?;
        Ok(())
    }
}
//! Scripting engine.

use rhai::Engine;

/// Scripting engine.
pub struct ScriptingEngine {
    engine: Engine,
}

impl Default for ScriptingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptingEngine {
    /// Create a new scripting engine.
    pub fn new() -> Self {
        let engine = Engine::new();
        Self { engine }
    }

    /// Run a script.
    pub fn run(&mut self, script: &str) -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        self.engine.eval(script)
    }
}
//! GPU renderer.

use crate::board::BoardRenderer;
use crate::timeline::Timeline;
use crate::waveforms::WaveformRenderer;
use crate::heatmap::MemoryHeatmap;

/// GPU renderer.
#[derive(Debug)]
pub struct Renderer {
    initialized: bool,
    board_renderer: Option<BoardRenderer>,
    timeline: Option<Timeline>,
    waveform_renderer: Option<WaveformRenderer>,
    memory_heatmap: Option<MemoryHeatmap>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            initialized: false,
            board_renderer: None,
            timeline: None,
            waveform_renderer: None,
            memory_heatmap: None,
        }
    }

    pub fn init(&mut self) {
        self.initialized = true;
    }

    /// Initialize board renderer.
    pub fn init_board(&mut self, width: f32, height: f32) {
        self.board_renderer = Some(BoardRenderer::new(width, height));
    }

    /// Initialize timeline.
    pub fn init_timeline(&mut self, width: f32, height: f32) {
        self.timeline = Some(Timeline::new(width, height));
    }

    /// Initialize waveform renderer.
    pub fn init_waveforms(&mut self, width: f32, height: f32) {
        self.waveform_renderer = Some(WaveformRenderer::new(width, height));
    }

    /// Initialize memory heatmap.
    pub fn init_heatmap(&mut self, width: f32, height: f32) {
        self.memory_heatmap = Some(MemoryHeatmap::new(width, height));
    }

    /// Render all components.
    pub fn render(&mut self) {
        if !self.initialized {
            return;
        }

        // Render board
        if let Some(ref board) = self.board_renderer {
            board.render();
        }

        // Render timeline
        if let Some(ref timeline) = self.timeline {
            timeline.render();
        }

        // Render waveforms
        if let Some(ref waveforms) = self.waveform_renderer {
            waveforms.render();
        }

        // Render heatmap
        if let Some(ref heatmap) = self.memory_heatmap {
            heatmap.render();
        }
    }

    /// Get board renderer.
    pub fn board(&self) -> Option<&BoardRenderer> {
        self.board_renderer.as_ref()
    }

    /// Get board renderer (mutable).
    pub fn board_mut(&mut self) -> Option<&mut BoardRenderer> {
        self.board_renderer.as_mut()
    }

    /// Get timeline.
    pub fn timeline(&self) -> Option<&Timeline> {
        self.timeline.as_ref()
    }

    /// Get timeline (mutable).
    pub fn timeline_mut(&mut self) -> Option<&mut Timeline> {
        self.timeline.as_mut()
    }

    /// Get waveform renderer.
    pub fn waveforms(&self) -> Option<&WaveformRenderer> {
        self.waveform_renderer.as_ref()
    }

    /// Get waveform renderer (mutable).
    pub fn waveforms_mut(&mut self) -> Option<&mut WaveformRenderer> {
        self.waveform_renderer.as_mut()
    }

    /// Check if initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
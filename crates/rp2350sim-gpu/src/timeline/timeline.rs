//! Timeline visualization.

use super::lanes::TimelineLanes;
use super::markers::TimelineMarkers;

/// Timeline renderer.
#[derive(Debug)]
pub struct Timeline {
    /// Timeline width.
    width: f32,
    /// Timeline height.
    height: f32,
    /// Zoom level.
    zoom: f32,
    /// Scroll offset.
    scroll_x: f32,
    /// Current time position.
    current_time: u64,
    /// Total time range.
    total_time: u64,
    /// Lanes.
    lanes: TimelineLanes,
    /// Markers.
    markers: TimelineMarkers,
    /// Time scale (pixels per unit time).
    time_scale: f32,
}

impl Timeline {
    /// Create a new timeline.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            zoom: 1.0,
            scroll_x: 0.0,
            current_time: 0,
            total_time: 1_000_000, // 1M time units default
            lanes: TimelineLanes::new(),
            markers: TimelineMarkers::new(),
            time_scale: 1.0,
        }
    }

    /// Set zoom level.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 100.0);
        self.update_time_scale();
    }

    /// Set scroll offset.
    pub fn set_scroll(&mut self, scroll_x: f32) {
        self.scroll_x = scroll_x.max(0.0);
    }

    /// Set current time position.
    pub fn set_current_time(&mut self, time: u64) {
        self.current_time = time;
    }

    /// Set total time range.
    pub fn set_total_time(&mut self, time: u64) {
        self.total_time = time.max(1);
        self.update_time_scale();
    }

    /// Update time scale based on zoom and total time.
    fn update_time_scale(&mut self) {
        self.time_scale = (self.width * self.zoom) / self.total_time as f32;
    }

    /// Convert time to x position.
    pub fn time_to_x(&self, time: u64) -> f32 {
        (time as f32 * self.time_scale) - self.scroll_x
    }

    /// Convert x position to time.
    pub fn x_to_time(&self, x: f32) -> u64 {
        ((x + self.scroll_x) / self.time_scale) as u64
    }

    /// Add a marker at the specified time.
    pub fn add_marker(&mut self, time: u64, label: &str, color: [f32; 4]) {
        self.markers.add(time, label, color);
    }

    /// Add a lane.
    pub fn add_lane(&mut self, name: &str) -> usize {
        self.lanes.add(name)
    }

    /// Add an event to a lane.
    pub fn add_event(&mut self, lane: usize, start_time: u64, end_time: u64, value: u64, color: [f32; 4]) {
        self.lanes.add_event(lane, start_time, end_time, value, color);
    }

    /// Render the timeline.
    pub fn render(&self) {
        // Render background
        self.render_background();
        
        // Render time grid
        self.render_time_grid();
        
        // Render lanes
        self.lanes.render(self.width, self.height, self.time_scale, self.scroll_x);
        
        // Render markers
        self.markers.render(self.time_scale, self.scroll_x, self.height);
        
        // Render current time indicator
        self.render_current_time();
    }

    /// Render background.
    fn render_background(&self) {
        // Background rendering
    }

    /// Render time grid.
    fn render_time_grid(&self) {
        // Calculate grid spacing based on zoom
        let grid_spacing = self.calculate_grid_spacing();
        
        // Draw vertical lines at grid intervals
        let start_time = self.x_to_time(0.0);
        let first_grid = (start_time / grid_spacing + 1) * grid_spacing;
        
        let mut time = first_grid;
        while self.time_to_x(time) < self.width {
            let x = self.time_to_x(time);
            // Draw grid line at x
            let _ = x;
            time += grid_spacing;
        }
    }

    /// Calculate appropriate grid spacing.
    fn calculate_grid_spacing(&self) -> u64 {
        let pixels_per_second = self.time_scale;
        if pixels_per_second > 1000.0 {
            1_000 // 1ms
        } else if pixels_per_second > 100.0 {
            10_000 // 10ms
        } else if pixels_per_second > 10.0 {
            100_000 // 100ms
        } else {
            1_000_000 // 1s
        }
    }

    /// Render current time indicator.
    fn render_current_time(&self) {
        let x = self.time_to_x(self.current_time);
        if x >= 0.0 && x < self.width {
            // Draw vertical line at current time
            let _ = x;
        }
    }

    /// Handle mouse wheel for zooming.
    pub fn handle_zoom(&mut self, delta: f32, mouse_x: f32) {
        let old_zoom = self.zoom;
        self.set_zoom(self.zoom * (1.0 + delta * 0.1));
        
        // Adjust scroll to keep mouse position fixed
        let zoom_ratio = self.zoom / old_zoom;
        self.scroll_x = mouse_x * zoom_ratio - (mouse_x + self.scroll_x) + self.scroll_x;
        self.scroll_x = self.scroll_x.max(0.0);
    }

    /// Handle mouse drag for scrolling.
    pub fn handle_drag(&mut self, delta_x: f32) {
        self.scroll_x = (self.scroll_x - delta_x).max(0.0);
    }
}
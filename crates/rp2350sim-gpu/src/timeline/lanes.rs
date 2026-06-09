#![allow(dead_code)]

//! Timeline lanes.

/// A single timeline lane.
#[derive(Debug, Clone)]
pub struct Lane {
    /// Lane name.
    pub name: String,
    /// Lane height.
    pub height: f32,
    /// Events in this lane.
    pub events: Vec<LaneEvent>,
}

/// An event in a lane.
#[derive(Debug, Clone)]
pub struct LaneEvent {
    /// Start time.
    pub start_time: u64,
    /// End time.
    pub end_time: u64,
    /// Value.
    pub value: u64,
    /// Color.
    pub color: [f32; 4],
}

/// Timeline lanes for displaying multiple signals.
#[derive(Debug, Default)]
pub struct TimelineLanes {
    /// Lanes.
    lanes: Vec<Lane>,
    /// Default lane height.
    default_height: f32,
}

impl TimelineLanes {
    /// Create new timeline lanes.
    pub fn new() -> Self {
        Self {
            lanes: Vec::new(),
            default_height: 24.0,
        }
    }

    /// Add a lane.
    pub fn add(&mut self, name: &str) -> usize {
        let idx = self.lanes.len();
        self.lanes.push(Lane {
            name: name.to_string(),
            height: self.default_height,
            events: Vec::new(),
        });
        idx
    }

    /// Add an event to a lane.
    pub fn add_event(&mut self, lane: usize, start_time: u64, end_time: u64, value: u64, color: [f32; 4]) {
        if let Some(lane) = self.lanes.get_mut(lane) {
            lane.events.push(LaneEvent {
                start_time,
                end_time,
                value,
                color,
            });
        }
    }

    /// Get the number of lanes.
    pub fn count(&self) -> usize {
        self.lanes.len()
    }

    /// Get the total height.
    pub fn total_height(&self) -> f32 {
        self.lanes.iter().map(|l| l.height).sum()
    }

    /// Get a lane by index.
    pub fn get(&self, index: usize) -> Option<&Lane> {
        self.lanes.get(index)
    }

    /// Get a lane by index (mutable).
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Lane> {
        self.lanes.get_mut(index)
    }

    /// Clear all events from all lanes.
    pub fn clear_events(&mut self) {
        for lane in &mut self.lanes {
            lane.events.clear();
        }
    }

    /// Render all lanes.
    pub fn render(&self, width: f32, _height: f32, time_scale: f32, scroll_x: f32) {
        let mut y = 0.0;
        
        for lane in &self.lanes {
            self.render_lane(lane, y, width, time_scale, scroll_x);
            y += lane.height;
        }
    }

    /// Render a single lane.
    fn render_lane(&self, lane: &Lane, y: f32, width: f32, time_scale: f32, scroll_x: f32) {
        // Render lane background
        let _ = (y, width);
        
        // Render events
        for event in &lane.events {
            let x = (event.start_time as f32 * time_scale) - scroll_x;
            let event_width = (event.end_time - event.start_time) as f32 * time_scale;
            
            // Only render if visible
            if x + event_width >= 0.0 && x < width {
                self.render_event(event, x, y, event_width, lane.height);
            }
        }
        
        // Render lane name
        let _ = &lane.name;
    }

    /// Render an event.
    fn render_event(&self, event: &LaneEvent, x: f32, y: f32, width: f32, height: f32) {
        // Event rendering would use macroquad or wgpu
        // For now, this is a placeholder
        let _ = (event.value, event.color, x, y, width, height);
    }
}
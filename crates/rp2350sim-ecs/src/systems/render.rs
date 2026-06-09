//! Render system.

use hecs::World;
use crate::components::{Position, Name, Renderable};

/// Render system for rendering entities.
pub struct RenderSystem {
    /// Camera offset X.
    camera_x: f32,
    /// Camera offset Y.
    camera_y: f32,
    /// Zoom level.
    zoom: f32,
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderSystem {
    /// Create a new render system.
    pub fn new() -> Self {
        Self {
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
        }
    }

    /// Set camera position.
    pub fn set_camera(&mut self, x: f32, y: f32) {
        self.camera_x = x;
        self.camera_y = y;
    }

    /// Set zoom level.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    /// Render all entities.
    pub fn render(&self, world: &World) {
        // Render entities with position and renderable components
        for (_, (pos, renderable, name)) in world.query::<(&Position, &Renderable, Option<&Name>)>().iter() {
            if !renderable.visible {
                continue;
            }
            
            let screen_x = (pos.x - self.camera_x) * self.zoom;
            let screen_y = (pos.y - self.camera_y) * self.zoom;

            // Render based on size
            self.render_entity(screen_x, screen_y, renderable, name);
        }
    }

    /// Render a single entity.
    fn render_entity(&self, x: f32, y: f32, renderable: &Renderable, name: Option<&Name>) {
        #[cfg(feature = "macroquad")]
        {
            use macroquad::prelude::*;
            
            let w = (renderable.width as f32) * self.zoom;
            let h = (renderable.height as f32) * self.zoom;
            
            // Choose color based on layer
            let color = match renderable.layer {
                0 => Color::from_rgba(100, 100, 120, 255), // Background
                1 => Color::from_rgba(80, 180, 80, 255),   // Default (green)
                2 => Color::from_rgba(80, 80, 180, 255),   // Displays (blue)
                _ => Color::from_rgba(180, 80, 80, 255),   // Other (red)
            };
            
            // Draw as rectangle
            draw_rectangle(x - w/2.0, y - h/2.0, w, h, color);
            
            // Draw border
            draw_rectangle_lines(x - w/2.0, y - h/2.0, w, h, 1.0, 
                Color::from_rgba(150, 150, 150, 255));
            
            // Draw name label if present
            if let Some(name) = name {
                let font_size = (10.0 * self.zoom).max(8.0) as u16;
                draw_text(&name.0, x - w/2.0, y - h/2.0 - 2.0, font_size as f32, WHITE);
            }
        }
        
        // Placeholder for non-macroquad builds
        let _ = (x, y, renderable, name);
    }

    /// Get visible entities.
    pub fn get_visible_entities(&self, world: &World, screen_width: f32, screen_height: f32) -> Vec<hecs::Entity> {
        let mut visible = Vec::new();

        for (entity, (pos, renderable)) in world.query::<(&Position, &Renderable)>().iter() {
            if !renderable.visible {
                continue;
            }
            
            let screen_x = (pos.x - self.camera_x) * self.zoom;
            let screen_y = (pos.y - self.camera_y) * self.zoom;

            if screen_x >= -100.0 && screen_x <= screen_width + 100.0 &&
               screen_y >= -100.0 && screen_y <= screen_height + 100.0 {
                visible.push(entity);
            }
        }

        visible
    }
}
//! Input system.

use hecs::World;
use crate::components::{Position, Selectable, GpioBind};

/// Input event.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse moved.
    MouseMove { x: f32, y: f32 },
    /// Mouse button pressed.
    MouseDown { x: f32, y: f32, button: u8 },
    /// Mouse button released.
    MouseUp { x: f32, y: f32, button: u8 },
    /// Mouse wheel scrolled.
    MouseWheel { delta: f32 },
    /// Key pressed.
    KeyDown { key: u32 },
    /// Key released.
    KeyUp { key: u32 },
}

/// Input system for handling user input.
pub struct InputSystem {
    /// Currently selected entity.
    selected_entity: Option<hecs::Entity>,
    /// Hovered entity.
    hovered_entity: Option<hecs::Entity>,
    /// Mouse position.
    mouse_x: f32,
    mouse_y: f32,
    /// Pending input events.
    events: Vec<InputEvent>,
    /// Drag state.
    dragging: bool,
    drag_start_x: f32,
    drag_start_y: f32,
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl InputSystem {
    /// Create a new input system.
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            hovered_entity: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            events: Vec::new(),
            dragging: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
        }
    }

    /// Add an input event.
    pub fn push_event(&mut self, event: InputEvent) {
        self.events.push(event);
    }

    /// Process input events.
    pub fn process(&mut self, world: &mut World) {
        // Process all pending events
        let events = std::mem::take(&mut self.events);

        for event in events {
            match event {
                InputEvent::MouseMove { x, y } => {
                    self.mouse_x = x;
                    self.mouse_y = y;
                    self.handle_hover(world, x, y);
                }
                InputEvent::MouseDown { x, y, button } => {
                    if button == 0 {
                        // Left click
                        self.handle_click(world, x, y);
                        self.dragging = true;
                        self.drag_start_x = x;
                        self.drag_start_y = y;
                    }
                }
                InputEvent::MouseUp { x, y, button } => {
                    if button == 0 {
                        self.dragging = false;
                        self.handle_release(world, x, y);
                    }
                }
                InputEvent::MouseWheel { delta } => {
                    self.handle_wheel(world, delta);
                }
                InputEvent::KeyDown { key } => {
                    self.handle_key_down(world, key);
                }
                InputEvent::KeyUp { key } => {
                    self.handle_key_up(world, key);
                }
            }
        }
    }

    /// Handle mouse hover.
    fn handle_hover(&mut self, world: &mut World, x: f32, y: f32) {
        // Find entity under mouse
        let mut found = None;

        for (entity, (pos, selectable)) in world.query_mut::<(&Position, &Selectable)>() {
            let dx = x - pos.x;
            let dy = y - pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < selectable.radius {
                found = Some(entity);
                break;
            }
        }

        self.hovered_entity = found;
    }

    /// Handle mouse click.
    fn handle_click(&mut self, world: &mut World, x: f32, y: f32) {
        // Select entity under mouse
        self.selected_entity = self.hovered_entity;

        // If entity has GPIO binding, toggle it
        if let Some(entity) = self.selected_entity {
            if let Ok(mut gpio) = world.query_one::<&GpioBind>(entity) {
                if let Some(bind) = gpio.get() {
                    // Toggle GPIO pin
                    let _ = bind; // Placeholder for GPIO toggle
                }
            }
        }

        let _ = (x, y);
    }

    /// Handle mouse release.
    fn handle_release(&mut self, _world: &mut World, _x: f32, _y: f32) {
        // Handle drag end if needed
    }

    /// Handle mouse wheel.
    fn handle_wheel(&mut self, _world: &mut World, delta: f32) {
        let _ = delta;
        // Zoom would be handled here
    }

    /// Handle key down.
    fn handle_key_down(&mut self, _world: &mut World, key: u32) {
        let _ = key;
        // Key handling would go here
    }

    /// Handle key up.
    fn handle_key_up(&mut self, _world: &mut World, key: u32) {
        let _ = key;
        // Key handling would go here
    }

    /// Get selected entity.
    pub fn selected(&self) -> Option<hecs::Entity> {
        self.selected_entity
    }

    /// Get hovered entity.
    pub fn hovered(&self) -> Option<hecs::Entity> {
        self.hovered_entity
    }

    /// Get mouse position.
    pub fn mouse_position(&self) -> (f32, f32) {
        (self.mouse_x, self.mouse_y)
    }
}
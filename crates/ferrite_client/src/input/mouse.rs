//! Mouse input handling.

use ferrite_core::math::Vec2;
use std::collections::HashSet;
use winit::event::MouseButton;

/// Mouse state
#[derive(Debug)]
pub struct MouseState {
    position: Vec2,
    delta: Vec2,
    pressed: HashSet<MouseButton>,
    just_pressed: HashSet<MouseButton>,
    just_released: HashSet<MouseButton>,
    scroll_delta: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            scroll_delta: 0.0,
        }
    }

    /// Get current mouse position
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Get mouse movement delta
    pub fn delta(&self) -> Vec2 {
        self.delta
    }

    /// Get scroll wheel delta
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    /// Check if a button is pressed
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    /// Check if a button was just pressed
    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    /// Check if a button was just released
    pub fn just_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    /// Set mouse position
    pub fn set_position(&mut self, position: Vec2) {
        self.delta = position - self.position;
        self.position = position;
    }

    /// Update position from separate x,y coordinates
    pub fn update_position(&mut self, x: f32, y: f32) {
        let new_position = Vec2::new(x, y);
        self.delta = new_position - self.position;
        self.position = new_position;
    }

    /// Register a button press
    pub fn press_button(&mut self, button: MouseButton) {
        if self.pressed.insert(button) {
            self.just_pressed.insert(button);
        }
    }

    /// Register a button release
    pub fn release_button(&mut self, button: MouseButton) {
        if self.pressed.remove(&button) {
            self.just_released.insert(button);
        }
    }

    /// Add scroll delta
    pub fn scroll(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    /// Add to mouse delta (used for device motion events when cursor is locked)
    pub fn add_motion_delta(&mut self, dx: f32, dy: f32) {
        self.delta.x += dx;
        self.delta.y += dy;
    }

    /// Update (clear frame-specific state)
    pub fn update(&mut self) {
        self.delta = Vec2::ZERO;
        self.just_pressed.clear();
        self.just_released.clear();
        self.scroll_delta = 0.0;
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

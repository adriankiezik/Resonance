//! Keyboard input handling.

use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Keyboard state
#[derive(Default, Debug)]
pub struct KeyboardState {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a key is currently pressed
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Check if a key was just pressed this frame
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    /// Register a key press
    pub fn press(&mut self, key: KeyCode) {
        if self.pressed.insert(key) {
            self.just_pressed.insert(key);
        }
    }

    /// Register a key release
    pub fn release(&mut self, key: KeyCode) {
        if self.pressed.remove(&key) {
            self.just_released.insert(key);
        }
    }

    /// Clear just_pressed and just_released (called each frame)
    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

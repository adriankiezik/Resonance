//! Input handling (keyboard, mouse, gamepad).

pub mod keyboard;
pub mod mouse;

use bevy_ecs::prelude::*;

/// Input state resource
#[derive(Resource, Default)]
pub struct Input {
    pub keyboard: keyboard::KeyboardState,
    pub mouse: mouse::MouseState,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update input state (called each frame)
    pub fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
    }
}

// TODO: Implement gamepad support
// TODO: Add input mapping system (action-based input)
// TODO: Implement input recording/playback for replays


pub mod keyboard;
pub mod mouse;

use bevy_ecs::prelude::*;

pub use keyboard::KeyboardState;
pub use mouse::MouseState;

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

#[derive(Resource, Default)]
pub struct Input {
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
}

impl Input {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
    }
}
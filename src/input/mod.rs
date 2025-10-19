pub mod keyboard;
pub mod mouse;

use bevy_ecs::prelude::*;
use crate::app::{Plugin, Resonance};
use std::any::TypeId;

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

#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, engine: &mut Resonance) {
        engine.world.insert_resource(Input::new());
    }

    fn name(&self) -> &str {
        "InputPlugin"
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn dependencies(&self) -> Vec<(TypeId, &str)> {
        vec![(
            TypeId::of::<crate::window::WindowPlugin>(),
            "resonance::window::WindowPlugin",
        )]
    }

    fn is_client_plugin(&self) -> bool {
        true
    }

    fn is_server_plugin(&self) -> bool {
        false
    }
}

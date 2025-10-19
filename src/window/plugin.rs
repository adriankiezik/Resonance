use crate::app::{Resonance, Plugin};
use crate::window::WindowConfig;

#[derive(Default)]
pub struct WindowPlugin {
    config: Option<WindowConfig>,
}

impl WindowPlugin {
    pub fn new(config: WindowConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    pub fn with_size(width: u32, height: u32, title: impl Into<String>) -> Self {
        Self {
            config: Some(WindowConfig::new(width, height, title)),
        }
    }

    fn get_config(&self) -> WindowConfig {
        self.config.clone().unwrap_or_default()
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, engine: &mut Resonance) {
        use crate::window::WindowEvent;

        engine.world.insert_resource(self.get_config());

        engine
            .world
            .init_resource::<bevy_ecs::prelude::Messages<WindowEvent>>();
    }
}

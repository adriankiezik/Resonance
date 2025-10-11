//! Client plugin that sets up all client-side systems.

use crate::{input::Input, window::WindowConfig};
use ferrite_app::{Engine, Plugin, Stage};

/// Plugin that sets up client functionality (rendering, input, audio)
///
/// Note: The window and renderer are created by the window runner, not by this plugin.
pub struct ClientPlugin {
    pub window_config: WindowConfig,
}

impl ClientPlugin {
    pub fn new(width: u32, height: u32, title: impl Into<String>) -> Self {
        Self {
            window_config: WindowConfig::new(width, height, title),
        }
    }

    pub fn with_config(window_config: WindowConfig) -> Self {
        Self { window_config }
    }
}

impl Default for ClientPlugin {
    fn default() -> Self {
        Self {
            window_config: WindowConfig::default(),
        }
    }
}

impl Plugin for ClientPlugin {
    fn build(&self, engine: &mut Engine) {
        // Only add client features in client mode
        if !engine.is_client() {
            log::warn!("ClientPlugin added to non-client engine, skipping");
            return;
        }

        // Add client resources (window and renderer will be added by the runner)
        engine.world.insert_resource(self.window_config.clone());
        engine.world.insert_resource(Input::new());
        // Note: AudioBackend is added by AudioPlugin

        // Register window events
        engine.world.init_resource::<bevy_ecs::message::Messages<crate::window::WindowEvent>>();

        // Add camera and model update systems to Update stage (before rendering)
        if let Some(schedule) = engine.schedules.get_mut(Stage::Update) {
            schedule.add_systems(crate::renderer::camera::update_camera_system);
            schedule.add_systems(crate::renderer::camera::update_model_system);
        }

        // Add rendering system to the Render stage
        if let Some(schedule) = engine.schedules.get_mut(Stage::Render) {
            schedule.add_systems(crate::renderer::systems::render_system);
        }

        // Components are automatically registered when first used

        log::info!(
            "ClientPlugin initialized (window config: {}x{})",
            self.window_config.width,
            self.window_config.height
        );
    }
}

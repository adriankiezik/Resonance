use crate::app::{Plugin, Resonance, Stage};
use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct WireframeState {
    pub enabled: bool,
}

impl Default for WireframeState {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[derive(Default)]
pub struct WireframePlugin;

impl Plugin for WireframePlugin {
    fn build(&self, engine: &mut Resonance) {
        engine.world.init_resource::<WireframeState>();

        if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
            schedule.add_systems(handle_wireframe_toggle);
        }
    }

    fn dependencies(&self) -> Vec<(std::any::TypeId, &str)> {
        vec![(
            std::any::TypeId::of::<crate::renderer::RenderPlugin>(),
            "resonance::renderer::RenderPlugin",
        )]
    }

    fn is_client_plugin(&self) -> bool {
        true
    }

    fn is_server_plugin(&self) -> bool {
        false
    }
}

fn handle_wireframe_toggle(
    mut state: bevy_ecs::prelude::ResMut<WireframeState>,
    input: Option<bevy_ecs::prelude::Res<crate::input::Input>>,
) {
    use winit::keyboard::KeyCode;

    let Some(input) = input else { return };

    if input.keyboard.just_pressed(KeyCode::F5) {
        state.enabled = !state.enabled;
        log::info!(
            "Wireframe mode: {}",
            if state.enabled { "ON" } else { "OFF" }
        );
    }
}

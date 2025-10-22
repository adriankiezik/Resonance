use crate::app::{Plugin, Resonance, Stage};
use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct DebugUiState {
    pub show_profiler: bool,
    pub show_performance: bool,
    pub show_general_info: bool,
}

impl Default for DebugUiState {
    fn default() -> Self {
        Self {
            show_profiler: false,
            show_performance: false,
            show_general_info: false,
        }
    }
}

#[derive(Default)]
pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, engine: &mut Resonance) {
        engine.world.init_resource::<DebugUiState>();

        if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
            schedule.add_systems(handle_keyboard_shortcuts);
        }
    }

    fn dependencies(&self) -> Vec<(std::any::TypeId, &str)> {
        vec![(
            std::any::TypeId::of::<crate::core::egui_plugin::EguiPlugin>(),
            "resonance::core::egui_plugin::EguiPlugin",
        )]
    }

    fn is_client_plugin(&self) -> bool {
        true
    }

    fn is_server_plugin(&self) -> bool {
        false
    }
}

fn handle_keyboard_shortcuts(
    mut state: bevy_ecs::prelude::ResMut<DebugUiState>,
    input: Option<bevy_ecs::prelude::Res<crate::input::Input>>,
) {
    use winit::keyboard::KeyCode;

    let Some(input) = input else { return };

    if input.keyboard.just_pressed(KeyCode::F1) {
        state.show_profiler = !state.show_profiler;
        log::debug!(
            "Profiler panel: {}",
            if state.show_profiler { "ON" } else { "OFF" }
        );
    }

    if input.keyboard.just_pressed(KeyCode::F2) {
        state.show_performance = !state.show_performance;
        log::debug!(
            "Performance panel: {}",
            if state.show_performance { "ON" } else { "OFF" }
        );
    }

    if input.keyboard.just_pressed(KeyCode::F3) {
        state.show_general_info = !state.show_general_info;
        log::debug!(
            "General info panel: {}",
            if state.show_general_info { "ON" } else { "OFF" }
        );
    }

    if input.keyboard.just_pressed(KeyCode::Escape) {
        if state.show_profiler || state.show_performance || state.show_general_info {
            state.show_profiler = false;
            state.show_performance = false;
            state.show_general_info = false;
            log::debug!("All debug panels closed");
        }
    }
}

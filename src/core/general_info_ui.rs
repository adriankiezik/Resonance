use crate::addons::DebugUiState;
use bevy_ecs::prelude::*;

pub fn render_general_info_panel(world: &mut World, ctx: &egui::Context) {
    let state = world.get_resource::<DebugUiState>();
    if state.map_or(false, |s| !s.show_general_info) {
        return;
    }

    egui::Window::new("General Info")
        .default_pos([10.0, 330.0])
        .default_size([300.0, 250.0])
        .show(ctx, |ui| {
            ui.heading("Engine Information");
            ui.separator();

            ui.label(format!("Engine: Resonance v{}", env!("CARGO_PKG_VERSION")));
            ui.label(format!(
                "Build: {}",
                if cfg!(debug_assertions) {
                    "Debug"
                } else {
                    "Release"
                }
            ));

            ui.add_space(10.0);
            ui.separator();
            ui.heading("ECS Statistics");

            let entity_count = world.entities().len();
            ui.label(format!("Entities: {}", entity_count));

            ui.add_space(10.0);
            ui.separator();
            ui.heading("Resources");

            let mut resource_names: Vec<String> = Vec::new();

            if world.contains_resource::<crate::window::Window>() {
                resource_names.push("Window".to_string());
            }
            if world.contains_resource::<crate::renderer::Renderer>() {
                resource_names.push("Renderer".to_string());
            }
            if world.contains_resource::<crate::core::Time>() {
                resource_names.push("Time".to_string());
            }
            if world.contains_resource::<crate::core::PerformanceAnalytics>() {
                resource_names.push("PerformanceAnalytics".to_string());
            }
            if world.contains_resource::<crate::core::Profiler>() {
                resource_names.push("Profiler".to_string());
            }
            if world.contains_resource::<crate::input::Input>() {
                resource_names.push("Input".to_string());
            }
            if world.contains_resource::<crate::audio::AudioBackend>() {
                resource_names.push("AudioBackend".to_string());
            }

            ui.label(format!("Active Resources: {}", resource_names.len()));

            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    for name in resource_names {
                        ui.label(format!("  • {}", name));
                    }
                });

            ui.add_space(10.0);
            ui.separator();

            if let Some(time) = world.get_resource::<crate::core::Time>() {
                ui.heading("Time");
                ui.label(format!("Elapsed: {:.2}s", time.elapsed().as_secs_f64()));
                ui.label(format!("Delta: {:.4}s", time.delta().as_secs_f64()));
            }
        });
}

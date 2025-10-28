//! Graphics Settings Example
//!
//! This example demonstrates how to configure graphics settings:
//! - MSAA (Multi-Sample Anti-Aliasing)
//! - VSync
//! - Window configuration
//!
//! Run with: `cargo run --example graphics_settings`

use resonance::prelude::*;

fn main() {
    Resonance::new()
        .with_log_level(log::LevelFilter::Info)
        // Configure window settings
        .with_resource(WindowConfig {
            width: 1920,
            height: 1080,
            title: "Graphics Settings Example".to_string(),
            mode: WindowMode::Windowed,
        })
        // Configure graphics settings (MSAA x4, VSync enabled)
        .with_graphics_settings(GraphicsSettings::new(MsaaSampleCount::X4, true))
        .add_plugin(DefaultPlugins)
        .add_system(Stage::Startup, setup_scene)
        .run();
}

fn setup_scene(world: &mut World) {
    // Spawn a camera
    world.spawn((
        Transform::from_xyz(0.0, 2.0, 5.0),
        Camera::perspective(16.0 / 9.0, 60.0),
    ));

    println!("Scene setup complete!");
    println!("- MSAA: 4x");
    println!("- VSync: Enabled");
    println!("- Resolution: 1920x1080");
}

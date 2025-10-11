//! Window Modes Example
//!
//! Demonstrates fullscreen and borderless window modes.
//!
//! Controls:
//! - F11: Toggle borderless fullscreen
//! - F: Toggle to exclusive fullscreen mode
//! - W: Switch to windowed mode
//! - ESC: Exit

use ferrite::prelude::*;
use ferrite_client::{run_with_window, Window, WindowConfig, WindowMode};

fn main() {
    // Initialize logger
    init_logger();

    log::info!("=== Window Modes Example ===");
    log::info!("Controls:");
    log::info!("  F11: Toggle borderless fullscreen");
    log::info!("  F: Toggle to exclusive fullscreen mode");
    log::info!("  W: Switch to windowed mode");
    log::info!("  ESC or close button: Exit");

    // Create window config - start in windowed mode
    let window_config = WindowConfig::new(1280, 720, "Window Modes Demo - Ferrite Engine")
        .with_mode(WindowMode::Windowed);

    // You can also start in fullscreen or borderless mode:
    // let window_config = WindowConfig::new(1920, 1080, "Ferrite Window Modes")
    //     .fullscreen();  // Start in exclusive fullscreen
    // or
    // let window_config = WindowConfig::new(1920, 1080, "Ferrite Window Modes")
    //     .borderless();  // Start in borderless fullscreen

    // Build engine with client plugin
    let engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(ClientPlugin::with_config(window_config.clone()))
        .add_startup_system(setup)
        .add_system(Stage::Update, display_info)
        .add_system(Stage::Update, handle_mode_switching);

    log::info!("Starting window...");

    // Run with window (this will block until window is closed)
    if let Err(e) = run_with_window(engine, window_config) {
        log::error!("Failed to run window: {}", e);
    }

    log::info!("Window closed, exiting");
}

/// Setup system that runs once
fn setup(mut commands: Commands) {
    log::info!("Window ready! Try switching between different window modes using the controls.");

    // Spawn a marker entity for tracking state
    commands.spawn((InfoDisplay { tick_count: 0 },));
}

/// Component to track display info
#[derive(Component)]
struct InfoDisplay {
    tick_count: u32,
}

/// System to display current window mode info
fn display_info(mut query: Query<&mut InfoDisplay>, window: Res<Window>) {
    for mut info in query.iter_mut() {
        info.tick_count += 1;

        // Display info every 60 frames (approximately once per second)
        if info.tick_count % 60 == 0 {
            let mode = window.current_mode();
            let (width, height) = window.size();
            log::info!(
                "Current mode: {:?} | Window size: {}x{}",
                mode,
                width,
                height
            );
        }
    }
}

/// System to handle window mode switching
fn handle_mode_switching(_window: Res<Window>) {
    // Note: This is a placeholder implementation since we haven't implemented
    // input handling yet in the engine. In a real implementation, this would
    // check for keyboard input events.

    // For now, users can test the modes by modifying the initial window_config
    // in the main function to start in different modes.

    // TODO: Once input system is implemented, add:
    // - F11 key: window.toggle_fullscreen()
    // - F key: window.set_mode(WindowMode::Fullscreen)
    // - W key: window.set_mode(WindowMode::Windowed)
}

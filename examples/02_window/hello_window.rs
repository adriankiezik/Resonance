//! Hello Window Example
//!
//! Demonstrates basic window creation with the Ferrite engine.
//! Opens a window that can be closed by clicking the close button.

use ferrite::prelude::*;
use ferrite_client::{run_with_window, WindowConfig};

fn main() {
    // Initialize logger
    init_logger();

    log::info!("=== Hello Window Example ===");

    // Create window config
    let window_config = WindowConfig::new(800, 600, "Hello Window - Ferrite Engine");

    // Build engine with client plugin
    let engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(ClientPlugin::with_config(window_config.clone()))
        .add_system(Stage::Update, hello_system);

    log::info!("Starting window...");

    // Run with window (this will block until window is closed)
    if let Err(e) = run_with_window(engine, window_config) {
        log::error!("Failed to run window: {}", e);
    }

    log::info!("Window closed, exiting");
}

/// Simple system that logs once when it runs
fn hello_system() {
    static mut RAN: bool = false;
    unsafe {
        if !RAN {
            log::info!("Hello from the engine! Press ESC or close the window to exit.");
            RAN = true;
        }
    }
}

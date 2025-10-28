//! Minimal Resonance Engine Example
//!
//! This example demonstrates the absolute minimum code needed to run the engine.
//! It creates a window and runs the engine with default settings.
//!
//! Run with: `cargo run --example minimal`

use resonance::prelude::*;

fn main() {
    // Initialize engine with default settings and all standard plugins
    Resonance::new()
        .with_log_level(log::LevelFilter::Info)
        .add_plugin(DefaultPlugins)
        .run();
}

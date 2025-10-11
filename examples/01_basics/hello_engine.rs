//! Minimal example showing the engine setup and basic loop.

use ferrite::prelude::*;

fn main() {
    // Initialize logging
    init_logger();

    log::info!("Starting Ferrite Engine - Hello Engine Example");

    // Create engine with core plugin
    let engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_system(Stage::Update, hello_system);

    // Run the engine (currently runs for 10 frames for testing)
    ferrite_app::run(engine);
}

fn hello_system(tick: Res<GameTick>, time: Res<Time>) {
    log::info!(
        "Tick: {}, Elapsed: {:.2}s",
        tick.get(),
        time.elapsed_seconds()
    );
}

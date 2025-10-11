//! Example of a basic dedicated server setup.
//!
//! Run with: cargo run --example server_basic --features server

#[cfg(feature = "server")]
use ferrite::prelude::*;

#[cfg(feature = "server")]
fn main() {
    init_logger();

    log::info!("Starting Dedicated Server");

    let engine = Engine::with_mode(EngineMode::Server)
        .add_plugin(CorePlugin)
        .add_plugin(TransformPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(ferrite::server::ServerPlugin::default())
        .add_startup_system(server_startup);

    log::info!("Server is running in headless mode (no rendering)");

    ferrite_app::run(engine);
}

#[cfg(feature = "server")]
fn server_startup() {
    log::info!("Server startup complete - ready for connections");
}

#[cfg(not(feature = "server"))]
fn main() {
    eprintln!("This example requires the 'server' feature.");
    eprintln!("Run with: cargo run --example server_basic --features server");
    std::process::exit(1);
}

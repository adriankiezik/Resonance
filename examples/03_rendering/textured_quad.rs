//! Textured Quad Example
//!
//! Demonstrates textured mesh rendering with the Ferrite engine.
//! Renders a quad with a procedurally generated checkerboard texture.

use ferrite::prelude::*;
use ferrite_client::renderer::mesh::Mesh;
use ferrite_client::renderer::pipeline::MeshBuffers;
use ferrite_client::renderer::texture::TextureHandle;
use ferrite_client::renderer::Renderer;
use ferrite_client::{run_with_window, WindowConfig};

fn main() {
    // Initialize logger
    init_logger();

    log::info!("=== Textured Quad Example ===");

    // Create window config
    let window_config = WindowConfig::new(800, 600, "Textured Quad - Ferrite Engine");

    // Build engine with client plugin
    let engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(ClientPlugin::with_config(window_config.clone()))
        .add_system(Stage::Startup, setup_textured_quad);

    log::info!("Starting window...");

    // Run with window (this will block until window is closed)
    if let Err(e) = run_with_window(engine, window_config) {
        log::error!("Failed to run window: {}", e);
    }

    log::info!("Window closed, exiting");
}

/// Component to mark our quad entity
#[derive(Component)]
struct QuadMarker;

/// Setup the textured quad
fn setup_textured_quad(mut commands: Commands, renderer: Option<Res<Renderer>>) {
    if let Some(renderer) = renderer {
        log::info!("Creating textured quad...");

        // Create quad vertices (using Mesh::quad helper)
        let mesh = Mesh::quad();

        // Create GPU buffers
        let mesh_buffers = MeshBuffers::new(&renderer.device, &mesh.vertices, &mesh.indices);

        // Create a checkerboard texture
        let texture = TextureHandle::create_checkerboard(
            &renderer.device,
            &renderer.queue,
            &renderer.texture_bind_group_layout,
            256,
        );

        log::info!("Textured quad created with checkerboard texture");

        // Spawn entity with mesh buffers and texture
        // The ClientPlugin's render_system will automatically render entities with both components
        commands.spawn((mesh, mesh_buffers, texture, QuadMarker));
    }
}

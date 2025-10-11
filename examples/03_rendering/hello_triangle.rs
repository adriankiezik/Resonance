//! Hello Triangle Example
//!
//! Demonstrates basic mesh rendering with the Ferrite engine.
//! Renders a colored triangle using the basic rendering pipeline.

use ferrite::prelude::*;
use ferrite_client::renderer::mesh::{Mesh, Vertex};
use ferrite_client::renderer::pipeline::MeshBuffers;
use ferrite_client::renderer::Renderer;
use ferrite_client::{run_with_window, WindowConfig};

fn main() {
    // Initialize logger
    init_logger();

    log::info!("=== Hello Triangle Example ===");

    // Create window config
    let window_config = WindowConfig::new(800, 600, "Hello Triangle - Ferrite Engine");

    // Build engine with client plugin
    let engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(ClientPlugin::with_config(window_config.clone()))
        .add_system(Stage::Startup, setup_triangle);

    log::info!("Starting window...");

    // Run with window (this will block until window is closed)
    if let Err(e) = run_with_window(engine, window_config) {
        log::error!("Failed to run window: {}", e);
    }

    log::info!("Window closed, exiting");
}

/// Component to mark our triangle entity
#[derive(Component)]
struct TriangleMarker;

/// Setup the triangle mesh
fn setup_triangle(mut commands: Commands, renderer: Option<Res<Renderer>>) {
    if let Some(renderer) = renderer {
        log::info!("Creating colored triangle...");

        // Create triangle vertices with different colors
        let vertices = vec![
            Vertex::new(
                Vec3::new(0.0, 0.5, 0.0),      // Top center
                Vec3::Z,
                Vec2::new(0.5, 0.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0), // Red
            ),
            Vertex::new(
                Vec3::new(-0.5, -0.5, 0.0),    // Bottom left
                Vec3::Z,
                Vec2::new(0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0), // Green
            ),
            Vertex::new(
                Vec3::new(0.5, -0.5, 0.0),     // Bottom right
                Vec3::Z,
                Vec2::new(1.0, 1.0),
                Vec4::new(0.0, 0.0, 1.0, 1.0), // Blue
            ),
        ];
        let indices = vec![0, 1, 2];

        // Create GPU buffers
        let mesh_buffers = MeshBuffers::new(&renderer.device, &vertices, &indices);

        // Spawn entity with mesh buffers
        // Note: The ClientPlugin's render_system will automatically render all entities with MeshBuffers
        commands.spawn((Mesh::new(vertices, indices), mesh_buffers, TriangleMarker));

        log::info!("Triangle created with {} vertices", 3);
    }
}

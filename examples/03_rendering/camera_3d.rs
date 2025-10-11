//! 3D Camera Example
//!
//! Demonstrates the camera system with a 3D perspective camera.
//! Renders multiple colored triangles at different depths to show 3D projection.

use ferrite::prelude::*;
use ferrite_client::renderer::camera::{Camera, CameraBuffer, CameraUniform, MainCamera};
use ferrite_client::renderer::mesh::{Mesh, Vertex};
use ferrite_client::renderer::pipeline::MeshBuffers;
use ferrite_client::renderer::Renderer;
use ferrite_client::{run_with_window, WindowConfig};
use std::f32::consts::PI;

fn main() {
    // Initialize logger
    init_logger();

    log::info!("=== 3D Camera Example ===");

    // Create window config
    let window_config = WindowConfig::new(800, 600, "3D Camera - Ferrite Engine");

    // Build engine with client plugin
    let engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(ClientPlugin::with_config(window_config.clone()))
        .add_system(Stage::Startup, setup_scene)
        .add_system(Stage::Update, rotate_camera);

    log::info!("Starting window...");

    // Run with window (this will block until window is closed)
    if let Err(e) = run_with_window(engine, window_config) {
        log::error!("Failed to run window: {}", e);
    }

    log::info!("Window closed, exiting");
}

/// Component to mark triangles
#[derive(Component)]
struct TriangleMarker;

/// Setup the 3D scene with camera and multiple triangles
fn setup_scene(mut commands: Commands, renderer: Option<Res<Renderer>>) {
    if let Some(renderer) = renderer {
        log::info!("Setting up 3D scene...");

        // Create camera entity with perspective projection
        let camera = Camera::perspective(
            PI / 4.0,           // 45 degree FOV
            800.0 / 600.0,      // aspect ratio
            0.1,                // near plane
            100.0,              // far plane
        );

        // Position camera looking at the scene
        let mut transform = Transform::default();
        transform.position = Vec3::new(0.0, 0.0, 5.0); // Move camera back

        // Create initial camera buffer
        let view = Camera::view_matrix(&transform);
        let uniform = CameraUniform::new(view, camera.projection);
        let camera_buffer = CameraBuffer::new(
            &renderer.device,
            &renderer.camera_bind_group_layout,
            &uniform,
        );

        // Insert camera buffer as resource
        commands.insert_resource(camera_buffer);

        // Spawn camera entity
        commands.spawn((camera, transform, MainCamera));

        log::info!("Camera created at position {:?}", Vec3::new(0.0, 0.0, 5.0));

        // Create multiple triangles at different positions and depths
        create_triangle(
            &mut commands,
            &renderer,
            Vec3::new(0.0, 0.0, 0.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
        );
        create_triangle(
            &mut commands,
            &renderer,
            Vec3::new(-1.5, 0.0, -2.0),
            Vec4::new(0.0, 1.0, 0.0, 1.0),
        );
        create_triangle(
            &mut commands,
            &renderer,
            Vec3::new(1.5, 0.0, -2.0),
            Vec4::new(0.0, 0.0, 1.0, 1.0),
        );

        log::info!("Scene setup complete with 3 triangles");
    }
}

/// Helper to create a triangle at a specific position
fn create_triangle(
    commands: &mut Commands,
    renderer: &Renderer,
    position: Vec3,
    color: Vec4,
) {
    let vertices = vec![
        Vertex::new(
            position + Vec3::new(0.0, 0.5, 0.0),
            Vec3::Z,
            Vec2::new(0.5, 0.0),
            color,
        ),
        Vertex::new(
            position + Vec3::new(-0.5, -0.5, 0.0),
            Vec3::Z,
            Vec2::new(0.0, 1.0),
            color,
        ),
        Vertex::new(
            position + Vec3::new(0.5, -0.5, 0.0),
            Vec3::Z,
            Vec2::new(1.0, 1.0),
            color,
        ),
    ];
    let indices = vec![0, 1, 2];

    let mesh = Mesh::new(vertices.clone(), indices.clone());
    let mesh_buffers = MeshBuffers::new(&renderer.device, &vertices, &indices);

    commands.spawn((mesh, mesh_buffers, TriangleMarker));
}

/// Rotate the camera around the scene
fn rotate_camera(
    time: Res<ferrite_core::Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if let Ok(mut transform) = camera_query.single_mut() {
        let elapsed = time.elapsed_seconds();
        let radius = 5.0;

        // Orbit camera around the scene
        transform.position.x = (elapsed * 0.5).sin() * radius;
        transform.position.z = (elapsed * 0.5).cos() * radius;
        transform.position.y = (elapsed * 0.3).sin() * 2.0;

        // Look at center
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

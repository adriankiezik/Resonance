//! Mesh Primitives Example
//!
//! Demonstrates the mesh and material system with various primitive shapes:
//! - Cube, sphere, and plane meshes
//! - Color and textured materials
//! - Multiple meshes rendered simultaneously
//!
//! Controls:
//! - WASD: Move camera forward/back/left/right
//! - Q/E: Move camera down/up
//! - Mouse Movement: Look around
//! - ESC: Toggle cursor lock

use ferrite::prelude::*;
use ferrite_client::input::Input;
use ferrite_client::renderer::camera::{Camera, CameraBuffer, CameraUniform, MainCamera, ModelBuffer, ModelUniform};
use ferrite_client::renderer::material::Material;
use ferrite_client::renderer::mesh::Mesh;
use ferrite_client::renderer::pipeline::MeshBuffers;
use ferrite_client::renderer::texture::TextureHandle;
use ferrite_client::renderer::Renderer;
use ferrite_client::window::WindowEvent;
use ferrite_client::{run_with_window, KeyCode, Window, WindowConfig};
use std::f32::consts::PI;

fn main() {
    // Initialize logger
    init_logger();

    log::info!("=== Mesh Primitives Example ===");

    // Create window config
    let window_config = WindowConfig::new(1024, 768, "Mesh Primitives - Ferrite Engine");

    // Build engine with client plugin
    let engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(ClientPlugin::with_config(window_config.clone()))
        .add_system(Stage::Startup, setup_scene)
        .add_system(Stage::Update, handle_window_resize)
        .add_system(Stage::Update, cursor_toggle)
        .add_system(Stage::Update, camera_controller)
        .add_system(Stage::Update, rotate_meshes);

    log::info!("Starting window...");

    // Run with window (this will block until window is closed)
    if let Err(e) = run_with_window(engine, window_config) {
        log::error!("Failed to run window: {}", e);
    }

    log::info!("Window closed, exiting");
}

/// Component to mark rotating meshes
#[derive(Component)]
struct Rotating {
    speed: f32,
    axis: Vec3,
}

/// Resource to track cursor state
#[derive(Resource)]
struct CursorState {
    locked: bool,
}

/// Resource to track camera rotation as euler angles
#[derive(Resource)]
struct CameraRotation {
    yaw: f32,   // Rotation around Y axis (left/right)
    pitch: f32, // Rotation around X axis (up/down)
}

/// Setup the 3D scene with camera and multiple primitive meshes
fn setup_scene(mut commands: Commands, renderer: Option<Res<Renderer>>, window: Option<Res<Window>>) {
    // Initialize cursor state and lock/hide cursor
    if let Some(window) = &window {
        window.set_cursor_visible(false);
        if let Err(e) = window.set_cursor_grab(true) {
            log::warn!("Failed to grab cursor: {}", e);
        }
    }
    commands.insert_resource(CursorState { locked: true });

    // Initialize camera rotation (calculate initial pitch from looking down at origin)
    // Camera is at (0, 3, 8) looking at (0, 0, 0)
    // Initial yaw is 0 (facing -Z)
    // Initial pitch is looking down
    let camera_pos = Vec3::new(0.0, 3.0, 8.0);
    let look_dir = (Vec3::ZERO - camera_pos).normalize();
    let initial_pitch = look_dir.y.asin(); // Angle looking down
    log::info!("Initial camera rotation - yaw: 0.0, pitch: {} rad ({} deg)",
        initial_pitch, initial_pitch.to_degrees());
    commands.insert_resource(CameraRotation {
        yaw: 0.0,
        pitch: initial_pitch,
    });

    if let Some(renderer) = renderer {
        log::info!("Setting up mesh primitives scene...");

        // Create camera entity with perspective projection
        let camera = Camera::perspective(
            PI / 4.0,        // 45 degree FOV
            1024.0 / 768.0,  // aspect ratio
            0.1,             // near plane
            100.0,           // far plane
        );

        // Position camera looking at the scene
        let mut transform = Transform::default();
        transform.position = Vec3::new(0.0, 3.0, 8.0); // Move camera back and up

        // Use look_at to orient camera toward the scene center
        transform.look_at(Vec3::ZERO, Vec3::Y);

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

        log::info!("Camera created at position {:?}", Vec3::new(0.0, 3.0, 8.0));

        // Create a textured cube
        create_textured_cube(
            &mut commands,
            &renderer,
            Vec3::new(-3.0, 0.0, 0.0),
            1.5,
        );

        // Create a colored sphere
        create_colored_sphere(
            &mut commands,
            &renderer,
            Vec3::new(0.0, 0.0, 0.0),
            1.0,
            Vec4::new(1.0, 0.3, 0.3, 1.0), // Red
        );

        // Create another colored sphere
        create_colored_sphere(
            &mut commands,
            &renderer,
            Vec3::new(3.0, 0.0, 0.0),
            1.0,
            Vec4::new(0.3, 1.0, 0.3, 1.0), // Green
        );

        // Create a ground plane
        create_textured_plane(
            &mut commands,
            &renderer,
            Vec3::new(0.0, -2.0, 0.0),
            10.0,
        );

        log::info!("Scene setup complete with cube, spheres, and plane");
    }
}

/// Helper to create a textured cube
fn create_textured_cube(
    commands: &mut Commands,
    renderer: &Renderer,
    position: Vec3,
    size: f32,
) {
    let mesh = Mesh::cube(size);
    let vertices = mesh.vertices.clone();
    let indices = mesh.indices.clone();
    let mesh_buffers = MeshBuffers::new(&renderer.device, &vertices, &indices);

    // Create checkerboard texture
    let texture = TextureHandle::create_checkerboard(
        &renderer.device,
        &renderer.queue,
        &renderer.texture_bind_group_layout,
        256,
    );

    let mut transform = Transform::default();
    transform.position = position;

    // Create model buffer
    let model_uniform = ModelUniform::from_transform(&transform);
    let model_buffer = ModelBuffer::new(
        &renderer.device,
        &renderer.model_bind_group_layout,
        &model_uniform,
    );

    let material = Material::textured();

    log::info!("Created cube at position: {:?}", position);

    commands.spawn((
        mesh,
        mesh_buffers,
        texture,
        material,
        transform,
        model_buffer,
        Rotating {
            speed: 0.5,
            axis: Vec3::new(1.0, 1.0, 0.0).normalize(),
        },
    ));
}

/// Helper to create a colored sphere
fn create_colored_sphere(
    commands: &mut Commands,
    renderer: &Renderer,
    position: Vec3,
    radius: f32,
    color: Vec4,
) {
    let mut mesh = Mesh::sphere(radius, 32, 16);

    // Apply color to all vertices
    for vertex in &mut mesh.vertices {
        vertex.color = color.to_array();
    }

    let vertices = mesh.vertices.clone();
    let indices = mesh.indices.clone();
    let mesh_buffers = MeshBuffers::new(&renderer.device, &vertices, &indices);

    let mut transform = Transform::default();
    transform.position = position;

    // Create model buffer
    let model_uniform = ModelUniform::from_transform(&transform);
    let model_buffer = ModelBuffer::new(
        &renderer.device,
        &renderer.model_bind_group_layout,
        &model_uniform,
    );

    let material = Material::color(color);

    log::info!("Created sphere at position: {:?} with color: {:?}", position, color);

    commands.spawn((
        mesh,
        mesh_buffers,
        material,
        transform,
        model_buffer,
        Rotating {
            speed: 0.3,
            axis: Vec3::Y,
        },
    ));
}

/// Helper to create a textured plane
fn create_textured_plane(
    commands: &mut Commands,
    renderer: &Renderer,
    position: Vec3,
    size: f32,
) {
    let mesh = Mesh::plane(size, 10);
    let vertices = mesh.vertices.clone();
    let indices = mesh.indices.clone();
    let mesh_buffers = MeshBuffers::new(&renderer.device, &vertices, &indices);

    // Create gradient texture for the plane
    let texture = TextureHandle::create_checkerboard(
        &renderer.device,
        &renderer.queue,
        &renderer.texture_bind_group_layout,
        512,
    );

    let mut transform = Transform::default();
    transform.position = position;

    // Create model buffer
    let model_uniform = ModelUniform::from_transform(&transform);
    let model_buffer = ModelBuffer::new(
        &renderer.device,
        &renderer.model_bind_group_layout,
        &model_uniform,
    );

    let material = Material::textured();

    commands.spawn((mesh, mesh_buffers, texture, material, transform, model_buffer));
}

/// System to handle window resize events and update camera aspect ratio
fn handle_window_resize(
    mut reader: MessageReader<WindowEvent>,
    mut camera_query: Query<&mut Camera, With<MainCamera>>,
) {
    for event in reader.read() {
        if let WindowEvent::Resized { width, height } = event {
            if let Ok(mut camera) = camera_query.single_mut() {
                let new_aspect = *width as f32 / *height as f32;
                camera.set_aspect_ratio(new_aspect);
                log::info!("Camera aspect ratio updated: {} ({}x{})", new_aspect, width, height);
            }
        }
    }
}

/// System to handle cursor toggle with ESC key
fn cursor_toggle(
    input: Res<Input>,
    window: Option<Res<Window>>,
    mut cursor_state: Option<ResMut<CursorState>>,
) {
    if let (Some(window), Some(cursor_state)) = (window, cursor_state.as_mut()) {
        // Check if ESC was just pressed (not held)
        if input.keyboard.just_pressed(KeyCode::Escape) {
            cursor_state.locked = !cursor_state.locked;

            window.set_cursor_visible(!cursor_state.locked);
            if let Err(e) = window.set_cursor_grab(cursor_state.locked) {
                log::warn!("Failed to set cursor grab: {}", e);
            }

            log::info!("Cursor {}", if cursor_state.locked { "locked" } else { "unlocked" });
        }
    }
}

/// Camera controller system - WASD to move, Mouse to look around
fn camera_controller(
    time: Res<ferrite_core::Time>,
    input: Res<Input>,
    cursor_state: Option<Res<CursorState>>,
    mut camera_rotation: Option<ResMut<CameraRotation>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if let Ok(mut transform) = camera_query.single_mut() {
        let delta = time.delta_seconds();
        let move_speed = 5.0;
        let mouse_sensitivity = 0.002; // Radians per pixel (same for both axes)

        // Movement (WASD + QE)
        let mut movement = Vec3::ZERO;

        if input.keyboard.is_pressed(KeyCode::KeyW) {
            movement += transform.forward();
        }
        if input.keyboard.is_pressed(KeyCode::KeyS) {
            movement -= transform.forward();
        }
        if input.keyboard.is_pressed(KeyCode::KeyA) {
            movement -= transform.right();
        }
        if input.keyboard.is_pressed(KeyCode::KeyD) {
            movement += transform.right();
        }
        if input.keyboard.is_pressed(KeyCode::KeyQ) {
            movement -= Vec3::Y;
        }
        if input.keyboard.is_pressed(KeyCode::KeyE) {
            movement += Vec3::Y;
        }

        if movement.length_squared() > 0.0 {
            transform.position += movement.normalize() * move_speed * delta;
        }

        // Only process mouse look when cursor is locked
        if let (Some(cursor_state), Some(camera_rotation)) = (cursor_state, camera_rotation.as_mut()) {
            if cursor_state.locked {
                // Update yaw and pitch from mouse delta
                let mouse_delta = input.mouse.delta();
                camera_rotation.yaw -= mouse_delta.x * mouse_sensitivity;
                camera_rotation.pitch -= mouse_delta.y * mouse_sensitivity;

                // Clamp pitch to prevent camera flipping (almost vertical but not quite)
                let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.01; // 89.4 degrees
                camera_rotation.pitch = camera_rotation.pitch.clamp(-pitch_limit, pitch_limit);

                // Reconstruct rotation from euler angles
                // Order: Yaw (Y-axis) * Pitch (X-axis)
                // This creates a rotation without any roll component
                let yaw_quat = Quat::from_axis_angle(Vec3::Y, camera_rotation.yaw);
                let pitch_quat = Quat::from_axis_angle(Vec3::X, camera_rotation.pitch);
                transform.rotation = yaw_quat * pitch_quat;
            }
        }
    }
}

/// Rotate meshes marked with Rotating component
fn rotate_meshes(
    time: Res<ferrite_core::Time>,
    mut query: Query<(&mut Transform, &Rotating)>,
) {
    let delta = time.delta_seconds();

    for (mut transform, rotating) in query.iter_mut() {
        // Rotate around the specified axis
        let angle = rotating.speed * delta;
        transform.rotation = Quat::from_axis_angle(rotating.axis, angle) * transform.rotation;
    }
}

//! Character Controller Example
//!
//! Demonstrates the MMORPG-style character controller with:
//! - Kinematic character movement (WASD)
//! - Jumping and gravity
//! - Ground detection via raycasting
//! - Collision with walls and obstacles
//! - Trigger zones (enter/exit events)
//! - Spatial partitioning for performance
//!
//! Controls:
//! - WASD: Move character (camera-relative)
//! - Space: Jump
//! - Mouse: Look around
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
use ferrite_physics::*;
use std::f32::consts::PI;

fn main() {
    init_logger();

    log::info!("=== Character Controller Example ===");
    log::info!("This example demonstrates MMORPG-style character movement");

    let window_config = WindowConfig::new(1280, 720, "Character Controller - Ferrite Engine");

    // Use 2x gravity for quicker, snappier jumps while maintaining same jump height
    let gravity = Vec3::new(0.0, -19.62, 0.0);

    let engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(PhysicsPlugin::with_gravity(gravity))
        .add_plugin(ClientPlugin::with_config(window_config.clone()))
        .add_system(Stage::Startup, setup_scene)
        .add_system(Stage::Update, handle_window_resize)
        .add_system(Stage::Update, cursor_toggle)
        .add_system(Stage::Update, camera_follow_character)
        .add_system(Stage::Update, character_input_system)
        .add_system(Stage::Update, trigger_zone_system)
        .add_system(Stage::Update, debug_info_system);

    log::info!("Starting character controller demo...");

    if let Err(e) = run_with_window(engine, window_config) {
        log::error!("Failed to run window: {}", e);
    }

    log::info!("Example closed");
}

/// Marker component for the player character
#[derive(Component)]
struct Player;

/// Cursor state
#[derive(Resource)]
struct CursorState {
    locked: bool,
}

/// Camera rotation tracking
#[derive(Resource)]
struct CameraRotation {
    yaw: f32,
    pitch: f32,
}


/// Setup the scene with character, obstacles, and triggers
fn setup_scene(mut commands: Commands, renderer: Option<Res<Renderer>>, window: Option<Res<Window>>) {
    // Setup cursor
    if let Some(window) = &window {
        window.set_cursor_visible(false);
        if let Err(e) = window.set_cursor_grab(true) {
            log::warn!("Failed to grab cursor: {}", e);
        }
    }
    commands.insert_resource(CursorState { locked: true });
    commands.insert_resource(CameraRotation { yaw: 0.0, pitch: -0.3 });

    let Some(renderer) = renderer else { return };

    log::info!("Setting up character controller scene...");

    // Create camera
    let camera = Camera::perspective(PI / 4.0, 1280.0 / 720.0, 0.1, 1000.0);
    let mut camera_transform = Transform::default();
    camera_transform.position = Vec3::new(0.0, 4.0, 8.0);
    camera_transform.look_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);

    let view = Camera::view_matrix(&camera_transform);
    let uniform = CameraUniform::new(view, camera.projection);
    let camera_buffer = CameraBuffer::new(
        &renderer.device,
        &renderer.camera_bind_group_layout,
        &uniform,
    );

    commands.insert_resource(camera_buffer);
    commands.spawn((camera, camera_transform, MainCamera));

    // Create player character (spawn at correct height: ground_top + half_height)
    // Ground box is at y=0 with half_extent_y=0.1, so top is at 0.1
    // Character half_height is 0.9, so spawn at 0.1 + 0.9 = 1.0
    create_player(&mut commands, &renderer, Vec3::new(0.0, 1.0, 0.0));

    // Create ground plane
    create_ground(&mut commands, &renderer);

    // Create obstacles
    create_obstacle(&mut commands, &renderer, Vec3::new(5.0, 0.5, 0.0), Vec3::new(1.0, 0.5, 3.0));
    create_obstacle(&mut commands, &renderer, Vec3::new(-5.0, 0.5, 0.0), Vec3::new(1.0, 0.5, 3.0));
    create_obstacle(&mut commands, &renderer, Vec3::new(0.0, 0.5, 8.0), Vec3::new(5.0, 0.5, 1.0));

    // Create trigger zones
    create_trigger_zone(
        &mut commands,
        &renderer,
        Vec3::new(0.0, 0.5, -5.0),
        Vec3::new(3.0, 1.0, 3.0),
        "Start Zone",
        Vec4::new(0.0, 1.0, 0.0, 0.3),
    );
    create_trigger_zone(
        &mut commands,
        &renderer,
        Vec3::new(10.0, 0.5, -5.0),
        Vec3::new(3.0, 1.0, 3.0),
        "Quest Zone",
        Vec4::new(1.0, 1.0, 0.0, 0.3),
    );
    create_trigger_zone(
        &mut commands,
        &renderer,
        Vec3::new(-10.0, 0.5, -5.0),
        Vec3::new(3.0, 1.0, 3.0),
        "Danger Zone",
        Vec4::new(1.0, 0.0, 0.0, 0.3),
    );

    // Create some scattered boxes for testing
    for i in 0..5 {
        let angle = (i as f32 / 5.0) * 2.0 * PI;
        let radius = 15.0;
        let pos = Vec3::new(angle.cos() * radius, 0.5, angle.sin() * radius);
        create_obstacle(&mut commands, &renderer, pos, Vec3::splat(0.5));
    }

    log::info!("Scene setup complete!");
    log::info!("Controls:");
    log::info!("  WASD - Move (camera-relative)");
    log::info!("  Space - Jump");
    log::info!("  Mouse - Look around");
    log::info!("  ESC - Toggle cursor");
}

/// Create the player character
fn create_player(commands: &mut Commands, renderer: &Renderer, position: Vec3) {
    // Create character mesh (capsule-like shape using a colored cube for now)
    let mesh = Mesh::cube(1.0);
    let mut colored_mesh = mesh.clone();
    let player_color = [0.2, 0.5, 1.0, 1.0]; // Blue player
    for vertex in &mut colored_mesh.vertices {
        vertex.color = player_color;
    }

    let mesh_buffers = MeshBuffers::new(&renderer.device, &colored_mesh.vertices, &colored_mesh.indices);
    let material = Material::color(Vec4::from_array(player_color));

    let mut transform = Transform::default();
    transform.position = position;
    transform.scale = Vec3::new(0.6, 1.8, 0.6); // Character proportions

    let model_uniform = ModelUniform::from_transform(&transform);
    let model_buffer = ModelBuffer::new(
        &renderer.device,
        &renderer.model_bind_group_layout,
        &model_uniform,
    );

    // Character controller setup
    let character_controller = CharacterController::new()
        .with_size(0.3, 0.9)
        .with_collision_filtering(
            CollisionLayer::PLAYER,
            CollisionMask::ALL.without_layer(CollisionLayer::TRIGGER),
        );

    let collider = Collider::capsule(0.9, 0.3)
        .with_layer(CollisionLayer::PLAYER)
        .with_mask(CollisionMask::ALL);

    commands.spawn((
        Player,
        character_controller,
        CharacterState::Grounded,
        GroundInfo::default(),
        CharacterMovement::new(),
        Velocity::default(),
        collider,
        colored_mesh,
        mesh_buffers,
        material,
        transform,
        model_buffer,
        CollisionState::new(),
    ));

    log::info!("Player spawned at {:?}", position);
}

/// Create ground plane
fn create_ground(commands: &mut Commands, renderer: &Renderer) {
    let mesh = Mesh::plane(50.0, 10);
    let mesh_buffers = MeshBuffers::new(&renderer.device, &mesh.vertices, &mesh.indices);

    let texture = TextureHandle::create_checkerboard(
        &renderer.device,
        &renderer.queue,
        &renderer.texture_bind_group_layout,
        256,
    );

    let mut transform = Transform::default();
    transform.position = Vec3::new(0.0, 0.0, 0.0);

    let model_uniform = ModelUniform::from_transform(&transform);
    let model_buffer = ModelBuffer::new(
        &renderer.device,
        &renderer.model_bind_group_layout,
        &model_uniform,
    );

    let collider = Collider::box_collider(Vec3::new(25.0, 0.1, 25.0))
        .with_layer(CollisionLayer::ENVIRONMENT);

    let material = Material::textured();

    commands.spawn((
        mesh,
        mesh_buffers,
        texture,
        material,
        transform,
        model_buffer,
        collider,
    ));
}

/// Create an obstacle
fn create_obstacle(
    commands: &mut Commands,
    renderer: &Renderer,
    position: Vec3,
    half_extents: Vec3,
) {
    let mesh = Mesh::cube(1.0);
    let mut colored_mesh = mesh.clone();
    let color = [0.7, 0.3, 0.2, 1.0]; // Orange obstacles
    for vertex in &mut colored_mesh.vertices {
        vertex.color = color;
    }

    let mesh_buffers = MeshBuffers::new(&renderer.device, &colored_mesh.vertices, &colored_mesh.indices);

    let mut transform = Transform::default();
    transform.position = position;
    transform.scale = half_extents * 2.0;

    let model_uniform = ModelUniform::from_transform(&transform);
    let model_buffer = ModelBuffer::new(
        &renderer.device,
        &renderer.model_bind_group_layout,
        &model_uniform,
    );

    let collider = Collider::box_collider(half_extents)
        .with_layer(CollisionLayer::ENVIRONMENT);

    let material = Material::color(Vec4::from_array(color));

    commands.spawn((
        colored_mesh,
        mesh_buffers,
        material,
        transform,
        model_buffer,
        collider,
    ));
}

/// Create a trigger zone
fn create_trigger_zone(
    commands: &mut Commands,
    renderer: &Renderer,
    position: Vec3,
    half_extents: Vec3,
    name: &str,
    color: Vec4,
) {
    let mesh = Mesh::cube(1.0);
    let mut colored_mesh = mesh.clone();
    let color_array = color.to_array();
    for vertex in &mut colored_mesh.vertices {
        vertex.color = color_array;
    }

    let mesh_buffers = MeshBuffers::new(&renderer.device, &colored_mesh.vertices, &colored_mesh.indices);

    let mut transform = Transform::default();
    transform.position = position;
    transform.scale = half_extents * 2.0;

    let model_uniform = ModelUniform::from_transform(&transform);
    let model_buffer = ModelBuffer::new(
        &renderer.device,
        &renderer.model_bind_group_layout,
        &model_uniform,
    );

    let collider = Collider::box_collider(half_extents)
        .with_layer(CollisionLayer::TRIGGER);

    let material = Material::color(color);

    commands.spawn((
        Trigger,
        TriggerZone::new(name),
        colored_mesh,
        mesh_buffers,
        material,
        transform,
        model_buffer,
        collider,
    ));

    log::info!("Created trigger zone: {} at {:?}", name, position);
}

/// Handle character input
fn character_input_system(
    input: Res<Input>,
    camera_rotation: Res<CameraRotation>,
    mut characters: Query<(&mut CharacterMovement, &CharacterState), With<Player>>,
) {
    for (mut movement, state) in characters.iter_mut() {
        // Get input direction in camera space
        let mut input_dir = Vec3::ZERO;

        if input.keyboard.is_pressed(KeyCode::KeyW) {
            input_dir.z -= 1.0;
        }
        if input.keyboard.is_pressed(KeyCode::KeyS) {
            input_dir.z += 1.0;
        }
        if input.keyboard.is_pressed(KeyCode::KeyA) {
            input_dir.x -= 1.0;
        }
        if input.keyboard.is_pressed(KeyCode::KeyD) {
            input_dir.x += 1.0;
        }

        // Transform input direction to world space based on camera yaw
        // Only use yaw (horizontal rotation), not pitch (vertical tilt)
        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, camera_rotation.yaw);
        let world_direction = yaw_rotation * input_dir;

        // Set character movement
        movement.direction = world_direction;
        movement.speed = 5.0;

        // Jump - allow jumping when grounded
        if input.keyboard.just_pressed(KeyCode::Space) && *state == CharacterState::Grounded {
            movement.jump = true;
            // Adjusted for 2x gravity: sqrt(2) * 5.0 = 7.07
            // Maintains same jump height but happens 30% faster
            movement.jump_velocity = 7.07;
        }
    }
}

/// Camera follows character with third-person view
fn camera_follow_character(
    input: Res<Input>,
    cursor_state: Res<CursorState>,
    mut camera_rotation: ResMut<CameraRotation>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    // Update camera rotation from mouse
    if cursor_state.locked {
        let mouse_delta = input.mouse.delta();
        let sensitivity = 0.002;

        camera_rotation.yaw -= mouse_delta.x * sensitivity;
        camera_rotation.pitch -= mouse_delta.y * sensitivity;

        // Clamp pitch
        camera_rotation.pitch = camera_rotation.pitch.clamp(-1.5, 1.5);
    }

    // Position camera behind and above player
    let distance = 8.0;
    let height_offset = 3.0;

    let yaw_quat = Quat::from_axis_angle(Vec3::Y, camera_rotation.yaw);
    let pitch_quat = Quat::from_axis_angle(Vec3::X, camera_rotation.pitch);
    let rotation = yaw_quat * pitch_quat;

    let offset = rotation * Vec3::new(0.0, 0.0, distance);
    camera_transform.position = player_transform.position + offset + Vec3::new(0.0, height_offset, 0.0);
    camera_transform.rotation = rotation;
}

/// System to handle trigger zone events
fn trigger_zone_system(
    collision_tracker: Res<CollisionTracker>,
    players: Query<Entity, With<Player>>,
    triggers: Query<&TriggerZone>,
) {
    let Ok(player_entity) = players.single() else {
        return;
    };

    for event in collision_tracker.events() {
        let (entity_a, entity_b) = event.entities();

        // Check if player is involved
        let trigger_entity = if entity_a == player_entity {
            Some(entity_b)
        } else if entity_b == player_entity {
            Some(entity_a)
        } else {
            None
        };

        if let Some(trigger_entity) = trigger_entity {
            if let Ok(trigger_zone) = triggers.get(trigger_entity) {
                match event {
                    CollisionEvent::Started(_, _) => {
                        log::info!("🚪 Entered trigger zone: {}", trigger_zone.name);
                    }
                    CollisionEvent::Ended(_, _) => {
                        log::info!("🚶 Left trigger zone: {}", trigger_zone.name);
                    }
                }
            }
        }
    }
}

/// Debug info system
fn debug_info_system(
    time: Res<ferrite_core::Time>,
    spatial_grid: Res<SpatialHashGrid>,
    collision_tracker: Res<CollisionTracker>,
    players: Query<(&Transform, &CharacterState, &GroundInfo), With<Player>>,
) {
    // Print debug info every 2 seconds
    if time.elapsed().as_secs_f32() % 2.0 < 0.016 {
        let stats = spatial_grid.stats();

        log::info!("=== Debug Info ===");
        log::info!("Spatial Grid: {} cells, {} entities, {:.2} entities/cell",
            stats.total_cells, stats.total_entities, stats.avg_entities_per_cell);
        log::info!("Collision events this frame: {}", collision_tracker.events().len());

        if let Ok((transform, state, ground_info)) = players.single() {
            log::info!("Player pos: {:?}", transform.position);
            log::info!("Player state: {:?}", state);
            log::info!("Ground distance: {:.2}", ground_info.distance);
        }
    }
}

/// Handle window resize
fn handle_window_resize(
    mut reader: MessageReader<WindowEvent>,
    mut camera_query: Query<&mut Camera, With<MainCamera>>,
) {
    for event in reader.read() {
        if let WindowEvent::Resized { width, height } = event {
            if let Ok(mut camera) = camera_query.single_mut() {
                let new_aspect = *width as f32 / *height as f32;
                camera.set_aspect_ratio(new_aspect);
            }
        }
    }
}

/// Toggle cursor lock
fn cursor_toggle(
    input: Res<Input>,
    window: Option<Res<Window>>,
    mut cursor_state: Option<ResMut<CursorState>>,
) {
    if let (Some(window), Some(cursor_state)) = (window, cursor_state.as_mut()) {
        if input.keyboard.just_pressed(KeyCode::Escape) {
            cursor_state.locked = !cursor_state.locked;

            window.set_cursor_visible(!cursor_state.locked);
            if let Err(e) = window.set_cursor_grab(cursor_state.locked) {
                log::warn!("Failed to set cursor grab: {}", e);
            }
        }
    }
}

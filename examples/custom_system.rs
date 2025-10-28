//! Custom System Example
//!
//! This example shows how to:
//! - Add custom systems to different stages
//! - Query entities and components
//! - Spawn entities with components
//! - Use the Transform component
//!
//! Run with: `cargo run --example custom_system`

use resonance::prelude::*;

fn main() {
    Resonance::new()
        .with_log_level(log::LevelFilter::Info)
        .add_plugin(DefaultPlugins)
        // Add setup system that runs once
        .add_system(Stage::Startup, setup_scene)
        // Add update system that runs every frame
        .add_system(Stage::Update, rotate_cubes)
        .run();
}

/// Setup system - runs once at startup
fn setup_scene(world: &mut World) {
    println!("Setting up scene...");

    // Spawn multiple entities with transforms
    for i in 0..5 {
        let x = (i as f32) * 2.0 - 4.0; // Spread entities along X axis

        world.spawn((
            Transform::from_xyz(x, 0.0, 0.0),
            RotatingCube {
                speed: 1.0 + (i as f32) * 0.5,
            },
        ));
    }

    println!("Spawned {} rotating cubes", 5);
}

/// Update system - runs every frame
fn rotate_cubes(mut query: Query<(&mut Transform, &RotatingCube)>, time: Res<Time>) {
    for (mut transform, cube) in query.iter_mut() {
        // Rotate around Y axis based on time and cube's speed
        let rotation_speed = cube.speed * time.delta();
        transform.rotation = Quat::from_rotation_y(rotation_speed) * transform.rotation;
    }
}

/// Custom component - marks an entity as a rotating cube
#[derive(Component)]
struct RotatingCube {
    speed: f32,
}

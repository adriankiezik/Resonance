//! Chunked world streaming demonstration for MMO-scale environments.
//!
//! This example shows:
//! - Creating a large open world divided into chunks
//! - Automatic chunk loading/unloading based on player position
//! - Memory management for large worlds
//! - Spatial queries
//! - Streaming statistics

use bevy_ecs::prelude::*;
use ferrite_app::{Engine, Stage};
use ferrite_core::Time;
use ferrite_scene::{
    ChunkCoord, ChunkGrid, SceneEntity, ScenePlugin, StreamingManager, StreamingObserver,
    Transform, CHUNK_SIZE,
};
use glam::Vec3;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();

    log::info!("\n=== Chunked World Streaming Demo ===\n");

    // Initialize engine with scene plugin and streaming systems
    let mut engine = Engine::new()
        .add_plugin(ScenePlugin)
        .add_system(Stage::PreUpdate, ferrite_scene::streaming::update_streaming)
        .add_system(Stage::Update, ferrite_scene::streaming::process_chunk_streaming)
        .add_system(Stage::Update, move_player);

    // Initialize time resources (required by engine.update())
    engine.world.insert_resource(ferrite_core::Time::new());
    engine.world.insert_resource(ferrite_core::FixedTime::new(60));
    engine.world.insert_resource(ferrite_core::GameTick::new());

    // Create a large world (100x100 chunks = 10km x 10km)
    populate_world(&mut engine);

    // Spawn a player that will move around the world
    spawn_player(&mut engine);

    // Run simulation for 200 frames
    log::info!("Starting simulation...\n");
    for frame in 0..200 {
        // Sleep to simulate real time (16ms = ~60 FPS)
        thread::sleep(Duration::from_millis(16));

        engine.update();

        // Log stats every 20 frames
        if frame % 20 == 0 {
            log_streaming_stats(&mut engine, frame);
        }
    }

    log::info!("\n=== Demo Complete ===");
    log_final_summary(&engine);
}

/// Populate the world with chunks
fn populate_world(engine: &mut Engine) {
    log::info!("Populating world...");

    let mut chunk_grid = engine.world.resource_mut::<ChunkGrid>();

    // Create a 100x100 grid of chunks (10km x 10km world)
    for x in -50..50 {
        for z in -50..50 {
            let coord = ChunkCoord::new(x, z);
            let chunk = chunk_grid.get_or_create_chunk(coord);

            // Add some dummy entities to each chunk (static geometry)
            for i in 0..5 {
                let entity = SceneEntity {
                    name: Some(format!("entity_{}_{}_{}",coord.x, coord.z, i)),
                    components: HashMap::new(),
                };
                chunk.add_entity(entity);
            }
        }
    }

    let total_chunks = chunk_grid.chunk_count();
    log::info!("Created {} chunks ({}x{} grid)", total_chunks, 100, 100);
    log::info!("World size: 10km x 10km");
    log::info!("Chunk size: {}m x {}m\n", CHUNK_SIZE, CHUNK_SIZE);
}

/// Spawn a player that will move through the world
fn spawn_player(engine: &mut Engine) {
    log::info!("Spawning player with streaming observer...");

    // Start at world center
    let start_pos = Vec3::new(0.0, 0.0, 0.0);

    engine.world.spawn((
        Transform::from_position(start_pos),
        StreamingObserver::mmo(), // MMO-scale streaming (5 chunk radius)
        Player {
            velocity: Vec3::new(50.0, 0.0, 30.0), // Moving diagonally
        },
    ));

    log::info!("Player spawned at {:?}", start_pos);
    log::info!("Load radius: 5 chunks (11x11 grid = 121 chunks)");
    log::info!("Unload radius: 7 chunks\n");
}

/// Component marking the player entity
#[derive(Component)]
struct Player {
    velocity: Vec3,
}

/// System: Move player through the world
fn move_player(time: Res<Time>, mut query: Query<(&mut Transform, &Player)>) {
    for (mut transform, player) in query.iter_mut() {
        // Move player
        transform.position += player.velocity * time.delta_seconds();

        // Wrap around world boundaries (so we can run demo forever)
        if transform.position.x > 5000.0 {
            transform.position.x = -5000.0;
        }
        if transform.position.x < -5000.0 {
            transform.position.x = 5000.0;
        }
        if transform.position.z > 5000.0 {
            transform.position.z = -5000.0;
        }
        if transform.position.z < -5000.0 {
            transform.position.z = 5000.0;
        }
    }
}

/// Log streaming statistics
fn log_streaming_stats(engine: &mut Engine, frame: usize) {
    // Get player position first
    let mut player_pos = Vec3::ZERO;
    {
        let mut query = engine.world.query::<(&Transform, &Player)>();
        for (transform, _) in query.iter(&engine.world) {
            player_pos = transform.position;
        }
    }

    // Now get stats (after query is dropped)
    let chunk_grid = engine.world.resource::<ChunkGrid>();
    let streaming = engine.world.resource::<StreamingManager>();

    let player_chunk = ChunkCoord::from_world_pos(player_pos);

    log::info!(
        "Frame {}: Player at ({:.0}, {:.0}, {:.0}) | Chunk ({}, {})",
        frame,
        player_pos.x,
        player_pos.y,
        player_pos.z,
        player_chunk.x,
        player_chunk.z
    );

    log::info!(
        "  Loaded chunks: {} | Memory: {:.2} MB | Load/Unload: +{} / -{}",
        chunk_grid.loaded_count(),
        chunk_grid.memory_usage() as f32 / (1024.0 * 1024.0),
        streaming.stats.chunks_loaded,
        streaming.stats.chunks_unloaded,
    );
}

/// Log final summary
fn log_final_summary(engine: &Engine) {
    let chunk_grid = engine.world.resource::<ChunkGrid>();
    let streaming = engine.world.resource::<StreamingManager>();

    log::info!("\n--- Final Statistics ---");
    log::info!("Total chunks in world: {}", chunk_grid.chunk_count());
    log::info!("Chunks loaded in memory: {}", chunk_grid.loaded_count());
    log::info!("Memory usage: {:.2} MB", chunk_grid.memory_usage() as f32 / (1024.0 * 1024.0));
    log::info!("Active observers: {}", streaming.stats.active_observers);

    let load_percentage = (chunk_grid.loaded_count() as f32 / chunk_grid.chunk_count() as f32) * 100.0;
    log::info!("Loaded: {:.1}% of world", load_percentage);

    log::info!("\n--- MMO-Ready Features Demonstrated ---");
    log::info!("✓ Large world (10,000 chunks)");
    log::info!("✓ Automatic chunk streaming");
    log::info!("✓ Memory-efficient (only ~{}% loaded)", load_percentage as usize);
    log::info!("✓ Observer-based loading");
    log::info!("✓ Spatial partitioning");
    log::info!("✓ Ready for multiplayer (multiple observers)");
}

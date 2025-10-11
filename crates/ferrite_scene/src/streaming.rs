//! Automatic chunk streaming system for open-world games.
//!
//! Monitors observer positions (players, cameras) and automatically
//! loads/unloads chunks to keep memory usage reasonable while
//! maintaining a seamless world experience.

use bevy_ecs::prelude::*;
use std::collections::HashSet;

use crate::chunk::{ChunkCoord, ChunkGrid};
use crate::Transform;

/// Component marking an entity as a streaming observer (player, camera)
/// Chunks will be loaded around entities with this component
#[derive(Component, Debug)]
pub struct StreamingObserver {
    /// Load radius in chunks (e.g., 3 = load 3 chunks in each direction)
    pub load_radius: i32,
    /// Unload radius in chunks (should be > load_radius to avoid thrashing)
    pub unload_radius: i32,
    /// Last chunk position (for detecting movement)
    last_chunk: Option<ChunkCoord>,
}

impl Default for StreamingObserver {
    fn default() -> Self {
        Self {
            load_radius: 3,      // Load 7x7 grid (49 chunks) ~700m x 700m
            unload_radius: 5,    // Unload beyond 11x11 grid
            last_chunk: None,
        }
    }
}

impl StreamingObserver {
    pub fn new(load_radius: i32, unload_radius: i32) -> Self {
        Self {
            load_radius,
            unload_radius,
            last_chunk: None,
        }
    }

    /// Create observer for MMO scale (large view distance)
    pub fn mmo() -> Self {
        Self::new(5, 7) // 11x11 load, 15x15 unload = ~1km x 1km
    }

    /// Create observer for small multiplayer (moderate view distance)
    pub fn multiplayer() -> Self {
        Self::new(3, 5) // 7x7 load, 11x11 unload = ~700m x 700m
    }

    /// Create observer for single-player (can use larger radius)
    pub fn singleplayer() -> Self {
        Self::new(4, 6) // 9x9 load, 13x13 unload = ~900m x 900m
    }
}

/// Streaming system state and statistics
#[derive(Debug, Resource)]
pub struct StreamingManager {
    /// Chunks that should be loaded (union of all observers)
    pub required_chunks: HashSet<ChunkCoord>,
    /// Statistics
    pub stats: StreamingStats,
}

impl Default for StreamingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingManager {
    pub fn new() -> Self {
        Self {
            required_chunks: HashSet::new(),
            stats: StreamingStats::default(),
        }
    }
}

/// Streaming statistics for debugging and monitoring
#[derive(Debug, Default)]
pub struct StreamingStats {
    /// Number of chunks loaded this frame
    pub chunks_loaded: usize,
    /// Number of chunks unloaded this frame
    pub chunks_unloaded: usize,
    /// Number of active observers
    pub active_observers: usize,
    /// Total chunks in memory
    pub total_loaded: usize,
    /// Memory usage estimate in MB
    pub memory_mb: f32,
}

/// System: Update streaming based on observer positions
/// Should run in PreUpdate stage
pub fn update_streaming(
    mut chunk_grid: ResMut<ChunkGrid>,
    mut streaming: ResMut<StreamingManager>,
    mut observers: Query<(&Transform, &mut StreamingObserver)>,
) {
    // Reset stats
    streaming.stats.chunks_loaded = 0;
    streaming.stats.chunks_unloaded = 0;
    streaming.stats.active_observers = 0;

    // Collect all required chunks from all observers
    let mut required_chunks = HashSet::new();

    for (transform, mut observer) in observers.iter_mut() {
        streaming.stats.active_observers += 1;

        let current_chunk = ChunkCoord::from_world_pos(transform.position);

        // Check if observer moved to a new chunk
        let moved = observer.last_chunk.map_or(true, |last| last != current_chunk);

        if moved {
            observer.last_chunk = Some(current_chunk);

            // Calculate chunks to load (square grid around observer)
            for x in (current_chunk.x - observer.load_radius)..=(current_chunk.x + observer.load_radius) {
                for z in (current_chunk.z - observer.load_radius)..=(current_chunk.z + observer.load_radius) {
                    required_chunks.insert(ChunkCoord::new(x, z));
                }
            }
        } else {
            // Observer didn't move, keep existing required chunks
            // (This is an optimization - in practice, re-calculate every frame is fine)
            for x in (current_chunk.x - observer.load_radius)..=(current_chunk.x + observer.load_radius) {
                for z in (current_chunk.z - observer.load_radius)..=(current_chunk.z + observer.load_radius) {
                    required_chunks.insert(ChunkCoord::new(x, z));
                }
            }
        }
    }

    // Update manager's required chunks
    streaming.required_chunks = required_chunks.clone();

    // Request load for chunks that aren't loaded yet
    for coord in &required_chunks {
        if !chunk_grid.is_loaded(*coord) {
            chunk_grid.request_load(*coord);
        }
    }

    // Request unload for chunks that are loaded but no longer required
    let loaded_chunks: Vec<ChunkCoord> = chunk_grid.loaded_chunks().iter().copied().collect();
    for coord in loaded_chunks {
        if !required_chunks.contains(&coord) {
            // Check if it's beyond unload radius of ALL observers
            let should_unload = observers.iter().all(|(transform, observer)| {
                let observer_chunk = ChunkCoord::from_world_pos(transform.position);
                coord.manhattan_distance(&observer_chunk) > observer.unload_radius
            });

            if should_unload {
                chunk_grid.request_unload(coord);
            }
        }
    }

    // Update stats
    streaming.stats.total_loaded = chunk_grid.loaded_count();
    streaming.stats.memory_mb = chunk_grid.memory_usage() as f32 / (1024.0 * 1024.0);
}

/// System: Process pending chunk loads/unloads
/// This is where you'd integrate with asset system in a real game
/// Should run in Update stage
pub fn process_chunk_streaming(
    mut chunk_grid: ResMut<ChunkGrid>,
    mut streaming: ResMut<StreamingManager>,
) {
    // Process pending loads
    let pending_load: Vec<ChunkCoord> = chunk_grid.pending_load().iter().copied().collect();
    for coord in pending_load {
        // In a real game, this would:
        // 1. Load chunk data from disk (async)
        // 2. Spawn entities from chunk data
        // 3. Initialize physics, rendering, etc.

        // For now, just mark as loaded
        chunk_grid.mark_loaded(coord);
        streaming.stats.chunks_loaded += 1;

        log::debug!("Loaded chunk {:?}", coord);
    }

    // Process pending unloads
    let pending_unload: Vec<ChunkCoord> = chunk_grid.pending_unload().iter().copied().collect();
    for coord in pending_unload {
        // In a real game, this would:
        // 1. Despawn all entities in chunk
        // 2. Save chunk data if modified (dirty flag)
        // 3. Free memory

        if let Some(chunk) = chunk_grid.get_chunk(coord) {
            // Despawn runtime entities
            for _entity in &chunk.spawned_entities {
                // commands.entity(entity).despawn_recursive();
            }
        }

        chunk_grid.mark_unloaded(coord);
        streaming.stats.chunks_unloaded += 1;

        log::debug!("Unloaded chunk {:?}", coord);
    }
}

/// Helper: Get chunk loading priority based on distance from observers
/// Higher priority = should load first
pub fn calculate_chunk_priority(
    coord: ChunkCoord,
    observers: &Query<&Transform, With<StreamingObserver>>,
) -> f32 {
    let mut min_distance = f32::MAX;

    for transform in observers.iter() {
        let observer_chunk = ChunkCoord::from_world_pos(transform.position);
        let distance = coord.distance_squared(&observer_chunk);
        min_distance = min_distance.min(distance);
    }

    // Invert distance so closer chunks have higher priority
    1.0 / (min_distance + 1.0)
}

/// Marker component for entities that should be cleaned up when chunk unloads
#[derive(Component)]
pub struct ChunkEntity {
    pub chunk_coord: ChunkCoord,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_observer_default() {
        let observer = StreamingObserver::default();
        assert_eq!(observer.load_radius, 3);
        assert_eq!(observer.unload_radius, 5);
    }

    #[test]
    fn test_observer_mmo_scale() {
        let observer = StreamingObserver::mmo();
        assert_eq!(observer.load_radius, 5);
        assert_eq!(observer.unload_radius, 7);
    }
}

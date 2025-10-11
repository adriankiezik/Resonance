//! Spatial chunk system for large open worlds.
//!
//! Divides the world into grid cells (chunks) for efficient streaming
//! and memory management. Essential for MMO-scale environments.

use bevy_ecs::prelude::*;
use glam::{IVec2, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::SceneEntity;

/// Size of each chunk in world units (meters)
/// 100m x 100m is a good balance for MMOs
pub const CHUNK_SIZE: f32 = 100.0;

/// Chunk coordinate (grid position)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Convert world position to chunk coordinate
    pub fn from_world_pos(pos: Vec3) -> Self {
        Self {
            x: (pos.x / CHUNK_SIZE).floor() as i32,
            z: (pos.z / CHUNK_SIZE).floor() as i32,
        }
    }

    /// Get center position of this chunk in world space
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.x as f32 + 0.5) * CHUNK_SIZE,
            0.0,
            (self.z as f32 + 0.5) * CHUNK_SIZE,
        )
    }

    /// Get corner position (min bounds) of this chunk
    pub fn min_bounds(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CHUNK_SIZE,
            0.0,
            self.z as f32 * CHUNK_SIZE,
        )
    }

    /// Get corner position (max bounds) of this chunk
    pub fn max_bounds(&self) -> Vec3 {
        Vec3::new(
            (self.x + 1) as f32 * CHUNK_SIZE,
            0.0,
            (self.z + 1) as f32 * CHUNK_SIZE,
        )
    }

    /// Get all 8 neighboring chunk coordinates
    pub fn neighbors(&self) -> [ChunkCoord; 8] {
        [
            ChunkCoord::new(self.x - 1, self.z - 1), // SW
            ChunkCoord::new(self.x, self.z - 1),     // S
            ChunkCoord::new(self.x + 1, self.z - 1), // SE
            ChunkCoord::new(self.x - 1, self.z),     // W
            ChunkCoord::new(self.x + 1, self.z),     // E
            ChunkCoord::new(self.x - 1, self.z + 1), // NW
            ChunkCoord::new(self.x, self.z + 1),     // N
            ChunkCoord::new(self.x + 1, self.z + 1), // NE
        ]
    }

    /// Manhattan distance between chunks
    pub fn manhattan_distance(&self, other: &ChunkCoord) -> i32 {
        (self.x - other.x).abs() + (self.z - other.z).abs()
    }

    /// Squared euclidean distance between chunk centers
    pub fn distance_squared(&self, other: &ChunkCoord) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        dx * dx + dz * dz
    }
}

impl From<IVec2> for ChunkCoord {
    fn from(v: IVec2) -> Self {
        Self { x: v.x, z: v.y }
    }
}

impl From<ChunkCoord> for IVec2 {
    fn from(c: ChunkCoord) -> Self {
        IVec2::new(c.x, c.z)
    }
}

/// A single chunk containing scene data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Chunk grid coordinate
    pub coord: ChunkCoord,
    /// Entities within this chunk (static geometry, spawn points, etc)
    pub entities: Vec<SceneEntity>,
    /// Whether this chunk has been modified (dirty flag for saving)
    #[serde(skip)]
    pub dirty: bool,
    /// Spawned runtime entities (not serialized)
    #[serde(skip)]
    pub spawned_entities: Vec<Entity>,
}

impl Chunk {
    /// Create a new empty chunk
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            entities: Vec::new(),
            dirty: false,
            spawned_entities: Vec::new(),
        }
    }

    /// Add an entity to this chunk
    pub fn add_entity(&mut self, entity: SceneEntity) {
        self.entities.push(entity);
        self.dirty = true;
    }

    /// Check if a world position is within this chunk's bounds
    pub fn contains_position(&self, pos: Vec3) -> bool {
        let min = self.coord.min_bounds();
        let max = self.coord.max_bounds();
        pos.x >= min.x && pos.x < max.x && pos.z >= min.z && pos.z < max.z
    }

    /// Get memory size estimate in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.entities.capacity() * std::mem::size_of::<SceneEntity>()
    }
}

/// Grid-based world manager for chunks
#[derive(Debug, Resource)]
pub struct ChunkGrid {
    /// All chunks in the world (coord -> chunk data)
    chunks: HashMap<ChunkCoord, Chunk>,
    /// Currently loaded chunks (in memory)
    loaded_chunks: HashSet<ChunkCoord>,
    /// Chunks pending load
    pending_load: HashSet<ChunkCoord>,
    /// Chunks pending unload
    pending_unload: HashSet<ChunkCoord>,
}

impl Default for ChunkGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkGrid {
    /// Create a new empty chunk grid
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            loaded_chunks: HashSet::new(),
            pending_load: HashSet::new(),
            pending_unload: HashSet::new(),
        }
    }

    /// Get chunk at coordinate (creates if doesn't exist)
    pub fn get_or_create_chunk(&mut self, coord: ChunkCoord) -> &mut Chunk {
        self.chunks.entry(coord).or_insert_with(|| Chunk::new(coord))
    }

    /// Get chunk at coordinate (read-only)
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    /// Get mutable chunk at coordinate
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    /// Check if chunk is loaded
    pub fn is_loaded(&self, coord: ChunkCoord) -> bool {
        self.loaded_chunks.contains(&coord)
    }

    /// Mark chunk as loaded
    pub fn mark_loaded(&mut self, coord: ChunkCoord) {
        self.loaded_chunks.insert(coord);
        self.pending_load.remove(&coord);
    }

    /// Mark chunk for unloading
    pub fn mark_unloaded(&mut self, coord: ChunkCoord) {
        self.loaded_chunks.remove(&coord);
        self.pending_unload.remove(&coord);
    }

    /// Request chunk load (async/deferred)
    pub fn request_load(&mut self, coord: ChunkCoord) {
        if !self.is_loaded(coord) && !self.pending_load.contains(&coord) {
            self.pending_load.insert(coord);
        }
    }

    /// Request chunk unload (async/deferred)
    pub fn request_unload(&mut self, coord: ChunkCoord) {
        if self.is_loaded(coord) && !self.pending_unload.contains(&coord) {
            self.pending_unload.insert(coord);
        }
    }

    /// Get all chunks within radius of a position
    pub fn get_chunks_in_radius(&self, center: Vec3, radius_chunks: i32) -> Vec<ChunkCoord> {
        let center_coord = ChunkCoord::from_world_pos(center);
        let mut result = Vec::new();

        for x in (center_coord.x - radius_chunks)..=(center_coord.x + radius_chunks) {
            for z in (center_coord.z - radius_chunks)..=(center_coord.z + radius_chunks) {
                let coord = ChunkCoord::new(x, z);
                if coord.manhattan_distance(&center_coord) <= radius_chunks {
                    result.push(coord);
                }
            }
        }

        result
    }

    /// Get all loaded chunks
    pub fn loaded_chunks(&self) -> &HashSet<ChunkCoord> {
        &self.loaded_chunks
    }

    /// Get chunks pending load
    pub fn pending_load(&self) -> &HashSet<ChunkCoord> {
        &self.pending_load
    }

    /// Get chunks pending unload
    pub fn pending_unload(&self) -> &HashSet<ChunkCoord> {
        &self.pending_unload
    }

    /// Clear all pending operations
    pub fn clear_pending(&mut self) {
        self.pending_load.clear();
        self.pending_unload.clear();
    }

    /// Get total number of chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get loaded chunk count
    pub fn loaded_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    /// Get total memory usage estimate
    pub fn memory_usage(&self) -> usize {
        self.chunks.values().map(|c| c.memory_size()).sum()
    }

    /// Spatial query: Get all entities near a position
    pub fn query_entities_near(&self, pos: Vec3, radius: f32) -> Vec<&SceneEntity> {
        let radius_chunks = (radius / CHUNK_SIZE).ceil() as i32;
        let nearby_chunks = self.get_chunks_in_radius(pos, radius_chunks);

        let mut result = Vec::new();

        for coord in nearby_chunks {
            if let Some(chunk) = self.get_chunk(coord) {
                for entity in &chunk.entities {
                    // Note: Would need position component to do proper distance check
                    // For now, just return all entities in nearby chunks
                    result.push(entity);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coord_conversion() {
        assert_eq!(ChunkCoord::from_world_pos(Vec3::new(0.0, 0.0, 0.0)), ChunkCoord::new(0, 0));
        assert_eq!(ChunkCoord::from_world_pos(Vec3::new(50.0, 0.0, 50.0)), ChunkCoord::new(0, 0));
        assert_eq!(ChunkCoord::from_world_pos(Vec3::new(100.0, 0.0, 100.0)), ChunkCoord::new(1, 1));
        assert_eq!(ChunkCoord::from_world_pos(Vec3::new(-50.0, 0.0, -50.0)), ChunkCoord::new(-1, -1));
    }

    #[test]
    fn test_chunk_neighbors() {
        let coord = ChunkCoord::new(0, 0);
        let neighbors = coord.neighbors();
        assert_eq!(neighbors.len(), 8);
        assert!(neighbors.contains(&ChunkCoord::new(-1, -1)));
        assert!(neighbors.contains(&ChunkCoord::new(1, 1)));
    }

    #[test]
    fn test_chunk_radius() {
        let grid = ChunkGrid::new();
        let chunks = grid.get_chunks_in_radius(Vec3::ZERO, 1);
        // 3x3 grid = 9 chunks
        assert_eq!(chunks.len(), 9);
    }
}

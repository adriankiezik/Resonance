//! Spatial partitioning for efficient collision broad-phase.
//!
//! Uses a spatial hash grid to quickly find nearby entities.
//! Essential for open-world MMORPGs with hundreds or thousands of entities.
//!
//! **Performance:**
//! - Insertion: O(1) average case
//! - Query: O(k) where k is entities in queried cells (typically << total entities)
//! - Without spatial partitioning: O(n²) collision checks
//! - With spatial partitioning: O(n*k) where k << n

use bevy_ecs::prelude::*;
use ferrite_core::math::*;
use std::collections::{HashMap, HashSet};

/// A spatial hash key (grid cell coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CellKey {
    x: i32,
    y: i32,
    z: i32,
}

impl CellKey {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Create a cell key from a world position and cell size
    fn from_position(position: Vec3, cell_size: f32) -> Self {
        Self {
            x: (position.x / cell_size).floor() as i32,
            y: (position.y / cell_size).floor() as i32,
            z: (position.z / cell_size).floor() as i32,
        }
    }
}

/// Spatial hash grid for efficient spatial queries
#[derive(Resource)]
pub struct SpatialHashGrid {
    /// Size of each cell in world units
    cell_size: f32,
    /// Map from cell coordinates to entities in that cell
    cells: HashMap<CellKey, HashSet<Entity>>,
    /// Map from entity to its current cell (for efficient removal)
    entity_cells: HashMap<Entity, HashSet<CellKey>>,
}

impl SpatialHashGrid {
    /// Create a new spatial hash grid with given cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            entity_cells: HashMap::new(),
        }
    }

    /// Insert an entity at a position
    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let key = CellKey::from_position(position, self.cell_size);

        // Add entity to cell
        self.cells.entry(key).or_insert_with(HashSet::new).insert(entity);

        // Track which cell this entity is in
        self.entity_cells.entry(entity).or_insert_with(HashSet::new).insert(key);
    }

    /// Insert an entity with a bounding radius (places in multiple cells if large)
    pub fn insert_with_radius(&mut self, entity: Entity, position: Vec3, radius: f32) {
        // Calculate how many cells the entity spans
        let cells_span = (radius / self.cell_size).ceil() as i32;
        let base_key = CellKey::from_position(position, self.cell_size);

        // Insert into all cells the entity might overlap
        for dx in -cells_span..=cells_span {
            for dy in -cells_span..=cells_span {
                for dz in -cells_span..=cells_span {
                    let key = CellKey::new(
                        base_key.x + dx,
                        base_key.y + dy,
                        base_key.z + dz,
                    );

                    self.cells.entry(key).or_insert_with(HashSet::new).insert(entity);
                    self.entity_cells.entry(entity).or_insert_with(HashSet::new).insert(key);
                }
            }
        }
    }

    /// Remove an entity from the grid
    pub fn remove(&mut self, entity: Entity) {
        if let Some(keys) = self.entity_cells.remove(&entity) {
            for key in keys {
                if let Some(cell) = self.cells.get_mut(&key) {
                    cell.remove(&entity);
                    // Remove empty cells to save memory
                    if cell.is_empty() {
                        self.cells.remove(&key);
                    }
                }
            }
        }
    }

    /// Update an entity's position (removes from old cell, inserts in new)
    pub fn update(&mut self, entity: Entity, position: Vec3) {
        self.remove(entity);
        self.insert(entity, position);
    }

    /// Update an entity's position with radius
    pub fn update_with_radius(&mut self, entity: Entity, position: Vec3, radius: f32) {
        self.remove(entity);
        self.insert_with_radius(entity, position, radius);
    }

    /// Query entities within a radius of a position
    pub fn query_radius(&self, position: Vec3, radius: f32) -> Vec<Entity> {
        let mut results = HashSet::new();
        let cells_to_check = (radius / self.cell_size).ceil() as i32 + 1;
        let center_key = CellKey::from_position(position, self.cell_size);

        // Check all cells within radius
        for dx in -cells_to_check..=cells_to_check {
            for dy in -cells_to_check..=cells_to_check {
                for dz in -cells_to_check..=cells_to_check {
                    let key = CellKey::new(
                        center_key.x + dx,
                        center_key.y + dy,
                        center_key.z + dz,
                    );

                    if let Some(cell) = self.cells.get(&key) {
                        results.extend(cell.iter());
                    }
                }
            }
        }

        results.into_iter().collect()
    }

    /// Query entities in a specific cell
    pub fn query_cell(&self, position: Vec3) -> Vec<Entity> {
        let key = CellKey::from_position(position, self.cell_size);

        self.cells
            .get(&key)
            .map(|cell| cell.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get the cell size
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Clear all entities from the grid
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_cells.clear();
    }

    /// Get statistics about the grid (for debugging/profiling)
    pub fn stats(&self) -> SpatialGridStats {
        let total_cells = self.cells.len();
        let total_entities = self.entity_cells.len();
        let entities_per_cell = if total_cells > 0 {
            self.cells.values().map(|cell| cell.len()).sum::<usize>() as f32 / total_cells as f32
        } else {
            0.0
        };

        SpatialGridStats {
            total_cells,
            total_entities,
            avg_entities_per_cell: entities_per_cell,
        }
    }
}

impl Default for SpatialHashGrid {
    fn default() -> Self {
        // Default cell size of 10 units (good for most MMORPG scenarios)
        Self::new(10.0)
    }
}

/// Statistics about the spatial grid
#[derive(Debug, Clone)]
pub struct SpatialGridStats {
    pub total_cells: usize,
    pub total_entities: usize,
    pub avg_entities_per_cell: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_key_from_position() {
        let key = CellKey::from_position(Vec3::new(15.0, 25.0, 35.0), 10.0);
        assert_eq!(key, CellKey::new(1, 2, 3));

        let key = CellKey::from_position(Vec3::new(-15.0, -25.0, -35.0), 10.0);
        assert_eq!(key, CellKey::new(-2, -3, -4));
    }

    #[test]
    fn test_insert_and_query() {
        let mut grid = SpatialHashGrid::new(10.0);
        let entity1 = Entity::from_bits(1);
        let entity2 = Entity::from_bits(2);

        grid.insert(entity1, Vec3::new(5.0, 5.0, 5.0));
        grid.insert(entity2, Vec3::new(15.0, 5.0, 5.0));

        let results = grid.query_cell(Vec3::new(5.0, 5.0, 5.0));
        assert_eq!(results.len(), 1);
        assert!(results.contains(&entity1));
    }

    #[test]
    fn test_query_radius() {
        let mut grid = SpatialHashGrid::new(10.0);
        let entity1 = Entity::from_bits(1);
        let entity2 = Entity::from_bits(2);
        let entity3 = Entity::from_bits(3);

        grid.insert(entity1, Vec3::new(0.0, 0.0, 0.0));
        grid.insert(entity2, Vec3::new(5.0, 0.0, 0.0));
        grid.insert(entity3, Vec3::new(50.0, 0.0, 0.0));

        let results = grid.query_radius(Vec3::ZERO, 20.0);
        assert!(results.contains(&entity1));
        assert!(results.contains(&entity2));
        assert!(!results.contains(&entity3));
    }

    #[test]
    fn test_remove() {
        let mut grid = SpatialHashGrid::new(10.0);
        let entity = Entity::from_bits(1);

        grid.insert(entity, Vec3::new(5.0, 5.0, 5.0));
        assert_eq!(grid.query_cell(Vec3::new(5.0, 5.0, 5.0)).len(), 1);

        grid.remove(entity);
        assert_eq!(grid.query_cell(Vec3::new(5.0, 5.0, 5.0)).len(), 0);
    }

    #[test]
    fn test_update() {
        let mut grid = SpatialHashGrid::new(10.0);
        let entity = Entity::from_bits(1);

        grid.insert(entity, Vec3::new(5.0, 5.0, 5.0));
        grid.update(entity, Vec3::new(15.0, 5.0, 5.0));

        assert_eq!(grid.query_cell(Vec3::new(5.0, 5.0, 5.0)).len(), 0);
        assert_eq!(grid.query_cell(Vec3::new(15.0, 5.0, 5.0)).len(), 1);
    }
}

/// Frustum culling for efficient entity visibility determination.
///
/// Performs CPU-side frustum tests on AABBs to avoid rendering off-screen entities.
/// Combined with distance-based culling to reduce GPU work for infinite worlds.
///
/// Design notes:
/// - Culling happens in PostUpdate, after camera transform is synced
/// - Results are used to build indirect draw buffers for GPU
/// - Entities without AABBs are always rendered (conservative fallback)

use crate::renderer::camera::Frustum;
use crate::renderer::components::Aabb;
use glam::Vec3;

/// Configuration for culling behavior
#[derive(Clone, Copy, Debug)]
pub struct CullingConfig {
    /// Enable frustum culling
    pub enable_frustum: bool,
    /// Maximum distance from camera for entities to be rendered
    /// Set to f32::INFINITY to disable distance culling
    pub max_render_distance: f32,
    /// Grid cell size for spatial partitioning (in world units)
    /// Helps optimize cache locality during frustum tests
    pub grid_cell_size: f32,
}

impl Default for CullingConfig {
    fn default() -> Self {
        Self {
            enable_frustum: true,
            max_render_distance: f32::INFINITY,
            grid_cell_size: 64.0, // Match terrain chunk size
        }
    }
}

/// Result of culling pass
pub struct CullingResult {
    /// Indices into the original entity list of visible entities
    pub visible_indices: Vec<u32>,
    /// Number of entities tested
    pub tested_count: usize,
    /// Number of entities culled by frustum
    pub frustum_culled: usize,
    /// Number of entities culled by distance
    pub distance_culled: usize,
}

/// Performs frustum culling on a set of entities with AABBs
///
/// # Arguments
/// * `frustum` - Camera frustum to test against
/// * `entities_with_aabbs` - List of (entity_index, transform_matrix, aabb)
/// * `camera_pos` - Camera position for distance calculations
/// * `config` - Culling configuration
///
/// # Returns
/// Indices of visible entities in the original list
pub fn frustum_cull_entities(
    frustum: &Frustum,
    entities_data: &[(u32, Vec3, Aabb)], // (index, position, aabb)
    camera_pos: Vec3,
    config: CullingConfig,
) -> CullingResult {
    let mut visible_indices = Vec::new();
    let mut frustum_culled = 0;
    let mut distance_culled = 0;

    let max_dist_sq = config.max_render_distance * config.max_render_distance;
    let enable_frustum = config.enable_frustum;
    let enable_distance = config.max_render_distance.is_finite();

    for (idx, entity_pos, aabb) in entities_data {
        // Quick distance cull first (cheaper than frustum test)
        if enable_distance {
            let to_entity = entity_pos - camera_pos;
            let dist_sq = to_entity.length_squared();
            if dist_sq > max_dist_sq {
                distance_culled += 1;
                continue;
            }
        }

        // Frustum cull
        if enable_frustum {
            let world_aabb_min = aabb.min + *entity_pos;
            let world_aabb_max = aabb.max + *entity_pos;

            if !frustum.contains_aabb(world_aabb_min, world_aabb_max) {
                frustum_culled += 1;
                continue;
            }
        }

        // Entity is visible
        visible_indices.push(*idx);
    }

    CullingResult {
        visible_indices,
        tested_count: entities_data.len(),
        frustum_culled,
        distance_culled,
    }
}

/// Sorts entities by spatial grid cell for improved cache locality
/// This helps the CPU cache during frustum tests
pub fn sort_by_spatial_grid(
    entities_data: &mut [(u32, Vec3, Aabb)],
    grid_cell_size: f32,
) {
    let inv_cell_size = 1.0 / grid_cell_size;

    entities_data.sort_unstable_by_key(|(_, pos, _)| {
        let grid_x = (pos.x * inv_cell_size).floor() as i32;
        let grid_z = (pos.z * inv_cell_size).floor() as i32;
        // Morton code for spatial locality (simple version: just use grid coords)
        // This is a simplified version - full Morton encoding would be more optimal
        (grid_x, grid_z)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_culling() {
        // Create a simple frustum (would normally come from camera)
        // For testing, we'll use default frustum which passes all tests
        let frustum = Frustum {
            planes: [Default::default(); 6],
        };

        let aabb = Aabb {
            min: Vec3::new(-1.0, -1.0, -1.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        };

        let entities = vec![(0u32, Vec3::ZERO, aabb)];
        let camera_pos = Vec3::new(0.0, 0.0, -10.0);

        let result = frustum_cull_entities(
            &frustum,
            &entities,
            camera_pos,
            CullingConfig::default(),
        );

        assert_eq!(result.visible_indices.len(), 1);
    }
}

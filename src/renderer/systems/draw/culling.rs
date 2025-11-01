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

/// Performs frustum culling on a set of entities with pre-computed world-space AABBs
///
/// # Arguments
/// * `frustum` - Camera frustum to test against
/// * `entities_data` - List of (entity_index, world_aabb) with AABBs already in world space
/// * `camera_pos` - Camera position for distance calculations
/// * `config` - Culling configuration
///
/// # Returns
/// Indices of visible entities in the original list
///
/// # Performance Notes
/// - Uses rayon for parallel processing when entity count > 1000
/// - AABBs should be pre-computed in world space to avoid redundant calculations
/// - Distance culling is performed first as it's cheaper than frustum tests
pub fn frustum_cull_entities(
    frustum: &Frustum,
    entities_data: &[(u32, Aabb)], // (index, world_aabb) - AABBs already in world space
    camera_pos: Vec3,
    config: CullingConfig,
) -> CullingResult {
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let max_dist_sq = config.max_render_distance * config.max_render_distance;
    let enable_frustum = config.enable_frustum;
    let enable_distance = config.max_render_distance.is_finite();

    // Use parallel processing for large entity counts
    let use_parallel = entities_data.len() > 1000;

    let frustum_culled = AtomicUsize::new(0);
    let distance_culled = AtomicUsize::new(0);

    let visible_indices: Vec<u32> = if use_parallel {
        entities_data
            .par_iter()
            .filter_map(|(idx, aabb)| {
                // Quick distance cull first (cheaper than frustum test)
                if enable_distance {
                    let aabb_center = (aabb.min + aabb.max) * 0.5;
                    let to_entity = aabb_center - camera_pos;
                    let dist_sq = to_entity.length_squared();
                    if dist_sq > max_dist_sq {
                        distance_culled.fetch_add(1, Ordering::Relaxed);
                        return None;
                    }
                }

                // Frustum cull using pre-computed world-space AABB
                if enable_frustum {
                    if !frustum.contains_aabb(aabb.min, aabb.max) {
                        frustum_culled.fetch_add(1, Ordering::Relaxed);
                        return None;
                    }
                }

                // Entity is visible
                Some(*idx)
            })
            .collect()
    } else {
        // Sequential processing for small entity counts
        let mut result = Vec::new();
        let mut fc = 0;
        let mut dc = 0;

        for (idx, aabb) in entities_data {
            if enable_distance {
                let aabb_center = (aabb.min + aabb.max) * 0.5;
                let to_entity = aabb_center - camera_pos;
                let dist_sq = to_entity.length_squared();
                if dist_sq > max_dist_sq {
                    dc += 1;
                    continue;
                }
            }

            if enable_frustum {
                if !frustum.contains_aabb(aabb.min, aabb.max) {
                    fc += 1;
                    continue;
                }
            }

            result.push(*idx);
        }

        frustum_culled.store(fc, Ordering::Relaxed);
        distance_culled.store(dc, Ordering::Relaxed);
        result
    };

    CullingResult {
        visible_indices,
        tested_count: entities_data.len(),
        frustum_culled: frustum_culled.load(Ordering::Relaxed),
        distance_culled: distance_culled.load(Ordering::Relaxed),
    }
}

/// Sorts entities by spatial grid cell for improved cache locality
/// This helps the CPU cache during frustum tests by processing spatially-close entities together
pub fn sort_by_spatial_grid(
    entities_data: &mut [(u32, Aabb)],
    grid_cell_size: f32,
) {
    let inv_cell_size = 1.0 / grid_cell_size;

    entities_data.sort_unstable_by_key(|(_, aabb)| {
        // Use AABB center for grid position
        let center = (aabb.min + aabb.max) * 0.5;
        let grid_x = (center.x * inv_cell_size).floor() as i32;
        let grid_z = (center.z * inv_cell_size).floor() as i32;
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

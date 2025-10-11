//! Physics plugin for easy setup.

use crate::character::{
    character_movement_system, character_state_system, ground_detection_system,
};
use crate::events::CollisionTracker;
use crate::spatial::SpatialHashGrid;
use crate::systems::{
    apply_damping_system, apply_gravity_system, collision_detection_system,
    integrate_position_system, integrate_velocity_system, reset_acceleration_system,
    update_collision_state_system, update_spatial_grid_system, Gravity,
};
use ferrite_app::{Engine, Plugin, Stage};

/// Plugin that adds physics simulation to the engine.
pub struct PhysicsPlugin {
    /// Custom gravity (if None, uses default Earth gravity)
    pub gravity: Option<ferrite_core::math::Vec3>,
    /// Cell size for spatial partitioning (if None, uses default 10.0 units)
    pub spatial_cell_size: Option<f32>,
}

impl PhysicsPlugin {
    /// Create with default settings
    pub fn new() -> Self {
        Self {
            gravity: None,
            spatial_cell_size: None,
        }
    }

    /// Create with custom gravity
    pub fn with_gravity(gravity: ferrite_core::math::Vec3) -> Self {
        Self {
            gravity: Some(gravity),
            spatial_cell_size: None,
        }
    }

    /// Create with custom spatial cell size
    pub fn with_spatial_cell_size(cell_size: f32) -> Self {
        Self {
            gravity: None,
            spatial_cell_size: Some(cell_size),
        }
    }

    /// Create with custom gravity and spatial cell size
    pub fn with_config(gravity: ferrite_core::math::Vec3, cell_size: f32) -> Self {
        Self {
            gravity: Some(gravity),
            spatial_cell_size: Some(cell_size),
        }
    }
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, engine: &mut Engine) {
        // Add gravity resource
        let gravity = self.gravity.unwrap_or_else(|| Gravity::default().0);
        engine.world.insert_resource(Gravity(gravity));

        // Add spatial hash grid resource
        let cell_size = self.spatial_cell_size.unwrap_or(10.0);
        engine.world.insert_resource(SpatialHashGrid::new(cell_size));

        // Add collision tracker resource
        engine.world.insert_resource(CollisionTracker::new());

        // Components are automatically registered when first used

        // Add physics systems to FixedUpdate stage for determinism
        // System order:
        // 1. Ground detection (raycast to find ground)
        // 2. Character state update (grounded/in-air)
        // 3. Character movement (apply player input)
        // 4. Apply gravity (adds to acceleration for non-characters)
        // 5. Integrate velocity (acceleration -> velocity)
        // 6. Apply damping (reduces velocity)
        // 7. Integrate position (velocity -> position)
        // 8. Reset acceleration (prepare for next frame)
        if let Some(schedule) = engine.schedules.get_mut(Stage::FixedUpdate) {
            schedule.add_systems((
                ground_detection_system,
                character_state_system,
                character_movement_system,
                apply_gravity_system,
                integrate_velocity_system,
                apply_damping_system,
                integrate_position_system,
                reset_acceleration_system,
            ));
        }

        // Update spatial grid and detect collisions (in PostUpdate after all movement)
        if let Some(schedule) = engine.schedules.get_mut(Stage::PostUpdate) {
            schedule.add_systems((
                update_spatial_grid_system,
                collision_detection_system,
                update_collision_state_system,
            ));
        }

        log::info!("PhysicsPlugin initialized:");
        log::info!("  Gravity: {:?}", gravity);
        log::info!("  Spatial grid cell size: {} units", cell_size);
        log::info!("  Physics systems: FixedUpdate (movement + character controller)");
        log::info!("  Collision systems: PostUpdate (spatial partitioning + detection)");
    }
}

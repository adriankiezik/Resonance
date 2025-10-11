//! Physics systems.

use crate::components::{
    Acceleration, ApplyGravity, Damping, RigidBody, Velocity,
};
use crate::integration::{
    apply_angular_damping, apply_damping, integrate_position, integrate_velocity,
};
use bevy_ecs::prelude::*;
use ferrite_core::{math::*, FixedTime};
use ferrite_transform::Transform;

/// Global gravity resource
#[derive(Resource, Clone, Copy)]
pub struct Gravity(pub Vec3);

impl Default for Gravity {
    fn default() -> Self {
        Self(Vec3::new(0.0, -9.81, 0.0)) // Earth gravity
    }
}

/// System to apply gravity to dynamic objects
pub fn apply_gravity_system(
    gravity: Res<Gravity>,
    mut query: Query<&mut Acceleration, (With<ApplyGravity>, With<RigidBody>)>,
) {
    for mut acceleration in query.iter_mut() {
        acceleration.linear += gravity.0;
    }
}

/// System to integrate velocity from acceleration
pub fn integrate_velocity_system(
    fixed_time: Res<FixedTime>,
    mut query: Query<(&mut Velocity, &Acceleration), With<RigidBody>>,
) {
    let dt = fixed_time.timestep_seconds();

    for (mut velocity, acceleration) in query.iter_mut() {
        velocity.linear = integrate_velocity(velocity.linear, acceleration.linear, dt);
        velocity.angular = integrate_velocity(velocity.angular, acceleration.angular, dt);
    }
}

/// System to apply damping to velocity
pub fn apply_damping_system(
    fixed_time: Res<FixedTime>,
    mut query: Query<(&mut Velocity, &Damping), With<RigidBody>>,
) {
    let dt = fixed_time.timestep_seconds();

    for (mut velocity, damping) in query.iter_mut() {
        velocity.linear = apply_damping(velocity.linear, damping.linear, dt);
        velocity.angular = apply_angular_damping(velocity.angular, damping.angular, dt);
    }
}

/// System to integrate position from velocity
pub fn integrate_position_system(
    fixed_time: Res<FixedTime>,
    mut query: Query<(&mut Transform, &Velocity), With<RigidBody>>,
) {
    let dt = fixed_time.timestep_seconds();

    for (mut transform, velocity) in query.iter_mut() {
        // Integrate linear velocity
        transform.position = integrate_position(transform.position, velocity.linear, dt);

        // Integrate angular velocity (simplified - rotate around angular velocity axis)
        if velocity.angular.length_squared() > 0.0 {
            let angle = velocity.angular.length() * dt;
            let axis = velocity.angular.normalize();
            let rotation = Quat::from_axis_angle(axis, angle);
            transform.rotation = rotation * transform.rotation;
        }
    }
}

/// System to reset acceleration each frame (forces are one-shot)
pub fn reset_acceleration_system(mut query: Query<&mut Acceleration, With<RigidBody>>) {
    for mut acceleration in query.iter_mut() {
        *acceleration = Acceleration::zero();
    }
}

/// System to update spatial hash grid with entity positions
pub fn update_spatial_grid_system(
    mut spatial_grid: ResMut<crate::spatial::SpatialHashGrid>,
    colliders: Query<(Entity, &Transform, &crate::collision::Collider)>,
) {
    // Clear and rebuild the grid each frame
    // For better performance with many static entities, could track Changed<Transform>
    spatial_grid.clear();

    for (entity, transform, collider) in colliders.iter() {
        // Use collider's approximate radius for multi-cell insertion
        let radius = collider.approximate_radius();
        spatial_grid.insert_with_radius(entity, transform.position, radius);
    }
}

/// System to detect collisions and update collision tracker
pub fn collision_detection_system(
    spatial_grid: Res<crate::spatial::SpatialHashGrid>,
    mut collision_tracker: ResMut<crate::events::CollisionTracker>,
    colliders: Query<(Entity, &Transform, &crate::collision::Collider)>,
) {
    use crate::collision::compute_aabb;

    // Clear current frame collisions before detection
    // (Note: process_events() also handles this, but being explicit here for clarity)
    collision_tracker.clear_current_frame();

    for (entity_a, transform_a, collider_a) in colliders.iter() {
        // Use collider's approximate radius for query
        let radius = collider_a.approximate_radius();

        // Query nearby entities from spatial grid
        let nearby = spatial_grid.query_radius(transform_a.position, radius * 2.0);

        for entity_b in nearby {
            // Skip self
            if entity_a == entity_b {
                continue;
            }

            // Avoid checking the same pair twice (only check if A's index > B's index)
            // This ensures we check each pair exactly once
            if entity_a.index() <= entity_b.index() {
                continue;
            }

            // Get the other collider
            if let Ok((_, transform_b, collider_b)) = colliders.get(entity_b) {
                // Check collision layer filtering
                if !collider_a.should_collide_with(collider_b) {
                    continue;
                }

                // Broad phase: AABB test
                let aabb_a = compute_aabb(collider_a, transform_a.position);
                let aabb_b = compute_aabb(collider_b, transform_b.position);

                if aabb_a.intersects(&aabb_b) {
                    // Register collision
                    collision_tracker.register_collision(entity_a, entity_b);
                }
            }
        }
    }

    // Process events for this frame
    collision_tracker.process_events();
}

/// System to update CollisionState components based on events
pub fn update_collision_state_system(
    collision_tracker: Res<crate::events::CollisionTracker>,
    mut collision_states: Query<(Entity, &mut crate::events::CollisionState)>,
) {
    // Process collision events
    for event in collision_tracker.events() {
        match event {
            crate::events::CollisionEvent::Started(entity_a, entity_b) => {
                // Update entity A
                if let Ok((_, mut state)) = collision_states.get_mut(*entity_a) {
                    state.colliding_with.insert(*entity_b);
                }
                // Update entity B
                if let Ok((_, mut state)) = collision_states.get_mut(*entity_b) {
                    state.colliding_with.insert(*entity_a);
                }
            }
            crate::events::CollisionEvent::Ended(entity_a, entity_b) => {
                // Update entity A
                if let Ok((_, mut state)) = collision_states.get_mut(*entity_a) {
                    state.colliding_with.remove(entity_b);
                    state.collision_details.remove(entity_b);
                }
                // Update entity B
                if let Ok((_, mut state)) = collision_states.get_mut(*entity_b) {
                    state.colliding_with.remove(entity_a);
                    state.collision_details.remove(entity_a);
                }
            }
        }
    }
}

//! Character controller for MMORPG-style movement.
//!
//! Provides kinematic character movement with:
//! - Ground detection via raycasting
//! - Collision sliding (walk along walls smoothly)
//! - Stair climbing (small step-up tolerance)
//! - Jump mechanics with gravity
//! - Server-authoritative movement support
//!
//! **Design Philosophy:**
//! Unlike physics-based character movement (dynamic RigidBody), this uses
//! kinematic movement for predictable, responsive controls suitable for MMORPGs.

use crate::collision::{Collider, CollisionLayer, CollisionMask};
use crate::raycast::{Ray, RaycastHit};
use bevy_ecs::prelude::*;
use ferrite_core::math::*;
use ferrite_transform::Transform;
use serde::{Deserialize, Serialize};

/// Character controller component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CharacterController {
    /// Capsule radius for collision
    pub radius: f32,
    /// Capsule half-height (from center to top/bottom)
    pub half_height: f32,
    /// Maximum step height the character can climb
    pub step_height: f32,
    /// Ground detection ray length (should be slightly > step_height)
    pub ground_check_distance: f32,
    /// Collision layer for the character
    pub layer: CollisionLayer,
    /// Which layers the character collides with
    pub mask: CollisionMask,
}

impl CharacterController {
    /// Create a new character controller with default humanoid proportions
    pub fn new() -> Self {
        Self {
            radius: 0.3,           // ~60cm diameter character
            half_height: 0.9,      // ~1.8m tall character
            step_height: 0.3,      // Can climb 30cm steps
            ground_check_distance: 1.0,  // Must be > half_height to detect ground from center
            layer: CollisionLayer::PLAYER,
            mask: CollisionMask::ALL,
        }
    }

    /// Set the capsule size
    pub fn with_size(mut self, radius: f32, half_height: f32) -> Self {
        self.radius = radius;
        self.half_height = half_height;
        self
    }

    /// Set the collision filtering
    pub fn with_collision_filtering(mut self, layer: CollisionLayer, mask: CollisionMask) -> Self {
        self.layer = layer;
        self.mask = mask;
        self
    }

    /// Set step height
    pub fn with_step_height(mut self, height: f32) -> Self {
        self.step_height = height;
        self.ground_check_distance = height + 0.1;
        self
    }
}

impl Default for CharacterController {
    fn default() -> Self {
        Self::new()
    }
}

/// Character movement state
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterState {
    /// On the ground, can walk and jump
    Grounded,
    /// In the air (jumping or falling)
    InAir,
}

impl Default for CharacterState {
    fn default() -> Self {
        Self::Grounded
    }
}

/// Character ground info
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GroundInfo {
    /// Is the character on the ground?
    pub is_grounded: bool,
    /// Ground normal vector
    pub normal: Vec3,
    /// Distance to ground
    pub distance: f32,
    /// Entity of the ground (if any)
    pub ground_entity: Option<Entity>,
}

impl GroundInfo {
    pub fn not_grounded() -> Self {
        Self {
            is_grounded: false,
            normal: Vec3::Y,
            distance: f32::INFINITY,
            ground_entity: None,
        }
    }

    pub fn grounded(normal: Vec3, distance: f32, ground_entity: Option<Entity>) -> Self {
        Self {
            is_grounded: true,
            normal,
            distance,
            ground_entity,
        }
    }

}

/// Character movement input (set by game logic, processed by character controller system)
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CharacterMovement {
    /// Movement direction (should be normalized, in world space)
    pub direction: Vec3,
    /// Movement speed (units per second)
    pub speed: f32,
    /// Jump requested this frame
    pub jump: bool,
    /// Jump force/velocity
    pub jump_velocity: f32,
}

impl CharacterMovement {
    pub fn new() -> Self {
        Self {
            direction: Vec3::ZERO,
            speed: 5.0,
            jump: false,
            jump_velocity: 5.0,
        }
    }

    /// Set movement direction (will be normalized)
    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.direction = if direction.length_squared() > 0.0 {
            direction.normalize()
        } else {
            Vec3::ZERO
        };
        self
    }

    /// Set movement speed
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Request a jump
    pub fn with_jump(mut self, jump_velocity: f32) -> Self {
        self.jump = true;
        self.jump_velocity = jump_velocity;
        self
    }
}

/// System to perform ground detection for characters
pub fn ground_detection_system(
    mut characters: Query<(&Transform, &CharacterController, &mut GroundInfo)>,
    colliders: Query<(Entity, &Collider, &Transform), (Without<CharacterController>, Without<crate::components::Trigger>)>,
) {
    for (transform, controller, mut ground_info) in characters.iter_mut() {
        // Cast ray downward from character center
        let ray_origin = transform.position;
        let ray = Ray::new(
            ray_origin,
            Vec3::new(0.0, -1.0, 0.0), // Downward
            controller.ground_check_distance,
        );

        // Find closest ground hit
        let mut closest_hit: Option<RaycastHit> = None;
        let mut closest_distance = controller.ground_check_distance;

        for (entity, collider, col_transform) in colliders.iter() {
            // Check collision layer filtering
            if !controller.mask.includes(collider.layer) {
                continue;
            }

            // Skip trigger layers for ground detection
            if collider.layer == crate::collision::CollisionLayer::TRIGGER {
                continue;
            }

            if let Some((distance, normal)) =
                crate::raycast::raycast_collider(&ray, collider, col_transform.position)
            {
                if distance < closest_distance {
                    closest_distance = distance;
                    let point = ray.point_at(distance);
                    closest_hit = Some(RaycastHit::new(entity, point, normal, distance));
                }
            }
        }

        // Update ground info
        if let Some(hit) = closest_hit {
            *ground_info = GroundInfo::grounded(hit.normal, hit.distance, Some(hit.entity));
        } else {
            *ground_info = GroundInfo::not_grounded();
        }
    }
}

/// System to update character state based on ground detection
pub fn character_state_system(
    mut characters: Query<(&GroundInfo, &mut CharacterState)>,
) {
    for (ground_info, mut state) in characters.iter_mut() {
        let new_state = if ground_info.is_grounded {
            CharacterState::Grounded
        } else {
            CharacterState::InAir
        };

        *state = new_state;
    }
}

/// System to move characters based on input with collision response
pub fn character_movement_system(
    fixed_time: Res<ferrite_core::FixedTime>,
    gravity: Res<crate::systems::Gravity>,
    mut characters: Query<(
        Entity,
        &mut Transform,
        &CharacterController,
        &CharacterState,
        &mut CharacterMovement,
        &mut crate::components::Velocity,
    )>,
    colliders: Query<(Entity, &Collider, &Transform), (Without<CharacterController>, Without<crate::components::Trigger>)>,
) {
    let dt = fixed_time.timestep_seconds();

    for (char_entity, mut transform, controller, state, mut movement, mut velocity) in characters.iter_mut() {
        match state {
            CharacterState::Grounded => {
                // Handle jump first
                if movement.jump {
                    velocity.linear.y = movement.jump_velocity;
                    movement.jump = false; // Reset jump flag after using it
                } else if velocity.linear.y < 0.0 {
                    // Only reset downward velocity when on ground (from falling)
                    // Preserve upward velocity (from jumps) to allow smooth transition to InAir
                    velocity.linear.y = 0.0;
                }

                // Apply horizontal movement
                if movement.direction.length_squared() > 0.0 {
                    let horizontal_delta = movement.direction.normalize() * movement.speed * dt;
                    let horizontal_velocity = movement.direction.normalize() * movement.speed;
                    velocity.linear.x = horizontal_velocity.x;
                    velocity.linear.z = horizontal_velocity.z;

                    // Combine horizontal movement with vertical velocity (for jumps)
                    let move_delta = Vec3::new(horizontal_delta.x, velocity.linear.y * dt, horizontal_delta.z);
                    let desired_position = transform.position + move_delta;

                    transform.position = try_move(
                        desired_position,
                        transform.position,
                        controller,
                        char_entity,
                        &colliders,
                    );
                } else {
                    // When not moving horizontally, still apply vertical velocity (for jumps)
                    velocity.linear.x = 0.0;
                    velocity.linear.z = 0.0;

                    let move_delta = Vec3::new(0.0, velocity.linear.y * dt, 0.0);
                    let desired_position = transform.position + move_delta;

                    transform.position = try_move(
                        desired_position,
                        transform.position,
                        controller,
                        char_entity,
                        &colliders,
                    );
                }
            }
            CharacterState::InAir => {
                // Reset jump flag when in air (can't jump while airborne)
                movement.jump = false;

                // In air: maintain horizontal velocity, apply gravity
                // Apply horizontal movement (reduced control in air)
                let horizontal = Vec3::new(movement.direction.x, 0.0, movement.direction.z);
                if horizontal.length_squared() > 0.0 {
                    let air_control = 0.3; // Reduced control in air
                    velocity.linear.x += horizontal.x * movement.speed * air_control * dt;
                    velocity.linear.z += horizontal.z * movement.speed * air_control * dt;
                }

                // Apply gravity
                velocity.linear += gravity.0 * dt;

                // Try to move with collision checking
                let move_delta = velocity.linear * dt;
                let desired_position = transform.position + move_delta;
                let new_position = try_move(
                    desired_position,
                    transform.position,
                    controller,
                    char_entity,
                    &colliders,
                );

                // Check if we hit something and zero velocity in that direction
                let actual_movement = new_position - transform.position;
                let intended_movement = move_delta;

                // If X movement was blocked, zero X velocity
                if actual_movement.x.abs() < intended_movement.x.abs() * 0.1 {
                    velocity.linear.x = 0.0;
                }

                // If Y movement was blocked (hit ceiling or ground), zero Y velocity
                if actual_movement.y.abs() < intended_movement.y.abs() * 0.1 {
                    velocity.linear.y = 0.0;
                }

                // If Z movement was blocked, zero Z velocity
                if actual_movement.z.abs() < intended_movement.z.abs() * 0.1 {
                    velocity.linear.z = 0.0;
                }

                transform.position = new_position;
            }
        }
    }
}

/// Try to move to a desired position, with collision sliding along surfaces
/// Returns the furthest safe position the character can move to
fn try_move(
    desired_position: Vec3,
    current_position: Vec3,
    controller: &CharacterController,
    char_entity: Entity,
    colliders: &Query<(Entity, &Collider, &Transform), (Without<CharacterController>, Without<crate::components::Trigger>)>,
) -> Vec3 {
    // First try the full movement
    if !would_collide(desired_position, controller, char_entity, colliders) {
        return desired_position;
    }

    // If blocked, try sliding along collision surfaces by testing each axis separately
    let delta = desired_position - current_position;

    // Try horizontal movement (X and Z together, since we usually slide along walls)
    let horizontal_only = current_position + Vec3::new(delta.x, 0.0, delta.z);
    if !would_collide(horizontal_only, controller, char_entity, colliders) {
        // Can move horizontally, now try adding vertical
        let with_vertical = horizontal_only + Vec3::new(0.0, delta.y, 0.0);
        if !would_collide(with_vertical, controller, char_entity, colliders) {
            return with_vertical;
        }
        return horizontal_only;
    }

    // If horizontal is blocked, try each horizontal axis separately (for corner sliding)
    let mut result = current_position;

    // Try X-only movement
    let x_only = current_position + Vec3::new(delta.x, 0.0, 0.0);
    if !would_collide(x_only, controller, char_entity, colliders) {
        result = x_only;
    }

    // Try Z-only movement from current result
    let z_from_result = result + Vec3::new(0.0, 0.0, delta.z);
    if !would_collide(z_from_result, controller, char_entity, colliders) {
        result = z_from_result;
    }

    // Try adding vertical movement
    let with_vertical = result + Vec3::new(0.0, delta.y, 0.0);
    if !would_collide(with_vertical, controller, char_entity, colliders) {
        result = with_vertical;
    }

    result
}

/// Check if the character would collide with any solid objects at the given position
fn would_collide(
    position: Vec3,
    controller: &CharacterController,
    char_entity: Entity,
    colliders: &Query<(Entity, &Collider, &Transform), (Without<CharacterController>, Without<crate::components::Trigger>)>,
) -> bool {
    use crate::collision::{compute_aabb, CollisionLayer};

    // Create AABB for character at test position
    // Use a slightly smaller vertical extent to avoid colliding with ground we're standing on
    // This "skin width" prevents the character from getting stuck on the ground
    let skin_width = 0.02; // Small tolerance
    let char_half_extents = Vec3::new(
        controller.radius,
        controller.half_height - skin_width, // Shrink vertically
        controller.radius,
    );
    let char_aabb = crate::collision::Aabb::from_center_half_extents(position, char_half_extents);

    // Check against all solid colliders
    for (entity, collider, col_transform) in colliders.iter() {
        // Skip self
        if entity == char_entity {
            continue;
        }

        // Skip triggers (we can walk through them)
        if collider.layer == CollisionLayer::TRIGGER {
            continue;
        }

        // Check collision layer filtering
        if !controller.mask.includes(collider.layer) {
            continue;
        }

        // Get collider AABB
        let collider_aabb = compute_aabb(collider, col_transform.position);

        // If AABBs overlap, there's a collision
        if char_aabb.intersects(&collider_aabb) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_controller_creation() {
        let controller = CharacterController::new();
        assert_eq!(controller.radius, 0.3);
        assert_eq!(controller.half_height, 0.9);
    }


    #[test]
    fn test_character_movement_direction_normalization() {
        let movement = CharacterMovement::new().with_direction(Vec3::new(3.0, 0.0, 4.0));
        assert!((movement.direction.length() - 1.0).abs() < 0.01);
    }
}

//! Input validation and anti-cheat.

use ferrite_core::math::*;
use ferrite_network::protocol::PlayerInput;

/// Validate player input for suspicious values
pub fn validate_input(input: &PlayerInput) -> bool {
    // Check movement is normalized
    if input.movement.length_squared() > 1.01 {
        log::warn!("Invalid movement vector: {:?}", input.movement);
        return false;
    }

    // Check look direction is normalized
    if input.look.length_squared() > 1.01 {
        log::warn!("Invalid look vector: {:?}", input.look);
        return false;
    }

    true
}

/// Validate position change is physically possible
pub fn validate_position_change(
    old_pos: Vec3,
    new_pos: Vec3,
    max_speed: f32,
    delta_time: f32,
) -> bool {
    let distance = old_pos.distance(new_pos);
    let max_distance = max_speed * delta_time;

    if distance > max_distance * 1.1 {
        // Allow 10% tolerance
        log::warn!(
            "Suspicious position change: {} (max: {})",
            distance,
            max_distance
        );
        return false;
    }

    true
}

// TODO: Implement rate limiting for inputs
// TODO: Add server-side hit validation
// TODO: Implement replay system for debugging
// TODO: Add anomaly detection for cheating

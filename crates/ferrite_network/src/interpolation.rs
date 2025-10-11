//! Client-side interpolation and prediction.

use ferrite_core::math::*;

/// Interpolate between two positions
pub fn interpolate_position(from: Vec3, to: Vec3, alpha: f32) -> Vec3 {
    from.lerp(to, alpha)
}

/// Interpolate between two rotations
pub fn interpolate_rotation(from: Quat, to: Quat, alpha: f32) -> Quat {
    from.slerp(to, alpha)
}

/// Calculate interpolation alpha between two ticks
pub fn calculate_alpha(current_tick: u64, from_tick: u64, to_tick: u64) -> f32 {
    if to_tick == from_tick {
        return 1.0;
    }
    let progress = (current_tick - from_tick) as f32;
    let total = (to_tick - from_tick) as f32;
    (progress / total).clamp(0.0, 1.0)
}

// TODO: Implement client-side prediction
// - Store input history
// - Predict movement locally
// - Reconcile with server corrections
//
// TODO: Implement lag compensation
// - Rewind server state for hit detection
// - Apply client's latency to determine "when" they shot

//! Physics integration (Euler, Verlet, etc.)

use ferrite_core::math::*;

/// Semi-implicit Euler integration for velocity
pub fn integrate_velocity(velocity: Vec3, acceleration: Vec3, dt: f32) -> Vec3 {
    velocity + acceleration * dt
}

/// Semi-implicit Euler integration for position
pub fn integrate_position(position: Vec3, velocity: Vec3, dt: f32) -> Vec3 {
    position + velocity * dt
}

/// Apply damping to velocity
pub fn apply_damping(velocity: Vec3, damping: f32, dt: f32) -> Vec3 {
    let damping_factor = 1.0 / (1.0 + damping * dt);
    velocity * damping_factor
}

/// Apply angular damping to angular velocity
pub fn apply_angular_damping(angular_velocity: Vec3, damping: f32, dt: f32) -> Vec3 {
    let damping_factor = 1.0 / (1.0 + damping * dt);
    angular_velocity * damping_factor
}

// TODO: Implement more sophisticated integration methods:
// - RK4 (Runge-Kutta 4th order) for better accuracy
// - Verlet integration for stability
// - Constraint solving for realistic collisions

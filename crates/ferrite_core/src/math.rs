//! Math utilities and re-exports.
//!
//! Re-exports glam math types for convenience. Glam is a high-performance
//! math library with SIMD support, perfect for game engines.

pub use glam::*;

/// Common math constants and utilities
pub mod consts {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const EPSILON: f32 = f32::EPSILON;
}

/// Check if two floats are approximately equal
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

// Note: lerp and clamp are now provided by glam's FloatExt trait
// For vectors: Vec2::lerp, Vec3::lerp, Vec4::lerp
// For quaternions: Quat::lerp (linear) or Quat::slerp (spherical)
// For floats: f32::lerp, f64::lerp

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        // Using glam's built-in lerp
        assert_eq!(0.0f32.lerp(10.0, 0.5), 5.0);
        assert_eq!(Vec3::ZERO.lerp(Vec3::ONE, 0.5), Vec3::splat(0.5));
    }

    #[test]
    fn test_clamp() {
        // Using glam's built-in clamp
        assert_eq!(5.0f32.clamp(0.0, 10.0), 5.0);
        assert_eq!((-5.0f32).clamp(0.0, 10.0), 0.0);
        assert_eq!(15.0f32.clamp(0.0, 10.0), 10.0);
    }

    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(1.0, 1.0000001, 0.001));
        assert!(!approx_eq(1.0, 1.1, 0.001));
    }
}

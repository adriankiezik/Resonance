//! Tests for math utilities

use ferrite_core::math::*;

#[test]
fn test_math_constants() {
    use ferrite_core::math::consts::*;

    assert_eq!(PI, std::f32::consts::PI);
    assert_eq!(TAU, std::f32::consts::TAU);
    assert_eq!(TAU, PI * 2.0);
}

#[test]
fn test_approx_eq() {
    use ferrite_core::math::approx_eq;

    // Exactly equal
    assert!(approx_eq(1.0, 1.0, 0.001));

    // Close enough
    assert!(approx_eq(1.0, 1.0001, 0.001));
    assert!(approx_eq(1.0, 0.9999, 0.001));

    // Too far apart
    assert!(!approx_eq(1.0, 1.1, 0.001));
    assert!(!approx_eq(1.0, 0.9, 0.001));

    // Different epsilon values
    assert!(approx_eq(1.0, 1.05, 0.1));
    assert!(!approx_eq(1.0, 1.05, 0.01));
}

#[test]
fn test_vec2_operations() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);

    // Addition
    let sum = a + b;
    assert_eq!(sum, Vec2::new(4.0, 6.0));

    // Subtraction
    let diff = b - a;
    assert_eq!(diff, Vec2::new(2.0, 2.0));

    // Multiplication
    let scaled = a * 2.0;
    assert_eq!(scaled, Vec2::new(2.0, 4.0));

    // Dot product
    let dot = a.dot(b);
    assert_eq!(dot, 11.0); // 1*3 + 2*4 = 11

    // Length
    let len = Vec2::new(3.0, 4.0).length();
    assert_eq!(len, 5.0);

    // Normalize
    let normalized = Vec2::new(3.0, 4.0).normalize();
    assert!((normalized.length() - 1.0).abs() < 0.0001);
}

#[test]
fn test_vec3_operations() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    // Addition
    let sum = a + b;
    assert_eq!(sum, Vec3::new(5.0, 7.0, 9.0));

    // Cross product
    let cross = Vec3::X.cross(Vec3::Y);
    assert_eq!(cross, Vec3::Z);

    let cross = Vec3::Y.cross(Vec3::Z);
    assert_eq!(cross, Vec3::X);

    let cross = Vec3::Z.cross(Vec3::X);
    assert_eq!(cross, Vec3::Y);

    // Dot product
    let dot = a.dot(b);
    assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32

    // Length
    let len = Vec3::new(2.0, 0.0, 0.0).length();
    assert_eq!(len, 2.0);

    // Constants
    assert_eq!(Vec3::ZERO, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(Vec3::ONE, Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(Vec3::X, Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(Vec3::Y, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(Vec3::Z, Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn test_quat_operations() {
    // Identity quaternion
    let identity = Quat::IDENTITY;
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let rotated = identity * vec;
    assert_eq!(rotated, vec);

    // 90 degree rotation around Y axis
    let rot_y = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let vec_x = Vec3::X;
    let rotated = rot_y * vec_x;

    // Should rotate X to -Z
    assert!((rotated.x - 0.0).abs() < 0.0001);
    assert!((rotated.y - 0.0).abs() < 0.0001);
    assert!((rotated.z - (-1.0)).abs() < 0.0001);

    // Quaternion multiplication
    let rot1 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let rot2 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let combined = rot1 * rot2;

    // 90 + 90 = 180 degrees
    let vec_x = Vec3::X;
    let rotated = combined * vec_x;

    // Should rotate X to -X (180 degrees)
    assert!((rotated.x - (-1.0)).abs() < 0.0001);
    assert!((rotated.y - 0.0).abs() < 0.0001);
    assert!((rotated.z - 0.0).abs() < 0.0001);
}

#[test]
fn test_mat4_operations() {
    // Identity matrix
    let identity = Mat4::IDENTITY;
    let vec = Vec3::new(1.0, 2.0, 3.0);
    let transformed = identity.transform_point3(vec);
    assert_eq!(transformed, vec);

    // Translation
    let translation = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
    let transformed = translation.transform_point3(Vec3::ZERO);
    assert_eq!(transformed, Vec3::new(5.0, 0.0, 0.0));

    // Scale
    let scale = Mat4::from_scale(Vec3::splat(2.0));
    let transformed = scale.transform_point3(Vec3::ONE);
    assert_eq!(transformed, Vec3::splat(2.0));

    // Rotation (90 degrees around Y)
    let rotation = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let transformed = rotation.transform_point3(Vec3::X);

    // X should rotate to -Z
    assert!((transformed.x - 0.0).abs() < 0.0001);
    assert!((transformed.z - (-1.0)).abs() < 0.0001);

    // Matrix multiplication
    let mat1 = Mat4::from_translation(Vec3::X);
    let mat2 = Mat4::from_scale(Vec3::splat(2.0));
    let combined = mat1 * mat2;

    let vec = Vec3::ONE;
    let transformed = combined.transform_point3(vec);

    // Should scale then translate: (1,1,1) * 2 = (2,2,2), then + (1,0,0) = (3,2,2)
    assert_eq!(transformed, Vec3::new(3.0, 2.0, 2.0));
}

#[test]
fn test_lerp() {
    // Float lerp
    assert_eq!(0.0f32.lerp(10.0, 0.0), 0.0);
    assert_eq!(0.0f32.lerp(10.0, 0.5), 5.0);
    assert_eq!(0.0f32.lerp(10.0, 1.0), 10.0);

    // Vec3 lerp
    let a = Vec3::ZERO;
    let b = Vec3::ONE;
    assert_eq!(a.lerp(b, 0.0), Vec3::ZERO);
    assert_eq!(a.lerp(b, 0.5), Vec3::splat(0.5));
    assert_eq!(a.lerp(b, 1.0), Vec3::ONE);
}

#[test]
fn test_clamp() {
    // Float clamp
    assert_eq!(5.0f32.clamp(0.0, 10.0), 5.0);
    assert_eq!((-5.0f32).clamp(0.0, 10.0), 0.0);
    assert_eq!(15.0f32.clamp(0.0, 10.0), 10.0);

    // Vec3 clamp
    let vec = Vec3::new(-1.0, 5.0, 15.0);
    let clamped = vec.clamp(Vec3::ZERO, Vec3::splat(10.0));
    assert_eq!(clamped, Vec3::new(0.0, 5.0, 10.0));
}

#[test]
fn test_vector_distance() {
    let a = Vec3::ZERO;
    let b = Vec3::new(3.0, 4.0, 0.0);

    let distance = a.distance(b);
    assert_eq!(distance, 5.0);

    let distance_squared = a.distance_squared(b);
    assert_eq!(distance_squared, 25.0);
}

#[test]
fn test_quaternion_slerp() {
    let start = Quat::IDENTITY;
    let end = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2); // 90 degrees instead of 180

    // Slerp at 0.0 should be start
    let result = start.slerp(end, 0.0);
    assert!((result.x - start.x).abs() < 0.001);
    assert!((result.y - start.y).abs() < 0.001);
    assert!((result.z - start.z).abs() < 0.001);
    assert!((result.w - start.w).abs() < 0.001);

    // Slerp at 1.0 should be end
    let result = start.slerp(end, 1.0);
    // Apply the rotation to a vector to test
    let vec = result * Vec3::X;
    // After 90 degree rotation around Y, X should point to -Z
    assert!((vec.x - 0.0).abs() < 0.001);
    assert!((vec.z - (-1.0)).abs() < 0.001);

    // Slerp at 0.5 should be halfway (45 degrees)
    let result = start.slerp(end, 0.5);
    let vec = result * Vec3::X;

    // At 45 degrees, both X and -Z components should be non-zero and roughly equal
    assert!(vec.x.abs() > 0.5); // Should be around 0.707
    assert!(vec.z.abs() > 0.5); // Should be around -0.707
    assert!((vec.x.abs() - vec.z.abs()).abs() < 0.1); // Should be roughly equal
}

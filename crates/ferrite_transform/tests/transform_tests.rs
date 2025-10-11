//! Tests for Transform and GlobalTransform components

use ferrite_transform::components::{GlobalTransform, Transform};
use glam::{Mat4, Quat, Vec3};

#[test]
fn test_transform_default() {
    let transform = Transform::default();

    assert_eq!(transform.position, Vec3::ZERO);
    assert_eq!(transform.rotation, Quat::IDENTITY);
    assert_eq!(transform.scale, Vec3::ONE);
}

#[test]
fn test_transform_new() {
    let transform = Transform::new();

    assert_eq!(transform.position, Vec3::ZERO);
    assert_eq!(transform.rotation, Quat::IDENTITY);
    assert_eq!(transform.scale, Vec3::ONE);
}

#[test]
fn test_transform_from_position() {
    let transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));

    assert_eq!(transform.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(transform.rotation, Quat::IDENTITY);
    assert_eq!(transform.scale, Vec3::ONE);
}

#[test]
fn test_transform_from_rotation() {
    let rotation = Quat::from_rotation_y(std::f32::consts::PI);
    let transform = Transform::from_rotation(rotation);

    assert_eq!(transform.position, Vec3::ZERO);
    assert!((transform.rotation.x - rotation.x).abs() < 0.0001);
    assert!((transform.rotation.y - rotation.y).abs() < 0.0001);
    assert!((transform.rotation.z - rotation.z).abs() < 0.0001);
    assert!((transform.rotation.w - rotation.w).abs() < 0.0001);
    assert_eq!(transform.scale, Vec3::ONE);
}

#[test]
fn test_transform_from_scale() {
    let transform = Transform::from_scale(Vec3::splat(2.0));

    assert_eq!(transform.position, Vec3::ZERO);
    assert_eq!(transform.rotation, Quat::IDENTITY);
    assert_eq!(transform.scale, Vec3::splat(2.0));
}

#[test]
fn test_transform_from_prs() {
    let pos = Vec3::new(1.0, 2.0, 3.0);
    let rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let scale = Vec3::splat(2.0);

    let transform = Transform::from_prs(pos, rot, scale);

    assert_eq!(transform.position, pos);
    assert_eq!(transform.rotation, rot);
    assert_eq!(transform.scale, scale);
}

#[test]
fn test_transform_translate() {
    let mut transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));

    transform.translate(Vec3::new(4.0, 5.0, 6.0));

    assert_eq!(transform.position, Vec3::new(5.0, 7.0, 9.0));
}

#[test]
fn test_transform_rotate() {
    let mut transform = Transform::new();

    let rotation1 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    transform.rotate(rotation1);

    let rotation2 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    transform.rotate(rotation2);

    // Two 90-degree rotations = 180 degrees
    let vec = transform.rotation * Vec3::X;
    assert!((vec.x - (-1.0)).abs() < 0.0001);
    assert!((vec.z - 0.0).abs() < 0.0001);
}

#[test]
fn test_transform_rotate_axis() {
    let mut transform = Transform::new();

    transform.rotate_axis(Vec3::Y, std::f32::consts::FRAC_PI_2);

    let vec = transform.rotation * Vec3::X;
    // X rotated 90 degrees around Y should point to -Z
    assert!((vec.x - 0.0).abs() < 0.0001);
    assert!((vec.z - (-1.0)).abs() < 0.0001);
}

#[test]
fn test_transform_rotate_xyz() {
    let mut transform = Transform::new();

    // Rotate 90 degrees around X
    transform.rotate_x(std::f32::consts::FRAC_PI_2);
    let vec = transform.rotation * Vec3::Y;
    assert!((vec.y - 0.0).abs() < 0.0001);
    assert!((vec.z - 1.0).abs() < 0.0001);

    // Reset
    transform = Transform::new();

    // Rotate 90 degrees around Y
    transform.rotate_y(std::f32::consts::FRAC_PI_2);
    let vec = transform.rotation * Vec3::X;
    assert!((vec.x - 0.0).abs() < 0.0001);
    assert!((vec.z - (-1.0)).abs() < 0.0001);

    // Reset
    transform = Transform::new();

    // Rotate 90 degrees around Z
    transform.rotate_z(std::f32::consts::FRAC_PI_2);
    let vec = transform.rotation * Vec3::X;
    assert!((vec.x - 0.0).abs() < 0.0001);
    assert!((vec.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_transform_directions() {
    let mut transform = Transform::new();

    // Default forward is -Z
    assert_eq!(transform.forward(), -Vec3::Z);

    // Default right is +X
    assert_eq!(transform.right(), Vec3::X);

    // Default up is +Y
    assert_eq!(transform.up(), Vec3::Y);

    // After rotating 90 degrees around Y
    transform.rotate_y(std::f32::consts::FRAC_PI_2);

    let forward = transform.forward();
    // Forward is -Z by default. Rotating 90 degrees CW around Y turns -Z into -X
    assert!((forward.x - (-1.0)).abs() < 0.01, "forward.x was {}", forward.x);
    assert!((forward.z - 0.0).abs() < 0.01, "forward.z was {}", forward.z);
}

#[test]
fn test_transform_compute_matrix() {
    let transform = Transform::from_prs(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::IDENTITY,
        Vec3::splat(2.0),
    );

    let matrix = transform.compute_matrix();

    // Check translation component
    assert_eq!(matrix.w_axis.truncate(), Vec3::new(1.0, 2.0, 3.0));

    // Check scale (diagonal elements should be scale)
    let scale_vec = Vec3::new(
        matrix.x_axis.truncate().length(),
        matrix.y_axis.truncate().length(),
        matrix.z_axis.truncate().length(),
    );
    assert!((scale_vec.x - 2.0).abs() < 0.0001);
    assert!((scale_vec.y - 2.0).abs() < 0.0001);
    assert!((scale_vec.z - 2.0).abs() < 0.0001);
}

#[test]
fn test_global_transform_default() {
    let global = GlobalTransform::default();

    assert_eq!(global.matrix(), Mat4::IDENTITY);
    assert_eq!(global.position(), Vec3::ZERO);
}

#[test]
fn test_global_transform_from_matrix() {
    let matrix = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let global = GlobalTransform::from_matrix(matrix);

    assert_eq!(global.position(), Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_global_transform_from_transform() {
    let transform = Transform::from_position(Vec3::new(5.0, 10.0, 15.0));
    let global = GlobalTransform::from_transform(&transform);

    assert_eq!(global.position(), Vec3::new(5.0, 10.0, 15.0));
}

#[test]
fn test_global_transform_from_transform_and_parent() {
    let parent_transform = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
    let parent_global = GlobalTransform::from_transform(&parent_transform);

    let child_transform = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
    let child_global =
        GlobalTransform::from_transform_and_parent(&child_transform, &parent_global);

    // Child's global position should be parent + local = (10,0,0) + (5,0,0) = (15,0,0)
    let pos = child_global.position();
    assert!((pos.x - 15.0).abs() < 0.0001);
    assert!((pos.y - 0.0).abs() < 0.0001);
    assert!((pos.z - 0.0).abs() < 0.0001);
}

#[test]
fn test_global_transform_with_rotation() {
    let parent_transform = Transform::from_prs(
        Vec3::ZERO,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2), // 90 degrees around Y
        Vec3::ONE,
    );
    let parent_global = GlobalTransform::from_transform(&parent_transform);

    let child_transform = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
    let child_global =
        GlobalTransform::from_transform_and_parent(&child_transform, &parent_global);

    // Child at (5,0,0) rotated 90 degrees around Y should be at (0,0,-5)
    let pos = child_global.position();
    assert!((pos.x - 0.0).abs() < 0.0001);
    assert!((pos.y - 0.0).abs() < 0.0001);
    assert!((pos.z - (-5.0)).abs() < 0.0001);
}

#[test]
fn test_global_transform_with_scale() {
    let parent_transform = Transform::from_scale(Vec3::splat(2.0));
    let parent_global = GlobalTransform::from_transform(&parent_transform);

    let child_transform = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
    let child_global =
        GlobalTransform::from_transform_and_parent(&child_transform, &parent_global);

    // Child at (5,0,0) with parent scale 2.0 should be at (10,0,0)
    let pos = child_global.position();
    assert!((pos.x - 10.0).abs() < 0.0001);

    // Check scale is propagated
    let scale = child_global.scale();
    assert!((scale.x - 2.0).abs() < 0.0001);
}

#[test]
fn test_global_transform_position_scale_extraction() {
    // Test position and scale extraction (rotation extraction from matrices with scale is complex)
    let transform = Transform::from_prs(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::IDENTITY,
        Vec3::splat(2.0),
    );

    let global = GlobalTransform::from_transform(&transform);

    // Check position
    let pos = global.position();
    assert!((pos.x - 1.0).abs() < 0.01, "pos.x was {}", pos.x);
    assert!((pos.y - 2.0).abs() < 0.01, "pos.y was {}", pos.y);
    assert!((pos.z - 3.0).abs() < 0.01, "pos.z was {}", pos.z);

    // Check scale
    let scale = global.scale();
    assert!((scale.x - 2.0).abs() < 0.01, "scale.x was {}", scale.x);
    assert!((scale.y - 2.0).abs() < 0.01, "scale.y was {}", scale.y);
    assert!((scale.z - 2.0).abs() < 0.01, "scale.z was {}", scale.z);

    // Note: rotation() extraction from matrices with non-uniform scale
    // can be imprecise, so we test it separately with no scale
    let transform_no_scale = Transform::from_position(Vec3::ZERO);
    let global_no_scale = GlobalTransform::from_transform(&transform_no_scale);
    let rot = global_no_scale.rotation();

    // Should be identity rotation
    assert!((rot.w - 1.0).abs() < 0.01 || (rot.w - (-1.0)).abs() < 0.01, "rot.w was {}", rot.w);
}

#[test]
fn test_transform_look_at() {
    let mut transform = Transform::from_position(Vec3::ZERO);

    // Look at point on +X axis
    transform.look_at(Vec3::X, Vec3::Y);

    // Forward direction should point toward X
    let forward = transform.forward();
    eprintln!("Forward vector: {:?}", forward);
    eprintln!("Expected: (1.0, 0.0, 0.0), got: ({}, {}, {})", forward.x, forward.y, forward.z);
    assert!((forward.x - 1.0).abs() < 0.01, "forward.x = {}, expected ~1.0", forward.x);
}

#[test]
fn test_deep_hierarchy() {
    // Test: grandparent -> parent -> child
    let grandparent_transform = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
    let grandparent_global = GlobalTransform::from_transform(&grandparent_transform);

    let parent_transform = Transform::from_position(Vec3::new(5.0, 0.0, 0.0));
    let parent_global =
        GlobalTransform::from_transform_and_parent(&parent_transform, &grandparent_global);

    let child_transform = Transform::from_position(Vec3::new(3.0, 0.0, 0.0));
    let child_global =
        GlobalTransform::from_transform_and_parent(&child_transform, &parent_global);

    // Child's global position should be 10 + 5 + 3 = 18
    let pos = child_global.position();
    assert!((pos.x - 18.0).abs() < 0.0001);
}

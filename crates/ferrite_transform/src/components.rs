//! Transform components for position, rotation, and scale.

use bevy_ecs::prelude::*;
use ferrite_core::math::*;
use serde::{Deserialize, Serialize};

/// Local transform component (relative to parent).
///
/// Represents position, rotation, and scale in local space.
/// If an entity has a parent, this is relative to the parent's transform.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    /// Local position
    pub position: Vec3,
    /// Local rotation (quaternion)
    pub rotation: Quat,
    /// Local scale
    pub scale: Vec3,
}

impl Transform {
    /// Create a new transform at the origin
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a transform with a specific position
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a transform with a specific rotation
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Create a transform with a specific scale
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Create a transform with position, rotation, and scale
    pub fn from_prs(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Translate by a vector
    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
    }

    /// Rotate by a quaternion
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    /// Rotate around an axis by an angle (in radians)
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis, angle));
    }

    /// Rotate around the X axis
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate_axis(Vec3::X, angle);
    }

    /// Rotate around the Y axis
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate_axis(Vec3::Y, angle);
    }

    /// Rotate around the Z axis
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate_axis(Vec3::Z, angle);
    }

    /// Get the forward direction (negative Z)
    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    /// Get the right direction (positive X)
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction (positive Y)
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Compute the transformation matrix
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        // Use glam's look_at function to compute the view matrix
        let view_matrix = Mat4::look_at_rh(self.position, target, up);

        // Extract rotation from the view matrix (inverse it since look_at creates a view matrix)
        let transform_matrix = view_matrix.inverse();
        self.rotation = Quat::from_mat4(&transform_matrix);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// Global (world-space) transform component.
///
/// This is computed from the Transform and parent hierarchy.
/// Should not be modified directly - modify Transform instead.
#[derive(Component, Debug, Clone, Copy)]
pub struct GlobalTransform {
    /// World-space transformation matrix
    matrix: Mat4,
}

impl GlobalTransform {
    /// Create a new global transform from a matrix
    pub fn from_matrix(matrix: Mat4) -> Self {
        Self { matrix }
    }

    /// Create from a local transform (no parent)
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            matrix: transform.compute_matrix(),
        }
    }

    /// Get the transformation matrix
    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }

    /// Get world position
    pub fn position(&self) -> Vec3 {
        self.matrix.w_axis.truncate()
    }

    /// Get world rotation (extracted from matrix)
    pub fn rotation(&self) -> Quat {
        Quat::from_mat4(&self.matrix)
    }

    /// Get world scale (extracted from matrix)
    pub fn scale(&self) -> Vec3 {
        Vec3::new(
            self.matrix.x_axis.truncate().length(),
            self.matrix.y_axis.truncate().length(),
            self.matrix.z_axis.truncate().length(),
        )
    }

    /// Compute from local transform and parent's global transform
    pub fn from_transform_and_parent(transform: &Transform, parent: &GlobalTransform) -> Self {
        Self {
            matrix: parent.matrix * transform.compute_matrix(),
        }
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }
}

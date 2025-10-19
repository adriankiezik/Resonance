
use bevy_ecs::prelude::*;
use crate::core::math::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    pub fn from_prs(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis, angle));
    }

    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate_axis(Vec3::X, angle);
    }

    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate_axis(Vec3::Y, angle);
    }

    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate_axis(Vec3::Z, angle);
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn look_at(&mut self, target: Vec3, up: Vec3) {

        let view_matrix = Mat4::look_at_rh(self.position, target, up);

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

#[derive(Component, Debug, Clone, Copy)]
pub struct GlobalTransform {

    matrix: Mat4,
}

impl GlobalTransform {

    pub fn from_matrix(matrix: Mat4) -> Self {
        Self { matrix }
    }

    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            matrix: transform.compute_matrix(),
        }
    }

    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }

    pub fn position(&self) -> Vec3 {
        self.matrix.w_axis.truncate()
    }

    pub fn rotation(&self) -> Quat {
        Quat::from_mat4(&self.matrix)
    }

    pub fn scale(&self) -> Vec3 {
        Vec3::new(
            self.matrix.x_axis.truncate().length(),
            self.matrix.y_axis.truncate().length(),
            self.matrix.z_axis.truncate().length(),
        )
    }

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

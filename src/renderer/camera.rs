use crate::core::math::*;
use crate::transform::GlobalTransform;
use bevy_ecs::prelude::*;
use bytemuck::{Pod, Zeroable};

#[derive(Component, Debug, Clone, Copy)]
pub struct Camera {
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            aspect,
            near,
            far,
        }
    }

    pub fn perspective(aspect: f32) -> Self {
        Self {
            fov: 45.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    pub fn view_matrix(&self, transform: &GlobalTransform) -> Mat4 {
        transform.matrix().inverse()
    }

    pub fn view_projection_matrix(&self, transform: &GlobalTransform) -> Mat4 {
        self.projection_matrix() * self.view_matrix(transform)
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::perspective(16.0 / 9.0)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, view_proj: Mat4) {
        self.view_proj = view_proj.to_cols_array_2d();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

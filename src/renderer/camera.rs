use crate::core::math::*;
use crate::transform::GlobalTransform;
use bevy_ecs::prelude::*;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn normalize(&mut self) {
        let len = self.normal.length();
        self.normal /= len;
        self.distance /= len;
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn from_view_projection(vp: Mat4) -> Self {
        let mut planes = [
            Plane::new(Vec3::ZERO, 0.0),
            Plane::new(Vec3::ZERO, 0.0),
            Plane::new(Vec3::ZERO, 0.0),
            Plane::new(Vec3::ZERO, 0.0),
            Plane::new(Vec3::ZERO, 0.0),
            Plane::new(Vec3::ZERO, 0.0),
        ];

        let m = vp.to_cols_array_2d();

        planes[0] = Plane::new(
            Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]),
            m[3][3] + m[3][0],
        );
        planes[1] = Plane::new(
            Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]),
            m[3][3] - m[3][0],
        );
        planes[2] = Plane::new(
            Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]),
            m[3][3] + m[3][1],
        );
        planes[3] = Plane::new(
            Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]),
            m[3][3] - m[3][1],
        );
        planes[4] = Plane::new(
            Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]),
            m[3][3] + m[3][2],
        );
        planes[5] = Plane::new(
            Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]),
            m[3][3] - m[3][2],
        );

        for plane in &mut planes {
            plane.normalize();
        }

        Self { planes }
    }

    pub fn contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            let p_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );

            if plane.distance_to_point(p_vertex) < 0.0 {
                return false;
            }
        }
        true
    }

    pub fn fully_contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            let n_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { min.x } else { max.x },
                if plane.normal.y >= 0.0 { min.y } else { max.y },
                if plane.normal.z >= 0.0 { min.z } else { max.z },
            );

            if plane.distance_to_point(n_vertex) < 0.0 {
                return false;
            }
        }
        true
    }
}

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

    pub fn frustum(&self, transform: &GlobalTransform) -> Frustum {
        Frustum::from_view_projection(self.view_projection_matrix(transform))
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

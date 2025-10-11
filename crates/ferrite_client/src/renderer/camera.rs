//! Camera component and systems.

use bevy_ecs::prelude::*;
use bytemuck::{Pod, Zeroable};
use ferrite_core::math::*;
use wgpu::util::DeviceExt;

/// Camera component
#[derive(Component, Debug, Clone)]
pub struct Camera {
    /// Projection matrix
    pub projection: Mat4,
    /// Field of view (radians)
    pub fov: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
    /// Aspect ratio (width / height)
    pub aspect_ratio: f32,
}

impl Camera {
    /// Create a perspective camera
    pub fn perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let projection = Mat4::perspective_rh(fov, aspect_ratio, near, far);
        Self {
            projection,
            fov,
            near,
            far,
            aspect_ratio,
        }
    }

    /// Create an orthographic camera
    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let projection = Mat4::orthographic_rh(left, right, bottom, top, near, far);
        Self {
            projection,
            fov: 0.0,
            near,
            far,
            aspect_ratio: (right - left) / (top - bottom),
        }
    }

    /// Update aspect ratio and recalculate projection
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.projection = Mat4::perspective_rh(self.fov, aspect_ratio, self.near, self.far);
    }

    /// Get view matrix from transform
    pub fn view_matrix(transform: &ferrite_transform::Transform) -> Mat4 {
        Mat4::look_at_rh(
            transform.position,
            transform.position + transform.forward(),
            transform.up(),
        )
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::perspective(
            std::f32::consts::PI / 4.0, // 45 degrees
            16.0 / 9.0,                 // Common aspect ratio
            0.1,
            1000.0,
        )
    }
}

/// Marker component for the main camera
#[derive(Component, Debug, Default)]
pub struct MainCamera;

/// Camera uniform data sent to GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    /// View matrix
    pub view: [[f32; 4]; 4],
    /// Projection matrix
    pub projection: [[f32; 4]; 4],
    /// View-projection matrix (projection * view)
    pub view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    /// Create camera uniform from view and projection matrices
    pub fn new(view: Mat4, projection: Mat4) -> Self {
        let view_projection = projection * view;
        Self {
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            view_projection: view_projection.to_cols_array_2d(),
        }
    }
}

/// GPU camera buffer resource
#[derive(Resource)]
pub struct CameraBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl CameraBuffer {
    /// Create a new camera buffer
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniform: &CameraUniform,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[*uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    /// Update the camera buffer with new uniform data
    pub fn update(&self, queue: &wgpu::Queue, uniform: &CameraUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[*uniform]));
    }
}

/// System to update camera uniform buffer from camera and transform components
pub fn update_camera_system(
    renderer: Option<Res<crate::renderer::Renderer>>,
    camera_buffer: Option<Res<CameraBuffer>>,
    camera_query: Query<(&Camera, &ferrite_transform::Transform), With<MainCamera>>,
) {
    let Some(renderer) = renderer else {
        return;
    };

    let Some(camera_buffer) = camera_buffer else {
        return;
    };

    // Get the main camera
    if let Ok((camera, transform)) = camera_query.single() {
        let view = Camera::view_matrix(transform);
        let uniform = CameraUniform::new(view, camera.projection);
        camera_buffer.update(&renderer.queue, &uniform);
    }
}

/// Model uniform data sent to GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ModelUniform {
    /// Model matrix (local to world transform)
    pub model: [[f32; 4]; 4],
}

impl ModelUniform {
    /// Create model uniform from transform
    pub fn from_transform(transform: &ferrite_transform::Transform) -> Self {
        let model = transform.compute_matrix();
        Self {
            model: model.to_cols_array_2d(),
        }
    }
}

/// GPU model buffer component (per-entity)
#[derive(Component)]
pub struct ModelBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl ModelBuffer {
    /// Create a new model buffer
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniform: &ModelUniform,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&[*uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Model Bind Group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    /// Update the model buffer with new uniform data
    pub fn update(&self, queue: &wgpu::Queue, uniform: &ModelUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[*uniform]));
    }
}

/// System to update model uniform buffers from transform components
pub fn update_model_system(
    renderer: Option<Res<crate::renderer::Renderer>>,
    query: Query<(&ferrite_transform::Transform, &ModelBuffer), Changed<ferrite_transform::Transform>>,
) {
    let Some(renderer) = renderer else {
        return;
    };

    // Update model buffers for entities with changed transforms
    for (transform, model_buffer) in query.iter() {
        let uniform = ModelUniform::from_transform(transform);
        model_buffer.update(&renderer.queue, &uniform);
    }
}

// TODO: Implement camera frustum culling
// TODO: Add camera controller (orbit, first-person, etc.)

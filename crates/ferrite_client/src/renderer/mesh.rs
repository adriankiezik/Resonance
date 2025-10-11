//! Mesh component and vertex data.

use bevy_ecs::prelude::*;
use bytemuck::{Pod, Zeroable};
use ferrite_core::math::*;

/// Vertex format for rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2, color: Vec4) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            uv: uv.to_array(),
            color: color.to_array(),
        }
    }
}

/// Mesh component
#[derive(Component, Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Create a new mesh
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Create a triangle mesh
    pub fn triangle() -> Self {
        let vertices = vec![
            Vertex::new(
                Vec3::new(0.0, 0.5, 0.0),
                Vec3::Z,
                Vec2::new(0.5, 0.0),
                Vec4::ONE,
            ),
            Vertex::new(
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::Z,
                Vec2::new(0.0, 1.0),
                Vec4::ONE,
            ),
            Vertex::new(
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::Z,
                Vec2::new(1.0, 1.0),
                Vec4::ONE,
            ),
        ];
        let indices = vec![0, 1, 2];
        Self::new(vertices, indices)
    }

    /// Create a quad mesh
    pub fn quad() -> Self {
        let vertices = vec![
            Vertex::new(
                Vec3::new(-0.5, 0.5, 0.0),
                Vec3::Z,
                Vec2::new(0.0, 0.0),
                Vec4::ONE,
            ),
            Vertex::new(
                Vec3::new(0.5, 0.5, 0.0),
                Vec3::Z,
                Vec2::new(1.0, 0.0),
                Vec4::ONE,
            ),
            Vertex::new(
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::Z,
                Vec2::new(1.0, 1.0),
                Vec4::ONE,
            ),
            Vertex::new(
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::Z,
                Vec2::new(0.0, 1.0),
                Vec4::ONE,
            ),
        ];
        // Counter-clockwise winding for front-facing triangles
        // Triangle 1: 0 -> 2 -> 1 (top-left -> bottom-right -> top-right)
        // Triangle 2: 0 -> 3 -> 2 (top-left -> bottom-left -> bottom-right)
        let indices = vec![0, 2, 1, 0, 3, 2];
        Self::new(vertices, indices)
    }

    /// Create a plane mesh (XZ plane)
    pub fn plane(size: f32, subdivisions: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let step = size / subdivisions as f32;
        let half_size = size / 2.0;

        // Generate vertices
        for z in 0..=subdivisions {
            for x in 0..=subdivisions {
                let px = -half_size + x as f32 * step;
                let pz = -half_size + z as f32 * step;
                let u = x as f32 / subdivisions as f32;
                let v = z as f32 / subdivisions as f32;

                vertices.push(Vertex::new(
                    Vec3::new(px, 0.0, pz),
                    Vec3::Y, // Normal points up
                    Vec2::new(u, v),
                    Vec4::ONE,
                ));
            }
        }

        // Generate indices (counter-clockwise winding)
        for z in 0..subdivisions {
            for x in 0..subdivisions {
                let top_left = z * (subdivisions + 1) + x;
                let top_right = top_left + 1;
                let bottom_left = top_left + (subdivisions + 1);
                let bottom_right = bottom_left + 1;

                // First triangle: top_left -> bottom_left -> top_right
                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                // Second triangle: top_right -> bottom_left -> bottom_right
                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        Self::new(vertices, indices)
    }

    /// Create a cube mesh
    pub fn cube(size: f32) -> Self {
        let half = size / 2.0;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Define 6 faces with proper normals and UVs
        // Front face (+Z)
        let front_normal = Vec3::Z;
        vertices.extend_from_slice(&[
            Vertex::new(Vec3::new(-half, -half, half), front_normal, Vec2::new(0.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, -half, half), front_normal, Vec2::new(1.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, half, half), front_normal, Vec2::new(1.0, 0.0), Vec4::ONE),
            Vertex::new(Vec3::new(-half, half, half), front_normal, Vec2::new(0.0, 0.0), Vec4::ONE),
        ]);
        indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

        // Back face (-Z)
        let back_normal = Vec3::NEG_Z;
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex::new(Vec3::new(half, -half, -half), back_normal, Vec2::new(0.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(-half, -half, -half), back_normal, Vec2::new(1.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(-half, half, -half), back_normal, Vec2::new(1.0, 0.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, half, -half), back_normal, Vec2::new(0.0, 0.0), Vec4::ONE),
        ]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        // Right face (+X)
        let right_normal = Vec3::X;
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex::new(Vec3::new(half, -half, half), right_normal, Vec2::new(0.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, -half, -half), right_normal, Vec2::new(1.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, half, -half), right_normal, Vec2::new(1.0, 0.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, half, half), right_normal, Vec2::new(0.0, 0.0), Vec4::ONE),
        ]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        // Left face (-X)
        let left_normal = Vec3::NEG_X;
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex::new(Vec3::new(-half, -half, -half), left_normal, Vec2::new(0.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(-half, -half, half), left_normal, Vec2::new(1.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(-half, half, half), left_normal, Vec2::new(1.0, 0.0), Vec4::ONE),
            Vertex::new(Vec3::new(-half, half, -half), left_normal, Vec2::new(0.0, 0.0), Vec4::ONE),
        ]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        // Top face (+Y)
        let top_normal = Vec3::Y;
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex::new(Vec3::new(-half, half, half), top_normal, Vec2::new(0.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, half, half), top_normal, Vec2::new(1.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, half, -half), top_normal, Vec2::new(1.0, 0.0), Vec4::ONE),
            Vertex::new(Vec3::new(-half, half, -half), top_normal, Vec2::new(0.0, 0.0), Vec4::ONE),
        ]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        // Bottom face (-Y)
        let bottom_normal = Vec3::NEG_Y;
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex::new(Vec3::new(-half, -half, -half), bottom_normal, Vec2::new(0.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, -half, -half), bottom_normal, Vec2::new(1.0, 1.0), Vec4::ONE),
            Vertex::new(Vec3::new(half, -half, half), bottom_normal, Vec2::new(1.0, 0.0), Vec4::ONE),
            Vertex::new(Vec3::new(-half, -half, half), bottom_normal, Vec2::new(0.0, 0.0), Vec4::ONE),
        ]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        Self::new(vertices, indices)
    }

    /// Create a UV sphere mesh
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Self {
        use std::f32::consts::PI;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Generate vertices
        for ring in 0..=rings {
            let phi = PI * ring as f32 / rings as f32;
            let y = radius * phi.cos();
            let ring_radius = radius * phi.sin();

            for segment in 0..=segments {
                let theta = 2.0 * PI * segment as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                let position = Vec3::new(x, y, z);
                let normal = position.normalize();
                let u = segment as f32 / segments as f32;
                let v = ring as f32 / rings as f32;

                vertices.push(Vertex::new(position, normal, Vec2::new(u, v), Vec4::ONE));
            }
        }

        // Generate indices
        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;

                // First triangle (counter-clockwise)
                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                // Second triangle (counter-clockwise)
                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self::new(vertices, indices)
    }
}

// TODO: Implement mesh asset loading from files (OBJ, GLTF)
// TODO: Add mesh optimization (deduplication, LOD, etc.)

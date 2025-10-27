use crate::assets::handle::AssetId;
use crate::assets::loader::mesh::MeshData;
use crate::core::math::*;
use bevy_ecs::prelude::Resource;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, BufferUsages, Device};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
    pub ao: f32,
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }

    pub fn from_data(position: Vec3, normal: Vec3, uv: Vec2, color: Vec3, ao: f32) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            uv: uv.to_array(),
            color: color.to_array(),
            ao,
        }
    }
}

pub struct GpuMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl GpuMesh {
    pub fn from_mesh_data(device: &Device, mesh_data: &MeshData) -> Self {
        let vertices: Vec<Vertex> = (0..mesh_data.positions.len())
            .map(|i| {
                let color = mesh_data.colors.get(i).copied().unwrap_or(Vec3::ONE);
                let ao = mesh_data.ao_values.get(i).copied().unwrap_or(1.0);
                Vertex::from_data(
                    mesh_data.positions[i],
                    mesh_data.normals[i],
                    mesh_data.uvs[i],
                    color,
                    ao,
                )
            })
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let optimized_indices =
            meshopt::optimize_vertex_cache(&mesh_data.indices, vertices.len());

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&optimized_indices),
            usage: BufferUsages::INDEX,
        });

        let index_count = optimized_indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}

#[derive(Resource, Default)]
pub struct GpuMeshCache {
    meshes: HashMap<AssetId, Arc<GpuMesh>>,
}

impl GpuMeshCache {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: AssetId, mesh: GpuMesh) {
        self.meshes.insert(id, Arc::new(mesh));
    }

    pub fn get(&self, id: &AssetId) -> Option<Arc<GpuMesh>> {
        self.meshes.get(id).cloned()
    }

    pub fn contains(&self, id: &AssetId) -> bool {
        self.meshes.contains_key(id)
    }

    pub fn remove(&mut self, id: &AssetId) -> Option<Arc<GpuMesh>> {
        self.meshes.remove(id)
    }

    pub fn clear(&mut self) {
        self.meshes.clear();
    }

    pub fn iter_ids(&self) -> impl Iterator<Item = AssetId> + '_ {
        self.meshes.keys().copied()
    }

    pub fn len(&self) -> usize {
        self.meshes.len()
    }
}

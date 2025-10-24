use crate::assets::handle::{AssetHandle, AssetId};
use crate::assets::loader::mesh::MeshData;
use crate::core::math::*;
use bevy_ecs::prelude::{Component, Resource};
use wgpu::{BindGroup, Buffer};

#[derive(Component, Clone)]
pub struct Mesh {
    pub handle: AssetHandle<Vec<MeshData>>,
    pub mesh_index: usize,
}

impl Mesh {
    pub fn new(handle: AssetHandle<Vec<MeshData>>) -> Self {
        Self {
            handle,
            mesh_index: 0,
        }
    }

    pub fn with_index(handle: AssetHandle<Vec<MeshData>>, mesh_index: usize) -> Self {
        Self { handle, mesh_index }
    }
}

#[derive(Component)]
pub struct MeshUploaded;

#[derive(Component, Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_positions(positions: &[Vec3]) -> Self {
        if positions.is_empty() {
            return Self::new(Vec3::ZERO, Vec3::ZERO);
        }

        let mut min = positions[0];
        let mut max = positions[0];

        for &pos in positions.iter().skip(1) {
            min = min.min(pos);
            max = max.max(pos);
        }

        Self { min, max }
    }

    pub fn transform(&self, transform_matrix: Mat4) -> Self {
        let corners = [
            transform_matrix.transform_point3(Vec3::new(self.min.x, self.min.y, self.min.z)),
            transform_matrix.transform_point3(Vec3::new(self.min.x, self.min.y, self.max.z)),
            transform_matrix.transform_point3(Vec3::new(self.min.x, self.max.y, self.min.z)),
            transform_matrix.transform_point3(Vec3::new(self.min.x, self.max.y, self.max.z)),
            transform_matrix.transform_point3(Vec3::new(self.max.x, self.min.y, self.min.z)),
            transform_matrix.transform_point3(Vec3::new(self.max.x, self.min.y, self.max.z)),
            transform_matrix.transform_point3(Vec3::new(self.max.x, self.max.y, self.min.z)),
            transform_matrix.transform_point3(Vec3::new(self.max.x, self.max.y, self.max.z)),
        ];

        let mut min = corners[0];
        let mut max = corners[0];

        for &corner in corners.iter().skip(1) {
            min = min.min(corner);
            max = max.max(corner);
        }

        Self { min, max }
    }
}

#[derive(Component)]
pub struct GpuModelData {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

#[derive(Resource)]
pub struct LightingData {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

#[derive(Resource)]
pub struct ModelStorageData {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub capacity: usize,
    pub entity_count: usize,
}

pub struct MeshDrawBatch {
    pub mesh_id: AssetId,
    pub indirect_buffer: Buffer,
    pub draw_count: u32,
    pub base_instance: u32,
    pub visible_instances: Vec<u32>,
    pub buffer_capacity: u32,
}

#[derive(Resource)]
pub struct IndirectDrawData {
    pub batches: Vec<MeshDrawBatch>,
}

#[derive(Resource)]
pub struct SsaoBindGroupCache {
    pub bind_group: BindGroup,
}

#[derive(Resource)]
pub struct CachedFrustum {
    pub frustum: crate::renderer::camera::Frustum,
    pub camera_transform: Mat4,
}

#[derive(Resource)]
pub struct CachedOctree {
    pub octree: crate::renderer::octree::Octree,
    pub octree_entities: Vec<crate::renderer::octree::OctreeEntity>,
    pub entity_count: usize,
    pub entity_hash: u64,
    pub last_visible: std::collections::HashSet<bevy_ecs::entity::Entity>,
}

use crate::assets::handle::AssetHandle;
use crate::assets::mesh_loader::MeshData;
use bevy_ecs::prelude::Component;
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
        Self {
            handle,
            mesh_index,
        }
    }
}

#[derive(Component)]
pub struct MeshUploaded;

#[derive(Component)]
pub struct GpuModelData {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

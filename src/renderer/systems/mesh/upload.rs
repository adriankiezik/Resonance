use crate::renderer::{GpuMeshCache, Renderer, components::{Mesh, MeshUploaded}, mesh::GpuMesh};
use bevy_ecs::prelude::*;

pub fn upload_meshes(
    mut commands: Commands,
    renderer: Option<Res<Renderer>>,
    mut gpu_mesh_cache: Option<ResMut<GpuMeshCache>>,
    mut memory_tracker: Option<ResMut<crate::core::MemoryTracker>>,
    query: Query<(Entity, &Mesh), Without<MeshUploaded>>,
) {
    let Some(renderer) = renderer else {
        return;
    };
    let Some(ref mut gpu_mesh_cache) = gpu_mesh_cache else {
        return;
    };

    let device = renderer.device();

    for (entity, mesh) in query.iter() {
        if gpu_mesh_cache.contains(&mesh.handle.id) {
            commands.entity(entity).insert(MeshUploaded);
            continue;
        }

        let mesh_data_vec = &mesh.handle.asset;
        if mesh_data_vec.is_empty() {
            continue;
        }
        if mesh.mesh_index < mesh_data_vec.len() {
            let mesh_data = &mesh_data_vec[mesh.mesh_index];
            let gpu_mesh = GpuMesh::from_mesh_data(device, mesh_data);

            let vertex_size = (mesh_data.positions.len() * std::mem::size_of::<crate::renderer::Vertex>()) as u64;
            let index_size = (mesh_data.indices.len() * std::mem::size_of::<u32>()) as u64;

            gpu_mesh_cache.insert(mesh.handle.id, gpu_mesh);
            commands.entity(entity).insert(MeshUploaded);

            if let Some(ref mut tracker) = memory_tracker {
                tracker.track_mesh_gpu(mesh.handle.id, vertex_size, index_size);
            }

            log::debug!(
                "Uploaded mesh: {:?} (vertices: {}, indices: {})",
                mesh.handle.id,
                mesh_data.positions.len(),
                mesh_data.indices.len()
            );
        } else {
            log::error!(
                "Mesh index {} out of bounds for entity {:?} (asset {:?} has {} meshes)",
                mesh.mesh_index,
                entity,
                mesh.handle.id,
                mesh_data_vec.len()
            );
        }
    }
}

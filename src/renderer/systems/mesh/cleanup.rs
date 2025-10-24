use crate::assets::handle::AssetId;
use crate::renderer::{GpuMeshCache, components::{Mesh, MeshUploaded}};
use bevy_ecs::prelude::*;
use std::collections::HashSet;

pub fn cleanup_unused_meshes(
    mut gpu_mesh_cache: Option<ResMut<GpuMeshCache>>,
    mut memory_tracker: Option<ResMut<crate::core::MemoryTracker>>,
    mesh_query: Query<&Mesh>,
) {
    let Some(ref mut gpu_mesh_cache) = gpu_mesh_cache else {
        return;
    };
    let active_mesh_ids: HashSet<AssetId> = mesh_query.iter().map(|mesh| mesh.handle.id).collect();

    let cached_ids: Vec<AssetId> = gpu_mesh_cache.iter_ids().collect();

    for mesh_id in cached_ids {
        if !active_mesh_ids.contains(&mesh_id) {
            if let Some(_) = gpu_mesh_cache.remove(&mesh_id) {
                if let Some(ref mut tracker) = memory_tracker {
                    tracker.untrack_mesh_gpu(&mesh_id);
                }

                log::debug!(
                    "Cleaned up GPU mesh: {:?} (no longer referenced)",
                    mesh_id
                );
            }
        }
    }
}

pub fn cleanup_mesh_components(
    mut commands: Commands,
    query: Query<Entity, (Without<Mesh>, With<MeshUploaded>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<MeshUploaded>();
    }
}

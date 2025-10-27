use crate::assets::handle::AssetId;
use crate::renderer::{
    GpuMeshCache, MeshPipeline, Renderer,
    components::{Aabb, IndirectDrawData, Mesh, MeshUploaded, ModelStorageData},
};
use crate::transform::GlobalTransform;
use bevy_ecs::prelude::*;

use super::utils::{batching, storage};

pub fn prepare_indirect_draw_data(
    mut commands: Commands,
    renderer: Option<Res<Renderer>>,
    pipeline: Option<Res<MeshPipeline>>,
    gpu_mesh_cache: Option<Res<GpuMeshCache>>,
    existing_storage: Option<ResMut<ModelStorageData>>,
    existing_indirect: Option<ResMut<IndirectDrawData>>,
    mut profiler: Option<ResMut<crate::core::Profiler>>,
    changed_query: Query<(Entity, &Mesh, &GlobalTransform, Option<&Aabb>), (With<MeshUploaded>, Changed<GlobalTransform>)>,
    all_query: Query<(Entity, &Mesh, &GlobalTransform, Option<&Aabb>), With<MeshUploaded>>,
) {
    let _start = std::time::Instant::now();

    let Some(renderer) = renderer else { return };
    let Some(pipeline) = pipeline else { return };
    let Some(gpu_mesh_cache) = gpu_mesh_cache else { return };

    let device = renderer.device();
    let queue = renderer.queue();
    let transforms_changed = !changed_query.is_empty();

    // Collect all entities - no culling, render everything
    let mut all_entities: Vec<(Entity, AssetId, GlobalTransform, Option<Aabb>)> = all_query
        .iter()
        .map(|(entity, mesh, transform, aabb)| (entity, mesh.handle.id, *transform, aabb.copied()))
        .collect();

    all_entities.sort_unstable_by_key(|(entity, mesh_id, _, _)| (mesh_id.0, *entity));

    let total_count = all_entities.len();
    if total_count == 0 {
        cleanup_resources(&mut commands, existing_storage, existing_indirect);
        return;
    }

    // Render all entities - no visibility filtering
    let visible_entities: Vec<u32> = (0..total_count as u32).collect();
    let mesh_groups = group_visible_meshes(&all_entities, &visible_entities);

    if transforms_changed && existing_storage.is_some() {
        if let Some(storage_data) = &existing_storage {
            if storage_data.entity_count == total_count {
                storage::update_changed_uniforms(
                    queue,
                    &storage_data.buffer,
                    &all_entities,
                    &changed_query.iter().map(|(e, _, _, _)| e).collect(),
                );

                let batches = batching::create_draw_batches(
                    device,
                    queue,
                    &gpu_mesh_cache,
                    mesh_groups,
                    existing_indirect.as_ref().map(|d| d.batches.as_slice()),
                );

                if !batches.is_empty() {
                    commands.insert_resource(IndirectDrawData { batches });
                }

                record_profiling(&mut profiler, _start);
                return;
            }
        }
    }

    let model_uniforms = storage::compute_model_uniforms(&all_entities);

    if try_update_existing_storage(
        &mut commands,
        device,
        queue,
        &gpu_mesh_cache,
        &existing_storage,
        &existing_indirect,
        &model_uniforms,
        total_count,
        mesh_groups.clone(),
    ) {
        record_profiling(&mut profiler, _start);
        return;
    }

    storage::update_or_create_storage_buffer(
        &mut commands,
        device,
        queue,
        &pipeline,
        existing_storage,
        &model_uniforms,
        total_count,
    );

    let batches = batching::create_draw_batches(
        device,
        queue,
        &gpu_mesh_cache,
        mesh_groups,
        None,
    );

    if !batches.is_empty() {
        commands.insert_resource(IndirectDrawData { batches });
    }

    record_profiling(&mut profiler, _start);
}

fn cleanup_resources(
    commands: &mut Commands,
    existing_storage: Option<ResMut<ModelStorageData>>,
    existing_indirect: Option<ResMut<IndirectDrawData>>,
) {
    if existing_storage.is_some() {
        commands.remove_resource::<ModelStorageData>();
    }
    if existing_indirect.is_some() {
        commands.remove_resource::<IndirectDrawData>();
    }
}

fn group_visible_meshes(
    all_entities: &[(Entity, AssetId, GlobalTransform, Option<Aabb>)],
    visible_instances: &[u32],
) -> ahash::AHashMap<AssetId, Vec<u32>> {
    let mut mesh_groups: ahash::AHashMap<AssetId, Vec<u32>> = ahash::AHashMap::new();

    for &idx in visible_instances {
        let idx_usize = idx as usize;
        if idx_usize < all_entities.len() {
            let (_entity, mesh_id, _, _) = &all_entities[idx_usize];
            mesh_groups
                .entry(*mesh_id)
                .or_default()
                .push(idx);
        }
    }

    mesh_groups
}

fn try_update_existing_storage(
    commands: &mut Commands,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_mesh_cache: &GpuMeshCache,
    existing_storage: &Option<ResMut<ModelStorageData>>,
    existing_indirect: &Option<ResMut<IndirectDrawData>>,
    model_uniforms: &[crate::renderer::ModelUniform],
    total_count: usize,
    mesh_groups: ahash::AHashMap<AssetId, Vec<u32>>,
) -> bool {
    let Some(storage_data) = existing_storage else {
        return false;
    };

    if storage_data.entity_count != total_count {
        return false;
    }

    queue.write_buffer(
        &storage_data.buffer,
        0,
        bytemuck::cast_slice(model_uniforms),
    );

    if let Some(existing) = existing_indirect {
        if can_reuse_indirect_buffers(existing, &mesh_groups) {
            return true;
        }
    }

    let batches = batching::create_draw_batches(
        device,
        queue,
        gpu_mesh_cache,
        mesh_groups,
        existing_indirect.as_ref().map(|d| d.batches.as_slice()),
    );

    if !batches.is_empty() {
        commands.insert_resource(IndirectDrawData { batches });
    }

    true
}

fn can_reuse_indirect_buffers(
    existing_indirect: &IndirectDrawData,
    mesh_groups: &ahash::AHashMap<AssetId, Vec<u32>>,
) -> bool {
    if existing_indirect.batches.len() != mesh_groups.len() {
        return false;
    }

    for existing_batch in &existing_indirect.batches {
        if let Some(new_instances) = mesh_groups.get(&existing_batch.mesh_id) {
            if existing_batch.visible_instances != *new_instances {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

fn record_profiling(profiler: &mut Option<ResMut<crate::core::Profiler>>, start_time: std::time::Instant) {
    if let Some(profiler) = profiler {
        profiler.record_timing(
            "PostUpdate::prepare_indirect_draw_data".to_string(),
            start_time.elapsed(),
        );
    }
}

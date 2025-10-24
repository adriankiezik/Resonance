use crate::assets::handle::AssetId;
use crate::renderer::{
    Camera, GpuMeshCache, MeshPipeline, Renderer,
    components::{Aabb, IndirectDrawData, Mesh, MeshUploaded, ModelStorageData},
};
use crate::transform::GlobalTransform;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

use super::utils::{batching, frustum, storage, visibility};

pub fn prepare_indirect_draw_data(
    mut commands: Commands,
    renderer: Option<Res<Renderer>>,
    pipeline: Option<Res<MeshPipeline>>,
    gpu_mesh_cache: Option<Res<GpuMeshCache>>,
    existing_storage: Option<ResMut<ModelStorageData>>,
    existing_indirect: Option<ResMut<IndirectDrawData>>,
    cached_frustum: Option<ResMut<crate::renderer::components::CachedFrustum>>,
    mut cached_octree: Option<ResMut<crate::renderer::components::CachedOctree>>,
    mut profiler: Option<ResMut<crate::core::Profiler>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
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

    let changed_entities: HashSet<Entity> = changed_query.iter().map(|(e, _, _, _)| e).collect();

    let (frustum, camera_changed) = if let Some((camera, transform)) = camera_query.iter().next() {
        frustum::calculate_frustum_with_cache(&mut commands, cached_frustum, camera, transform)
    } else {
        (None, false)
    };

    if !transforms_changed && !camera_changed && cached_octree.is_some() && existing_storage.is_some() && existing_indirect.is_some() {
        record_profiling(&mut profiler, _start);
        return;
    }

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

    let visible_entities = if let Some(ref frustum) = frustum {
        let visible_set = visibility::update_octree_and_visibility(
            &mut commands,
            &mut cached_octree,
            &all_entities,
            frustum,
            transforms_changed,
        );

        if try_fast_visibility_update(
            &mut commands,
            device,
            queue,
            &gpu_mesh_cache,
            &all_entities,
            &visible_set,
            &mut cached_octree,
            &existing_storage,
            &existing_indirect,
            transforms_changed,
            &mut profiler,
            _start,
        ) {
            return;
        }

        visible_set
    } else {
        all_entities.iter().map(|(entity, _, _, _)| *entity).collect()
    };

    let mesh_groups = group_visible_meshes(&all_entities, &visible_entities);

    if transforms_changed && existing_storage.is_some() {
        if let Some(storage_data) = &existing_storage {
            if storage_data.entity_count == total_count {
                storage::update_changed_uniforms(
                    queue,
                    &storage_data.buffer,
                    &all_entities,
                    &changed_entities,
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
    visible_entities: &HashSet<Entity>,
) -> ahash::AHashMap<AssetId, Vec<u32>> {
    let mut mesh_groups: ahash::AHashMap<AssetId, Vec<u32>> = ahash::AHashMap::new();

    for (idx, (entity, mesh_id, _, _)) in all_entities.iter().enumerate() {
        if visible_entities.contains(entity) {
            mesh_groups
                .entry(*mesh_id)
                .or_default()
                .push(idx as u32);
        }
    }

    mesh_groups
}

#[allow(clippy::too_many_arguments)]
fn try_fast_visibility_update(
    commands: &mut Commands,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_mesh_cache: &GpuMeshCache,
    all_entities: &[(Entity, AssetId, GlobalTransform, Option<Aabb>)],
    visible_set: &HashSet<Entity>,
    cached_octree: &mut Option<ResMut<crate::renderer::components::CachedOctree>>,
    existing_storage: &Option<ResMut<ModelStorageData>>,
    existing_indirect: &Option<ResMut<IndirectDrawData>>,
    transforms_changed: bool,
    profiler: &mut Option<ResMut<crate::core::Profiler>>,
    start_time: std::time::Instant,
) -> bool {
    if transforms_changed || existing_storage.is_none() {
        return false;
    }

    let Some(cache) = cached_octree else {
        return false;
    };

    if cache.octree_entities.is_empty() {
        return false;
    }

    if visible_set == &cache.last_visible {
        record_profiling(profiler, start_time);
        return true;
    }

    let mesh_groups = group_visible_meshes(all_entities, visible_set);
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

    cache.last_visible = visible_set.clone();
    record_profiling(profiler, start_time);
    true
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


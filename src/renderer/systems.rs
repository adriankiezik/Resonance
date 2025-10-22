use crate::assets::cache::AssetCache;
use crate::assets::handle::AssetId;
use crate::assets::loader::mesh::MeshData;
use crate::core::math::Mat3;
use crate::renderer::lighting::{
    AmbientLight, AmbientLightUniform, DirectionalLight, DirectionalLightUniform, LightingUniform,
};
use crate::renderer::{
    Camera, GpuMeshCache, MeshPipeline, ModelUniform, Renderer,
    components::{Aabb, IndirectDrawData, LightingData, Mesh, MeshUploaded, ModelStorageData},
    mesh::GpuMesh,
};
use crate::transform::GlobalTransform;
use crate::window::WindowEvent;
use bevy_ecs::prelude::*;
use std::collections::HashSet;
use wgpu::util::DeviceExt;

pub fn upload_meshes(
    mut commands: Commands,
    renderer: Option<Res<Renderer>>,
    asset_cache: Res<AssetCache>,
    mut gpu_mesh_cache: Option<ResMut<GpuMeshCache>>,
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

        if let Some(mesh_data_vec) = asset_cache.get::<Vec<MeshData>>(mesh.handle.id) {
            if mesh.mesh_index < mesh_data_vec.len() {
                let mesh_data = &mesh_data_vec[mesh.mesh_index];
                let gpu_mesh = GpuMesh::from_mesh_data(device, mesh_data);

                gpu_mesh_cache.insert(mesh.handle.id, gpu_mesh);
                commands.entity(entity).insert(MeshUploaded);

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
}

pub fn compute_mesh_aabbs(
    mut commands: Commands,
    asset_cache: Res<AssetCache>,
    query: Query<(Entity, &Mesh, &MeshUploaded), Without<Aabb>>,
) {
    for (entity, mesh, _) in query.iter() {
        if let Some(mesh_data_vec) = asset_cache.get::<Vec<MeshData>>(mesh.handle.id) {
            if mesh.mesh_index < mesh_data_vec.len() {
                let mesh_data = &mesh_data_vec[mesh.mesh_index];
                let aabb = Aabb::from_positions(&mesh_data.positions);
                commands.entity(entity).insert(aabb);
            }
        }
    }
}

pub fn cleanup_unused_meshes(
    mut gpu_mesh_cache: Option<ResMut<GpuMeshCache>>,
    asset_cache: Res<AssetCache>,
    mesh_query: Query<&Mesh>,
) {
    let Some(ref mut gpu_mesh_cache) = gpu_mesh_cache else {
        return;
    };
    let active_mesh_ids: HashSet<AssetId> = mesh_query.iter().map(|mesh| mesh.handle.id).collect();

    let cached_ids: Vec<AssetId> = gpu_mesh_cache.iter_ids().collect();

    for mesh_id in cached_ids {
        let is_asset_loaded = asset_cache.contains::<Vec<MeshData>>(mesh_id);
        let is_referenced = active_mesh_ids.contains(&mesh_id);

        if !is_asset_loaded || !is_referenced {
            if let Some(_) = gpu_mesh_cache.remove(&mesh_id) {
                log::debug!(
                    "Cleaned up GPU mesh: {:?} (asset_loaded: {}, referenced: {})",
                    mesh_id,
                    is_asset_loaded,
                    is_referenced
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

pub fn update_camera_aspect_ratio(
    mut cameras: Query<&mut Camera>,
    If(mut window_events): If<MessageReader<WindowEvent>>,
) {
    for event in window_events.read() {
        if let WindowEvent::Resized { width, height } = event {
            let aspect = *width as f32 / (*height as f32).max(1.0);

            for mut camera in cameras.iter_mut() {
                camera.set_aspect(aspect);
            }

            log::debug!("Updated camera aspect ratio to: {:.3}", aspect);
        }
    }
}

pub fn initialize_lighting(
    mut commands: Commands,
    renderer: Option<Res<Renderer>>,
    pipeline: Option<Res<MeshPipeline>>,
    lighting_data: Option<Res<LightingData>>,
) {
    if lighting_data.is_some() {
        return;
    }

    let Some(renderer) = renderer else {
        return;
    };
    let Some(pipeline) = pipeline else {
        return;
    };

    let device = renderer.device();
    let default_lighting = LightingUniform::default();

    let lighting_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Lighting Buffer"),
        contents: bytemuck::cast_slice(&[default_lighting]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let lighting_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Lighting Bind Group"),
        layout: &pipeline.lighting_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: lighting_buffer.as_entire_binding(),
        }],
    });

    commands.insert_resource(LightingData {
        buffer: lighting_buffer,
        bind_group: lighting_bind_group,
    });

    log::debug!("Initialized lighting system with default values");
}

pub fn update_lighting(
    renderer: Option<Res<Renderer>>,
    lighting_data: Option<Res<LightingData>>,
    ao_mode: Option<Res<crate::renderer::AOMode>>,
    ao_debug: Option<Res<crate::renderer::AODebugMode>>,
    mut profiler: Option<ResMut<crate::core::Profiler>>,
    directional_light_query: Query<&DirectionalLight>,
    ambient_light_query: Query<&AmbientLight>,
) {
    let _start = std::time::Instant::now();
    let Some(renderer) = renderer else {
        return;
    };
    let Some(lighting_data) = lighting_data else {
        return;
    };

    let directional_uniform = directional_light_query
        .iter()
        .next()
        .map(DirectionalLightUniform::from_light)
        .unwrap_or_default();

    let ambient_uniform = ambient_light_query
        .iter()
        .next()
        .map(AmbientLightUniform::from_light)
        .unwrap_or_default();

    let lighting_uniform = LightingUniform {
        directional: directional_uniform,
        ambient: ambient_uniform,
        point_light_count: 0,
        ao_mode: ao_mode.map(|m| *m as u32).unwrap_or(0),
        ao_debug: ao_debug.map(|d| d.enabled as u32).unwrap_or(0),
        _padding1: 0.0,
        _padding2: [0.0; 3],
        _padding3: 0.0,
        _padding4: [0.0; 3],
        _padding5: 0.0,
    };

    renderer.queue().write_buffer(
        &lighting_data.buffer,
        0,
        bytemuck::cast_slice(&[lighting_uniform]),
    );

    if let Some(ref mut profiler) = profiler {
        profiler.record_timing("PostUpdate::update_lighting".to_string(), _start.elapsed());
    }
}

pub fn prepare_indirect_draw_data(
    mut commands: Commands,
    renderer: Option<Res<Renderer>>,
    pipeline: Option<Res<MeshPipeline>>,
    gpu_mesh_cache: Option<Res<GpuMeshCache>>,
    existing_storage: Option<ResMut<ModelStorageData>>,
    _existing_indirect: Option<ResMut<IndirectDrawData>>,
    cached_frustum: Option<ResMut<crate::renderer::components::CachedFrustum>>,
    mut profiler: Option<ResMut<crate::core::Profiler>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &Mesh, &GlobalTransform, Option<&Aabb>), With<MeshUploaded>>,
) {
    let _start = std::time::Instant::now();
    let Some(renderer) = renderer else {
        return;
    };
    let Some(pipeline) = pipeline else {
        return;
    };
    let Some(gpu_mesh_cache) = gpu_mesh_cache else {
        return;
    };

    let device = renderer.device();
    let queue = renderer.queue();

    let frustum = if let Some((camera, transform)) = camera_query.iter().next() {
        let current_matrix = transform.matrix();

        if let Some(mut cache) = cached_frustum {
            if cache.camera_transform == current_matrix {
                Some(cache.frustum.clone())
            } else {
                let frustum = camera.frustum(transform);
                cache.frustum = frustum.clone();
                cache.camera_transform = current_matrix;
                Some(frustum)
            }
        } else {
            let frustum = camera.frustum(transform);
            commands.insert_resource(crate::renderer::components::CachedFrustum {
                frustum: frustum.clone(),
                camera_transform: current_matrix,
            });
            Some(frustum)
        }
    } else {
        None
    };

    let mut all_entities: Vec<(Entity, AssetId, GlobalTransform, Option<Aabb>)> = query
        .iter()
        .map(|(entity, mesh, transform, aabb)| (entity, mesh.handle.id, *transform, aabb.copied()))
        .collect();

    all_entities.sort_by_key(|(entity, mesh_id, _, _)| (mesh_id.0, *entity));

    let total_count = all_entities.len();
    if total_count == 0 {
        return;
    }

    let draw_data: Vec<(AssetId, Entity, u32, Mat3, GlobalTransform, bool)> = all_entities
        .iter()
        .enumerate()
        .map(|(instance_index, (entity, mesh_id, transform, aabb))| {
            let visible = if let (Some(frustum), Some(aabb)) = (&frustum, aabb) {
                let world_aabb = aabb.transform(transform.matrix());
                frustum.contains_aabb(world_aabb.min, world_aabb.max)
            } else {
                true
            };
            (
                *mesh_id,
                *entity,
                instance_index as u32,
                Mat3::from_mat4(transform.matrix()).inverse().transpose(),
                *transform,
                visible,
            )
        })
        .collect();

    // Prepare model uniforms for ALL entities
    // Group visible draws by mesh_id ONCE (used in both paths below)
    use crate::renderer::components::MeshDrawBatch;
    use std::collections::HashMap;

    let mut mesh_groups: HashMap<AssetId, Vec<u32>> = HashMap::new();

    for (mesh_id, _, instance_index, _, _, visible) in &draw_data {
        if *visible {
            mesh_groups
                .entry(*mesh_id)
                .or_default()
                .push(*instance_index);
        }
    }

    let model_uniforms: Vec<ModelUniform> = draw_data
        .iter()
        .map(|(_, _, _, normal_matrix, transform, _)| {
            let model_matrix = transform.matrix();
            let normal_matrix_cols: [[f32; 4]; 3] = [
                [
                    normal_matrix.x_axis.x,
                    normal_matrix.x_axis.y,
                    normal_matrix.x_axis.z,
                    0.0,
                ],
                [
                    normal_matrix.y_axis.x,
                    normal_matrix.y_axis.y,
                    normal_matrix.y_axis.z,
                    0.0,
                ],
                [
                    normal_matrix.z_axis.x,
                    normal_matrix.z_axis.y,
                    normal_matrix.z_axis.z,
                    0.0,
                ],
            ];

            ModelUniform {
                model: model_matrix.to_cols_array_2d(),
                normal_matrix: normal_matrix_cols,
            }
        })
        .collect();

    // Check if we can reuse existing buffers (entity count unchanged)
    if let Some(ref storage_data) = existing_storage {
        if storage_data.entity_count == total_count {
            // Just update model matrices, don't recreate buffers
            queue.write_buffer(
                &storage_data.buffer,
                0,
                bytemuck::cast_slice(&model_uniforms),
            );

            // Check if we can reuse existing indirect buffers
            if let Some(ref existing_indirect) = _existing_indirect {
                let mut can_reuse = existing_indirect.batches.len() == mesh_groups.len();

                if can_reuse {
                    for existing_batch in &existing_indirect.batches {
                        if let Some(new_instances) = mesh_groups.get(&existing_batch.mesh_id) {
                            if existing_batch.visible_instances != *new_instances {
                                can_reuse = false;
                                break;
                            }
                        } else {
                            can_reuse = false;
                            break;
                        }
                    }
                }

                if can_reuse {
                    if let Some(ref mut profiler) = profiler {
                        profiler.record_timing(
                            "PostUpdate::prepare_indirect_draw_data".to_string(),
                            _start.elapsed(),
                        );
                    }
                    return;
                }
            }

            // Visibility changed, recreate indirect buffers but reuse model buffer
            let mut batches = Vec::new();

            for (mesh_id, instances) in mesh_groups {
                if let Some(gpu_mesh) = gpu_mesh_cache.get(&mesh_id) {
                    let mut indirect_commands = Vec::new();

                    for first_instance in instances.iter() {
                        indirect_commands.push(gpu_mesh.index_count);
                        indirect_commands.push(1u32);
                        indirect_commands.push(0u32);
                        indirect_commands.push(0i32 as u32);
                        indirect_commands.push(*first_instance);
                    }

                    let indirect_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("Indirect Draw Buffer {:?}", mesh_id)),
                            contents: bytemuck::cast_slice(&indirect_commands),
                            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                        });

                    batches.push(MeshDrawBatch {
                        mesh_id,
                        indirect_buffer,
                        draw_count: instances.len() as u32,
                        base_instance: instances[0],
                        visible_instances: instances,
                    });
                }
            }

            if !batches.is_empty() {
                commands.insert_resource(IndirectDrawData { batches });
            }

            if let Some(ref mut profiler) = profiler {
                profiler.record_timing(
                    "PostUpdate::prepare_indirect_draw_data".to_string(),
                    _start.elapsed(),
                );
            }
            return;
        }
    }

    // Create new buffers
    let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Model Storage Buffer"),
        contents: bytemuck::cast_slice(&model_uniforms),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Model Storage Bind Group"),
        layout: &pipeline.model_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: model_buffer.as_entire_binding(),
        }],
    });

    commands.insert_resource(ModelStorageData {
        buffer: model_buffer,
        bind_group: model_bind_group,
        capacity: total_count,
        entity_count: total_count,
    });

    // mesh_groups already computed above, reuse it
    let mut batches = Vec::new();

    for (mesh_id, instances) in mesh_groups {
        if let Some(gpu_mesh) = gpu_mesh_cache.get(&mesh_id) {
            let mut indirect_commands = Vec::new();

            for first_instance in instances.iter() {
                // DrawIndexedIndirect structure:
                // index_count, instance_count, first_index, base_vertex, first_instance
                indirect_commands.push(gpu_mesh.index_count); // index_count
                indirect_commands.push(1u32); // instance_count
                indirect_commands.push(0u32); // first_index
                indirect_commands.push(0i32 as u32); // base_vertex
                indirect_commands.push(*first_instance); // first_instance
            }

            let indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Indirect Draw Buffer {:?}", mesh_id)),
                contents: bytemuck::cast_slice(&indirect_commands),
                usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            });

            batches.push(MeshDrawBatch {
                mesh_id,
                indirect_buffer,
                draw_count: instances.len() as u32,
                base_instance: instances[0],
                visible_instances: instances,
            });
        }
    }

    if !batches.is_empty() {
        commands.insert_resource(IndirectDrawData { batches });
    }

    if let Some(ref mut profiler) = profiler {
        profiler.record_timing(
            "PostUpdate::prepare_indirect_draw_data".to_string(),
            _start.elapsed(),
        );
    }
}

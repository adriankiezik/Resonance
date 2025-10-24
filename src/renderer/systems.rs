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
    octree::{Octree, OctreeEntity},
};
use crate::transform::GlobalTransform;
use crate::window::WindowEvent;
use bevy_ecs::prelude::*;
use std::collections::HashSet;
use wgpu::util::DeviceExt;

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

pub fn compute_mesh_aabbs(
    mut commands: Commands,
    query: Query<(Entity, &Mesh, &MeshUploaded), Without<Aabb>>,
) {
    for (entity, mesh, _) in query.iter() {
        let mesh_data_vec = &mesh.handle.asset;
        if mesh_data_vec.is_empty() {
            continue;
        }
        if mesh.mesh_index < mesh_data_vec.len() {
            let mesh_data = &mesh_data_vec[mesh.mesh_index];
            let aabb = Aabb::from_positions(&mesh_data.positions);
            commands.entity(entity).insert(aabb);
        }
    }
}

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

pub fn update_camera_aspect_ratio(
    mut commands: Commands,
    mut cameras: Query<&mut Camera>,
    cached_frustum: Option<ResMut<crate::renderer::components::CachedFrustum>>,
    If(mut window_events): If<MessageReader<WindowEvent>>,
) {
    for event in window_events.read() {
        if let WindowEvent::Resized { width, height } = event {
            let aspect = *width as f32 / (*height as f32).max(1.0);

            for mut camera in cameras.iter_mut() {
                camera.set_aspect(aspect);
            }

            if cached_frustum.is_some() {
                commands.remove_resource::<crate::renderer::components::CachedFrustum>();
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
    mut cached_octree: Option<ResMut<crate::renderer::components::CachedOctree>>,
    mut profiler: Option<ResMut<crate::core::Profiler>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &Mesh, &GlobalTransform, Option<&Aabb>), (With<MeshUploaded>, Changed<GlobalTransform>)>,
    all_query: Query<(Entity, &Mesh, &GlobalTransform, Option<&Aabb>), With<MeshUploaded>>,
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

    let transforms_changed = !query.is_empty();

    let (frustum, camera_changed) = if let Some((camera, transform)) = camera_query.iter().next() {
        let current_matrix = transform.matrix();

        if let Some(mut cache) = cached_frustum {
            if cache.camera_transform == current_matrix {
                (Some(cache.frustum.clone()), false)
            } else {
                let frustum = camera.frustum(transform);
                cache.frustum = frustum.clone();
                cache.camera_transform = current_matrix;
                (Some(frustum), true)
            }
        } else {
            let frustum = camera.frustum(transform);
            commands.insert_resource(crate::renderer::components::CachedFrustum {
                frustum: frustum.clone(),
                camera_transform: current_matrix,
            });
            (Some(frustum), true)
        }
    } else {
        (None, false)
    };

    if !transforms_changed && !camera_changed && cached_octree.is_some() && existing_storage.is_some() && _existing_indirect.is_some() {
        if let Some(ref mut profiler) = profiler {
            profiler.record_timing(
                "PostUpdate::prepare_indirect_draw_data".to_string(),
                _start.elapsed(),
            );
        }
        return;
    }

    let mut all_entities: Vec<(Entity, AssetId, GlobalTransform, Option<Aabb>)> = all_query
        .iter()
        .map(|(entity, mesh, transform, aabb)| (entity, mesh.handle.id, *transform, aabb.copied()))
        .collect();

    all_entities.sort_unstable_by_key(|(entity, mesh_id, _, _)| (mesh_id.0, *entity));

    let total_count = all_entities.len();
    if total_count == 0 {
        if existing_storage.is_some() {
            commands.remove_resource::<ModelStorageData>();
        }
        if _existing_indirect.is_some() {
            commands.remove_resource::<IndirectDrawData>();
        }
        return;
    }

    let visible_entities = if let Some(ref frustum) = frustum {
        let entity_hash = all_entities.iter().fold(0u64, |acc, (entity, _, _, _)| {
            let gen_bits = entity.generation().to_bits();
            acc.wrapping_mul(31)
                .wrapping_add(entity.index() as u64)
                .wrapping_add(gen_bits as u64)
        });

        let needs_rebuild = transforms_changed || cached_octree
            .as_ref()
            .map(|cache| cache.entity_count != all_entities.len() || cache.entity_hash != entity_hash)
            .unwrap_or(true);

        if needs_rebuild {
            let octree_entities: Vec<OctreeEntity> = all_entities
                .iter()
                .filter_map(|(entity, mesh_id, transform, aabb)| {
                    aabb.map(|aabb| {
                        let world_aabb = aabb.transform(transform.matrix());
                        OctreeEntity {
                            entity: *entity,
                            mesh_id: *mesh_id,
                            aabb: world_aabb,
                        }
                    })
                })
                .collect();

            if !octree_entities.is_empty() {
                let new_octree = Octree::from_entities(&octree_entities);
                let visible = new_octree.query_frustum(frustum);
                let visible_set: HashSet<Entity> = visible.iter().map(|e| e.entity).collect();

                if let Some(ref mut cache) = cached_octree {
                    cache.octree = new_octree;
                    cache.octree_entities = octree_entities;
                    cache.entity_count = all_entities.len();
                    cache.entity_hash = entity_hash;
                    cache.last_visible = visible_set.clone();
                } else {
                    commands.insert_resource(crate::renderer::components::CachedOctree {
                        octree: new_octree,
                        octree_entities,
                        entity_count: all_entities.len(),
                        entity_hash,
                        last_visible: visible_set.clone(),
                    });
                }
                visible_set
            } else {
                all_entities.iter().map(|(entity, _, _, _)| *entity).collect()
            }
        } else if let Some(ref mut cache) = cached_octree {
            if !cache.octree_entities.is_empty() {
                let visible = cache.octree.query_frustum(frustum);
                let visible_set: HashSet<Entity> = visible.iter().map(|e| e.entity).collect();

                if visible_set == cache.last_visible && !transforms_changed {
                    if let Some(ref mut profiler) = profiler {
                        profiler.record_timing(
                            "PostUpdate::prepare_indirect_draw_data".to_string(),
                            _start.elapsed(),
                        );
                    }
                    return;
                }

                if !transforms_changed && existing_storage.is_some() && visible_set != cache.last_visible {
                    let mut mesh_groups: ahash::AHashMap<AssetId, Vec<u32>> = ahash::AHashMap::new();

                    for (idx, (entity, mesh_id, _, _)) in all_entities.iter().enumerate() {
                        if visible_set.contains(entity) {
                            mesh_groups
                                .entry(*mesh_id)
                                .or_default()
                                .push(idx as u32);
                        }
                    }

                    let mut batches = Vec::new();

                    let existing_batches_map: ahash::AHashMap<AssetId, &crate::renderer::components::MeshDrawBatch> =
                        _existing_indirect.as_ref()
                            .map(|d| d.batches.iter().map(|b| (b.mesh_id, b)).collect())
                            .unwrap_or_default();

                    for (mesh_id, instances) in mesh_groups {
                        if let Some(gpu_mesh) = gpu_mesh_cache.get(&mesh_id) {
                            let existing_batch = existing_batches_map.get(&mesh_id);

                            let instances_changed = existing_batch
                                .map(|b| {
                                    b.visible_instances.len() != instances.len() ||
                                    b.visible_instances != instances
                                })
                                .unwrap_or(true);

                            let (indirect_buffer, buffer_capacity) = if let Some(existing) = existing_batch {
                                if instances.len() as u32 <= existing.buffer_capacity {
                                    if instances_changed {
                                        let mut indirect_commands = Vec::new();
                                        for first_instance in instances.iter() {
                                            indirect_commands.push(gpu_mesh.index_count);
                                            indirect_commands.push(1u32);
                                            indirect_commands.push(0u32);
                                            indirect_commands.push(0i32 as u32);
                                            indirect_commands.push(*first_instance);
                                        }
                                        queue.write_buffer(
                                            &existing.indirect_buffer,
                                            0,
                                            bytemuck::cast_slice(&indirect_commands),
                                        );
                                    }
                                    (existing.indirect_buffer.clone(), existing.buffer_capacity)
                                } else {
                                    let mut indirect_commands = Vec::new();
                                    for first_instance in instances.iter() {
                                        indirect_commands.push(gpu_mesh.index_count);
                                        indirect_commands.push(1u32);
                                        indirect_commands.push(0u32);
                                        indirect_commands.push(0i32 as u32);
                                        indirect_commands.push(*first_instance);
                                    }
                                    let new_capacity = (instances.len() as u32 * 3 / 2).max(instances.len() as u32 + 16);
                                    let buffer_size = new_capacity as usize * 5 * std::mem::size_of::<u32>();
                                    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                                        label: Some(&format!("Indirect Draw Buffer {:?}", mesh_id)),
                                        size: buffer_size as u64,
                                        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                                        mapped_at_creation: false,
                                    });
                                    queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&indirect_commands));
                                    (buffer, new_capacity)
                                }
                            } else {
                                let mut indirect_commands = Vec::new();
                                for first_instance in instances.iter() {
                                    indirect_commands.push(gpu_mesh.index_count);
                                    indirect_commands.push(1u32);
                                    indirect_commands.push(0u32);
                                    indirect_commands.push(0i32 as u32);
                                    indirect_commands.push(*first_instance);
                                }
                                let capacity = (instances.len() as u32 * 3 / 2).max(instances.len() as u32 + 16);
                                let buffer_size = capacity as usize * 5 * std::mem::size_of::<u32>();
                                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some(&format!("Indirect Draw Buffer {:?}", mesh_id)),
                                    size: buffer_size as u64,
                                    usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                                    mapped_at_creation: false,
                                });
                                queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&indirect_commands));
                                (buffer, capacity)
                            };

                            batches.push(crate::renderer::components::MeshDrawBatch {
                                mesh_id,
                                indirect_buffer,
                                draw_count: instances.len() as u32,
                                base_instance: instances[0],
                                visible_instances: instances,
                                buffer_capacity,
                            });
                        }
                    }

                    if !batches.is_empty() {
                        commands.insert_resource(IndirectDrawData { batches });
                    }

                    cache.last_visible = visible_set;

                    if let Some(ref mut profiler) = profiler {
                        profiler.record_timing(
                            "PostUpdate::prepare_indirect_draw_data".to_string(),
                            _start.elapsed(),
                        );
                    }
                    return;
                }

                cache.last_visible = visible_set.clone();
                visible_set
            } else {
                all_entities.iter().map(|(entity, _, _, _)| *entity).collect()
            }
        } else {
            all_entities.iter().map(|(entity, _, _, _)| *entity).collect()
        }
    } else {
        all_entities.iter().map(|(entity, _, _, _)| *entity).collect()
    };

    use crate::renderer::components::MeshDrawBatch;
    use rayon::prelude::*;

    let mut mesh_groups: ahash::AHashMap<AssetId, Vec<u32>> = ahash::AHashMap::new();

    for (idx, (entity, mesh_id, _, _)) in all_entities.iter().enumerate() {
        if visible_entities.contains(entity) {
            mesh_groups
                .entry(*mesh_id)
                .or_default()
                .push(idx as u32);
        }
    }

    let model_uniforms: Vec<ModelUniform> = all_entities
        .par_iter()
        .map(|(_, _, transform, _)| {
            let model_matrix = transform.matrix();
            let normal_matrix = Mat3::from_mat4(model_matrix).inverse().transpose();
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

    if let Some(ref storage_data) = existing_storage {
        if storage_data.entity_count == total_count {
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

            let mut batches = Vec::new();
            let existing_batches = _existing_indirect.as_ref().map(|d| &d.batches);

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

                    let existing_batch = existing_batches.and_then(|batches| {
                        batches.iter().find(|b| b.mesh_id == mesh_id)
                    });

                    let (indirect_buffer, buffer_capacity) = if let Some(existing) = existing_batch {
                        if instances.len() as u32 <= existing.buffer_capacity {
                            queue.write_buffer(
                                &existing.indirect_buffer,
                                0,
                                bytemuck::cast_slice(&indirect_commands),
                            );
                            (existing.indirect_buffer.clone(), existing.buffer_capacity)
                        } else {
                            let new_capacity = (instances.len() as u32 * 3 / 2).max(instances.len() as u32 + 16);
                            let buffer_size = new_capacity as usize * 5 * std::mem::size_of::<u32>();
                            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some(&format!("Indirect Draw Buffer {:?}", mesh_id)),
                                size: buffer_size as u64,
                                usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            });
                            queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&indirect_commands));
                            (buffer, new_capacity)
                        }
                    } else {
                        let capacity = (instances.len() as u32 * 3 / 2).max(instances.len() as u32 + 16);
                        let buffer_size = capacity as usize * 5 * std::mem::size_of::<u32>();
                        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some(&format!("Indirect Draw Buffer {:?}", mesh_id)),
                            size: buffer_size as u64,
                            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });
                        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&indirect_commands));
                        (buffer, capacity)
                    };

                    batches.push(MeshDrawBatch {
                        mesh_id,
                        indirect_buffer,
                        draw_count: instances.len() as u32,
                        base_instance: instances[0],
                        visible_instances: instances,
                        buffer_capacity,
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

            let capacity = (instances.len() as u32 * 3 / 2).max(instances.len() as u32 + 16);
            let buffer_size = capacity as usize * 5 * std::mem::size_of::<u32>();
            let indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Indirect Draw Buffer {:?}", mesh_id)),
                size: buffer_size as u64,
                usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&indirect_buffer, 0, bytemuck::cast_slice(&indirect_commands));

            batches.push(MeshDrawBatch {
                mesh_id,
                indirect_buffer,
                draw_count: instances.len() as u32,
                base_instance: instances[0],
                visible_instances: instances,
                buffer_capacity: capacity,
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

pub fn update_gpu_memory_stats(
    renderer: Option<Res<Renderer>>,
    mut memory_tracker: Option<ResMut<crate::core::MemoryTracker>>,
) {
    let Some(renderer) = renderer else {
        return;
    };
    let Some(ref mut memory_tracker) = memory_tracker else {
        return;
    };

    let (depth_size, ssao_size, msaa_size) = renderer.calculate_texture_memory();
    let camera_buffer_size = renderer.camera_buffer_size();

    memory_tracker.track_depth_texture(depth_size);
    memory_tracker.track_ssao_textures(ssao_size);
    memory_tracker.track_msaa_textures(msaa_size);
    memory_tracker.track_camera_buffer(camera_buffer_size);
}

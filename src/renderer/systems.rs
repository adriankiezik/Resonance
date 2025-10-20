use crate::assets::cache::AssetCache;
use crate::assets::handle::AssetId;
use crate::assets::mesh_loader::MeshData;
use crate::core::math::Mat3;
use crate::renderer::{
    components::{GpuModelData, LightingData, Mesh, MeshUploaded},
    mesh::GpuMesh,
    Camera, GpuMeshCache, MeshPipeline, ModelUniform, Renderer,
};
use crate::renderer::lighting::{
    AmbientLight, AmbientLightUniform, DirectionalLight, DirectionalLightUniform, LightingUniform,
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
    let Some(renderer) = renderer else { return; };
    let Some(ref mut gpu_mesh_cache) = gpu_mesh_cache else { return; };

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

pub fn cleanup_unused_meshes(
    mut gpu_mesh_cache: Option<ResMut<GpuMeshCache>>,
    asset_cache: Res<AssetCache>,
    mesh_query: Query<&Mesh>,
) {
    let Some(ref mut gpu_mesh_cache) = gpu_mesh_cache else { return; };
    let active_mesh_ids: HashSet<AssetId> = mesh_query
        .iter()
        .map(|mesh| mesh.handle.id)
        .collect();

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

pub fn create_model_buffers(
    mut commands: Commands,
    renderer: Option<Res<Renderer>>,
    pipeline: Option<Res<MeshPipeline>>,
    query: Query<(Entity, &GlobalTransform), (With<Mesh>, With<MeshUploaded>, Without<GpuModelData>)>,
) {
    let Some(renderer) = renderer else {
        return;
    };
    let Some(pipeline) = pipeline else {
        return;
    };

    let device = renderer.device();

    for (entity, transform) in query.iter() {
        let model_matrix = transform.matrix();
        let normal_matrix = Mat3::from_mat4(model_matrix).inverse().transpose();

        let normal_matrix_cols: [[f32; 4]; 3] = [
            [normal_matrix.x_axis.x, normal_matrix.x_axis.y, normal_matrix.x_axis.z, 0.0],
            [normal_matrix.y_axis.x, normal_matrix.y_axis.y, normal_matrix.y_axis.z, 0.0],
            [normal_matrix.z_axis.x, normal_matrix.z_axis.y, normal_matrix.z_axis.z, 0.0],
        ];

        let model_uniform = ModelUniform {
            model: model_matrix.to_cols_array_2d(),
            normal_matrix: normal_matrix_cols,
        };

        let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&[model_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Model Bind Group"),
            layout: &pipeline.model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_buffer.as_entire_binding(),
            }],
        });

        commands.entity(entity).insert(GpuModelData {
            buffer: model_buffer,
            bind_group: model_bind_group,
        });
    }
}

pub fn update_model_buffers(
    renderer: Option<Res<Renderer>>,
    query: Query<(&GlobalTransform, &GpuModelData), (With<Mesh>, With<MeshUploaded>, Changed<GlobalTransform>)>,
) {
    let Some(renderer) = renderer else {
        return;
    };

    let queue = renderer.queue();

    for (transform, gpu_model_data) in query.iter() {
        let model_matrix = transform.matrix();
        let normal_matrix = Mat3::from_mat4(model_matrix).inverse().transpose();

        let normal_matrix_cols: [[f32; 4]; 3] = [
            [normal_matrix.x_axis.x, normal_matrix.x_axis.y, normal_matrix.x_axis.z, 0.0],
            [normal_matrix.y_axis.x, normal_matrix.y_axis.y, normal_matrix.y_axis.z, 0.0],
            [normal_matrix.z_axis.x, normal_matrix.z_axis.y, normal_matrix.z_axis.z, 0.0],
        ];

        let model_uniform = ModelUniform {
            model: model_matrix.to_cols_array_2d(),
            normal_matrix: normal_matrix_cols,
        };

        queue.write_buffer(
            &gpu_model_data.buffer,
            0,
            bytemuck::cast_slice(&[model_uniform]),
        );
    }
}

pub fn cleanup_mesh_components(
    mut commands: Commands,
    query: Query<Entity, (Without<Mesh>, Or<(With<MeshUploaded>, With<GpuModelData>)>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<MeshUploaded>()
            .remove::<GpuModelData>();
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
    directional_light_query: Query<&DirectionalLight>,
    ambient_light_query: Query<&AmbientLight>,
) {
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
        _padding1: [0.0; 3],
        _padding2: [0.0; 3],
        _padding3: 0.0,
    };

    renderer.queue().write_buffer(
        &lighting_data.buffer,
        0,
        bytemuck::cast_slice(&[lighting_uniform]),
    );
}

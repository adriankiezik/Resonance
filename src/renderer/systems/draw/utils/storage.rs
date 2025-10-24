use crate::assets::handle::AssetId;
use crate::core::math::Mat3;
use crate::renderer::{ModelUniform, components::{Aabb, ModelStorageData}};
use crate::transform::GlobalTransform;
use bevy_ecs::prelude::*;
use rayon::prelude::*;
use std::collections::HashSet;
use wgpu::util::DeviceExt;

fn compute_uniform_for_transform(transform: &GlobalTransform) -> ModelUniform {
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
}

pub fn compute_model_uniforms(
    entities: &[(Entity, AssetId, GlobalTransform, Option<Aabb>)],
) -> Vec<ModelUniform> {
    entities
        .par_iter()
        .map(|(_, _, transform, _)| compute_uniform_for_transform(transform))
        .collect()
}

pub fn update_changed_uniforms(
    queue: &wgpu::Queue,
    storage_buffer: &wgpu::Buffer,
    entities: &[(Entity, AssetId, GlobalTransform, Option<Aabb>)],
    changed_entities: &HashSet<Entity>,
) {
    for (idx, (entity, _, transform, _)) in entities.iter().enumerate() {
        if changed_entities.contains(entity) {
            let uniform = compute_uniform_for_transform(transform);
            let offset = (idx * std::mem::size_of::<ModelUniform>()) as u64;
            queue.write_buffer(
                storage_buffer,
                offset,
                bytemuck::cast_slice(&[uniform]),
            );
        }
    }
}

pub fn update_or_create_storage_buffer(
    commands: &mut Commands,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &crate::renderer::MeshPipeline,
    existing_storage: Option<ResMut<ModelStorageData>>,
    model_uniforms: &[ModelUniform],
    total_count: usize,
) {
    if let Some(ref storage_data) = existing_storage {
        if storage_data.entity_count == total_count {
            queue.write_buffer(
                &storage_data.buffer,
                0,
                bytemuck::cast_slice(model_uniforms),
            );
            return;
        }
    }

    let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Model Storage Buffer"),
        contents: bytemuck::cast_slice(model_uniforms),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let all_visible = vec![1u32; total_count];
    let visibility_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Visibility Buffer"),
        contents: bytemuck::cast_slice(&all_visible),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Model Storage Bind Group"),
        layout: &pipeline.model_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: model_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: visibility_buffer.as_entire_binding(),
            },
        ],
    });

    commands.insert_resource(ModelStorageData {
        buffer: model_buffer,
        visibility_buffer: Some(visibility_buffer),
        bind_group: model_bind_group,
        capacity: total_count,
        entity_count: total_count,
    });
}

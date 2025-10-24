use crate::assets::handle::AssetId;
use crate::renderer::{GpuMeshCache, components::MeshDrawBatch, mesh::GpuMesh};
use std::sync::Arc;

pub fn create_indirect_commands(gpu_mesh: &GpuMesh, instances: &[u32]) -> Vec<u32> {
    let mut commands = Vec::new();
    for first_instance in instances.iter() {
        commands.push(gpu_mesh.index_count);
        commands.push(1u32);
        commands.push(0u32);
        commands.push(0i32 as u32);
        commands.push(*first_instance);
    }
    commands
}

pub fn create_or_update_indirect_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mesh_id: AssetId,
    gpu_mesh: Arc<GpuMesh>,
    instances: &[u32],
    existing_batch: Option<&MeshDrawBatch>,
) -> (wgpu::Buffer, u32) {
    let indirect_commands = create_indirect_commands(&gpu_mesh, instances);

    if let Some(existing) = existing_batch {
        let instances_changed = existing.visible_instances.len() != instances.len()
            || existing.visible_instances != instances;

        if instances.len() as u32 <= existing.buffer_capacity {
            if instances_changed {
                queue.write_buffer(
                    &existing.indirect_buffer,
                    0,
                    bytemuck::cast_slice(&indirect_commands),
                );
            }
            return (existing.indirect_buffer.clone(), existing.buffer_capacity);
        }
    }

    let capacity = calculate_buffer_capacity(instances.len());
    let buffer = create_indirect_buffer(device, mesh_id, capacity);
    queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&indirect_commands));
    (buffer, capacity)
}

fn calculate_buffer_capacity(instance_count: usize) -> u32 {
    (instance_count as u32 * 3 / 2).max(instance_count as u32 + 16)
}

fn create_indirect_buffer(device: &wgpu::Device, mesh_id: AssetId, capacity: u32) -> wgpu::Buffer {
    let buffer_size = capacity as usize * 5 * std::mem::size_of::<u32>();
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("Indirect Draw Buffer {:?}", mesh_id)),
        size: buffer_size as u64,
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn create_draw_batches(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_mesh_cache: &GpuMeshCache,
    mesh_groups: ahash::AHashMap<AssetId, Vec<u32>>,
    existing_batches: Option<&[MeshDrawBatch]>,
) -> Vec<MeshDrawBatch> {
    let mut batches = Vec::new();

    for (mesh_id, instances) in mesh_groups {
        if let Some(gpu_mesh) = gpu_mesh_cache.get(&mesh_id) {
            let existing_batch = existing_batches
                .and_then(|batches| batches.iter().find(|b| b.mesh_id == mesh_id));

            let (indirect_buffer, buffer_capacity) = create_or_update_indirect_buffer(
                device,
                queue,
                mesh_id,
                gpu_mesh,
                &instances,
                existing_batch,
            );

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

    batches
}

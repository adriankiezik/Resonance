use crate::renderer::components::{Aabb, Mesh, MeshUploaded};
use bevy_ecs::prelude::*;

pub fn compute_mesh_aabbs(
    mut commands: Commands,
    query: Query<(Entity, &Mesh, &MeshUploaded), Without<Aabb>>,
) {
    static FRAME_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let frame_num = FRAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let should_log = frame_num % 300 == 0;

    let mut computed_count = 0;
    for (entity, mesh, _) in query.iter() {
        let mesh_data_vec = &mesh.handle.asset;
        if mesh_data_vec.is_empty() {
            continue;
        }
        if mesh.mesh_index < mesh_data_vec.len() {
            let mesh_data = &mesh_data_vec[mesh.mesh_index];
            let aabb = Aabb::from_positions(&mesh_data.positions);

            if should_log {
                log::warn!(
                    "[compute_mesh_aabbs] Frame {}: Entity {:?}, Aabb min={:.2?}, max={:.2?}, vertices={}",
                    frame_num, entity, aabb.min, aabb.max, mesh_data.positions.len()
                );
            }

            commands.entity(entity).insert(aabb);
            computed_count += 1;
        }
    }

    if should_log && computed_count > 0 {
        log::warn!("[compute_mesh_aabbs] Frame {}: Computed {} AABBs total", frame_num, computed_count);
    }
}

use crate::renderer::components::{Aabb, Mesh, MeshUploaded};
use bevy_ecs::prelude::*;

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

use crate::assets::handle::AssetId;
use crate::renderer::{
    camera::Frustum,
    components::{Aabb, CachedOctree},
    octree::{Octree, OctreeEntity},
};
use crate::transform::GlobalTransform;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

pub fn update_octree_and_visibility(
    commands: &mut Commands,
    cached_octree: &mut Option<ResMut<CachedOctree>>,
    all_entities: &[(Entity, AssetId, GlobalTransform, Option<Aabb>)],
    frustum: &Frustum,
    transforms_changed: bool,
) -> HashSet<Entity> {
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
        rebuild_octree(commands, cached_octree, all_entities, frustum, entity_hash)
    } else if let Some(cache) = cached_octree {
        query_existing_octree(cache, frustum)
    } else {
        all_entities.iter().map(|(entity, _, _, _)| *entity).collect()
    }
}

fn rebuild_octree(
    commands: &mut Commands,
    cached_octree: &mut Option<ResMut<CachedOctree>>,
    all_entities: &[(Entity, AssetId, GlobalTransform, Option<Aabb>)],
    frustum: &Frustum,
    entity_hash: u64,
) -> HashSet<Entity> {
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

        if let Some(cache) = cached_octree {
            cache.octree = new_octree;
            cache.octree_entities = octree_entities;
            cache.entity_count = all_entities.len();
            cache.entity_hash = entity_hash;
            cache.last_visible = visible_set.clone();
        } else {
            commands.insert_resource(CachedOctree {
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
}

fn query_existing_octree(
    cache: &CachedOctree,
    frustum: &Frustum,
) -> HashSet<Entity> {
    if !cache.octree_entities.is_empty() {
        let visible = cache.octree.query_frustum(frustum);
        visible.iter().map(|e| e.entity).collect()
    } else {
        HashSet::new()
    }
}

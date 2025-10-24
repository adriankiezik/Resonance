use crate::assets::handle::AssetId;
use crate::core::math::*;
use crate::renderer::camera::Frustum;
use crate::renderer::components::Aabb;
use bevy_ecs::entity::Entity;

const MAX_DEPTH: usize = 8;
const MAX_ENTITIES_PER_NODE: usize = 16;

#[derive(Debug, Clone)]
pub struct OctreeEntity {
    pub entity: Entity,
    pub mesh_id: AssetId,
    pub aabb: Aabb,
}

#[derive(Debug)]
struct OctreeNode {
    bounds: Aabb,
    entities: Vec<OctreeEntity>,
    children: Option<Box<[OctreeNode; 8]>>,
}

impl OctreeNode {
    fn new(bounds: Aabb) -> Self {
        Self {
            bounds,
            entities: Vec::new(),
            children: None,
        }
    }

    fn subdivide(&mut self) {
        let center = (self.bounds.min + self.bounds.max) * 0.5;
        let min = self.bounds.min;
        let max = self.bounds.max;

        let children = [
            OctreeNode::new(Aabb::new(min, center)),
            OctreeNode::new(Aabb::new(
                Vec3::new(center.x, min.y, min.z),
                Vec3::new(max.x, center.y, center.z),
            )),
            OctreeNode::new(Aabb::new(
                Vec3::new(min.x, center.y, min.z),
                Vec3::new(center.x, max.y, center.z),
            )),
            OctreeNode::new(Aabb::new(
                Vec3::new(center.x, center.y, min.z),
                Vec3::new(max.x, max.y, center.z),
            )),
            OctreeNode::new(Aabb::new(
                Vec3::new(min.x, min.y, center.z),
                Vec3::new(center.x, center.y, max.z),
            )),
            OctreeNode::new(Aabb::new(
                Vec3::new(center.x, min.y, center.z),
                Vec3::new(max.x, center.y, max.z),
            )),
            OctreeNode::new(Aabb::new(
                Vec3::new(min.x, center.y, center.z),
                Vec3::new(center.x, max.y, max.z),
            )),
            OctreeNode::new(Aabb::new(center, max)),
        ];

        self.children = Some(Box::new(children));
    }

    fn get_child_index(&self, aabb: &Aabb) -> Option<usize> {
        let center = (self.bounds.min + self.bounds.max) * 0.5;
        let aabb_center = (aabb.min + aabb.max) * 0.5;

        let x = if aabb_center.x >= center.x { 1 } else { 0 };
        let y = if aabb_center.y >= center.y { 1 } else { 0 };
        let z = if aabb_center.z >= center.z { 1 } else { 0 };

        let fits_in_child = if x == 1 {
            aabb.min.x >= center.x
        } else {
            aabb.max.x <= center.x
        } && if y == 1 {
            aabb.min.y >= center.y
        } else {
            aabb.max.y <= center.y
        } && if z == 1 {
            aabb.min.z >= center.z
        } else {
            aabb.max.z <= center.z
        };

        if fits_in_child {
            Some(x | (y << 1) | (z << 2))
        } else {
            None
        }
    }

    fn insert(&mut self, entity: OctreeEntity, depth: usize) {
        if depth >= MAX_DEPTH {
            self.entities.push(entity);
            return;
        }

        if self.children.is_none() {
            if self.entities.len() < MAX_ENTITIES_PER_NODE {
                self.entities.push(entity);
                return;
            }

            self.subdivide();

            let mut remaining_entities = Vec::new();
            std::mem::swap(&mut self.entities, &mut remaining_entities);

            for e in remaining_entities {
                if let Some(child_idx) = self.get_child_index(&e.aabb) {
                    self.children.as_mut().unwrap()[child_idx].insert(e, depth + 1);
                } else {
                    self.entities.push(e);
                }
            }
        }

        if let Some(child_idx) = self.get_child_index(&entity.aabb) {
            self.children.as_mut().unwrap()[child_idx].insert(entity, depth + 1);
        } else {
            self.entities.push(entity);
        }
    }

    fn query_frustum(&self, frustum: &Frustum, results: &mut Vec<OctreeEntity>) {
        if !frustum.contains_aabb(self.bounds.min, self.bounds.max) {
            return;
        }

        for entity in &self.entities {
            if frustum.contains_aabb(entity.aabb.min, entity.aabb.max) {
                results.push(entity.clone());
            }
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_frustum(frustum, results);
            }
        }
    }
}

pub struct Octree {
    root: OctreeNode,
}

impl Octree {
    pub fn new(bounds: Aabb) -> Self {
        Self {
            root: OctreeNode::new(bounds),
        }
    }

    pub fn from_entities(entities: &[OctreeEntity]) -> Self {
        if entities.is_empty() {
            return Self::new(Aabb::new(Vec3::splat(-1000.0), Vec3::splat(1000.0)));
        }

        let mut min = entities[0].aabb.min;
        let mut max = entities[0].aabb.max;

        for entity in entities {
            min = min.min(entity.aabb.min);
            max = max.max(entity.aabb.max);
        }

        let padding = (max - min).length() * 0.1;
        min -= Vec3::splat(padding);
        max += Vec3::splat(padding);

        let mut octree = Self::new(Aabb::new(min, max));
        for entity in entities {
            octree.insert(entity.clone());
        }

        octree
    }

    pub fn insert(&mut self, entity: OctreeEntity) {
        self.root.insert(entity, 0);
    }

    pub fn query_frustum(&self, frustum: &Frustum) -> Vec<OctreeEntity> {
        let mut results = Vec::new();
        self.root.query_frustum(frustum, &mut results);
        results
    }
}

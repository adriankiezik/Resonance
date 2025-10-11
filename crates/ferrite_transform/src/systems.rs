//! Systems for transform propagation.

use crate::components::{GlobalTransform, Transform};
use crate::hierarchy::{Children, Parent};
use bevy_ecs::prelude::*;

/// System to propagate transforms from parents to children.
///
/// This ensures GlobalTransform is correctly computed for all entities
/// in the hierarchy. Runs in PostUpdate stage.
pub fn propagate_transforms(
    mut root_query: Query<
        (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
        Without<Parent>,
    >,
    mut child_query: Query<
        (Entity, &Transform, &mut GlobalTransform, &Parent, Option<&Children>),
    >,
    children_query: Query<&Children>,
) {
    // Update root entities (no parent)
    for (_entity, transform, mut global_transform, children) in root_query.iter_mut() {
        *global_transform = GlobalTransform::from_transform(transform);

        // Recursively update children
        if let Some(children) = children {
            for &child in children.iter() {
                propagate_recursive(child, global_transform.as_ref(), &mut child_query, &children_query);
            }
        }
    }
}

/// Recursively propagate transforms to children
fn propagate_recursive(
    entity: Entity,
    parent_global: &GlobalTransform,
    child_query: &mut Query<
        (Entity, &Transform, &mut GlobalTransform, &Parent, Option<&Children>),
    >,
    children_query: &Query<&Children>,
) {
    // Get the transform and compute the new global transform
    let (new_global, children_list) = if let Ok((_entity, transform, mut global_transform, _parent, _)) = child_query.get_mut(entity) {
        *global_transform = GlobalTransform::from_transform_and_parent(transform, parent_global);
        let new_global = *global_transform;

        // Get children list
        let children_list: Vec<Entity> = if let Ok(children) = children_query.get(entity) {
            children.iter().copied().collect()
        } else {
            Vec::new()
        };

        (new_global, children_list)
    } else {
        return;
    };

    // Recursively update children with the new global transform
    for child in children_list {
        propagate_recursive(child, &new_global, child_query, children_query);
    }
}

/// System to sync Transform to GlobalTransform for entities without parents.
///
/// This is a simpler version for entities that don't have hierarchy.
pub fn sync_simple_transforms(
    mut query: Query<(&Transform, &mut GlobalTransform), (Without<Parent>, Without<Children>)>,
) {
    for (transform, mut global_transform) in query.iter_mut() {
        *global_transform = GlobalTransform::from_transform(transform);
    }
}

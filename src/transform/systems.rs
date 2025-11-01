use super::components::{GlobalTransform, Transform};
use super::hierarchy::{Children, Parent};
use bevy_ecs::prelude::*;

/// Propagates transforms through the entity hierarchy using an iterative approach
/// to avoid per-entity allocations. Uses a persistent stack buffer for traversal.
pub fn propagate_transforms(
    mut root_query: Query<
        (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
        Without<Parent>,
    >,
    mut child_query: Query<(
        Entity,
        &Transform,
        &mut GlobalTransform,
        &Parent,
        Option<&Children>,
    )>,
    children_query: Query<&Children>,
) {
    // Reusable stack for iterative traversal to avoid allocations per entity
    let mut stack = Vec::with_capacity(256);

    for (_entity, transform, mut global_transform, children) in root_query.iter_mut() {
        let new_global = GlobalTransform::from_transform(transform);
        if *global_transform != new_global {
            *global_transform = new_global;
        }

        if let Some(children) = children {
            // Initialize stack with root's children
            stack.clear();
            for &child in children.iter() {
                stack.push((child, *global_transform));
            }

            // Iterative traversal instead of recursion
            while let Some((entity, parent_global)) = stack.pop() {
                if let Ok((_entity, transform, mut global_transform, _parent, _)) =
                    child_query.get_mut(entity)
                {
                    let computed_global =
                        GlobalTransform::from_transform_and_parent(transform, &parent_global);
                    if *global_transform != computed_global {
                        *global_transform = computed_global;
                    }
                    let new_global = *global_transform;

                    // Push children onto stack for processing
                    if let Ok(children) = children_query.get(entity) {
                        for &child in children.iter() {
                            stack.push((child, new_global));
                        }
                    }
                }
            }
        }
    }
}

pub fn sync_simple_transforms(
    mut query: Query<(&Transform, &mut GlobalTransform), (Without<Parent>, Without<Children>)>,
) {
    for (transform, mut global_transform) in query.iter_mut() {
        let new_global = GlobalTransform::from_transform(transform);
        if *global_transform != new_global {
            *global_transform = new_global;
        }
    }
}

use super::components::{GlobalTransform, Transform};
use super::hierarchy::{Children, Parent};
use bevy_ecs::prelude::*;

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
    for (_entity, transform, mut global_transform, children) in root_query.iter_mut() {
        let new_global = GlobalTransform::from_transform(transform);
        if *global_transform != new_global {
            *global_transform = new_global;
        }

        if let Some(children) = children {
            for &child in children.iter() {
                propagate_recursive(
                    child,
                    global_transform.as_ref(),
                    &mut child_query,
                    &children_query,
                );
            }
        }
    }
}

fn propagate_recursive(
    entity: Entity,
    parent_global: &GlobalTransform,
    child_query: &mut Query<(
        Entity,
        &Transform,
        &mut GlobalTransform,
        &Parent,
        Option<&Children>,
    )>,
    children_query: &Query<&Children>,
) {
    let (new_global, children_list) =
        if let Ok((_entity, transform, mut global_transform, _parent, _)) =
            child_query.get_mut(entity)
        {
            let computed_global =
                GlobalTransform::from_transform_and_parent(transform, parent_global);
            if *global_transform != computed_global {
                *global_transform = computed_global;
            }
            let new_global = *global_transform;

            let children_list: Vec<Entity> = if let Ok(children) = children_query.get(entity) {
                children.iter().copied().collect()
            } else {
                Vec::new()
            };

            (new_global, children_list)
        } else {
            return;
        };

    for child in children_list {
        propagate_recursive(child, &new_global, child_query, children_query);
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

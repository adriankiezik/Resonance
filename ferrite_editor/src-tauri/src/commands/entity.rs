use crate::engine::{entity_to_id, id_to_entity, EntityData, EntityId};
use crate::state::EditorState;
use bevy_ecs::prelude::*;
use ferrite_transform::{GlobalTransform, Transform};
use tauri::State;

/// Create a new entity
#[tauri::command]
pub fn create_entity(
    name: Option<String>,
    parent: Option<EntityId>,
    state: State<EditorState>,
) -> Result<EntityData, String> {
    let base_name = name.unwrap_or_else(|| "Entity".to_string());
    let entity_name = state.generate_entity_name(&base_name);

    let mut world = state.world.write();

    // Spawn entity with Transform
    let entity = world.spawn((Transform::default(), GlobalTransform::default())).id();

    drop(world);

    // Register entity
    state.register_entity(entity, entity_name.clone());

    // Set parent if specified
    if let Some(parent_id) = parent {
        let parent_entity = id_to_entity(parent_id);
        state.add_child(parent_entity, entity);
    }

    Ok(EntityData {
        id: entity_to_id(entity),
        name: entity_name,
        parent,
        children: Vec::new(),
        components: Vec::new(),
    })
}

/// Delete an entity
#[tauri::command]
pub fn delete_entity(entity_id: EntityId, state: State<EditorState>) -> Result<(), String> {
    let entity = id_to_entity(entity_id);

    // Get children to delete recursively
    let children = state.get_children(entity);

    // Delete all children first
    for child in children {
        let child_id = entity_to_id(child);
        delete_entity(child_id, state.clone())?;
    }

    // Delete the entity from ECS
    let mut world = state.world.write();
    world
        .despawn(entity);

    drop(world);

    // Unregister entity
    state.unregister_entity(entity);

    Ok(())
}

/// Rename an entity
#[tauri::command]
pub fn rename_entity(
    entity_id: EntityId,
    new_name: String,
    state: State<EditorState>,
) -> Result<(), String> {
    let entity = id_to_entity(entity_id);

    if state.get_entity_by_name(&new_name).is_some() {
        return Err(format!("Entity with name '{}' already exists", new_name));
    }

    state.entity_names.write().insert(entity, new_name);

    Ok(())
}

/// Get the entity hierarchy
#[tauri::command]
pub fn get_entity_hierarchy(state: State<EditorState>) -> Result<Vec<EntityData>, String> {
    let root_entities = state.get_root_entities();

    let mut result = Vec::new();

    for entity in root_entities {
        result.push(build_entity_tree(entity, &state));
    }

    Ok(result)
}

/// Build entity tree recursively
fn build_entity_tree(entity: Entity, state: &EditorState) -> EntityData {
    let children = state.get_children(entity);
    let name = state
        .get_entity_name(entity)
        .unwrap_or_else(|| "Unnamed".to_string());

    EntityData {
        id: entity_to_id(entity),
        name,
        parent: None, // Parent is implicit in hierarchy
        children: children
            .iter()
            .map(|child| build_entity_tree(*child, state))
            .map(|child_data| child_data.id)
            .collect(),
        components: Vec::new(), // Components fetched separately if needed
    }
}

/// Set entity parent
#[tauri::command]
pub fn set_entity_parent(
    entity_id: EntityId,
    parent_id: Option<EntityId>,
    state: State<EditorState>,
) -> Result<(), String> {
    let entity = id_to_entity(entity_id);

    // Remove from old parent
    let hierarchy = state.entity_hierarchy.read();
    let old_parent = hierarchy
        .iter()
        .find(|(_, children)| children.contains(&entity))
        .map(|(parent, _)| *parent);
    drop(hierarchy);

    if let Some(old_parent) = old_parent {
        state.remove_child(old_parent, entity);
    }

    // Add to new parent
    if let Some(parent_id) = parent_id {
        let parent_entity = id_to_entity(parent_id);
        state.add_child(parent_entity, entity);
    }

    Ok(())
}

/// Get a single entity's data
#[tauri::command]
pub fn get_entity(entity_id: EntityId, state: State<EditorState>) -> Result<EntityData, String> {
    let entity = id_to_entity(entity_id);
    let name = state
        .get_entity_name(entity)
        .ok_or("Entity not found")?;
    let children = state.get_children(entity);

    // Find parent
    let hierarchy = state.entity_hierarchy.read();
    let parent = hierarchy
        .iter()
        .find(|(_, children)| children.contains(&entity))
        .map(|(parent, _)| entity_to_id(*parent));

    Ok(EntityData {
        id: entity_id,
        name,
        parent,
        children: children.iter().map(|e| entity_to_id(*e)).collect(),
        components: Vec::new(),
    })
}

use crate::engine::{
    deserialize_transform, id_to_entity, serialize_transform, ComponentInfo, EntityId,
};
use crate::state::EditorState;
use bevy_ecs::prelude::*;
use ferrite_transform::Transform;
use tauri::State;

/// Add a component to an entity
#[tauri::command]
pub fn add_component(
    entity_id: EntityId,
    component_type: String,
    data: serde_json::Value,
    state: State<EditorState>,
) -> Result<(), String> {
    let entity = id_to_entity(entity_id);
    let mut world = state.world.write();

    match component_type.as_str() {
        "Transform" => {
            let transform = deserialize_transform(&data)?;
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.insert(transform);
            } else {
                return Err("Entity not found".to_string());
            }
        }
        _ => {
            return Err(format!("Unknown component type: {}", component_type));
        }
    }

    Ok(())
}

/// Update a component on an entity
#[tauri::command]
pub fn update_component(
    entity_id: EntityId,
    component_type: String,
    data: serde_json::Value,
    state: State<EditorState>,
) -> Result<(), String> {
    let entity = id_to_entity(entity_id);
    let mut world = state.world.write();

    match component_type.as_str() {
        "Transform" => {
            let transform = deserialize_transform(&data)?;
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                if let Some(mut existing_transform) = entity_mut.get_mut::<Transform>() {
                    *existing_transform = transform;
                } else {
                    return Err("Transform component not found on entity".to_string());
                }
            } else {
                return Err("Entity not found".to_string());
            }
        }
        _ => {
            return Err(format!("Unknown component type: {}", component_type));
        }
    }

    Ok(())
}

/// Remove a component from an entity
#[tauri::command]
pub fn remove_component(
    entity_id: EntityId,
    component_type: String,
    state: State<EditorState>,
) -> Result<(), String> {
    let entity = id_to_entity(entity_id);
    let mut world = state.world.write();

    match component_type.as_str() {
        "Transform" => {
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.remove::<Transform>();
            } else {
                return Err("Entity not found".to_string());
            }
        }
        _ => {
            return Err(format!("Unknown component type: {}", component_type));
        }
    }

    Ok(())
}

/// Get all components for an entity
#[tauri::command]
pub fn get_entity_components(
    entity_id: EntityId,
    state: State<EditorState>,
) -> Result<Vec<ComponentInfo>, String> {
    let entity = id_to_entity(entity_id);
    let world = state.world.read();

    let mut components = Vec::new();

    if let Ok(entity_ref) = world.get_entity(entity) {
        // Check for Transform
        if let Some(transform) = entity_ref.get::<Transform>() {
            components.push(ComponentInfo {
                component_type: "Transform".to_string(),
                data: serialize_transform(transform),
            });
        }
    } else {
        return Err("Entity not found".to_string());
    }

    Ok(components)
}

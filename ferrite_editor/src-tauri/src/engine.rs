use bevy_ecs::prelude::*;
use ferrite_transform::Transform;
use serde::{Deserialize, Serialize};

/// Serializable version of Entity ID
pub type EntityId = u64;

/// Convert Entity to EntityId
pub fn entity_to_id(entity: Entity) -> EntityId {
    entity.to_bits()
}

/// Convert EntityId to Entity
pub fn id_to_entity(id: EntityId) -> Entity {
    Entity::from_bits(id)
}

/// Serializable entity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: EntityId,
    pub name: String,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
    pub components: Vec<ComponentInfo>,
}

/// Component information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub component_type: String,
    pub data: serde_json::Value,
}

/// Helper to serialize Transform
pub fn serialize_transform(transform: &Transform) -> serde_json::Value {
    serde_json::json!({
        "position": {
            "x": transform.position.x,
            "y": transform.position.y,
            "z": transform.position.z,
        },
        "rotation": {
            "x": transform.rotation.x,
            "y": transform.rotation.y,
            "z": transform.rotation.z,
            "w": transform.rotation.w,
        },
        "scale": {
            "x": transform.scale.x,
            "y": transform.scale.y,
            "z": transform.scale.z,
        }
    })
}

/// Helper to deserialize Transform
pub fn deserialize_transform(data: &serde_json::Value) -> Result<Transform, String> {
    let position = glam::Vec3::new(
        data["position"]["x"].as_f64().ok_or("Invalid position.x")? as f32,
        data["position"]["y"].as_f64().ok_or("Invalid position.y")? as f32,
        data["position"]["z"].as_f64().ok_or("Invalid position.z")? as f32,
    );

    let rotation = glam::Quat::from_xyzw(
        data["rotation"]["x"].as_f64().ok_or("Invalid rotation.x")? as f32,
        data["rotation"]["y"].as_f64().ok_or("Invalid rotation.y")? as f32,
        data["rotation"]["z"].as_f64().ok_or("Invalid rotation.z")? as f32,
        data["rotation"]["w"].as_f64().ok_or("Invalid rotation.w")? as f32,
    );

    let scale = glam::Vec3::new(
        data["scale"]["x"].as_f64().ok_or("Invalid scale.x")? as f32,
        data["scale"]["y"].as_f64().ok_or("Invalid scale.y")? as f32,
        data["scale"]["z"].as_f64().ok_or("Invalid scale.z")? as f32,
    );

    Ok(Transform {
        position,
        rotation,
        scale,
    })
}

//! Component registry stub
//!
//! This module will provide automatic serialization/deserialization for components
//! once glam gains Bevy reflection support. For now, it's a stub.

use bevy_ecs::prelude::*;
use bevy_reflect::TypeRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Registry for serializable component types (currently unused)
pub struct ComponentRegistry {
    _type_registry: Arc<RwLock<TypeRegistry>>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            _type_registry: Arc::new(RwLock::new(TypeRegistry::default())),
        }
    }

    /// Serialize all components from an entity (not yet implemented)
    #[allow(dead_code)]
    pub fn serialize_entity(&self, _world: &World, _entity: Entity) -> Result<HashMap<String, String>, String> {
        // This will be implemented when glam gains Bevy reflection support
        Ok(HashMap::new())
    }

    /// Deserialize and add components to an entity (not yet implemented)
    #[allow(dead_code)]
    pub fn deserialize_entity(
        &self,
        _world: &mut World,
        _entity: Entity,
        _components: &HashMap<String, String>,
    ) -> Result<(), String> {
        // This will be implemented when glam gains Bevy reflection support
        Ok(())
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

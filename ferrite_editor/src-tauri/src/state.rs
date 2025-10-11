use bevy_ecs::prelude::*;
use ferrite_scene::SceneManager;
use parking_lot::RwLock;
use std::collections::HashMap;

/// Editor state that holds the ECS world and scene information
pub struct EditorState {
    pub world: RwLock<World>,
    pub entity_names: RwLock<HashMap<Entity, String>>,
    pub entity_hierarchy: RwLock<HashMap<Entity, Vec<Entity>>>,
}

impl EditorState {
    pub fn new() -> Self {
        let mut world = World::new();

        // Initialize scene manager
        world.insert_resource(SceneManager::new());

        Self {
            world: RwLock::new(world),
            entity_names: RwLock::new(HashMap::new()),
            entity_hierarchy: RwLock::new(HashMap::new()),
        }
    }

    /// Generate a unique name for an entity
    pub fn generate_entity_name(&self, base_name: &str) -> String {
        let names = self.entity_names.read();
        let existing_names: Vec<&String> = names.values().collect();

        let mut counter = 1;
        let mut name = base_name.to_string();

        while existing_names.contains(&&name) {
            name = format!("{}_{}", base_name, counter);
            counter += 1;
        }

        name
    }

    /// Register an entity with a name
    pub fn register_entity(&self, entity: Entity, name: String) {
        self.entity_names.write().insert(entity, name);
    }

    /// Get entity by name
    pub fn get_entity_by_name(&self, name: &str) -> Option<Entity> {
        self.entity_names
            .read()
            .iter()
            .find(|(_, n)| n.as_str() == name)
            .map(|(e, _)| *e)
    }

    /// Get entity name
    pub fn get_entity_name(&self, entity: Entity) -> Option<String> {
        self.entity_names.read().get(&entity).cloned()
    }

    /// Remove entity from tracking
    pub fn unregister_entity(&self, entity: Entity) {
        self.entity_names.write().remove(&entity);
        self.entity_hierarchy.write().remove(&entity);

        // Remove from parent's children list
        let mut hierarchy = self.entity_hierarchy.write();
        for children in hierarchy.values_mut() {
            children.retain(|e| *e != entity);
        }
    }

    /// Add child to parent
    pub fn add_child(&self, parent: Entity, child: Entity) {
        self.entity_hierarchy
            .write()
            .entry(parent)
            .or_insert_with(Vec::new)
            .push(child);
    }

    /// Remove child from parent
    pub fn remove_child(&self, parent: Entity, child: Entity) {
        if let Some(children) = self.entity_hierarchy.write().get_mut(&parent) {
            children.retain(|e| *e != child);
        }
    }

    /// Get children of an entity
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        self.entity_hierarchy
            .read()
            .get(&entity)
            .cloned()
            .unwrap_or_default()
    }

    /// Get root entities (entities with no parent)
    pub fn get_root_entities(&self) -> Vec<Entity> {
        let names = self.entity_names.read();
        let hierarchy = self.entity_hierarchy.read();

        let all_children: Vec<Entity> = hierarchy.values().flatten().copied().collect();

        names
            .keys()
            .filter(|e| !all_children.contains(e))
            .copied()
            .collect()
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

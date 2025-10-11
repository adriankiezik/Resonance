//! Entity replication system.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network ID component - uniquely identifies an entity across the network.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

impl NetworkId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

/// Component marking an entity for replication.
///
/// Entities with this component will have their state sent to clients.
#[derive(Component, Debug, Clone, Copy)]
pub struct Replicate;

/// Resource mapping NetworkId to Entity for quick lookups
#[derive(Resource, Default)]
pub struct NetworkIdMap {
    map: HashMap<u64, Entity>,
    next_id: u64,
}

impl NetworkIdMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register an entity and assign it a network ID
    pub fn register(&mut self, entity: Entity) -> NetworkId {
        let id = self.next_id;
        self.next_id += 1;
        self.map.insert(id, entity);
        NetworkId(id)
    }

    /// Register an entity with a specific network ID (for client-side spawning)
    pub fn register_with_id(&mut self, entity: Entity, network_id: NetworkId) {
        self.map.insert(network_id.0, entity);
    }

    /// Get the entity for a network ID
    pub fn get_entity(&self, network_id: NetworkId) -> Option<Entity> {
        self.map.get(&network_id.0).copied()
    }

    /// Remove an entity from the map
    pub fn remove(&mut self, network_id: NetworkId) {
        self.map.remove(&network_id.0);
    }
}

// TODO: Implement change detection for efficient replication
// TODO: Add support for component-level replication
// TODO: Implement ownership system (who can modify what)

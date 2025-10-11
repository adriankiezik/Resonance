//! Entity prefab system for reusable entity templates.

use crate::SerializationFormat;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// A prefab is a reusable entity template with predefined components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefab {
    /// Prefab name/identifier
    pub name: String,
    /// Component data as key-value pairs
    pub components: HashMap<String, ron::Value>,
    /// Optional child prefabs (for hierarchies)
    pub children: Vec<Prefab>,
}

impl Prefab {
    /// Create a new empty prefab
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            components: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Add a component to the prefab
    pub fn with_component(mut self, component_name: impl Into<String>, value: ron::Value) -> Self {
        self.components.insert(component_name.into(), value);
        self
    }

    /// Add a child prefab
    pub fn with_child(mut self, child: Prefab) -> Self {
        self.children.push(child);
        self
    }

    // ===== RON Serialization (Human-readable, Development) =====

    /// Serialize prefab to RON format (human-readable, slower)
    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, Default::default())
    }

    /// Deserialize prefab from RON format
    pub fn from_ron(data: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(data)
    }

    // ===== Bincode Serialization (Binary, Production) =====

    /// Save prefab to Bincode format (binary, 3-5x faster than RON)
    pub fn to_bincode(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
    }

    /// Load prefab from Bincode format
    pub fn from_bincode(data: &[u8]) -> Result<Self, bincode::error::DecodeError> {
        bincode::serde::decode_from_slice(data, bincode::config::standard())
            .map(|(prefab, _size)| prefab)
    }

    // ===== Auto-detecting Serialization =====

    /// Save prefab with specified format
    pub fn serialize(&self, format: SerializationFormat) -> Result<Vec<u8>, String> {
        match format {
            SerializationFormat::Ron => {
                self.to_ron()
                    .map(|s| s.into_bytes())
                    .map_err(|e| format!("RON serialization error: {}", e))
            }
            SerializationFormat::Bincode => {
                self.to_bincode()
                    .map_err(|e| format!("Bincode serialization error: {}", e))
            }
        }
    }

    /// Load prefab from bytes with specified format
    pub fn deserialize(data: &[u8], format: SerializationFormat) -> Result<Self, String> {
        match format {
            SerializationFormat::Ron => {
                let text = std::str::from_utf8(data)
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                Self::from_ron(text)
                    .map_err(|e| format!("RON deserialization error: {}", e))
            }
            SerializationFormat::Bincode => {
                Self::from_bincode(data)
                    .map_err(|e| format!("Bincode deserialization error: {}", e))
            }
        }
    }

    /// Auto-detect format from file path and load prefab
    pub fn load_from_path(path: impl AsRef<Path>, data: &[u8]) -> Result<Self, String> {
        let format = SerializationFormat::from_path(path);
        Self::deserialize(data, format)
    }

    /// Convert prefab from one format to another
    pub fn convert(&self, target_format: SerializationFormat) -> Result<Vec<u8>, String> {
        self.serialize(target_format)
    }
}

/// Registry for storing and managing prefabs
#[derive(Debug, Default, Resource)]
pub struct PrefabRegistry {
    prefabs: HashMap<String, Prefab>,
}

impl PrefabRegistry {
    /// Create a new empty prefab registry
    pub fn new() -> Self {
        Self {
            prefabs: HashMap::new(),
        }
    }

    /// Register a prefab
    pub fn register(&mut self, prefab: Prefab) {
        let name = prefab.name.clone();
        self.prefabs.insert(name, prefab);
    }

    /// Get a prefab by name
    pub fn get(&self, name: &str) -> Option<&Prefab> {
        self.prefabs.get(name)
    }

    /// Remove a prefab by name
    pub fn remove(&mut self, name: &str) -> Option<Prefab> {
        self.prefabs.remove(name)
    }

    /// Check if a prefab exists
    pub fn contains(&self, name: &str) -> bool {
        self.prefabs.contains_key(name)
    }

    /// Get all registered prefab names
    pub fn prefab_names(&self) -> Vec<&String> {
        self.prefabs.keys().collect()
    }

    /// Load prefab from RON file
    pub fn load_from_ron(&mut self, name: impl Into<String>, data: &str) -> Result<(), ron::error::SpannedError> {
        let prefab = Prefab::from_ron(data)?;
        self.prefabs.insert(name.into(), prefab);
        Ok(())
    }
}

/// Helper trait for spawning entities from prefabs
pub trait PrefabSpawner {
    /// Spawn an entity from a prefab
    /// Note: This returns the raw component data - the actual component deserialization
    /// must be handled by the caller based on registered component types
    fn spawn_prefab(&self, name: &str) -> Option<&Prefab>;
}

impl PrefabSpawner for PrefabRegistry {
    fn spawn_prefab(&self, name: &str) -> Option<&Prefab> {
        self.get(name)
    }
}

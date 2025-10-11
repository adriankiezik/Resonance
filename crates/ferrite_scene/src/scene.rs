//! Scene representation and serialization.

use ferrite_core::{FerriteError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Scene serialization format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// RON format - Human-readable, slower, larger files (~3-5x slower than Bincode)
    /// Best for: Development, hand-editing, debugging
    /// Extension: .ron
    Ron,
    /// Bincode format - Binary, faster, smaller files (~3-5x faster than RON)
    /// Best for: Production builds, large scenes, runtime performance
    /// Extension: .scene or .bin
    Bincode,
}

impl SerializationFormat {
    /// Detect format from file extension
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        match path.extension().and_then(|e| e.to_str()) {
            Some("ron") => SerializationFormat::Ron,
            Some("scene") | Some("bin") => SerializationFormat::Bincode,
            _ => SerializationFormat::Ron, // Default to RON for unknown extensions
        }
    }

    /// Get recommended file extension
    pub fn extension(&self) -> &'static str {
        match self {
            SerializationFormat::Ron => "ron",
            SerializationFormat::Bincode => "scene",
        }
    }
}

/// A scene containing serializable entity and component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Scene name/identifier
    pub name: String,
    /// List of entities with their components
    pub entities: Vec<SceneEntity>,
}

/// Represents a single entity in a scene with its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity {
    /// Entity name (optional identifier)
    pub name: Option<String>,
    /// Component data as key-value pairs
    pub components: HashMap<String, ron::Value>,
}

impl Scene {
    /// Create a new empty scene
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entities: Vec::new(),
        }
    }

    /// Add an entity to the scene
    pub fn add_entity(&mut self, entity: SceneEntity) {
        self.entities.push(entity);
    }

    // ===== RON Serialization (Human-readable, Development) =====

    /// Save scene to RON format (human-readable, slower)
    pub fn to_ron(&self) -> Result<String> {
        ron::ser::to_string_pretty(self, Default::default())
            .map_err(|e| FerriteError::serialization(format!("RON serialization failed: {}", e)))
    }

    /// Load scene from RON format
    pub fn from_ron(data: &str) -> Result<Self> {
        ron::from_str(data)
            .map_err(|e| FerriteError::serialization(format!("RON deserialization failed: {}", e)))
    }

    // ===== Bincode Serialization (Binary, Production) =====

    /// Save scene to Bincode format (binary, 3-5x faster than RON)
    pub fn to_bincode(&self) -> Result<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| FerriteError::serialization(format!("Bincode serialization failed: {}", e)))
    }

    /// Load scene from Bincode format
    pub fn from_bincode(data: &[u8]) -> Result<Self> {
        bincode::serde::decode_from_slice(data, bincode::config::standard())
            .map(|(scene, _size)| scene)
            .map_err(|e| FerriteError::serialization(format!("Bincode deserialization failed: {}", e)))
    }

    // ===== Auto-detecting Serialization =====

    /// Save scene with specified format
    pub fn serialize(&self, format: SerializationFormat) -> Result<Vec<u8>> {
        match format {
            SerializationFormat::Ron => {
                self.to_ron().map(|s| s.into_bytes())
            }
            SerializationFormat::Bincode => {
                self.to_bincode()
            }
        }
    }

    /// Load scene from bytes with specified format
    pub fn deserialize(data: &[u8], format: SerializationFormat) -> Result<Self> {
        match format {
            SerializationFormat::Ron => {
                let text = std::str::from_utf8(data)
                    .map_err(|e| FerriteError::serialization(format!("Invalid UTF-8: {}", e)))?;
                Self::from_ron(text)
            }
            SerializationFormat::Bincode => {
                Self::from_bincode(data)
            }
        }
    }

    /// Auto-detect format from file path and load scene
    pub fn load_from_path(path: impl AsRef<Path>, data: &[u8]) -> Result<Self> {
        let format = SerializationFormat::from_path(path);
        Self::deserialize(data, format)
    }

    /// Convert scene from one format to another
    pub fn convert(&self, target_format: SerializationFormat) -> Result<Vec<u8>> {
        self.serialize(target_format)
    }
}

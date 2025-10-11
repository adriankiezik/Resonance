//! Scene format conversion utilities for build pipelines.
//!
//! This module provides utilities to convert scenes between RON (development)
//! and Bincode (production) formats. Typically used in build scripts to
//! "bake" human-readable RON scenes into fast-loading binary scenes.

use crate::{Prefab, Scene, SerializationFormat};

/// Scene format converter for build pipelines
pub struct SceneConverter;

impl SceneConverter {
    /// Convert a scene from one format to another
    pub fn convert_scene(
        input_data: &[u8],
        source_format: SerializationFormat,
        target_format: SerializationFormat,
    ) -> Result<Vec<u8>, String> {
        // Deserialize from source format
        let scene = Scene::deserialize(input_data, source_format)?;

        // Serialize to target format
        scene.serialize(target_format)
    }

    /// Convert a prefab from one format to another
    pub fn convert_prefab(
        input_data: &[u8],
        source_format: SerializationFormat,
        target_format: SerializationFormat,
    ) -> Result<Vec<u8>, String> {
        // Deserialize from source format
        let prefab = Prefab::deserialize(input_data, source_format)?;

        // Serialize to target format
        prefab.serialize(target_format)
    }

    /// Batch convert scenes from RON to Bincode (typical build pipeline use)
    /// Returns converted data and compression ratio
    pub fn bake_scene(ron_data: &[u8]) -> Result<(Vec<u8>, f32), String> {
        let bincode_data = Self::convert_scene(
            ron_data,
            SerializationFormat::Ron,
            SerializationFormat::Bincode,
        )?;

        let compression_ratio = ron_data.len() as f32 / bincode_data.len() as f32;

        Ok((bincode_data, compression_ratio))
    }

    /// Batch convert prefabs from RON to Bincode
    /// Returns converted data and compression ratio
    pub fn bake_prefab(ron_data: &[u8]) -> Result<(Vec<u8>, f32), String> {
        let bincode_data = Self::convert_prefab(
            ron_data,
            SerializationFormat::Ron,
            SerializationFormat::Bincode,
        )?;

        let compression_ratio = ron_data.len() as f32 / bincode_data.len() as f32;

        Ok((bincode_data, compression_ratio))
    }

    /// Calculate size savings when converting from RON to Bincode
    pub fn calculate_savings(ron_data: &[u8], bincode_data: &[u8]) -> SizeSavings {
        let ron_size = ron_data.len();
        let bincode_size = bincode_data.len();
        let saved_bytes = ron_size.saturating_sub(bincode_size);
        let compression_ratio = ron_size as f32 / bincode_size.max(1) as f32;
        let size_reduction_percent = (saved_bytes as f32 / ron_size as f32) * 100.0;

        SizeSavings {
            ron_size,
            bincode_size,
            saved_bytes,
            compression_ratio,
            size_reduction_percent,
        }
    }
}

/// Statistics about size savings from format conversion
#[derive(Debug, Clone)]
pub struct SizeSavings {
    /// Original RON size in bytes
    pub ron_size: usize,
    /// Bincode size in bytes
    pub bincode_size: usize,
    /// Bytes saved
    pub saved_bytes: usize,
    /// Compression ratio (RON size / Bincode size)
    pub compression_ratio: f32,
    /// Size reduction percentage
    pub size_reduction_percent: f32,
}

impl SizeSavings {
    /// Format as a human-readable string
    pub fn format(&self) -> String {
        format!(
            "RON: {} bytes → Bincode: {} bytes | Saved: {} bytes ({:.1}% smaller) | Compression: {:.2}x",
            self.ron_size,
            self.bincode_size,
            self.saved_bytes,
            self.size_reduction_percent,
            self.compression_ratio
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SceneEntity;
    use std::collections::HashMap;

    #[test]
    fn test_scene_conversion() {
        let mut scene = Scene::new("test_scene");
        scene.add_entity(SceneEntity {
            name: Some("entity1".to_string()),
            components: HashMap::new(),
        });

        // Convert to RON
        let ron_data = scene.to_ron().unwrap();
        let ron_bytes = ron_data.as_bytes();

        // Convert to Bincode
        let bincode_data = scene.to_bincode().unwrap();

        // Verify conversion works both ways
        let scene_from_ron = Scene::from_ron(&ron_data).unwrap();
        let scene_from_bincode = Scene::from_bincode(&bincode_data).unwrap();

        assert_eq!(scene.name, scene_from_ron.name);
        assert_eq!(scene.name, scene_from_bincode.name);

        // Verify size savings
        let savings = SceneConverter::calculate_savings(ron_bytes, &bincode_data);
        assert!(savings.compression_ratio > 1.0); // Bincode should be smaller
        println!("{}", savings.format());
    }
}

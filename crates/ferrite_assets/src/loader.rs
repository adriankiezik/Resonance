//! Asset loading system.

use crate::cache::AssetCache;
use crate::handle::{AssetHandle, AssetId};
use anyhow::Result;
use std::path::Path;
use thiserror::Error;

/// Asset loading errors
#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Asset not found: {0}")]
    NotFound(String),
    #[error("Failed to load asset: {0}")]
    LoadFailed(String),
    #[error("Unsupported asset type: {0}")]
    UnsupportedType(String),
}

/// Asset loader trait
pub trait AssetLoader: Send + Sync {
    type Asset: Send + Sync + 'static;

    /// Load an asset from a file path
    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError>;

    /// Get supported file extensions
    fn extensions(&self) -> &[&str];
}

/// Image asset loader
pub struct ImageLoader;

impl AssetLoader for ImageLoader {
    type Asset = image::DynamicImage;

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {
        image::open(path).map_err(|e| LoadError::LoadFailed(e.to_string()))
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "gif"]
    }
}

// TODO: Add more asset loaders
// - TextureLoader (convert image to GPU texture)
// - MeshLoader (load .obj, .gltf, etc.)
// - AudioLoader (load .wav, .mp3, etc.)
// - FontLoader
// - ShaderLoader

/// Helper function to load an asset
pub fn load_asset<L: AssetLoader>(
    loader: &L,
    path: impl AsRef<Path>,
    cache: &AssetCache,
) -> Result<AssetHandle<L::Asset>, LoadError> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy().to_string();
    let id = AssetId::from_path(&path_str);

    // Check cache first
    if cache.contains::<L::Asset>(id) {
        log::debug!("Asset already cached: {}", path_str);
        return Ok(AssetHandle::new(id, path_str));
    }

    // Load asset
    let asset = loader.load(path)?;

    // Cache it
    cache.insert(id, asset);

    Ok(AssetHandle::new(id, path_str))
}

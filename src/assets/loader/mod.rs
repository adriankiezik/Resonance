pub mod audio;
pub mod font;
pub mod mesh;
pub mod shader;
pub mod texture;

use crate::assets::cache::{AssetCache, CachePolicy};
use crate::assets::handle::{AssetHandle, AssetId};
use anyhow::Result;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Asset not found: {0}")]
    NotFound(String),
    #[error("Failed to load asset: {0}")]
    LoadFailed(String),
    #[error("Unsupported asset type: {0}")]
    UnsupportedType(String),
}

pub trait AssetLoader: Send + Sync {
    type Asset: Send + Sync + 'static;
    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError>;
    fn extensions(&self) -> &[&str];
    fn cache_policy(&self) -> CachePolicy {
        CachePolicy::Weak
    }
}

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

pub fn load_asset<L: AssetLoader>(
    loader: &L,
    path: impl AsRef<Path>,
    cache: &AssetCache,
) -> Result<AssetHandle<L::Asset>, LoadError> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy().to_string();
    let id = AssetId::from_path(&path_str);

    if cache.contains::<L::Asset>(id) {
        log::debug!("Asset already cached: {}", path_str);
        return Ok(AssetHandle::new(id, path_str));
    }

    let asset = loader.load(path)?;
    let policy = loader.cache_policy();

    cache.insert(id, asset, policy);

    Ok(AssetHandle::new(id, path_str))
}

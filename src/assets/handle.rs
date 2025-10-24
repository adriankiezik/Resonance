use bevy_ecs::component::Component;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(pub u64);

impl AssetId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn from_path(path: &str) -> Self {
        let hash = path
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        Self(hash)
    }
}

#[derive(Debug, Clone, Component)]
pub struct AssetHandle<T> {
    pub asset: Arc<T>,
    pub id: AssetId,
    pub path: String,
}

impl<T> AssetHandle<T> {
    pub fn new(asset: Arc<T>, id: AssetId, path: impl Into<String>) -> Self {
        Self {
            asset,
            id,
            path: path.into(),
        }
    }

    pub fn from_path_and_asset(path: impl Into<String>, asset: Arc<T>) -> Self {
        let path = path.into();
        let id = AssetId::from_path(&path);
        Self::new(asset, id, path)
    }
}

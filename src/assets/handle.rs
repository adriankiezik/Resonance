use std::marker::PhantomData;

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

#[cfg(not(target_arch = "wasm32"))]
use bevy_ecs::prelude::Component;

#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Component))]
pub struct AssetHandle<T> {
    pub id: AssetId,
    pub path: String,
    _phantom: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    pub fn new(id: AssetId, path: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            _phantom: PhantomData,
        }
    }

    pub fn from_path(path: impl Into<String>) -> Self {
        let path = path.into();
        let id = AssetId::from_path(&path);
        Self::new(id, path)
    }
}

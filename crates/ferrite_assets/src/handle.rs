//! Asset handles for safe references.

use std::marker::PhantomData;

/// Unique identifier for an asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(pub u64);

impl AssetId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn from_path(path: &str) -> Self {
        // Simple hash of path for now
        // TODO: Use a better hash function
        let hash = path.bytes().fold(0u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        Self(hash)
    }
}

/// Type-safe handle to an asset
#[derive(Debug, Clone)]
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

// TODO: Implement weak handles that don't keep assets loaded
// TODO: Add reference counting for automatic unloading

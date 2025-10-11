//! Asset caching system.

use crate::handle::AssetId;
use bevy_ecs::prelude::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Asset cache resource
#[derive(Resource)]
pub struct AssetCache {
    assets: Arc<RwLock<HashMap<(TypeId, AssetId), Arc<dyn Any + Send + Sync>>>>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            assets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert an asset into the cache
    pub fn insert<T: Send + Sync + 'static>(&self, id: AssetId, asset: T) {
        let type_id = TypeId::of::<T>();
        let mut assets = self.assets.write().unwrap();
        assets.insert((type_id, id), Arc::new(asset));
        log::debug!("Cached asset: {:?}", id);
    }

    /// Get an asset from the cache
    pub fn get<T: Send + Sync + 'static>(&self, id: AssetId) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        let assets = self.assets.read().unwrap();
        assets
            .get(&(type_id, id))
            .and_then(|arc| arc.clone().downcast::<T>().ok())
    }

    /// Check if an asset is cached
    pub fn contains<T: Send + Sync + 'static>(&self, id: AssetId) -> bool {
        let type_id = TypeId::of::<T>();
        let assets = self.assets.read().unwrap();
        assets.contains_key(&(type_id, id))
    }

    /// Remove an asset from the cache
    pub fn remove<T: Send + Sync + 'static>(&self, id: AssetId) {
        let type_id = TypeId::of::<T>();
        let mut assets = self.assets.write().unwrap();
        assets.remove(&(type_id, id));
    }

    /// Clear all assets of a specific type
    pub fn clear_type<T: Send + Sync + 'static>(&self) {
        let type_id = TypeId::of::<T>();
        let mut assets = self.assets.write().unwrap();
        assets.retain(|(tid, _), _| *tid != type_id);
    }

    /// Clear all cached assets
    pub fn clear_all(&self) {
        let mut assets = self.assets.write().unwrap();
        assets.clear();
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}

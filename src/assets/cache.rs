use crate::assets::handle::AssetId;
use bevy_ecs::prelude::*;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::{Arc, Weak};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CachePolicy {
    Weak,
    Strong,
}

enum CachedAsset {
    Weak(Weak<dyn Any + Send + Sync>),
    Strong(Arc<dyn Any + Send + Sync>),
}

#[derive(Resource, Clone)]
pub struct AssetCache {
    assets: Arc<DashMap<(TypeId, AssetId), CachedAsset>>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            assets: Arc::new(DashMap::new()),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(
        &self,
        path: impl Into<String>,
        asset: T,
        policy: CachePolicy,
    ) -> crate::assets::handle::AssetHandle<T> {
        let path = path.into();
        let id = crate::assets::handle::AssetId::from_path(&path);
        let type_id = TypeId::of::<T>();
        let arc = Arc::new(asset);

        let cached = match policy {
            CachePolicy::Weak => {
                CachedAsset::Weak(Arc::downgrade(&arc) as Weak<dyn Any + Send + Sync>)
            }
            CachePolicy::Strong => CachedAsset::Strong(arc.clone() as Arc<dyn Any + Send + Sync>),
        };

        self.assets.insert((type_id, id), cached);
        log::trace!("Cached asset {:?} with policy {:?}", id, policy);

        crate::assets::handle::AssetHandle::new(arc, id, path)
    }

    pub fn get<T: Send + Sync + 'static>(&self, id: AssetId) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        let key = (type_id, id);

        let entry = self.assets.get(&key)?;

        match &*entry {
            CachedAsset::Strong(arc) => arc.clone().downcast::<T>().ok(),
            CachedAsset::Weak(weak) => {
                if let Some(arc) = weak.upgrade() {
                    arc.downcast::<T>().ok()
                } else {
                    drop(entry);
                    self.assets.remove(&key);
                    None
                }
            }
        }
    }

    pub fn contains<T: Send + Sync + 'static>(&self, id: AssetId) -> bool {
        let type_id = TypeId::of::<T>();
        self.assets.contains_key(&(type_id, id))
    }

    pub fn remove<T: Send + Sync + 'static>(&self, id: AssetId) {
        let type_id = TypeId::of::<T>();
        self.assets.remove(&(type_id, id));
    }

    pub fn clear_type<T: Send + Sync + 'static>(&self) {
        let type_id = TypeId::of::<T>();
        self.assets.retain(|(tid, _), _| *tid != type_id);
    }

    pub fn clear_all(&self) {
        self.assets.clear();
    }

    pub fn iter_type<T: Send + Sync + 'static>(&self) -> Vec<Arc<T>> {
        let type_id = TypeId::of::<T>();
        let mut results = Vec::new();

        for entry in self.assets.iter() {
            let ((tid, id), cached) = entry.pair();
            if *tid == type_id {
                let arc_any = match cached {
                    CachedAsset::Strong(arc) => Some(arc.clone()),
                    CachedAsset::Weak(weak) => weak.upgrade(),
                };

                if let Some(arc_any) = arc_any {
                    let typed_arc = unsafe {
                        let raw = Arc::into_raw(arc_any);
                        let typed_raw = raw as *const T;
                        Arc::from_raw(typed_raw)
                    };
                    results.push(typed_arc);
                }
            }
        }

        results
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}

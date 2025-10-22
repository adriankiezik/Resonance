use crate::assets::cache::AssetCache;
use crate::assets::handle::{AssetHandle, AssetId};
use crate::assets::loader::AssetLoader;
use bevy_ecs::prelude::*;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
pub enum LoadState<T> {
    Loading,
    Loaded(Arc<T>),
    Failed(String),
}

impl<T> Clone for LoadState<T> {
    fn clone(&self) -> Self {
        match self {
            LoadState::Loading => LoadState::Loading,
            LoadState::Loaded(arc) => LoadState::Loaded(arc.clone()),
            LoadState::Failed(s) => LoadState::Failed(s.clone()),
        }
    }
}

#[derive(Resource)]
pub struct Assets {
    runtime: tokio::runtime::Handle,
    cache: Arc<AssetCache>,
    states: Arc<DashMap<AssetId, Box<dyn std::any::Any + Send + Sync>>>,
}

impl Assets {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Handle::try_current()
            .expect("Assets requires a tokio runtime to be running");

        Self {
            runtime,
            cache: Arc::new(AssetCache::new()),
            states: Arc::new(DashMap::new()),
        }
    }

    pub fn with_cache(cache: Arc<AssetCache>) -> Self {
        let runtime = tokio::runtime::Handle::try_current()
            .expect("Assets requires a tokio runtime to be running");

        Self {
            runtime,
            cache,
            states: Arc::new(DashMap::new()),
        }
    }

    pub fn cache(&self) -> &Arc<AssetCache> {
        &self.cache
    }

    pub fn load<L: AssetLoader + 'static>(
        &self,
        loader: L,
        path: impl AsRef<Path>,
    ) -> AssetHandle<L::Asset> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();
        let id = AssetId::from_path(&path_str);
        let handle = AssetHandle::new(id, path_str.clone());

        if self.cache.contains::<L::Asset>(id) {
            log::debug!("Asset already cached: {}", path_str);
            return handle;
        }

        self.states
            .insert(id, Box::new(LoadState::<L::Asset>::Loading));

        let states_clone = self.states.clone();
        let cache_clone = self.cache.clone();
        let path_buf = PathBuf::from(path);
        let policy = loader.cache_policy();

        self.runtime.spawn(async move {
            let result = tokio::task::spawn_blocking(move || loader.load(&path_buf)).await;

            let state = match result {
                Ok(Ok(asset)) => {
                    let arc = cache_clone.insert(id, asset, policy);
                    log::debug!("Async loaded asset: {}", path_str);
                    LoadState::Loaded(arc)
                }
                Ok(Err(e)) => {
                    log::error!("Failed to load asset {}: {}", path_str, e);
                    LoadState::Failed(e.to_string())
                }
                Err(e) => {
                    log::error!("Task panicked while loading {}: {}", path_str, e);
                    LoadState::Failed(format!("Task panicked: {}", e))
                }
            };

            states_clone.insert(id, Box::new(state));
        });

        handle
    }

    pub fn get<T: Send + Sync + 'static>(&self, id: AssetId) -> Option<Arc<T>> {
        self.cache.get::<T>(id)
    }

    pub fn get_state<T: Send + Sync + 'static>(&self, id: AssetId) -> Option<LoadState<T>> {
        self.states
            .get(&id)
            .and_then(|boxed| boxed.downcast_ref::<LoadState<T>>().cloned())
    }

    pub fn is_loaded<T: Send + Sync + 'static>(&self, id: AssetId) -> bool {
        matches!(self.get_state::<T>(id), Some(LoadState::Loaded(_)))
    }

    pub fn is_loading<T: Send + Sync + 'static>(&self, id: AssetId) -> bool {
        matches!(self.get_state::<T>(id), Some(LoadState::Loading))
    }

    pub fn clear_state(&self, id: AssetId) {
        self.states.remove(&id);
    }

    pub fn load_batch<L: AssetLoader + Clone + 'static>(
        &self,
        loader: L,
        paths: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> Vec<AssetHandle<L::Asset>> {
        let paths: Vec<_> = paths.into_iter().collect();
        let mut handles = Vec::with_capacity(paths.len());

        for path in paths {
            let handle = self.load(loader.clone(), path);
            handles.push(handle);
        }

        handles
    }

    pub fn all_loaded<T: Send + Sync + 'static>(&self, handles: &[AssetHandle<T>]) -> bool {
        handles.iter().all(|handle| {
            self.states
                .get(&handle.id)
                .and_then(|state| {
                    state
                        .downcast_ref::<LoadState<T>>()
                        .map(|s| matches!(s, LoadState::Loaded(_)))
                })
                .unwrap_or(false)
        })
    }

    pub fn any_failed<T: Send + Sync + 'static>(&self, handles: &[AssetHandle<T>]) -> bool {
        handles.iter().any(|handle| {
            self.states
                .get(&handle.id)
                .and_then(|state| {
                    state
                        .downcast_ref::<LoadState<T>>()
                        .map(|s| matches!(s, LoadState::Failed(_)))
                })
                .unwrap_or(false)
        })
    }

    pub fn loading_progress<T: Send + Sync + 'static>(
        &self,
        handles: &[AssetHandle<T>],
    ) -> (usize, usize) {
        let total = handles.len();
        let loaded = handles
            .iter()
            .filter(|handle| {
                self.states
                    .get(&handle.id)
                    .and_then(|state| {
                        state
                            .downcast_ref::<LoadState<T>>()
                            .map(|s| matches!(s, LoadState::Loaded(_)))
                    })
                    .unwrap_or(false)
            })
            .count();

        (loaded, total)
    }
}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}

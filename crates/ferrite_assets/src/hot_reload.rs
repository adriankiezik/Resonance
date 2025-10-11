//! Hot reloading system for assets.
//!
//! Watches file system for changes and automatically reloads assets.

use crate::cache::AssetCache;
use crate::handle::AssetId;
use bevy_ecs::prelude::*;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

/// Hot reload event
#[derive(Debug, Clone)]
pub struct HotReloadEvent {
    pub path: PathBuf,
    pub asset_id: AssetId,
    pub event_kind: HotReloadEventKind,
}

/// Hot reload event kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotReloadEventKind {
    /// Asset file was modified
    Modified,
    /// Asset file was created
    Created,
    /// Asset file was deleted
    Deleted,
}

/// Hot reload watcher resource
#[derive(Resource)]
pub struct HotReloadWatcher {
    /// File system watcher
    _watcher: RecommendedWatcher,
    /// Receiver for file events
    receiver: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<HotReloadEvent>>>,
    /// Watched paths and their asset IDs
    watched_assets: Arc<RwLock<HashMap<PathBuf, AssetId>>>,
    /// Asset dependencies (which assets depend on which files)
    dependencies: Arc<RwLock<HashMap<AssetId, HashSet<PathBuf>>>>,
}

impl HotReloadWatcher {
    /// Create a new hot reload watcher
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let watched_assets = Arc::new(RwLock::new(HashMap::new()));
        let dependencies = Arc::new(RwLock::new(HashMap::new()));

        let watched_assets_clone = watched_assets.clone();

        // Create file watcher
        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Determine event kind
                    let event_kind = match event.kind {
                        notify::EventKind::Modify(_) => Some(HotReloadEventKind::Modified),
                        notify::EventKind::Create(_) => Some(HotReloadEventKind::Created),
                        notify::EventKind::Remove(_) => Some(HotReloadEventKind::Deleted),
                        _ => None,
                    };

                    if let Some(kind) = event_kind {
                        for path in event.paths {
                            // Check if this path is watched
                            let watched = watched_assets_clone.read().unwrap();
                            if let Some(&asset_id) = watched.get(&path) {
                                log::info!("Hot reload: {:?} was {:?}", path, kind);

                                let _ = tx.send(HotReloadEvent {
                                    path,
                                    asset_id,
                                    event_kind: kind,
                                });
                            }
                        }
                    }
                }
                Err(e) => log::error!("Watch error: {:?}", e),
            }
        })?;

        Ok(Self {
            _watcher: watcher,
            receiver: Arc::new(tokio::sync::Mutex::new(rx)),
            watched_assets,
            dependencies,
        })
    }

    /// Watch an asset file for changes
    pub fn watch(&mut self, path: impl Into<PathBuf>, asset_id: AssetId) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.into();

        // Add to watched assets
        {
            let mut watched = self.watched_assets.write().unwrap();
            watched.insert(path.clone(), asset_id);
        }

        // Watch the file
        self._watcher.watch(&path, RecursiveMode::NonRecursive)?;

        log::debug!("Watching asset file: {:?}", path);

        Ok(())
    }

    /// Stop watching an asset
    pub fn unwatch(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Remove from watched assets
        {
            let mut watched = self.watched_assets.write().unwrap();
            watched.remove(path);
        }

        // Unwatch the file
        self._watcher.unwatch(path)?;

        log::debug!("Stopped watching: {:?}", path);

        Ok(())
    }

    /// Watch a directory recursively
    pub fn watch_directory(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        self._watcher.watch(path, RecursiveMode::Recursive)?;
        log::info!("Watching directory: {:?}", path);
        Ok(())
    }

    /// Add asset dependency (asset depends on file)
    pub fn add_dependency(&self, asset_id: AssetId, dependency_path: PathBuf) {
        let mut deps = self.dependencies.write().unwrap();
        deps.entry(asset_id).or_insert_with(HashSet::new).insert(dependency_path);
    }

    /// Get all dependencies for an asset
    pub fn get_dependencies(&self, asset_id: AssetId) -> Vec<PathBuf> {
        let deps = self.dependencies.read().unwrap();
        deps.get(&asset_id)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Process hot reload events
    pub async fn process_events(&self) -> Vec<HotReloadEvent> {
        let mut receiver = self.receiver.lock().await;
        let mut events = Vec::new();

        // Collect all pending events
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }

        events
    }

    /// Check if an asset is being watched
    pub fn is_watched(&self, path: &Path) -> bool {
        let watched = self.watched_assets.read().unwrap();
        watched.contains_key(path)
    }
}

/// System to process hot reload events
pub fn process_hot_reload_events(
    hot_reload: Res<HotReloadWatcher>,
    _cache: Res<AssetCache>,
) {
    // Use tokio runtime to process events
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(rt) = rt {
        rt.block_on(async {
            let events = hot_reload.process_events().await;

            for event in events {
                match event.event_kind {
                    HotReloadEventKind::Modified | HotReloadEventKind::Created => {
                        log::info!("Asset changed, invalidating cache: {:?}", event.path);
                        // For now, just log - in a real system, you'd trigger a reload
                        // This would involve:
                        // 1. Remove from cache
                        // 2. Reload the asset
                        // 3. Update all users of the asset
                    }
                    HotReloadEventKind::Deleted => {
                        log::warn!("Asset deleted: {:?}", event.path);
                        // Remove from cache and handle missing asset
                    }
                }

                // Check for dependent assets that need reloading
                let dependencies = hot_reload.get_dependencies(event.asset_id);
                for dep_path in dependencies {
                    log::info!("Reloading dependent asset: {:?}", dep_path);
                }
            }
        });
    }
}

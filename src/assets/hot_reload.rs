use crate::assets::cache::AssetCache;
use crate::assets::handle::AssetId;
use bevy_ecs::prelude::*;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct HotReloadEvent {
    pub path: PathBuf,
    pub asset_id: AssetId,
    pub event_kind: HotReloadEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotReloadEventKind {

    Modified,

    Created,

    Deleted,
}

#[derive(Resource)]
pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    receiver: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<HotReloadEvent>>>,
    watched_assets: Arc<RwLock<HashMap<PathBuf, AssetId>>>,
    dependencies: Arc<RwLock<HashMap<AssetId, HashSet<PathBuf>>>>,
}

impl HotReloadWatcher {

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let watched_assets = Arc::new(RwLock::new(HashMap::new()));
        let dependencies = Arc::new(RwLock::new(HashMap::new()));

        let watched_assets_clone = watched_assets.clone();

        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {

                    let event_kind = match event.kind {
                        notify::EventKind::Modify(_) => Some(HotReloadEventKind::Modified),
                        notify::EventKind::Create(_) => Some(HotReloadEventKind::Created),
                        notify::EventKind::Remove(_) => Some(HotReloadEventKind::Deleted),
                        _ => None,
                    };

                    if let Some(kind) = event_kind {
                        for path in event.paths {

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

    pub fn watch(&mut self, path: impl Into<PathBuf>, asset_id: AssetId) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.into();

        {
            let mut watched = self.watched_assets.write().unwrap();
            watched.insert(path.clone(), asset_id);
        }

        self._watcher.watch(&path, RecursiveMode::NonRecursive)?;

        log::debug!("Watching asset file: {:?}", path);

        Ok(())
    }

    pub fn unwatch(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut watched = self.watched_assets.write().unwrap();
            watched.remove(path);
        }

        self._watcher.unwatch(path)?;

        log::debug!("Stopped watching: {:?}", path);

        Ok(())
    }

    pub fn watch_directory(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        self._watcher.watch(path, RecursiveMode::Recursive)?;
        log::info!("Watching directory: {:?}", path);
        Ok(())
    }

    pub fn add_dependency(&self, asset_id: AssetId, dependency_path: PathBuf) {
        let mut deps = self.dependencies.write().unwrap();
        deps.entry(asset_id).or_insert_with(HashSet::new).insert(dependency_path);
    }

    pub fn get_dependencies(&self, asset_id: AssetId) -> Vec<PathBuf> {
        let deps = self.dependencies.read().unwrap();
        deps.get(&asset_id)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn process_events(&self) -> Vec<HotReloadEvent> {
        let mut receiver = self.receiver.lock().await;
        let mut events = Vec::new();

        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }

        events
    }

    pub fn is_watched(&self, path: &Path) -> bool {
        let watched = self.watched_assets.read().unwrap();
        watched.contains_key(path)
    }
}

pub fn process_hot_reload_events(
    hot_reload: Res<HotReloadWatcher>,
    _cache: Res<AssetCache>,
) {

    let rt = tokio::runtime::Handle::try_current();
    if let Ok(rt) = rt {
        rt.block_on(async {
            let events = hot_reload.process_events().await;

            for event in events {
                match event.event_kind {
                    HotReloadEventKind::Modified | HotReloadEventKind::Created => {
                        log::info!("Asset changed, invalidating cache: {:?}", event.path);

                    }
                    HotReloadEventKind::Deleted => {
                        log::warn!("Asset deleted: {:?}", event.path);

                    }
                }

                let dependencies = hot_reload.get_dependencies(event.asset_id);
                for dep_path in dependencies {
                    log::info!("Reloading dependent asset: {:?}", dep_path);
                }
            }
        });
    }
}

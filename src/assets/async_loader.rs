use crate::assets::cache::AssetCache;
use crate::assets::handle::{AssetHandle, AssetId};
use crate::assets::loader::LoadError;
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Queued,
    Loading,
    Loaded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct LoadProgress {
    pub state: LoadState,
    pub progress: f32,
    pub error: Option<String>,
}

impl LoadProgress {
    pub fn new(state: LoadState) -> Self {
        Self {
            state,
            progress: match state {
                LoadState::Queued => 0.0,
                LoadState::Loading => 0.5,
                LoadState::Loaded => 1.0,
                LoadState::Failed => 0.0,
            },
            error: None,
        }
    }

    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }
}

#[derive(Debug)]
pub enum LoaderMessage<T: Send + Sync + 'static> {
    Progress {
        id: AssetId,
        progress: LoadProgress,
    },
    Loaded {
        id: AssetId,
        asset: T,
        path: String,
    },
    Failed {
        id: AssetId,
        error: String,
    },
}

#[derive(Resource)]
pub struct AsyncAssetLoader {
    progress: Arc<RwLock<HashMap<AssetId, LoadProgress>>>,
    receiver: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<LoaderMessage<Vec<u8>>>>>,
    sender: mpsc::UnboundedSender<LoaderMessage<Vec<u8>>>,
}

impl AsyncAssetLoader {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            progress: Arc::new(RwLock::new(HashMap::new())),
            receiver: Arc::new(tokio::sync::Mutex::new(receiver)),
            sender,
        }
    }

    pub fn load_async<T: Send + Sync + 'static>(
        &self,
        path: impl Into<PathBuf>,
        _loader: impl Fn(&[u8]) -> Result<T, LoadError> + Send + 'static,
    ) -> AssetHandle<T> {
        let path = path.into();
        let path_str = path.to_string_lossy().to_string();
        let id = AssetId::from_path(&path_str);

        {
            let mut progress = self.progress.write().unwrap();
            progress.insert(id, LoadProgress::new(LoadState::Queued));
        }

        let sender = self.sender.clone();
        let progress = self.progress.clone();
        let path_str_clone = path_str.clone();

        tokio::spawn(async move {

            {
                let mut prog = progress.write().unwrap();
                prog.insert(id, LoadProgress::new(LoadState::Loading));
            }

            let _ = sender.send(LoaderMessage::Progress {
                id,
                progress: LoadProgress::new(LoadState::Loading),
            });

            match tokio::fs::read(&path).await {
                Ok(data) => {
                    log::debug!("Loaded asset file: {} ({} bytes)", path_str_clone, data.len());

                    let _ = sender.send(LoaderMessage::Loaded {
                        id,
                        asset: data,
                        path: path_str_clone.clone(),
                    });

                    let mut prog = progress.write().unwrap();
                    prog.insert(id, LoadProgress::new(LoadState::Loaded));
                }
                Err(e) => {
                    let error = format!("Failed to load {}: {}", path_str_clone, e);
                    log::error!("{}", error);

                    let _ = sender.send(LoaderMessage::Failed {
                        id,
                        error: error.clone(),
                    });

                    let mut prog = progress.write().unwrap();
                    prog.insert(
                        id,
                        LoadProgress::new(LoadState::Failed).with_error(error),
                    );
                }
            }
        });

        AssetHandle::new(id, path_str)
    }

    pub fn get_progress(&self, id: AssetId) -> Option<LoadProgress> {
        let progress = self.progress.read().unwrap();
        progress.get(&id).cloned()
    }

    pub fn is_loaded(&self, id: AssetId) -> bool {
        self.get_progress(id)
            .map(|p| p.state == LoadState::Loaded)
            .unwrap_or(false)
    }

    pub fn is_failed(&self, id: AssetId) -> bool {
        self.get_progress(id)
            .map(|p| p.state == LoadState::Failed)
            .unwrap_or(false)
    }

    pub fn get_total_progress(&self) -> f32 {
        let progress = self.progress.read().unwrap();
        if progress.is_empty() {
            return 1.0;
        }

        let total: f32 = progress.values().map(|p| p.progress).sum();
        total / progress.len() as f32
    }

    pub async fn process_messages(&self, cache: &AssetCache) -> usize {
        let mut receiver = self.receiver.lock().await;
        let mut processed = 0;

        while let Ok(msg) = receiver.try_recv() {
            match msg {
                LoaderMessage::Progress { id, progress } => {
                    log::debug!("Asset {:?} progress: {:?}", id, progress.state);
                }
                LoaderMessage::Loaded { id, asset, path: _ } => {
                    cache.insert(id, asset);
                    processed += 1;
                }
                LoaderMessage::Failed { id: _, error: _ } => {
                    processed += 1;
                }
            }
        }

        processed
    }

    pub fn clear_progress(&self, id: AssetId) {
        let mut progress = self.progress.write().unwrap();
        progress.remove(&id);
    }

    pub fn clear_all_progress(&self) {
        let mut progress = self.progress.write().unwrap();
        progress.clear();
    }
}

impl Default for AsyncAssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

pub fn process_asset_loading(
    async_loader: Res<AsyncAssetLoader>,
    cache: Res<AssetCache>,
) {
    let rt = tokio::runtime::Handle::try_current();

    if let Ok(rt) = rt {
        rt.block_on(async {
            async_loader.process_messages(&cache).await;
        });
    }
}

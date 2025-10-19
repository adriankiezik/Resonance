use crate::app::{Engine, Plugin, Stage};
use crate::assets::async_loader::{AsyncAssetLoader, process_asset_loading};
use crate::assets::cache::AssetCache;
use crate::assets::hot_reload::{HotReloadWatcher, process_hot_reload_events};
use crate::assets::source::AssetSourceConfig;

pub struct AssetsPluginConfig {
    pub enable_hot_reload: bool,
    pub enable_async_loading: bool,
    pub asset_source: AssetSourceConfig,
}

impl Default for AssetsPluginConfig {
    fn default() -> Self {
        Self {
            enable_hot_reload: true,
            enable_async_loading: true,
            asset_source: AssetSourceConfig::Auto,
        }
    }
}

pub struct AssetsPlugin {
    config: AssetsPluginConfig,
}

impl AssetsPlugin {
    pub fn new() -> Self {
        Self {
            config: AssetsPluginConfig::default(),
        }
    }

    pub fn with_config(config: AssetsPluginConfig) -> Self {
        Self { config }
    }

    pub fn without_hot_reload(mut self) -> Self {
        self.config.enable_hot_reload = false;
        self
    }

    pub fn without_async_loading(mut self) -> Self {
        self.config.enable_async_loading = false;
        self
    }

    pub fn with_asset_source(mut self, source: AssetSourceConfig) -> Self {
        self.config.asset_source = source;
        self
    }
}

impl Default for AssetsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AssetsPlugin {
    fn build(&self, engine: &mut Engine) {
        engine.world.insert_resource(AssetCache::new());

        let source = match self.config.asset_source.clone().resolve() {
            Ok(source) => source,
            Err(e) => {
                log::error!("Failed to initialize asset source: {}", e);
                log::error!("AssetsPlugin initialization failed");
                return;
            }
        };

        let supports_hot_reload = source.supports_hot_reload();

        if self.config.enable_async_loading {
            engine.world.insert_resource(AsyncAssetLoader::new(source));

            if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
                schedule.add_systems(process_asset_loading);
            }
        }

        if self.config.enable_hot_reload {
            if !supports_hot_reload {
                log::warn!(
                    "Hot reload requested but asset source does not support it (PAK archives cannot be hot-reloaded)"
                );
            } else {
                match HotReloadWatcher::new() {
                    Ok(watcher) => {
                        engine.world.insert_resource(watcher);

                        if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
                            schedule.add_systems(process_hot_reload_events);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to initialize hot reload watcher: {}", e);
                    }
                }
            }
        }
    }
}


use crate::assets::async_loader::{process_asset_loading, AsyncAssetLoader};
use crate::assets::cache::AssetCache;
use crate::assets::hot_reload::{process_hot_reload_events, HotReloadWatcher};
use crate::app::{Engine, Plugin, Stage};

pub struct AssetsPluginConfig {
    pub enable_hot_reload: bool,
    pub enable_async_loading: bool,
}

impl Default for AssetsPluginConfig {
    fn default() -> Self {
        Self {
            enable_hot_reload: true,
            enable_async_loading: true,
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
}

impl Default for AssetsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AssetsPlugin {
    fn build(&self, engine: &mut Engine) {

        engine.world.insert_resource(AssetCache::new());

        if self.config.enable_async_loading {
            engine.world.insert_resource(AsyncAssetLoader::new());

            if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
                schedule.add_systems(process_asset_loading);
            }
            log::info!("Async asset loading enabled");
        }

        if self.config.enable_hot_reload {
            match HotReloadWatcher::new() {
                Ok(watcher) => {
                    engine.world.insert_resource(watcher);

                    if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
                        schedule.add_systems(process_hot_reload_events);
                    }
                    log::info!("Hot reloading enabled");
                }
                Err(e) => {
                    log::error!("Failed to initialize hot reload watcher: {}", e);
                }
            }
        }

        log::info!("AssetsPlugin initialized");
    }
}

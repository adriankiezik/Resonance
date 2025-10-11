//! Assets plugin.

use crate::async_loader::{process_asset_loading, AsyncAssetLoader};
use crate::cache::AssetCache;
use crate::hot_reload::{process_hot_reload_events, HotReloadWatcher};
use ferrite_app::{Engine, Plugin, Stage};

/// Configuration for the assets plugin
pub struct AssetsPluginConfig {
    /// Enable hot reloading (file watching)
    pub enable_hot_reload: bool,
    /// Enable async loading
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

/// Plugin for asset management
pub struct AssetsPlugin {
    config: AssetsPluginConfig,
}

impl AssetsPlugin {
    /// Create with default configuration
    pub fn new() -> Self {
        Self {
            config: AssetsPluginConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AssetsPluginConfig) -> Self {
        Self { config }
    }

    /// Disable hot reloading
    pub fn without_hot_reload(mut self) -> Self {
        self.config.enable_hot_reload = false;
        self
    }

    /// Disable async loading
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
        // Insert asset cache
        engine.world.insert_resource(AssetCache::new());

        // Insert async loader if enabled
        if self.config.enable_async_loading {
            engine.world.insert_resource(AsyncAssetLoader::new());

            // Add system directly to schedule
            if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
                schedule.add_systems(process_asset_loading);
            }
            log::info!("Async asset loading enabled");
        }

        // Insert hot reload watcher if enabled
        if self.config.enable_hot_reload {
            match HotReloadWatcher::new() {
                Ok(watcher) => {
                    engine.world.insert_resource(watcher);

                    // Add system directly to schedule
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

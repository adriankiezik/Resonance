use crate::app::{Plugin, Resonance, Stage};
use crate::assets::assets::Assets;
use crate::assets::cache::AssetCache;
use crate::assets::hot_reload::{HotReloadWatcher, process_hot_reload_events};
use crate::assets::loader::mesh::MeshData;
use crate::assets::loader::texture::TextureData;
use crate::assets::source::AssetSourceConfig;
use crate::core::MemoryTracker;
use bevy_ecs::prelude::*;

pub struct AssetsPluginConfig {
    pub enable_hot_reload: bool,
    pub asset_source: AssetSourceConfig,
}

impl Default for AssetsPluginConfig {
    fn default() -> Self {
        Self {
            enable_hot_reload: true,
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
    fn build(&self, engine: &mut Resonance) {
        let assets = Assets::new();
        let cache = (**assets.cache()).clone();
        engine.world.insert_resource(assets);
        engine.world.insert_resource(cache);

        if let Some(schedule) = engine.schedules.get_mut(Stage::PostUpdate) {
            schedule.add_systems(update_asset_memory_stats);
        }

        let source = match self.config.asset_source.clone().resolve() {
            Ok(source) => source,
            Err(e) => {
                log::error!("Failed to initialize asset source: {}", e);
                log::error!("AssetsPlugin initialization failed");
                return;
            }
        };

        let supports_hot_reload = source.supports_hot_reload();

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

fn update_asset_memory_stats(
    asset_cache: Res<AssetCache>,
    mut memory_tracker: ResMut<MemoryTracker>,
) {
    memory_tracker.assets.textures = 0;
    memory_tracker.assets.meshes = 0;

    let textures = asset_cache.iter_type::<TextureData>();
    for texture in textures {
        memory_tracker.assets.textures += texture.memory_size();
    }

    let meshes = asset_cache.iter_type::<Vec<MeshData>>();
    for mesh_vec in meshes {
        for mesh in mesh_vec.iter() {
            memory_tracker.assets.meshes += mesh.memory_size();
        }
    }
}

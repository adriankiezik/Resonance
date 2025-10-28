use crate::app::{Plugin, Resonance, Stage};
use crate::assets::assets::Assets;
use crate::assets::cache::AssetCache;
use crate::assets::loader::mesh::MeshData;
use crate::assets::loader::texture::TextureData;
use crate::assets::source::AssetSourceConfig;
use crate::core::MemoryTracker;
use bevy_ecs::prelude::*;

pub struct AssetsPluginConfig {
    pub asset_source: AssetSourceConfig,
}

impl Default for AssetsPluginConfig {
    fn default() -> Self {
        Self {
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

        let _source = match self.config.asset_source.clone().resolve() {
            Ok(source) => source,
            Err(e) => {
                log::error!("Failed to initialize asset source: {}", e);
                log::error!("AssetsPlugin initialization failed");
                return;
            }
        };
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

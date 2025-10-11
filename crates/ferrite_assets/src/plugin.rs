//! Assets plugin.

use crate::cache::AssetCache;
use ferrite_app::{Engine, Plugin};

/// Plugin for asset management
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, engine: &mut Engine) {
        engine.world.insert_resource(AssetCache::new());

        log::info!("AssetsPlugin initialized");
    }
}

//! Scene plugin for integrating scene management with the engine.

use crate::{ChunkGrid, PrefabRegistry, SceneManager, StreamingManager};
use ferrite_app::{Engine, Plugin};

/// Plugin that adds scene management functionality to the engine
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, engine: &mut Engine) {
        // Add scene management resources
        engine.world.insert_resource(SceneManager::new());
        engine.world.insert_resource(PrefabRegistry::new());

        // Add chunked world resources
        engine.world.insert_resource(ChunkGrid::new());
        engine.world.insert_resource(StreamingManager::new());

        log::info!("ScenePlugin initialized (with chunk streaming)");
    }

    fn name(&self) -> &str {
        "ScenePlugin"
    }
}

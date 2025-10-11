//! Transform plugin for easy setup.

use crate::systems::{propagate_transforms, sync_simple_transforms};
use ferrite_app::{Engine, Plugin, Stage};

/// Plugin that adds transform system to the engine.
pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, engine: &mut Engine) {
        // Components are automatically registered when first used

        // Add transform propagation systems to PostUpdate stage
        // sync_simple_transforms handles entities without hierarchy (faster)
        // propagate_transforms handles parent-child hierarchies
        *engine = std::mem::take(engine)
            .add_system(Stage::PostUpdate, sync_simple_transforms)
            .add_system(Stage::PostUpdate, propagate_transforms);

        log::info!("TransformPlugin initialized");
    }
}

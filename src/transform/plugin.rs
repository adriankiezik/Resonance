use super::systems::{propagate_transforms, sync_simple_transforms};
use crate::app::{Resonance, Plugin, Stage};

#[derive(Default)]
pub struct TransformPlugin;

impl TransformPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for TransformPlugin {
    fn build(&self, engine: &mut Resonance) {
        *engine = std::mem::take(engine)
            .add_system(Stage::PostUpdate, sync_simple_transforms)
            .add_system(Stage::PostUpdate, propagate_transforms);
    }
}

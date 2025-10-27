use super::systems::{propagate_transforms, sync_simple_transforms};
use crate::app::{Plugin, Resonance, Stage};

#[derive(Default)]
pub struct TransformPlugin;

impl TransformPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for TransformPlugin {
    fn build(&self, engine: &mut Resonance) {
        use bevy_ecs::schedule::IntoScheduleConfigs;

        // IMPORTANT: System ordering for transform sync.
        // propagate_transforms MUST run AFTER sync_simple_transforms to ensure:
        // 1. Simple entities (no parents) have their GlobalTransform updated from Transform
        // 2. Child entities can then use parent's updated GlobalTransform when propagating
        // This prevents stale parent transforms from being used by children.
        *engine = std::mem::take(engine)
            .add_systems(
                Stage::PostUpdate,
                (
                    sync_simple_transforms,
                    propagate_transforms.after(sync_simple_transforms),
                ),
            );
    }
}

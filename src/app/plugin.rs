use super::engine::Resonance;
use std::any::TypeId;

pub trait Plugin: Default + Send + Sync + 'static {
    fn build(&self, engine: &mut Resonance);

    fn new() -> Self {
        Self::default()
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn dependencies(&self) -> Vec<(TypeId, &str)> {
        Vec::new()
    }

    fn is_client_plugin(&self) -> bool {
        true
    }

    fn is_server_plugin(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Ready,
    Building,
    Built,
    Failed,
}

pub struct PluginMetadata {
    pub type_id: TypeId,
    pub name: String,
    pub state: PluginState,
    pub dependencies: Vec<TypeId>,
}

pub struct CorePlugin {}

impl Default for CorePlugin {
    fn default() -> Self {
        Self {}
    }
}

impl CorePlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Plugin for CorePlugin {
    fn build(&self, engine: &mut Resonance) {
        use crate::core::{FixedTime, GameTick, MemoryTracker, Time};

        engine.world.insert_resource(Time::new());
        engine.world.insert_resource(FixedTime::default());
        engine.world.insert_resource(GameTick::new());
        engine.world.insert_resource(MemoryTracker::new());
    }
}

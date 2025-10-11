//! Server plugin for dedicated server functionality.

use crate::connection::ConnectionManager;
use ferrite_app::{Engine, Plugin};

/// Plugin for server functionality
pub struct ServerPlugin {
    pub max_clients: usize,
}

impl ServerPlugin {
    pub fn new(max_clients: usize) -> Self {
        Self { max_clients }
    }
}

impl Default for ServerPlugin {
    fn default() -> Self {
        Self::new(32)
    }
}

impl Plugin for ServerPlugin {
    fn build(&self, engine: &mut Engine) {
        // Only add server features in server mode
        if !engine.is_server() {
            log::warn!("ServerPlugin added to non-server engine, skipping");
            return;
        }

        // Add server resources
        engine.world.insert_resource(ConnectionManager::new());

        // Components are automatically registered when first used

        // TODO: Add server systems
        // - Handle client connections/disconnections
        // - Process client inputs
        // - Validate actions
        // - Broadcast state updates

        log::info!("ServerPlugin initialized (max {} clients)", self.max_clients);
    }
}

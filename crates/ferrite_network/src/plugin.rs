//! Network plugins for client and server.

use crate::replication::NetworkIdMap;
use crate::snapshot::SnapshotHistory;
use ferrite_app::{Engine, Plugin};

/// Plugin for network client functionality
pub struct NetworkClientPlugin {
    pub server_addr: String,
}

impl NetworkClientPlugin {
    pub fn new(server_addr: String) -> Self {
        Self { server_addr }
    }
}

impl Plugin for NetworkClientPlugin {
    fn build(&self, engine: &mut Engine) {
        // Add client resources
        engine.world.insert_resource(NetworkIdMap::new());
        engine
            .world
            .insert_resource(SnapshotHistory::new(60)); // Keep 1 second of snapshots at 60Hz

        // Components are automatically registered when first used

        // TODO: Add client systems
        // - Connect to server
        // - Send input
        // - Receive snapshots
        // - Apply interpolation

        log::info!("NetworkClientPlugin initialized (connecting to {})", self.server_addr);
    }
}

/// Plugin for network server functionality
pub struct NetworkServerPlugin {
    pub bind_addr: String,
    pub max_clients: usize,
}

impl NetworkServerPlugin {
    pub fn new(bind_addr: String, max_clients: usize) -> Self {
        Self {
            bind_addr,
            max_clients,
        }
    }
}

impl Default for NetworkServerPlugin {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:5000".to_string(),
            max_clients: 32,
        }
    }
}

impl Plugin for NetworkServerPlugin {
    fn build(&self, engine: &mut Engine) {
        // Add server resources
        engine.world.insert_resource(NetworkIdMap::new());

        // Components are automatically registered when first used

        // TODO: Add server systems
        // - Accept connections
        // - Receive input from clients
        // - Create and send snapshots
        // - Validate client actions

        log::info!(
            "NetworkServerPlugin initialized (binding to {}, max {} clients)",
            self.bind_addr,
            self.max_clients
        );
    }
}

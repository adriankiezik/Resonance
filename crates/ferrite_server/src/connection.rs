//! Connection management.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Resource managing active client connections
#[derive(Resource, Default)]
pub struct ConnectionManager {
    connections: HashMap<u64, ClientConnection>,
    next_client_id: u64,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            next_client_id: 1,
        }
    }

    /// Add a new client connection
    pub fn add_client(&mut self, player_name: String) -> u64 {
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        self.connections.insert(
            client_id,
            ClientConnection {
                client_id,
                player_name,
                connected_at: std::time::Instant::now(),
            },
        );

        log::info!("Client {} connected", client_id);
        client_id
    }

    /// Remove a client connection
    pub fn remove_client(&mut self, client_id: u64) {
        if self.connections.remove(&client_id).is_some() {
            log::info!("Client {} disconnected", client_id);
        }
    }

    /// Get all connected client IDs
    pub fn get_client_ids(&self) -> Vec<u64> {
        self.connections.keys().copied().collect()
    }

    /// Get connection info for a client
    pub fn get_connection(&self, client_id: u64) -> Option<&ClientConnection> {
        self.connections.get(&client_id)
    }

    /// Get number of connected clients
    pub fn client_count(&self) -> usize {
        self.connections.len()
    }
}

/// Information about a connected client
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub client_id: u64,
    pub player_name: String,
    pub connected_at: std::time::Instant,
}

// TODO: Add connection timeout detection
// TODO: Implement ping/latency tracking
// TODO: Add bandwidth monitoring

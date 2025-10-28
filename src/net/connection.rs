/// Connection management layer
///
/// Manages server and client connections, handling message sending/receiving
/// on top of the transport layer.

use std::time::Instant;
use renet::{RenetServer, RenetClient, ClientId, ConnectionConfig};
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Manages server connections
pub struct ServerConnection {
    server: RenetServer,
    last_update: Instant,
}

impl ServerConnection {
    pub fn new(connection_config: ConnectionConfig) -> Self {
        let server = RenetServer::new(connection_config);
        Self {
            server,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.server.update(delta);
        self.last_update = now;
    }

    /// Send a message to a specific client
    pub fn send_message<T: Serialize>(
        &mut self,
        client_id: ClientId,
        message: &T,
        channel: u8,
    ) -> Result<()> {
        let bytes = bincode::serialize(message)?;
        self.server.send_message(client_id, channel, bytes);
        Ok(())
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast_message<T: Serialize>(
        &mut self,
        message: &T,
        channel: u8,
    ) -> Result<()> {
        let bytes = bincode::serialize(message)?;
        for client_id in self.server.clients_id() {
            self.server.send_message(client_id, channel, bytes.clone());
        }
        Ok(())
    }

    /// Receive all pending messages from clients
    pub fn receive_messages<T: for<'de> Deserialize<'de>>(
        &mut self,
        channel: u8,
    ) -> Vec<(ClientId, T)> {
        let mut messages = Vec::new();

        for client_id in self.server.clients_id() {
            while let Some(bytes) = self.server.receive_message(client_id, channel) {
                if let Ok(message) = bincode::deserialize::<T>(&bytes) {
                    messages.push((client_id, message));
                }
            }
        }

        messages
    }

    pub fn connected_clients(&self) -> Vec<ClientId> {
        self.server.clients_id().into_iter().collect()
    }

    pub fn inner_mut(&mut self) -> &mut RenetServer {
        &mut self.server
    }

    pub fn disconnect_client(&mut self, client_id: ClientId) {
        self.server.disconnect(client_id);
    }
}

/// Manages client connection
pub struct ClientConnection {
    client: RenetClient,
    last_update: Instant,
}

impl ClientConnection {
    pub fn new(connection_config: ConnectionConfig) -> Self {
        let client = RenetClient::new(connection_config);
        Self {
            client,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.client.update(delta);
        self.last_update = now;
    }

    /// Send a message to the server
    pub fn send_message<T: Serialize>(
        &mut self,
        message: &T,
        channel: u8,
    ) -> Result<()> {
        let bytes = bincode::serialize(message)?;
        self.client.send_message(channel, bytes);
        Ok(())
    }

    /// Receive all pending messages from server
    pub fn receive_messages<T: for<'de> Deserialize<'de>>(
        &mut self,
        channel: u8,
    ) -> Vec<T> {
        let mut messages = Vec::new();

        while let Some(bytes) = self.client.receive_message(channel) {
            if let Ok(message) = bincode::deserialize::<T>(&bytes) {
                messages.push(message);
            }
        }

        messages
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    pub fn disconnect(&mut self) {
        self.client.disconnect();
    }

    pub fn inner_mut(&mut self) -> &mut RenetClient {
        &mut self.client
    }
}

//! Network protocol and message definitions.

use ferrite_core::math::*;
use serde::{Deserialize, Serialize};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Client wants to join the game
    Join { player_name: String },
    /// Client input (movement, actions, etc.)
    Input { tick: u64, input: PlayerInput },
    /// Client is disconnecting
    Disconnect,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Server accepts the client's join request
    JoinAccepted { client_id: u64 },
    /// Server rejects the client (e.g., game full)
    JoinRejected { reason: String },
    /// World state snapshot
    Snapshot { tick: u64, entities: Vec<EntitySnapshot> },
    /// Entity spawned
    EntitySpawned { network_id: u64, entity_type: String },
    /// Entity despawned
    EntityDespawned { network_id: u64 },
    /// Server is shutting down
    ServerShutdown,
}

/// Player input data (customize based on your game)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Movement direction
    pub movement: Vec2,
    /// Look direction
    pub look: Vec2,
    /// Jump button
    pub jump: bool,
    /// Action button
    pub action: bool,
}

/// Snapshot of an entity's state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// Network ID (unique across network)
    pub network_id: u64,
    /// Position
    pub position: Vec3,
    /// Rotation (quaternion)
    pub rotation: Quat,
    /// Velocity (optional)
    pub velocity: Option<Vec3>,
    // TODO: Add more fields as needed (health, etc.)
}

// TODO: Implement delta compression for snapshots
// TODO: Add interest management (only send nearby entities)
// TODO: Implement priority system for important updates

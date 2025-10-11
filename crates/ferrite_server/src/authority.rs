//! Server authority system.

use bevy_ecs::prelude::*;

/// Marker component indicating the server has authority over this entity.
///
/// Clients cannot modify these entities directly - they must send inputs
/// to the server which will apply them after validation.
#[derive(Component, Debug, Default)]
pub struct ServerAuthority;

/// Marker component for player-controlled entities.
#[derive(Component, Debug)]
pub struct PlayerControlled {
    pub client_id: u64,
}

impl PlayerControlled {
    pub fn new(client_id: u64) -> Self {
        Self { client_id }
    }
}

// TODO: Implement authority transfer system
// TODO: Add ownership validation
// TODO: Implement server reconciliation for client predictions

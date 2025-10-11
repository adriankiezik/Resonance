//! State snapshot system for networking.

use crate::protocol::EntitySnapshot;
use crate::replication::{NetworkId, Replicate};
use bevy_ecs::prelude::*;
use ferrite_core::GameTick;
use ferrite_transform::Transform;

/// Resource storing recent snapshots for interpolation
#[derive(Resource, Default)]
pub struct SnapshotHistory {
    snapshots: Vec<TickSnapshot>,
    max_snapshots: usize,
}

impl SnapshotHistory {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Vec::with_capacity(max_snapshots),
            max_snapshots,
        }
    }

    /// Add a new snapshot
    pub fn add(&mut self, snapshot: TickSnapshot) {
        self.snapshots.push(snapshot);
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    /// Get snapshots for interpolation
    pub fn get_surrounding(&self, tick: u64) -> Option<(&TickSnapshot, &TickSnapshot)> {
        // Find the two snapshots surrounding the given tick
        for window in self.snapshots.windows(2) {
            if window[0].tick <= tick && window[1].tick >= tick {
                return Some((&window[0], &window[1]));
            }
        }
        None
    }
}

/// Snapshot of world state at a specific tick
#[derive(Debug, Clone)]
pub struct TickSnapshot {
    pub tick: u64,
    pub entities: Vec<EntitySnapshot>,
}

/// System to create snapshots on the server
pub fn create_snapshot_system(
    tick: Res<GameTick>,
    query: Query<(&NetworkId, &Transform), With<Replicate>>,
) -> TickSnapshot {
    let entities: Vec<EntitySnapshot> = query
        .iter()
        .map(|(network_id, transform)| EntitySnapshot {
            network_id: network_id.get(),
            position: transform.position,
            rotation: transform.rotation,
            velocity: None, // TODO: Add velocity if component exists
        })
        .collect();

    TickSnapshot {
        tick: tick.get(),
        entities,
    }
}

// TODO: Implement delta snapshots (only send changed data)
// TODO: Add snapshot compression
// TODO: Implement snapshot acknowledgment for reliable delivery

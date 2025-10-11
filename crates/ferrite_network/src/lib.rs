//! Networking system for multiplayer games.
//!
//! Provides:
//! - Client-server architecture
//! - Entity replication
//! - State snapshots
//! - Network messages and protocols
//!
//! Built on top of renet for reliable UDP networking.

pub mod interpolation;
pub mod plugin;
pub mod protocol;
pub mod replication;
pub mod snapshot;
pub mod transport;

pub use plugin::{NetworkClientPlugin, NetworkServerPlugin};
pub use protocol::{ClientMessage, ServerMessage};
pub use replication::{NetworkId, Replicate};

/// Network channel IDs
pub mod channels {
    pub const RELIABLE: u8 = 0;
    pub const UNRELIABLE: u8 = 1;
}

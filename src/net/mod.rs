/// Networking module for Resonance Engine
///
/// This module provides game-agnostic networking primitives:
/// - Transport layer (UDP with reliability via renet)
/// - Connection management
/// - Protocol definitions
/// - Serialization utilities
/// - Network time synchronization
///
/// This layer knows nothing about game-specific concepts like terrain,
/// players, or entities. Games must implement their own message types
/// and replication logic on top of these primitives.

pub mod protocol;
pub mod serialization;
pub mod connection;
pub mod transport;
pub mod clock;

// Re-exports for convenience
pub use protocol::{NetworkChannel, SystemMessage, MessageEnvelope, MessageStats};
pub use serialization::{serialize, deserialize, serialize_with_length, deserialize_with_length};
pub use connection::{ServerConnection, ClientConnection};
pub use transport::{ServerTransport, ClientTransport, TransportConfig};
pub use clock::NetworkClock;

/// Protocol definitions for the Resonance networking layer
///
/// This module defines core networking protocols that are game-agnostic.
/// Games must define their own message types that implement the GameMessage trait.

use serde::{Serialize, Deserialize};

/// Network channels for different message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetworkChannel {
    /// Unreliable, unordered (for position updates, etc.)
    Unreliable = 0,
    /// Reliable, ordered (for important game events)
    Reliable = 1,
    /// Reliable, ordered, high priority (for connection management)
    System = 2,
}

impl From<NetworkChannel> for u8 {
    fn from(channel: NetworkChannel) -> u8 {
        channel as u8
    }
}

/// Base message envelope that wraps all game messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    /// Timestamp when message was sent
    pub timestamp: f64,
    /// Sequence number for ordering
    pub sequence: u64,
    /// The actual message payload
    pub payload: T,
}

impl<T> MessageEnvelope<T> {
    pub fn new(timestamp: f64, sequence: u64, payload: T) -> Self {
        Self {
            timestamp,
            sequence,
            payload,
        }
    }
}

/// Standard system messages handled by the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemMessage {
    /// Server informs client of their assigned ID
    ConnectionAccepted { client_id: u64 },
    /// Server rejects connection
    ConnectionRejected { reason: String },
    /// Graceful disconnect
    Disconnect { reason: String },
    /// Ping request (server → client)
    Ping { timestamp: f64 },
    /// Pong response (client → server)
    Pong { timestamp: f64 },
    /// Server sends current time for synchronization
    TimeSync { server_time: f64 },
}

/// Trait that game messages must implement
pub trait GameMessage: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {}

/// Message statistics for debugging
#[derive(Debug, Clone, Default)]
pub struct MessageStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_lost: u64,
}

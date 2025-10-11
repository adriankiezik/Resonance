//! Server-only functionality for multiplayer games.
//!
//! Provides:
//! - Server authority and validation
//! - Connection management
//! - Input processing and validation

pub mod authority;
pub mod connection;
pub mod plugin;
pub mod validation;

pub use plugin::ServerPlugin;

//! Scene management and entity prefab system for Ferrite.
//!
//! This crate provides:
//! - Scene serialization and deserialization
//! - Scene loading and unloading
//! - Entity prefab system
//! - Scene transitions

pub mod scene;
pub mod prefab;
pub mod manager;
pub mod plugin;
pub mod converter;
pub mod chunk;
pub mod streaming;

pub use scene::*;
pub use prefab::*;
pub use manager::*;
pub use plugin::*;
pub use converter::*;
pub use chunk::*;
pub use streaming::*;

// Re-export Transform for convenience
pub use ferrite_transform::Transform;

//! Common imports for Ferrite engine users.
//!
//! This module re-exports the most commonly used types and traits,
//! allowing users to get started quickly with a single import:
//!
//! ```rust
//! use ferrite::prelude::*;
//! ```

// Core
pub use ferrite_core::{init_logger, FixedTime, GameTick, Time};

// App
pub use ferrite_app::{Engine, EngineMode, Plugin, Stage};

// Transform
pub use ferrite_transform::{Children, GlobalTransform, Parent, Transform, TransformPlugin};

// Physics
pub use ferrite_physics::{
    Aabb, Acceleration, Collider, ColliderShape, Mass, PhysicsPlugin, RigidBody, Velocity,
};

// Network
pub use ferrite_network::{NetworkId, Replicate};

// Assets
pub use ferrite_assets::{AssetCache, AssetHandle, AssetId, AssetsPlugin};

// Client (optional)
#[cfg(feature = "client")]
pub use ferrite_client::{ClientPlugin, Window};

#[cfg(feature = "client")]
pub use ferrite_client::renderer::camera::{Camera, MainCamera};

#[cfg(feature = "client")]
pub use ferrite_client::renderer::mesh::Mesh;

#[cfg(feature = "client")]
pub use ferrite_client::input::Input;

// Server (optional)
#[cfg(feature = "server")]
pub use ferrite_server::{ServerPlugin};

// ECS re-exports
pub use bevy_ecs::prelude::*;

// Math re-exports
pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

// Built-in plugins
pub use ferrite_app::plugin::CorePlugin;

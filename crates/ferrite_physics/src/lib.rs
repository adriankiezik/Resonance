//! Physics system for the Ferrite engine.
//!
//! Provides MMORPG-focused physics simulation including:
//! - Kinematic character movement
//! - Raycasting for ground detection and line-of-sight
//! - Spatial partitioning for efficient collision queries
//! - Trigger volumes for zone detection
//!
//! **Design Philosophy:**
//! This physics system is designed for MMORPGs, not realistic physics simulation.
//! Characters use kinematic movement (not dynamic physics) for predictable,
//! server-authoritative gameplay. Physics runs on the server; clients predict locally.

pub mod character;
pub mod collision;
pub mod components;
pub mod events;
pub mod integration;
pub mod plugin;
pub mod raycast;
pub mod spatial;
pub mod systems;

pub use character::{
    CharacterController, CharacterMovement, CharacterState, GroundInfo,
};
pub use collision::{Aabb, Collider, ColliderShape, CollisionLayer, CollisionMask};
pub use components::{Acceleration, ApplyGravity, Damping, Mass, RigidBody, Trigger, TriggerZone, Velocity};
pub use events::{CollisionEvent, CollisionInfo, CollisionState, CollisionTracker};
pub use plugin::PhysicsPlugin;
pub use raycast::{raycast_world, Ray, RaycastHit};
pub use spatial::{SpatialHashGrid, SpatialGridStats};
pub use systems::Gravity;

//! Physics components.

use bevy_ecs::prelude::*;
use ferrite_core::math::*;
use serde::{Deserialize, Serialize};

/// Rigid body component - marks an entity as a physics object.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RigidBody {
    /// Affected by forces and collisions
    Dynamic,
    /// Not affected by forces, but can be moved programmatically
    Kinematic,
    /// Never moves, infinite mass
    Static,
}

impl Default for RigidBody {
    fn default() -> Self {
        RigidBody::Dynamic
    }
}

/// Velocity component (linear and angular).
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Velocity {
    /// Linear velocity (units per second)
    pub linear: Vec3,
    /// Angular velocity (radians per second around each axis)
    pub angular: Vec3,
}

impl Velocity {
    /// Create a new velocity
    pub fn new(linear: Vec3, angular: Vec3) -> Self {
        Self { linear, angular }
    }

    /// Create with only linear velocity
    pub fn linear(linear: Vec3) -> Self {
        Self {
            linear,
            angular: Vec3::ZERO,
        }
    }

    /// Create with only angular velocity
    pub fn angular(angular: Vec3) -> Self {
        Self {
            linear: Vec3::ZERO,
            angular,
        }
    }

    /// Zero velocity
    pub fn zero() -> Self {
        Self::default()
    }
}

/// Acceleration component (forces applied per frame).
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Acceleration {
    /// Linear acceleration (units per second squared)
    pub linear: Vec3,
    /// Angular acceleration
    pub angular: Vec3,
}

impl Acceleration {
    /// Create a new acceleration
    pub fn new(linear: Vec3, angular: Vec3) -> Self {
        Self { linear, angular }
    }

    /// Create with only linear acceleration (e.g., gravity)
    pub fn linear(linear: Vec3) -> Self {
        Self {
            linear,
            angular: Vec3::ZERO,
        }
    }

    /// Zero acceleration
    pub fn zero() -> Self {
        Self::default()
    }
}

/// Mass component.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Mass(pub f32);

impl Mass {
    /// Create a new mass
    pub fn new(mass: f32) -> Self {
        Self(mass.max(0.0))
    }

    /// Get the mass value
    pub fn get(&self) -> f32 {
        self.0
    }

    /// Get inverse mass (useful for physics calculations)
    pub fn inverse(&self) -> f32 {
        if self.0 > 0.0 {
            1.0 / self.0
        } else {
            0.0
        }
    }
}

impl Default for Mass {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Gravity component - objects with this will be affected by gravity.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ApplyGravity;

/// Damping component - reduces velocity over time (air resistance).
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Damping {
    /// Linear damping factor (0.0 = no damping, 1.0 = full damping)
    pub linear: f32,
    /// Angular damping factor
    pub angular: f32,
}

impl Damping {
    pub fn new(linear: f32, angular: f32) -> Self {
        Self { linear, angular }
    }
}

impl Default for Damping {
    fn default() -> Self {
        Self {
            linear: 0.01,
            angular: 0.01,
        }
    }
}

/// Trigger component - marks a collider as a trigger volume.
/// Trigger volumes detect overlaps but don't provide collision response.
/// Used for:
/// - Zone transitions
/// - Quest areas
/// - PvP boundaries
/// - Dungeon entrances
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Trigger;

/// Trigger event data - additional information about what triggered
#[derive(Component, Debug, Clone)]
pub struct TriggerZone {
    /// Name or identifier for this trigger zone
    pub name: String,
    /// Optional data payload (zone ID, quest ID, etc.)
    pub data: Option<u32>,
}

impl TriggerZone {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: u32) -> Self {
        self.data = Some(data);
        self
    }
}

//! Collision detection primitives.

use bevy_ecs::prelude::*;
use ferrite_core::math::*;
use serde::{Deserialize, Serialize};

/// Collision layers for filtering collisions.
/// Uses bitflags for efficient layer masking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollisionLayer(pub u32);

impl CollisionLayer {
    /// Layer 0: Default
    pub const DEFAULT: Self = Self(1 << 0);
    /// Layer 1: Player characters
    pub const PLAYER: Self = Self(1 << 1);
    /// Layer 2: NPCs and enemies
    pub const NPC: Self = Self(1 << 2);
    /// Layer 3: Environment (walls, floor, obstacles)
    pub const ENVIRONMENT: Self = Self(1 << 3);
    /// Layer 4: Trigger volumes (zones, quests)
    pub const TRIGGER: Self = Self(1 << 4);
    /// Layer 5: Projectiles
    pub const PROJECTILE: Self = Self(1 << 5);
    /// Layer 6: Items (pickups, drops)
    pub const ITEM: Self = Self(1 << 6);
    /// Layer 7: Terrain
    pub const TERRAIN: Self = Self(1 << 7);

    /// All layers
    pub const ALL: Self = Self(u32::MAX);
    /// No layers
    pub const NONE: Self = Self(0);

    /// Create a custom layer
    pub const fn custom(bit: u32) -> Self {
        Self(1 << bit)
    }

    /// Combine layers
    pub const fn combine(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Check if this layer intersects with a mask
    pub const fn intersects(self, mask: CollisionMask) -> bool {
        (self.0 & mask.0) != 0
    }
}

impl Default for CollisionLayer {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Collision mask - defines which layers this collider can interact with
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollisionMask(pub u32);

impl CollisionMask {
    /// Collide with nothing
    pub const NONE: Self = Self(0);
    /// Collide with everything
    pub const ALL: Self = Self(u32::MAX);

    /// Create a mask from multiple layers
    pub const fn from_layers(layers: &[CollisionLayer]) -> Self {
        let mut mask = 0;
        let mut i = 0;
        while i < layers.len() {
            mask |= layers[i].0;
            i += 1;
        }
        Self(mask)
    }

    /// Create a mask that collides with a single layer
    pub const fn single(layer: CollisionLayer) -> Self {
        Self(layer.0)
    }

    /// Add a layer to the mask
    pub const fn with_layer(self, layer: CollisionLayer) -> Self {
        Self(self.0 | layer.0)
    }

    /// Remove a layer from the mask
    pub const fn without_layer(self, layer: CollisionLayer) -> Self {
        Self(self.0 & !layer.0)
    }

    /// Check if mask includes a layer
    pub const fn includes(self, layer: CollisionLayer) -> bool {
        (self.0 & layer.0) != 0
    }
}

impl Default for CollisionMask {
    fn default() -> Self {
        Self::ALL
    }
}

/// Collider component - defines collision shape and layers.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    /// Which layer this collider belongs to
    pub layer: CollisionLayer,
    /// Which layers this collider can collide with
    pub mask: CollisionMask,
}

impl Collider {
    /// Create a box collider with default layer
    pub fn box_collider(half_extents: Vec3) -> Self {
        Self {
            shape: ColliderShape::Box { half_extents },
            layer: CollisionLayer::default(),
            mask: CollisionMask::default(),
        }
    }

    /// Create a sphere collider with default layer
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere { radius },
            layer: CollisionLayer::default(),
            mask: CollisionMask::default(),
        }
    }

    /// Create a capsule collider with default layer
    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule {
                half_height,
                radius,
            },
            layer: CollisionLayer::default(),
            mask: CollisionMask::default(),
        }
    }

    /// Set the collision layer
    pub fn with_layer(mut self, layer: CollisionLayer) -> Self {
        self.layer = layer;
        self
    }

    /// Set the collision mask
    pub fn with_mask(mut self, mask: CollisionMask) -> Self {
        self.mask = mask;
        self
    }

    /// Set both layer and mask
    pub fn with_collision_filtering(mut self, layer: CollisionLayer, mask: CollisionMask) -> Self {
        self.layer = layer;
        self.mask = mask;
        self
    }

    /// Check if this collider should collide with another
    pub fn should_collide_with(&self, other: &Collider) -> bool {
        // Both must have the other in their mask
        self.mask.includes(other.layer) && other.mask.includes(self.layer)
    }
}

/// Collider shapes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderShape {
    /// Box with half extents (half width, half height, half depth)
    Box { half_extents: Vec3 },
    /// Sphere with radius
    Sphere { radius: f32 },
    /// Capsule with half height and radius
    Capsule { half_height: f32, radius: f32 },
}

/// Axis-Aligned Bounding Box for broad-phase collision detection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Create a new AABB
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from a center and half extents
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Check if this AABB intersects with another
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Check if a point is inside the AABB
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Get the center of the AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the half extents of the AABB
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Expand the AABB to include a point
    pub fn expand_to_include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
}

/// Compute AABB from collider and position
pub fn compute_aabb(collider: &Collider, position: Vec3) -> Aabb {
    match &collider.shape {
        ColliderShape::Box { half_extents } => {
            Aabb::from_center_half_extents(position, *half_extents)
        }
        ColliderShape::Sphere { radius } => {
            let half_extents = Vec3::splat(*radius);
            Aabb::from_center_half_extents(position, half_extents)
        }
        ColliderShape::Capsule {
            half_height,
            radius,
        } => {
            let half_extents = Vec3::new(*radius, half_height + radius, *radius);
            Aabb::from_center_half_extents(position, half_extents)
        }
    }
}

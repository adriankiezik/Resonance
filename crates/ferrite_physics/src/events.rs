//! Collision events system.
//!
//! Tracks collision states and emits events when collisions start or end.
//! Essential for gameplay logic (trigger zones, damage, interactions).

use bevy_ecs::prelude::*;
use ferrite_core::math::*;
use std::collections::{HashMap, HashSet};

/// Collision event - fired when two entities collide
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionEvent {
    /// Collision started this frame
    Started(Entity, Entity),
    /// Collision ended this frame
    Ended(Entity, Entity),
}

impl CollisionEvent {
    /// Get the two entities involved in the collision
    pub fn entities(&self) -> (Entity, Entity) {
        match self {
            CollisionEvent::Started(a, b) => (*a, *b),
            CollisionEvent::Ended(a, b) => (*a, *b),
        }
    }

    /// Check if this event involves a specific entity
    pub fn involves(&self, entity: Entity) -> bool {
        let (a, b) = self.entities();
        a == entity || b == entity
    }
}

/// Collision pair - represents two entities that are colliding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CollisionPair {
    a: Entity,
    b: Entity,
}

impl CollisionPair {
    fn new(a: Entity, b: Entity) -> Self {
        // Always store smaller entity first for consistent hashing
        if a.index() < b.index() {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

/// Resource that tracks active collisions and emits events
#[derive(Resource, Default)]
pub struct CollisionTracker {
    /// Collisions that were active last frame
    previous_frame: HashSet<CollisionPair>,
    /// Collisions that are active this frame
    current_frame: HashSet<CollisionPair>,
    /// Events to be processed this frame
    events: Vec<CollisionEvent>,
}

impl CollisionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a collision between two entities
    pub fn register_collision(&mut self, entity_a: Entity, entity_b: Entity) {
        self.current_frame.insert(CollisionPair::new(entity_a, entity_b));
    }

    /// Process collisions and generate events (call once per frame after collision detection)
    pub fn process_events(&mut self) {
        self.events.clear();

        // Find new collisions (in current but not previous)
        for pair in &self.current_frame {
            if !self.previous_frame.contains(pair) {
                self.events.push(CollisionEvent::Started(pair.a, pair.b));
            }
        }

        // Find ended collisions (in previous but not current)
        for pair in &self.previous_frame {
            if !self.current_frame.contains(pair) {
                self.events.push(CollisionEvent::Ended(pair.a, pair.b));
            }
        }

        // Swap frames
        std::mem::swap(&mut self.previous_frame, &mut self.current_frame);
        self.current_frame.clear();
    }

    /// Get events from this frame
    pub fn events(&self) -> &[CollisionEvent] {
        &self.events
    }

    /// Clear all tracked collisions (useful for cleanup)
    pub fn clear(&mut self) {
        self.previous_frame.clear();
        self.current_frame.clear();
        self.events.clear();
    }
}

/// Detailed collision information
#[derive(Debug, Clone, Copy)]
pub struct CollisionInfo {
    /// The other entity involved in the collision
    pub other_entity: Entity,
    /// Contact point (approximate)
    pub contact_point: Vec3,
    /// Contact normal (from self to other)
    pub contact_normal: Vec3,
    /// Penetration depth
    pub penetration_depth: f32,
}

impl CollisionInfo {
    pub fn new(
        other_entity: Entity,
        contact_point: Vec3,
        contact_normal: Vec3,
        penetration_depth: f32,
    ) -> Self {
        Self {
            other_entity,
            contact_point,
            contact_normal,
            penetration_depth,
        }
    }
}

/// Component that stores collision information for an entity
#[derive(Component, Default)]
pub struct CollisionState {
    /// Entities this entity is currently colliding with
    pub colliding_with: HashSet<Entity>,
    /// Detailed collision info (if available)
    pub collision_details: HashMap<Entity, CollisionInfo>,
}

impl CollisionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if colliding with a specific entity
    pub fn is_colliding_with(&self, entity: Entity) -> bool {
        self.colliding_with.contains(&entity)
    }

    /// Get collision info for a specific entity
    pub fn get_collision_info(&self, entity: Entity) -> Option<&CollisionInfo> {
        self.collision_details.get(&entity)
    }

    /// Clear all collision data
    pub fn clear(&mut self) {
        self.colliding_with.clear();
        self.collision_details.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_pair_ordering() {
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);

        let pair1 = CollisionPair::new(e1, e2);
        let pair2 = CollisionPair::new(e2, e1);

        assert_eq!(pair1, pair2);
    }

    #[test]
    fn test_collision_tracker_new_collision() {
        let mut tracker = CollisionTracker::new();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);

        tracker.register_collision(e1, e2);
        tracker.process_events();

        let events = tracker.events();
        assert_eq!(events.len(), 1);
        // Check both entities are involved, order doesn't matter due to CollisionPair normalization
        assert!(events[0].involves(e1));
        assert!(events[0].involves(e2));
        match events[0] {
            CollisionEvent::Started(_, _) => {},
            _ => panic!("Expected Started event"),
        }
    }

    #[test]
    fn test_collision_tracker_ended_collision() {
        let mut tracker = CollisionTracker::new();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);

        // First frame: collision starts
        tracker.register_collision(e1, e2);
        tracker.process_events();

        // Second frame: collision continues
        tracker.register_collision(e1, e2);
        tracker.process_events();
        assert_eq!(tracker.events().len(), 0); // No new events

        // Third frame: collision ends (no registration)
        tracker.process_events();
        let events = tracker.events();
        assert_eq!(events.len(), 1);
        // Check both entities are involved, order doesn't matter due to CollisionPair normalization
        assert!(events[0].involves(e1));
        assert!(events[0].involves(e2));
        match events[0] {
            CollisionEvent::Ended(_, _) => {},
            _ => panic!("Expected Ended event"),
        }
    }
}

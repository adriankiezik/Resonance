//! Parent-child hierarchy for transforms.

use bevy_ecs::prelude::*;

/// Component marking an entity as a child of another entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

impl Parent {
    /// Create a new parent component
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// Get the parent entity
    pub fn get(&self) -> Entity {
        self.0
    }
}

/// Component storing an entity's children.
#[derive(Component, Debug, Clone, Default)]
pub struct Children(pub Vec<Entity>);

impl Children {
    /// Create a new children component
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create with initial children
    pub fn with_children(children: Vec<Entity>) -> Self {
        Self(children)
    }

    /// Add a child
    pub fn add(&mut self, child: Entity) {
        if !self.0.contains(&child) {
            self.0.push(child);
        }
    }

    /// Remove a child
    pub fn remove(&mut self, child: Entity) {
        self.0.retain(|&e| e != child);
    }

    /// Get all children
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }

    /// Get the number of children
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if there are no children
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// TODO: Implement helper functions for building hierarchies
// pub fn add_child(commands: &mut Commands, parent: Entity, child: Entity) { }
// pub fn remove_child(commands: &mut Commands, parent: Entity, child: Entity) { }

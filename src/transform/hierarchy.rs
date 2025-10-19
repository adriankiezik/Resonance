
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

impl Parent {

    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct Children(pub Vec<Entity>);

impl Children {

    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_children(children: Vec<Entity>) -> Self {
        Self(children)
    }

    pub fn add(&mut self, child: Entity) {
        if !self.0.contains(&child) {
            self.0.push(child);
        }
    }

    pub fn remove(&mut self, child: Entity) {
        self.0.retain(|&e| e != child);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

//! Tests for Parent and Children hierarchy components

use bevy_ecs::prelude::*;
use ferrite_transform::hierarchy::{Children, Parent};

#[test]
fn test_parent_component() {
    let mut world = World::new();
    let parent_entity = world.spawn_empty().id();
    let parent = Parent(parent_entity);

    assert_eq!(parent.0, parent_entity);
}

#[test]
fn test_children_component() {
    let mut world = World::new();
    let child1 = world.spawn_empty().id();
    let child2 = world.spawn_empty().id();
    let child3 = world.spawn_empty().id();

    let children = Children(vec![child1, child2, child3]);

    assert_eq!(children.0.len(), 3);
    assert_eq!(children.0[0], child1);
    assert_eq!(children.0[1], child2);
    assert_eq!(children.0[2], child3);
}

#[test]
fn test_children_iter() {
    let mut world = World::new();
    let child1 = world.spawn_empty().id();
    let child2 = world.spawn_empty().id();

    let children = Children(vec![child1, child2]);

    let mut iter = children.iter();
    assert_eq!(iter.next(), Some(&child1));
    assert_eq!(iter.next(), Some(&child2));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_children_empty() {
    let children = Children(vec![]);

    assert_eq!(children.0.len(), 0);
    assert_eq!(children.iter().count(), 0);
}

#[test]
fn test_parent_clone() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let parent1 = Parent(entity);
    let parent2 = parent1;

    assert_eq!(parent1.0, parent2.0);
}

#[test]
fn test_children_clone() {
    let mut world = World::new();
    let child = world.spawn_empty().id();
    let children1 = Children(vec![child]);
    let children2 = children1.clone();

    assert_eq!(children1.0.len(), children2.0.len());
    assert_eq!(children1.0[0], children2.0[0]);
}

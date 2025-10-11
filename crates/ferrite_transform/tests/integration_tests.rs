//! Integration tests for transform propagation system

use bevy_ecs::prelude::*;
use ferrite_transform::{
    components::{GlobalTransform, Transform},
    hierarchy::{Children, Parent},
    systems::{propagate_transforms, sync_simple_transforms},
};
use glam::{Quat, Vec3};

#[test]
fn test_simple_transform_sync() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(sync_simple_transforms);

    // Create entity with transform but no parent/children
    let entity = world.spawn((
        Transform::from_position(Vec3::new(5.0, 10.0, 15.0)),
        GlobalTransform::default(),
    )).id();

    // Run system
    schedule.run(&mut world);

    // Check that GlobalTransform was updated
    let global = world.get::<GlobalTransform>(entity).unwrap();
    let pos = global.position();

    assert!((pos.x - 5.0).abs() < 0.0001);
    assert!((pos.y - 10.0).abs() < 0.0001);
    assert!((pos.z - 15.0).abs() < 0.0001);
}

#[test]
fn test_parent_child_propagation() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(propagate_transforms);

    // Create parent at (10, 0, 0)
    let parent = world.spawn((
        Transform::from_position(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::default(),
    )).id();

    // Create child at local (5, 0, 0)
    let child = world.spawn((
        Transform::from_position(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    // Add Children component to parent
    world.entity_mut(parent).insert(Children(vec![child]));

    // Run propagation system
    schedule.run(&mut world);

    // Parent's global should be (10, 0, 0)
    let parent_global = world.get::<GlobalTransform>(parent).unwrap();
    let parent_pos = parent_global.position();
    assert!((parent_pos.x - 10.0).abs() < 0.0001);

    // Child's global should be (15, 0, 0) = parent + local
    let child_global = world.get::<GlobalTransform>(child).unwrap();
    let child_pos = child_global.position();
    assert!((child_pos.x - 15.0).abs() < 0.0001);
    assert!((child_pos.y - 0.0).abs() < 0.0001);
    assert!((child_pos.z - 0.0).abs() < 0.0001);
}

#[test]
fn test_three_level_hierarchy() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(propagate_transforms);

    // Grandparent at (100, 0, 0)
    let grandparent = world.spawn((
        Transform::from_position(Vec3::new(100.0, 0.0, 0.0)),
        GlobalTransform::default(),
    )).id();

    // Parent at local (10, 0, 0)
    let parent = world.spawn((
        Transform::from_position(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(grandparent),
    )).id();

    // Child at local (1, 0, 0)
    let child = world.spawn((
        Transform::from_position(Vec3::new(1.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    // Setup hierarchy
    world.entity_mut(grandparent).insert(Children(vec![parent]));
    world.entity_mut(parent).insert(Children(vec![child]));

    // Run propagation
    schedule.run(&mut world);

    // Check global positions
    let grandparent_pos = world.get::<GlobalTransform>(grandparent).unwrap().position();
    assert!((grandparent_pos.x - 100.0).abs() < 0.0001);

    let parent_pos = world.get::<GlobalTransform>(parent).unwrap().position();
    assert!((parent_pos.x - 110.0).abs() < 0.0001); // 100 + 10

    let child_pos = world.get::<GlobalTransform>(child).unwrap().position();
    assert!((child_pos.x - 111.0).abs() < 0.0001); // 100 + 10 + 1
}

#[test]
fn test_hierarchy_with_rotation() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(propagate_transforms);

    // Parent rotated 90 degrees around Y
    let parent = world.spawn((
        Transform::from_prs(
            Vec3::ZERO,
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Vec3::ONE,
        ),
        GlobalTransform::default(),
    )).id();

    // Child at local (5, 0, 0)
    let child = world.spawn((
        Transform::from_position(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    world.entity_mut(parent).insert(Children(vec![child]));

    // Run propagation
    schedule.run(&mut world);

    // Child at (5, 0, 0) rotated 90 degrees around Y should be at (0, 0, -5)
    let child_pos = world.get::<GlobalTransform>(child).unwrap().position();
    assert!((child_pos.x - 0.0).abs() < 0.0001);
    assert!((child_pos.y - 0.0).abs() < 0.0001);
    assert!((child_pos.z - (-5.0)).abs() < 0.0001);
}

#[test]
fn test_hierarchy_with_scale() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(propagate_transforms);

    // Parent with scale 2.0
    let parent = world.spawn((
        Transform::from_scale(Vec3::splat(2.0)),
        GlobalTransform::default(),
    )).id();

    // Child at local (5, 0, 0)
    let child = world.spawn((
        Transform::from_position(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    world.entity_mut(parent).insert(Children(vec![child]));

    // Run propagation
    schedule.run(&mut world);

    // Child at (5, 0, 0) with parent scale 2.0 should be at (10, 0, 0)
    let child_pos = world.get::<GlobalTransform>(child).unwrap().position();
    assert!((child_pos.x - 10.0).abs() < 0.0001);

    // Child should inherit parent's scale
    let child_scale = world.get::<GlobalTransform>(child).unwrap().scale();
    assert!((child_scale.x - 2.0).abs() < 0.0001);
}

#[test]
fn test_multiple_children() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(propagate_transforms);

    // Parent at (10, 0, 0)
    let parent = world.spawn((
        Transform::from_position(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::default(),
    )).id();

    // Three children at different positions
    let child1 = world.spawn((
        Transform::from_position(Vec3::new(1.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    let child2 = world.spawn((
        Transform::from_position(Vec3::new(0.0, 2.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    let child3 = world.spawn((
        Transform::from_position(Vec3::new(0.0, 0.0, 3.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    world.entity_mut(parent).insert(Children(vec![child1, child2, child3]));

    // Run propagation
    schedule.run(&mut world);

    // Check all children have correct global positions
    let pos1 = world.get::<GlobalTransform>(child1).unwrap().position();
    assert!((pos1.x - 11.0).abs() < 0.0001);
    assert!((pos1.y - 0.0).abs() < 0.0001);

    let pos2 = world.get::<GlobalTransform>(child2).unwrap().position();
    assert!((pos2.x - 10.0).abs() < 0.0001);
    assert!((pos2.y - 2.0).abs() < 0.0001);

    let pos3 = world.get::<GlobalTransform>(child3).unwrap().position();
    assert!((pos3.x - 10.0).abs() < 0.0001);
    assert!((pos3.z - 3.0).abs() < 0.0001);
}

#[test]
fn test_transform_update_propagates() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(propagate_transforms);

    let parent = world.spawn((
        Transform::from_position(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::default(),
    )).id();

    let child = world.spawn((
        Transform::from_position(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    world.entity_mut(parent).insert(Children(vec![child]));

    // Initial propagation
    schedule.run(&mut world);

    let initial_child_pos = world.get::<GlobalTransform>(child).unwrap().position();
    assert!((initial_child_pos.x - 15.0).abs() < 0.0001);

    // Update parent position
    world.get_mut::<Transform>(parent).unwrap().position = Vec3::new(20.0, 0.0, 0.0);

    // Propagate again
    schedule.run(&mut world);

    // Child should now be at 25.0
    let updated_child_pos = world.get::<GlobalTransform>(child).unwrap().position();
    assert!((updated_child_pos.x - 25.0).abs() < 0.0001);
}

#[test]
fn test_mixed_entities_with_and_without_hierarchy() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    schedule.add_systems(sync_simple_transforms);
    schedule.add_systems(propagate_transforms);

    // Simple entity (no parent/children)
    let simple = world.spawn((
        Transform::from_position(Vec3::new(1.0, 0.0, 0.0)),
        GlobalTransform::default(),
    )).id();

    // Parent with child
    let parent = world.spawn((
        Transform::from_position(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::default(),
    )).id();

    let child = world.spawn((
        Transform::from_position(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    )).id();

    world.entity_mut(parent).insert(Children(vec![child]));

    // Run both systems
    schedule.run(&mut world);

    // Simple entity should be at (1, 0, 0)
    let simple_pos = world.get::<GlobalTransform>(simple).unwrap().position();
    assert!((simple_pos.x - 1.0).abs() < 0.0001);

    // Parent should be at (10, 0, 0)
    let parent_pos = world.get::<GlobalTransform>(parent).unwrap().position();
    assert!((parent_pos.x - 10.0).abs() < 0.0001);

    // Child should be at (15, 0, 0)
    let child_pos = world.get::<GlobalTransform>(child).unwrap().position();
    assert!((child_pos.x - 15.0).abs() < 0.0001);
}

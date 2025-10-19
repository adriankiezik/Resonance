use crate::core::math::*;
use crate::core::time::Time;
use crate::input::{Input, KeyCode};
use crate::transform::Transform;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct FlyCam {
    pub speed: f32,
    pub sensitivity: f32,
}

impl FlyCam {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self { speed, sensitivity }
    }
}

impl Default for FlyCam {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sensitivity: 0.1,
        }
    }
}

pub fn flycam_movement(
    input: Option<Res<Input>>,
    time: Option<Res<Time>>,
    mut query: Query<(&mut Transform, &FlyCam)>,
) {
    let Some(input) = input else { return };
    let Some(time) = time else { return };

    for (mut transform, flycam) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;

        if input.keyboard.is_pressed(KeyCode::KeyW) {
            velocity += transform.forward();
        }
        if input.keyboard.is_pressed(KeyCode::KeyS) {
            velocity -= transform.forward();
        }
        if input.keyboard.is_pressed(KeyCode::KeyA) {
            velocity -= transform.right();
        }
        if input.keyboard.is_pressed(KeyCode::KeyD) {
            velocity += transform.right();
        }
        if input.keyboard.is_pressed(KeyCode::Space) {
            velocity += Vec3::Y;
        }
        if input.keyboard.is_pressed(KeyCode::ShiftLeft) {
            velocity -= Vec3::Y;
        }

        if velocity != Vec3::ZERO {
            velocity = velocity.normalize();
        }

        transform.translate(velocity * flycam.speed * time.delta_seconds());
    }
}

pub fn flycam_look(input: Option<Res<Input>>, mut query: Query<(&mut Transform, &FlyCam)>) {
    let Some(input) = input else { return };

    let mouse_delta = input.mouse.delta();

    if mouse_delta.x == 0.0 && mouse_delta.y == 0.0 {
        return;
    }

    for (mut transform, flycam) in query.iter_mut() {
        let yaw = -mouse_delta.x * flycam.sensitivity * 0.01;
        let pitch = -mouse_delta.y * flycam.sensitivity * 0.01;

        transform.rotate_y(yaw);

        let right = transform.right();
        transform.rotate(Quat::from_axis_angle(right, pitch));
    }
}

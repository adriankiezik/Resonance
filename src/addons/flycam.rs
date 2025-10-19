use crate::core::math::*;
use crate::core::time::Time;
use crate::input::{Input, KeyCode};
use crate::transform::Transform;
use crate::window::Window;
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
            sensitivity: 0.2,
        }
    }
}

pub fn flycam_system(
    input: Option<Res<Input>>,
    time: Option<Res<Time>>,
    window: Option<Res<Window>>,
    mut active: Local<bool>,
    mut initialized: Local<bool>,
    mut query: Query<(&mut Transform, &FlyCam)>,
) {
    if query.is_empty() {
        return;
    }

    if !*initialized {
        if let Some(window) = window.as_ref() {
            window.set_cursor_visible(false);
            let _ = window.set_cursor_grab(true);
        }
        *active = true;
        *initialized = true;
    }

    let Some(input) = input else { return };

    if input.keyboard.just_pressed(KeyCode::Escape) {
        *active = !*active;

        if let Some(window) = window.as_ref() {
            if *active {
                window.set_cursor_visible(false);
                let _ = window.set_cursor_grab(true);
            } else {
                window.set_cursor_visible(true);
                let _ = window.set_cursor_grab(false);
            }
        }
    }

    let Some(time) = time else { return };

    let mouse_delta = input.mouse.delta();

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

        if *active && (mouse_delta.x != 0.0 || mouse_delta.y != 0.0) {
            let yaw = -mouse_delta.x * flycam.sensitivity * 0.01;
            let pitch = -mouse_delta.y * flycam.sensitivity * 0.01;

            transform.rotate_y(yaw);

            let right = transform.right();
            transform.rotate(Quat::from_axis_angle(right, pitch));
        }
    }
}

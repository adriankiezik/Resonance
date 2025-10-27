use crate::renderer::Camera;
use crate::window::WindowEvent;
use bevy_ecs::prelude::*;

pub fn update_camera_aspect_ratio(
    mut cameras: Query<&mut Camera>,
    If(mut window_events): If<MessageReader<WindowEvent>>,
) {
    for event in window_events.read() {
        if let WindowEvent::Resized { width, height } = event {
            let aspect = *width as f32 / (*height as f32).max(1.0);

            for mut camera in cameras.iter_mut() {
                camera.set_aspect(aspect);
            }

            log::debug!("Updated camera aspect ratio to: {:.3}", aspect);
        }
    }
}

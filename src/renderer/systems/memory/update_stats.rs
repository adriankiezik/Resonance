use crate::renderer::Renderer;
use bevy_ecs::prelude::*;

pub fn update_gpu_memory_stats(
    renderer: Option<Res<Renderer>>,
    mut memory_tracker: Option<ResMut<crate::core::MemoryTracker>>,
) {
    let Some(renderer) = renderer else {
        return;
    };
    let Some(ref mut memory_tracker) = memory_tracker else {
        return;
    };

    let (depth_size, ssao_size, msaa_size) = renderer.calculate_texture_memory();
    let camera_buffer_size = renderer.camera_buffer_size();

    memory_tracker.track_depth_texture(depth_size);
    memory_tracker.track_ssao_textures(ssao_size);
    memory_tracker.track_msaa_textures(msaa_size);
    memory_tracker.track_camera_buffer(camera_buffer_size);
}

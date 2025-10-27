use crate::renderer::{
    Renderer,
    components::LightingData,
    lighting::{AmbientLight, AmbientLightUniform, DirectionalLight, DirectionalLightUniform, LightingUniform},
};
use bevy_ecs::prelude::*;

pub fn update_lighting(
    renderer: Option<Res<Renderer>>,
    lighting_data: Option<Res<LightingData>>,
    ao_mode: Option<Res<crate::renderer::AOMode>>,
    ao_debug: Option<Res<crate::renderer::AODebugMode>>,
    mut profiler: Option<ResMut<crate::core::Profiler>>,
    directional_light_query: Query<&DirectionalLight>,
    ambient_light_query: Query<&AmbientLight>,
) {
    let _start = std::time::Instant::now();
    let Some(renderer) = renderer else {
        return;
    };
    let Some(lighting_data) = lighting_data else {
        return;
    };

    let directional_uniform = directional_light_query
        .iter()
        .next()
        .map(DirectionalLightUniform::from_light)
        .unwrap_or_default();

    let ambient_uniform = ambient_light_query
        .iter()
        .next()
        .map(AmbientLightUniform::from_light)
        .unwrap_or_default();

    let lighting_uniform = LightingUniform {
        directional: directional_uniform,
        ambient: ambient_uniform,
        point_light_count: 0,
        ao_mode: ao_mode.map(|m| *m as u32).unwrap_or(0),
        ao_debug: ao_debug.map(|d| d.enabled as u32).unwrap_or(0),
        _padding1: 0.0,
        _padding2: [0.0; 3],
        _padding3: 0.0,
        _padding4: [0.0; 3],
        _padding5: 0.0,
    };

    renderer.queue().write_buffer(
        &lighting_data.buffer,
        0,
        bytemuck::cast_slice(&[lighting_uniform]),
    );

    if let Some(ref mut profiler) = profiler {
        profiler.record_timing("PostUpdate::update_lighting", _start.elapsed());
    }
}

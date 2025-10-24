use crate::renderer::{MeshPipeline, Renderer, components::LightingData, lighting::LightingUniform};
use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;

pub fn initialize_lighting(
    mut commands: Commands,
    renderer: Option<Res<Renderer>>,
    pipeline: Option<Res<MeshPipeline>>,
    lighting_data: Option<Res<LightingData>>,
) {
    if lighting_data.is_some() {
        return;
    }

    let Some(renderer) = renderer else {
        return;
    };
    let Some(pipeline) = pipeline else {
        return;
    };

    let device = renderer.device();
    let default_lighting = LightingUniform::default();

    let lighting_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Lighting Buffer"),
        contents: bytemuck::cast_slice(&[default_lighting]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let lighting_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Lighting Bind Group"),
        layout: &pipeline.lighting_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: lighting_buffer.as_entire_binding(),
        }],
    });

    commands.insert_resource(LightingData {
        buffer: lighting_buffer,
        bind_group: lighting_bind_group,
    });

    log::debug!("Initialized lighting system with default values");
}

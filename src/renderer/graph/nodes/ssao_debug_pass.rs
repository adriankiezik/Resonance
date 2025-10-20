use crate::renderer::graph::node::{RenderContext, RenderNode};
use crate::renderer::SSAODebugMode;
use anyhow::Result;
use bevy_ecs::prelude::World;
use wgpu::CommandEncoder;

pub struct SSAODebugPassNode;

impl SSAODebugPassNode {
    pub fn new() -> Self {
        Self
    }
}

impl RenderNode for SSAODebugPassNode {
    fn name(&self) -> &str {
        "ssao_debug_pass"
    }

    fn dependencies(&self) -> &[&str] {
        &["ssao_blur_pass"]
    }

    fn execute(
        &mut self,
        world: &mut World,
        context: &RenderContext,
        encoder: &mut CommandEncoder,
    ) -> Result<()> {
        let debug_mode = world.get_resource::<SSAODebugMode>().copied().unwrap_or_default();

        if debug_mode == SSAODebugMode::Off {
            return Ok(());
        }

        let texture_view = match debug_mode {
            SSAODebugMode::RawSSAO => context.ssao_view,
            SSAODebugMode::BlurredSSAO => context.ssao_blurred_view,
            SSAODebugMode::Off => return Ok(()),
        };

        let shader_code = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32(vertex_index & 2u);
    return vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
}

@group(0) @binding(0)
var ssao_tex: texture_2d<f32>;

@group(0) @binding(1)
var ssao_sampler: sampler;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / vec2<f32>(textureDimensions(ssao_tex));
    let ao = textureSample(ssao_tex, ssao_sampler, uv).r;
    return vec4<f32>(ao, ao, ao, 1.0);
}
"#;

        let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SSAO Debug Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO Debug Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SSAO Debug Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSAO Debug Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Debug Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SSAO Debug Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SSAO Debug Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: context.surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

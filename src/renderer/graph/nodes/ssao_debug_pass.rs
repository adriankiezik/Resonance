use crate::renderer::graph::node::{RenderContext, RenderNode};
use crate::renderer::{SSAODebugMode, SSAODebugPipeline};
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

        if world.get_resource::<SSAODebugPipeline>().is_none() {
            log::debug!("SSAODebugPipeline resource not available, skipping SSAO debug pass");
            return Ok(());
        }

        let texture_view = match debug_mode {
            SSAODebugMode::RawSSAO => context.ssao_view,
            SSAODebugMode::BlurredSSAO => context.ssao_blurred_view,
            SSAODebugMode::Off => return Ok(()),
        };

        let debug_pipeline = world.get_resource::<SSAODebugPipeline>().unwrap();
        let bind_group = debug_pipeline.create_bind_group(context.device, texture_view);

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

        render_pass.set_pipeline(&debug_pipeline.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

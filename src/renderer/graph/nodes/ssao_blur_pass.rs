use crate::renderer::graph::node::{RenderContext, RenderNode};
use crate::renderer::SSAOBlurPipeline;
use anyhow::Result;
use bevy_ecs::prelude::World;
use wgpu::CommandEncoder;

pub struct SSAOBlurPassNode;

impl SSAOBlurPassNode {
    pub fn new() -> Self {
        Self
    }
}

impl RenderNode for SSAOBlurPassNode {
    fn name(&self) -> &str {
        "ssao_blur_pass"
    }

    fn dependencies(&self) -> &[&str] {
        &["ssao_pass"]
    }

    fn execute(
        &mut self,
        world: &mut World,
        context: &RenderContext,
        encoder: &mut CommandEncoder,
    ) -> Result<()> {
        if world.get_resource::<SSAOBlurPipeline>().is_none() {
            log::debug!("SSAOBlurPipeline resource not available, skipping SSAO blur pass");
            return Ok(());
        }

        let (width, height) = (context.surface_config.width, context.surface_config.height);

        let blur_pipeline = world.get_resource::<SSAOBlurPipeline>().unwrap();
        blur_pipeline.update_params(context.queue, width, height);

        let bind_group = blur_pipeline.create_bind_group(
            context.device,
            context.ssao_view,
            context.depth_view,
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SSAO Blur Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: context.ssao_blurred_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&blur_pipeline.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

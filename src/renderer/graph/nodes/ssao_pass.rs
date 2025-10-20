use crate::renderer::graph::node::{RenderContext, RenderNode};
use crate::renderer::pipeline::SSAOParams;
use crate::renderer::{AOMode, Camera, SSAOPipeline};
use crate::transform::GlobalTransform;
use anyhow::Result;
use bevy_ecs::prelude::World;
use wgpu::CommandEncoder;

pub struct SSAOPassNode;

impl SSAOPassNode {
    pub fn new() -> Self {
        Self
    }
}

impl RenderNode for SSAOPassNode {
    fn name(&self) -> &str {
        "ssao_pass"
    }

    fn dependencies(&self) -> &[&str] {
        &["depth_prepass"]
    }

    fn execute(
        &mut self,
        world: &mut World,
        context: &RenderContext,
        encoder: &mut CommandEncoder,
    ) -> Result<()> {
        let ao_mode = world.get_resource::<AOMode>().copied().unwrap_or_default();
        if ao_mode == AOMode::VertexOnly {
            return Ok(());
        }

        if world.get_resource::<SSAOPipeline>().is_none() {
            log::debug!("SSAOPipeline resource not available, skipping SSAO pass");
            return Ok(());
        }

        let camera_data = world
            .query::<(&Camera, &GlobalTransform)>()
            .iter(world)
            .next();

        let Some((camera, _transform)) = camera_data else {
            log::debug!("No active camera found, skipping SSAO pass");
            return Ok(());
        };

        let projection = camera.projection_matrix();
        let inv_projection = projection.inverse();

        let params = SSAOParams {
            projection: projection.to_cols_array_2d(),
            inv_projection: inv_projection.to_cols_array_2d(),
            radius: 0.1,
            bias: 0.01,
            sample_count: 16.0,
            intensity: 0.2,
        };

        let ssao_pipeline = world.get_resource::<SSAOPipeline>().unwrap();
        context.queue.write_buffer(
            &ssao_pipeline.params_buffer,
            0,
            bytemuck::cast_slice(&[params]),
        );
        let bind_group = ssao_pipeline.create_bind_group(context.device, context.depth_view);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SSAO Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: context.ssao_view,
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

        render_pass.set_pipeline(&ssao_pipeline.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

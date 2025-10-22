use crate::core::math::Mat4;
use crate::renderer::components::{IndirectDrawData, ModelStorageData};
use crate::renderer::graph::node::{RenderContext, RenderNode};
use crate::renderer::{Camera, CameraUniform, DepthPrepassPipeline, GpuMeshCache};
use crate::transform::GlobalTransform;
use anyhow::Result;
use bevy_ecs::prelude::World;
use wgpu::CommandEncoder;

pub struct DepthPrepassNode;

impl DepthPrepassNode {
    pub fn new() -> Self {
        Self
    }
}

impl RenderNode for DepthPrepassNode {
    fn name(&self) -> &str {
        "depth_prepass"
    }

    fn dependencies(&self) -> &[&str] {
        &[]
    }

    fn execute(
        &mut self,
        world: &mut World,
        context: &RenderContext,
        encoder: &mut CommandEncoder,
    ) -> Result<()> {
        let camera_view_proj: Option<Mat4> = world
            .query::<(&Camera, &GlobalTransform)>()
            .iter(world)
            .next()
            .map(|(camera, transform)| camera.view_projection_matrix(transform));

        let Some(view_proj) = camera_view_proj else {
            log::debug!("No active camera found, skipping depth prepass");
            return Ok(());
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(view_proj);

        context.queue.write_buffer(
            context.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        if world.get_resource::<DepthPrepassPipeline>().is_none() {
            log::debug!("DepthPrepassPipeline resource not available, skipping depth prepass");
            return Ok(());
        }

        if world.get_resource::<GpuMeshCache>().is_none() {
            log::debug!("GpuMeshCache resource not available, skipping depth prepass");
            return Ok(());
        }

        if context.camera_bind_group.is_none() {
            log::debug!("Camera bind group not initialized, skipping depth prepass");
            return Ok(());
        }

        let Some(model_storage_data) = world.get_resource::<ModelStorageData>() else {
            log::debug!("ModelStorageData resource not available, skipping depth prepass");
            return Ok(());
        };

        let Some(indirect_draw_data) = world.get_resource::<IndirectDrawData>() else {
            log::debug!("IndirectDrawData resource not available, skipping depth prepass");
            return Ok(());
        };

        let pipeline = world.get_resource::<DepthPrepassPipeline>().unwrap();
        let gpu_mesh_cache = world.get_resource::<GpuMeshCache>().unwrap();

        let depth_view = context.msaa_depth_view.unwrap_or(context.depth_view);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Depth Prepass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, context.camera_bind_group.unwrap(), &[]);
        render_pass.set_bind_group(1, &model_storage_data.bind_group, &[]);

        for batch in &indirect_draw_data.batches {
            if let Some(gpu_mesh) = gpu_mesh_cache.get(&batch.mesh_id) {
                if gpu_mesh.index_count == 0 {
                    continue;
                }
                render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.multi_draw_indexed_indirect(
                    &batch.indirect_buffer,
                    0,
                    batch.draw_count,
                );
            }
        }

        Ok(())
    }
}

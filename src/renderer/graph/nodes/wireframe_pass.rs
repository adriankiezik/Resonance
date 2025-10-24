use crate::addons::WireframeState;
use crate::core::math::Mat4;
use crate::renderer::components::{IndirectDrawData, ModelStorageData};
use crate::renderer::graph::node::{RenderContext, RenderNode};
use crate::renderer::{Camera, GpuMeshCache, WireframePipeline};
use crate::transform::GlobalTransform;
use anyhow::Result;
use bevy_ecs::prelude::World;
use wgpu::CommandEncoder;

pub struct WireframePassNode;

impl WireframePassNode {
    pub fn new() -> Self {
        Self
    }
}

impl RenderNode for WireframePassNode {
    fn name(&self) -> &str {
        "wireframe_pass"
    }

    fn dependencies(&self) -> &[&str] {
        &["main_pass"]
    }

    fn execute(
        &mut self,
        world: &mut World,
        context: &RenderContext,
        encoder: &mut CommandEncoder,
    ) -> Result<()> {
        let wireframe_state = world
            .get_resource::<WireframeState>()
            .map(|s| s.enabled)
            .unwrap_or(false);

        if !wireframe_state {
            return Ok(());
        }

        let camera_view_proj: Option<Mat4> = world
            .query::<(&Camera, &GlobalTransform)>()
            .iter(world)
            .next()
            .map(|(camera, transform)| camera.view_projection_matrix(transform));

        {
            let (color_view, resolve_target) = if let Some(msaa_view) = context.msaa_color_view {
                (msaa_view, Some(context.surface_view))
            } else {
                (context.surface_view, None)
            };

            let depth_view = context.msaa_depth_view.unwrap_or(context.depth_view);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Wireframe Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if camera_view_proj.is_none() {
                log::debug!("No active camera found, skipping wireframe rendering");
            } else if world.get_resource::<WireframePipeline>().is_none() {
                log::debug!("WireframePipeline resource not available, skipping wireframe rendering");
            } else if world.get_resource::<GpuMeshCache>().is_none() {
                log::debug!("GpuMeshCache resource not available, skipping wireframe rendering");
            } else if context.camera_bind_group.is_none() {
                log::debug!("Camera bind group not initialized, skipping wireframe rendering");
            } else if world.get_resource::<ModelStorageData>().is_none() {
                log::debug!("ModelStorageData resource not available, skipping wireframe rendering");
            } else if world.get_resource::<IndirectDrawData>().is_none() {
                log::debug!("IndirectDrawData resource not available, skipping wireframe rendering");
            } else {
                let pipeline = world.get_resource::<WireframePipeline>().unwrap();
                let gpu_mesh_cache = world.get_resource::<GpuMeshCache>().unwrap();
                let model_storage_data = world.get_resource::<ModelStorageData>().unwrap();
                let indirect_draw_data = world.get_resource::<IndirectDrawData>().unwrap();

                render_pass.set_pipeline(&pipeline.pipeline);
                render_pass.set_bind_group(0, context.camera_bind_group.unwrap(), &[]);
                render_pass.set_bind_group(1, &model_storage_data.bind_group, &[]);

                for batch in &indirect_draw_data.batches {
                    if let Some(gpu_mesh) = gpu_mesh_cache.get(&batch.mesh_id) {
                        if gpu_mesh.index_count == 0 {
                            continue;
                        }
                        render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            gpu_mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.multi_draw_indexed_indirect(
                            &batch.indirect_buffer,
                            0,
                            batch.draw_count,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

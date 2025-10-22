use crate::core::math::Mat4;
use crate::renderer::components::{IndirectDrawData, ModelStorageData};
use crate::renderer::graph::node::{RenderContext, RenderNode};
use crate::renderer::{Camera, GpuMeshCache, LightingData, MeshPipeline, SSAODebugMode};
use crate::transform::GlobalTransform;
use anyhow::Result;
use bevy_ecs::prelude::World;
use wgpu::CommandEncoder;

pub struct MainPassNode;

impl MainPassNode {
    pub fn new() -> Self {
        Self
    }
}

impl RenderNode for MainPassNode {
    fn name(&self) -> &str {
        "main_pass"
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
        let debug_mode = world
            .get_resource::<SSAODebugMode>()
            .copied()
            .unwrap_or_default();
        if debug_mode != SSAODebugMode::Off {
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
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
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
                log::debug!("No active camera found, skipping mesh rendering");
            } else if world.get_resource::<MeshPipeline>().is_none() {
                log::debug!("MeshPipeline resource not available, skipping mesh rendering");
            } else if world.get_resource::<GpuMeshCache>().is_none() {
                log::debug!("GpuMeshCache resource not available, skipping mesh rendering");
            } else if context.camera_bind_group.is_none() {
                log::debug!("Camera bind group not initialized, skipping mesh rendering");
            } else if world.get_resource::<LightingData>().is_none() {
                log::debug!("LightingData resource not available, skipping mesh rendering");
            } else if world.get_resource::<ModelStorageData>().is_none() {
                log::debug!("ModelStorageData resource not available, skipping mesh rendering");
            } else if world.get_resource::<IndirectDrawData>().is_none() {
                log::debug!("IndirectDrawData resource not available, skipping mesh rendering");
            } else {
                use crate::renderer::components::SsaoBindGroupCache;

                if world.get_resource::<SsaoBindGroupCache>().is_none() {
                    let pipeline_temp = world.get_resource::<MeshPipeline>().unwrap();
                    let bind_group = pipeline_temp
                        .create_ssao_bind_group(context.device, context.ssao_blurred_view);
                    world.insert_resource(SsaoBindGroupCache { bind_group });
                }

                let pipeline = world.get_resource::<MeshPipeline>().unwrap();
                let gpu_mesh_cache = world.get_resource::<GpuMeshCache>().unwrap();
                let lighting_data = world.get_resource::<LightingData>().unwrap();
                let model_storage_data = world.get_resource::<ModelStorageData>().unwrap();
                let indirect_draw_data = world.get_resource::<IndirectDrawData>().unwrap();
                let ssao_cache = world.get_resource::<SsaoBindGroupCache>().unwrap();
                let ssao_bind_group = &ssao_cache.bind_group;

                render_pass.set_pipeline(&pipeline.pipeline);
                render_pass.set_bind_group(0, context.camera_bind_group.unwrap(), &[]);
                render_pass.set_bind_group(1, &model_storage_data.bind_group, &[]);
                render_pass.set_bind_group(2, &lighting_data.bind_group, &[]);
                render_pass.set_bind_group(3, ssao_bind_group, &[]);

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

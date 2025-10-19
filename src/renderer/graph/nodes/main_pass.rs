use crate::assets::handle::AssetId;
use crate::core::math::Mat4;
use crate::renderer::graph::node::{RenderContext, RenderNode};
use crate::renderer::{Camera, CameraUniform, GpuMeshCache, MeshPipeline};
use crate::renderer::components::{GpuModelData, Mesh, MeshUploaded};
use crate::transform::GlobalTransform;
use anyhow::Result;
use bevy_ecs::prelude::{With, World};
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

        let mesh_data: Vec<(AssetId, &GpuModelData)> = {
            let mut mesh_query = world.query_filtered::<(&Mesh, &GpuModelData), With<MeshUploaded>>();
            mesh_query
                .iter(world)
                .map(|(mesh, gpu_data)| (mesh.handle.id, gpu_data))
                .collect()
        };

        if let Some(view_proj) = camera_view_proj {
            let mut camera_uniform = CameraUniform::new();
            camera_uniform.update_view_proj(view_proj);

            context.queue.write_buffer(
                context.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniform]),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: context.surface_view,
                    resolve_target: None,
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
                    view: context.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
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
            } else {
                let pipeline = world.get_resource::<MeshPipeline>().unwrap();
                let gpu_mesh_cache = world.get_resource::<GpuMeshCache>().unwrap();

                render_pass.set_pipeline(&pipeline.pipeline);
                render_pass.set_bind_group(0, context.camera_bind_group.unwrap(), &[]);

                for (mesh_id, gpu_model_data) in mesh_data.iter() {
                    if let Some(gpu_mesh) = gpu_mesh_cache.get(mesh_id) {
                        render_pass.set_bind_group(1, &gpu_model_data.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            gpu_mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
                    }
                }
            }
        }

        Ok(())
    }
}

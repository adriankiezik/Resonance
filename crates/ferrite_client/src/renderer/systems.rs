//! Rendering systems.

use super::camera::{CameraBuffer, ModelBuffer};
use super::pipeline::MeshBuffers;
use super::texture::TextureHandle;
use super::Renderer;
use bevy_ecs::prelude::*;

/// Main render system that renders the current frame
///
/// This system acquires the surface ONCE per frame, clears it, renders all meshes,
/// and presents it. This prevents flickering caused by multiple surface acquisitions.
pub fn render_system(
    renderer: Option<Res<Renderer>>,
    camera_buffer: Option<Res<CameraBuffer>>,
    colored_mesh_query: Query<(&MeshBuffers, &ModelBuffer), Without<TextureHandle>>,
    textured_mesh_query: Query<(&MeshBuffers, &ModelBuffer, &TextureHandle)>,
) {
    let Some(renderer) = renderer else {
        return;
    };

    let Some(camera_buffer) = camera_buffer else {
        // No camera, skip rendering
        return;
    };

    // Acquire surface texture once per frame
    let output = match renderer.surface.get_current_texture() {
        Ok(output) => output,
        Err(wgpu::SurfaceError::Lost) => {
            log::warn!("Surface lost, will be recreated");
            return;
        }
        Err(wgpu::SurfaceError::OutOfMemory) => {
            log::error!("Out of memory!");
            return;
        }
        Err(e) => {
            log::warn!("Render error: {:?}", e);
            return;
        }
    };

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    // Single render pass that clears and draws all meshes
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(renderer.clear_color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &renderer.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Render all colored meshes (without texture)
        for (mesh_buffers, model_buffer) in colored_mesh_query.iter() {
            render_pass.set_pipeline(&renderer.basic_pipeline);
            render_pass.set_bind_group(0, &camera_buffer.bind_group, &[]);
            render_pass.set_bind_group(1, &model_buffer.bind_group, &[]);
            render_pass.set_vertex_buffer(0, mesh_buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                mesh_buffers.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..mesh_buffers.num_indices, 0, 0..1);
        }

        // Render all textured meshes
        for (mesh_buffers, model_buffer, texture) in textured_mesh_query.iter() {
            render_pass.set_pipeline(&renderer.textured_pipeline);
            render_pass.set_bind_group(0, &camera_buffer.bind_group, &[]);
            render_pass.set_bind_group(1, &model_buffer.bind_group, &[]);
            render_pass.set_bind_group(2, &texture.bind_group, &[]);
            render_pass.set_vertex_buffer(0, mesh_buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                mesh_buffers.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..mesh_buffers.num_indices, 0, 0..1);
        }
    }

    // Submit commands and present (once per frame)
    renderer.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

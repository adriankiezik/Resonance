//! Rendering system using wgpu.

pub mod backend;
pub mod camera;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod systems;
pub mod texture;

use bevy_ecs::prelude::*;
use wgpu::{BindGroupLayout, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration, Texture, TextureView};

/// Renderer resource holding wgpu state
#[derive(Resource)]
pub struct Renderer {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub clear_color: wgpu::Color,
    pub basic_pipeline: RenderPipeline,
    pub textured_pipeline: RenderPipeline,
    pub camera_bind_group_layout: BindGroupLayout,
    pub model_bind_group_layout: BindGroupLayout,
    pub texture_bind_group_layout: BindGroupLayout,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
}

impl Renderer {
    /// Create a new renderer (surface will be configured later)
    pub fn new(
        surface: Surface<'static>,
        device: Device,
        queue: Queue,
        config: SurfaceConfiguration,
        basic_pipeline: RenderPipeline,
        textured_pipeline: RenderPipeline,
        camera_bind_group_layout: BindGroupLayout,
        model_bind_group_layout: BindGroupLayout,
        texture_bind_group_layout: BindGroupLayout,
        depth_texture: Texture,
        depth_view: TextureView,
    ) -> Self {
        Self {
            surface,
            device,
            queue,
            config,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            basic_pipeline,
            textured_pipeline,
            camera_bind_group_layout,
            model_bind_group_layout,
            texture_bind_group_layout,
            depth_texture,
            depth_view,
        }
    }

    /// Render a frame with just a clear color
    pub fn render_clear(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render a frame with mesh buffers
    pub fn render_mesh(
        &self,
        mesh_buffers: &pipeline::MeshBuffers,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.basic_pipeline);
            render_pass.set_vertex_buffer(0, mesh_buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                mesh_buffers.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..mesh_buffers.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Resize the surface
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);

            // Recreate depth texture with new size
            self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: new_width,
                    height: new_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

            log::debug!("Renderer resized to {}x{}", new_width, new_height);
        }
    }
}

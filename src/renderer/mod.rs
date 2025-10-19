pub mod camera;
pub mod components;
pub mod mesh;
pub mod pipeline;
pub mod plugin;
pub mod systems;

use anyhow::Result;
use bevy_ecs::prelude::{Resource, With, World};
use std::sync::Arc;
use wgpu::{BindGroup, Buffer, Device, Queue, Surface, SurfaceConfiguration, Texture, TextureView};
use winit::window::Window;

pub use camera::{Camera, CameraUniform};
pub use components::{GpuModelData, Mesh, MeshUploaded};
pub use mesh::{GpuMesh, GpuMeshCache, Vertex};
pub use pipeline::MeshPipeline;
pub use plugin::RenderPlugin;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ModelUniform {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 3],
}

#[derive(Resource)]
pub struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: (u32, u32),
    camera_buffer: Buffer,
    camera_bind_group: Option<BindGroup>,
    depth_texture: Texture,
    depth_view: TextureView,
}

impl Renderer {
    fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::empty(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("Resonance Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: wgpu::Trace::Off,
            }))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let depth_texture = Self::create_depth_texture(&device, width, height);
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        log::info!(
            "Renderer initialized: {}x{}, format: {:?}",
            width,
            height,
            surface_format
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size: (width, height),
            camera_buffer,
            camera_bind_group: None,
            depth_texture,
            depth_view,
        })
    }

    fn create_depth_texture(device: &Device, width: u32, height: u32) -> Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);

        if self.size != (width, height) {
            self.size = (width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            self.depth_texture = Self::create_depth_texture(&self.device, width, height);
            self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.camera_bind_group = None;

            log::debug!("Renderer resized to {}x{}", width, height);
        }
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn surface(&self) -> &Surface<'_> {
        &self.surface
    }

    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }

    pub fn camera_buffer(&self) -> &Buffer {
        &self.camera_buffer
    }

    pub fn set_camera_bind_group(&mut self, bind_group: BindGroup) {
        self.camera_bind_group = Some(bind_group);
    }

    pub fn has_camera_bind_group(&self) -> bool {
        self.camera_bind_group.is_some()
    }

    pub fn render(&mut self, world: &mut World) -> Result<()> {
        use crate::assets::handle::AssetId;
        use crate::core::math::Mat4;
        use crate::transform::GlobalTransform;

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

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        if let Some(view_proj) = camera_view_proj {
            let mut camera_uniform = CameraUniform::new();
            camera_uniform.update_view_proj(view_proj);

            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniform]),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                    view: &self.depth_view,
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
            } else if self.camera_bind_group.is_none() {
                log::debug!("Camera bind group not initialized, skipping mesh rendering");
            } else {
                let pipeline = world.get_resource::<MeshPipeline>().unwrap();
                let gpu_mesh_cache = world.get_resource::<GpuMeshCache>().unwrap();

                render_pass.set_pipeline(&pipeline.pipeline);
                render_pass.set_bind_group(0, self.camera_bind_group.as_ref().unwrap(), &[]);

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

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub fn create_renderer_sync(window: Arc<Window>) -> Result<Renderer> {
    Renderer::new(window)
}

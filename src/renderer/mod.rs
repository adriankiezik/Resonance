pub mod camera;
pub mod components;
pub mod graph;
pub mod lighting;
pub mod mesh;
pub mod pipeline;
pub mod plugin;
pub mod systems;

use anyhow::Result;
use bevy_ecs::prelude::Resource;
use std::sync::Arc;
use wgpu::{BindGroup, Buffer, Device, Queue, Surface, SurfaceConfiguration, Texture, TextureView};
use winit::window::Window;

pub use camera::{Camera, CameraUniform};
pub use components::{GpuModelData, LightingData, Mesh, MeshUploaded};
pub use graph::node::{RenderContext, RenderNode};
pub use graph::nodes::{DepthPrepassNode, MainPassNode, SSAOBlurPassNode, SSAODebugPassNode, SSAOPassNode};
pub use graph::RenderGraph;
pub use lighting::{AmbientLight, DirectionalLight, LightingUniform, PointLight};
pub use mesh::{GpuMesh, GpuMeshCache, Vertex};
pub use pipeline::{DepthPrepassPipeline, MeshPipeline, SSAOBlurPipeline, SSAODebugPipeline, SSAOPipeline};
pub use plugin::RenderPlugin;

use bytemuck::{Pod, Zeroable};

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SSAODebugMode {
    Off,
    RawSSAO,
    BlurredSSAO,
}

impl Default for SSAODebugMode {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AOMode {
    VertexOnly,
    SSAOOnly,
    Hybrid,
}

impl Default for AOMode {
    fn default() -> Self {
        Self::VertexOnly
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AODebugMode {
    pub enabled: bool,
}

impl Default for AODebugMode {
    fn default() -> Self {
        Self { enabled: false }
    }
}

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
    ssao_texture: Texture,
    ssao_view: TextureView,
    ssao_blurred_texture: Texture,
    ssao_blurred_view: TextureView,
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

        let ssao_texture = Self::create_ssao_texture(&device, width, height);
        let ssao_view = ssao_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let ssao_blurred_texture = Self::create_ssao_texture(&device, width, height);
        let ssao_blurred_view = ssao_blurred_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
            ssao_texture,
            ssao_view,
            ssao_blurred_texture,
            ssao_blurred_view,
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

    fn create_ssao_texture(device: &Device, width: u32, height: u32) -> Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R16Float,
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
            self.ssao_texture = Self::create_ssao_texture(&self.device, width, height);
            self.ssao_view = self.ssao_texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.ssao_blurred_texture = Self::create_ssao_texture(&self.device, width, height);
            self.ssao_blurred_view = self.ssao_blurred_texture.create_view(&wgpu::TextureViewDescriptor::default());
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

    pub fn camera_bind_group(&self) -> Option<&BindGroup> {
        self.camera_bind_group.as_ref()
    }

    pub fn depth_view(&self) -> &TextureView {
        &self.depth_view
    }

    pub fn ssao_view(&self) -> &TextureView {
        &self.ssao_view
    }

    pub fn ssao_blurred_view(&self) -> &TextureView {
        &self.ssao_blurred_view
    }
}

pub fn create_renderer_sync(window: Arc<Window>) -> Result<Renderer> {
    Renderer::new(window)
}

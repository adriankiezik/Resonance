pub mod camera;
pub mod components;
pub mod graph;
pub mod graphics_settings;
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
pub use components::{Aabb, GpuModelData, LightingData, Mesh, MeshUploaded};
pub use graph::RenderGraph;
pub use graph::node::{RenderContext, RenderNode};
pub use graph::nodes::{
    DepthPrepassNode, MainPassNode, SSAOBlurPassNode, SSAODebugPassNode, SSAOPassNode,
    WireframePassNode,
};
pub use graphics_settings::{GraphicsSettings, MsaaSampleCount};
pub use lighting::{AmbientLight, DirectionalLight, LightingUniform, PointLight};
pub use mesh::{GpuMesh, GpuMeshCache, Vertex};
pub use pipeline::{
    DepthPrepassPipeline, MeshPipeline, SSAOBlurPipeline, SSAODebugPipeline,
    SSAOPipeline, WireframePipeline,
};
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
    msaa_sample_count: u32,
    msaa_color_texture: Option<Texture>,
    msaa_color_view: Option<TextureView>,
    msaa_depth_texture: Option<Texture>,
    msaa_depth_view: Option<TextureView>,
    available_present_modes: Vec<wgpu::PresentMode>,
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

        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Immediate)
        {
            wgpu::PresentMode::Immediate
        } else if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        };

        log::info!(
            "Present mode: {:?} (available: {:?})",
            present_mode,
            surface_caps.present_modes
        );

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 3,
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
        let ssao_blurred_view =
            ssao_blurred_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
            msaa_sample_count: 1,
            msaa_color_texture: None,
            msaa_color_view: None,
            msaa_depth_texture: None,
            msaa_depth_view: None,
            available_present_modes: surface_caps.present_modes,
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
            self.depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.ssao_texture = Self::create_ssao_texture(&self.device, width, height);
            self.ssao_view = self
                .ssao_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.ssao_blurred_texture = Self::create_ssao_texture(&self.device, width, height);
            self.ssao_blurred_view = self
                .ssao_blurred_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.camera_bind_group = None;

            if self.msaa_sample_count > 1 {
                let msaa_color_texture = Self::create_msaa_color_texture(
                    &self.device,
                    width,
                    height,
                    self.config.format,
                    self.msaa_sample_count,
                );
                let msaa_color_view =
                    msaa_color_texture.create_view(&wgpu::TextureViewDescriptor::default());

                let msaa_depth_texture = Self::create_msaa_depth_texture(
                    &self.device,
                    width,
                    height,
                    self.msaa_sample_count,
                );
                let msaa_depth_view =
                    msaa_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

                self.msaa_color_texture = Some(msaa_color_texture);
                self.msaa_color_view = Some(msaa_color_view);
                self.msaa_depth_texture = Some(msaa_depth_texture);
                self.msaa_depth_view = Some(msaa_depth_view);
            }

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

    pub fn set_camera_bind_group_invalid(&mut self) {
        self.camera_bind_group = None;
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

    pub fn msaa_sample_count(&self) -> u32 {
        self.msaa_sample_count
    }

    pub fn msaa_color_view(&self) -> Option<&TextureView> {
        self.msaa_color_view.as_ref()
    }

    pub fn msaa_depth_view(&self) -> Option<&TextureView> {
        self.msaa_depth_view.as_ref()
    }

    fn create_msaa_color_texture(
        device: &Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Color Texture"),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    fn create_msaa_depth_texture(
        device: &Device,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    pub fn update_msaa_settings(&mut self, sample_count: u32) {
        if self.msaa_sample_count == sample_count {
            return;
        }

        self.msaa_sample_count = sample_count;

        let (width, height) = self.size;

        if sample_count > 1 {
            let msaa_color_texture = Self::create_msaa_color_texture(
                &self.device,
                width,
                height,
                self.config.format,
                sample_count,
            );
            let msaa_color_view =
                msaa_color_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let msaa_depth_texture =
                Self::create_msaa_depth_texture(&self.device, width, height, sample_count);
            let msaa_depth_view =
                msaa_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.msaa_color_texture = Some(msaa_color_texture);
            self.msaa_color_view = Some(msaa_color_view);
            self.msaa_depth_texture = Some(msaa_depth_texture);
            self.msaa_depth_view = Some(msaa_depth_view);
        } else {
            self.msaa_color_texture = None;
            self.msaa_color_view = None;
            self.msaa_depth_texture = None;
            self.msaa_depth_view = None;
        }
    }

    pub fn update_vsync(&mut self, enabled: bool) {
        let desired_present_mode = if enabled {
            wgpu::PresentMode::Fifo
        } else {
            if self
                .available_present_modes
                .contains(&wgpu::PresentMode::Immediate)
            {
                wgpu::PresentMode::Immediate
            } else if self
                .available_present_modes
                .contains(&wgpu::PresentMode::Mailbox)
            {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Fifo
            }
        };

        if self.config.present_mode == desired_present_mode {
            return;
        }

        log::info!(
            "Changing present mode: {:?} -> {:?} (VSync: {})",
            self.config.present_mode,
            desired_present_mode,
            enabled
        );

        self.config.present_mode = desired_present_mode;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn calculate_texture_memory(&self) -> (u64, u64, u64) {
        let (width, height) = self.size;

        let depth_size = (width * height * 4) as u64;

        let ssao_size = (width * height * 2 * 2) as u64;

        let msaa_size = if self.msaa_sample_count > 1 {
            let bytes_per_pixel = match self.config.format {
                wgpu::TextureFormat::Bgra8UnormSrgb | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
                _ => 4,
            };
            let color_size = (width * height * bytes_per_pixel * self.msaa_sample_count) as u64;
            let depth_size = (width * height * 4 * self.msaa_sample_count) as u64;
            color_size + depth_size
        } else {
            0
        };

        (depth_size, ssao_size, msaa_size)
    }

    pub fn camera_buffer_size(&self) -> u64 {
        std::mem::size_of::<CameraUniform>() as u64
    }
}

pub fn create_renderer_sync(window: Arc<Window>) -> Result<Renderer> {
    Renderer::new(window)
}

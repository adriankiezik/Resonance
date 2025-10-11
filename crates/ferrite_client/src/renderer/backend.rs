//! Graphics backend setup (wgpu).

use crate::renderer::Renderer;
use std::sync::Arc;
use wgpu::{
    Backends, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, PowerPreference,
    PresentMode, RequestAdapterOptions, SurfaceConfiguration, TextureUsages,
};
use winit::window::Window;

/// Initialize wgpu and create a renderer
pub async fn create_renderer(window: Arc<Window>) -> anyhow::Result<Renderer> {
    let size = window.inner_size();

    // Create wgpu instance
    let instance = Instance::new(&InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    // Create surface
    let surface = instance.create_surface(window)?;

    // Request adapter
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await?;

    log::info!("Using adapter: {:?}", adapter.get_info());

    // Request device and queue
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: Some("Main Device"),
            required_features: Features::empty(),
            required_limits: Limits::default(),
            memory_hints: Default::default(),
            experimental_features: Default::default(),
            trace: Default::default(),
        })
        .await?;

    // Get surface capabilities and configure
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    log::info!("Using surface format: {:?}", surface_format);

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo, // VSync
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    // Create depth texture
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create bind group layouts
    let camera_bind_group_layout =
        crate::renderer::pipeline::create_camera_bind_group_layout(&device);
    let model_bind_group_layout =
        crate::renderer::pipeline::create_model_bind_group_layout(&device);
    let texture_bind_group_layout =
        crate::renderer::pipeline::create_texture_bind_group_layout(&device);

    // Create basic render pipeline
    let basic_pipeline = crate::renderer::pipeline::create_basic_pipeline(
        &device,
        &config,
        &camera_bind_group_layout,
        &model_bind_group_layout,
    );

    // Create textured render pipeline
    let textured_pipeline = crate::renderer::pipeline::create_textured_pipeline(
        &device,
        &config,
        &camera_bind_group_layout,
        &model_bind_group_layout,
        &texture_bind_group_layout,
    );

    log::info!("Renderer initialized ({}x{})", size.width, size.height);

    Ok(Renderer::new(
        surface,
        device,
        queue,
        config,
        basic_pipeline,
        textured_pipeline,
        camera_bind_group_layout,
        model_bind_group_layout,
        texture_bind_group_layout,
        depth_texture,
        depth_view,
    ))
}

/// Synchronous wrapper for create_renderer using pollster
pub fn create_renderer_sync(window: Arc<Window>) -> anyhow::Result<Renderer> {
    pollster::block_on(create_renderer(window))
}

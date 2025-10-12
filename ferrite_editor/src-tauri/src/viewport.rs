//! Viewport renderer for scene editor
//!
//! This module provides a simple scene renderer that visualizes entities
//! without running the full game engine. It directly renders scene data
//! on demand using wgpu.

use glam::{Mat4, Quat, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;


/// Camera for viewport
#[derive(Debug, Clone)]
pub struct ViewportCamera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

impl ViewportCamera {
    pub fn new(aspect: f32) -> Self {
        // Position camera looking at origin from an angle
        let position = Vec3::new(5.0, 5.0, 8.0);
        let target = Vec3::ZERO;

        // Calculate rotation to look at target
        let forward = (target - position).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward).normalize();
        let rotation = Quat::from_mat3(&glam::Mat3::from_cols(right, up, -forward));

        Self {
            position,
            rotation,
            fov: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        let forward = self.rotation * Vec3::NEG_Z;
        let up = self.rotation * Vec3::Y;
        Mat4::look_at_rh(self.position, self.position + forward, up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Orbit camera around a target point
    pub fn orbit(&mut self, target: Vec3, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.005;

        // Calculate current offset from target
        let offset = self.position - target;
        let distance = offset.length();

        // Convert to spherical coordinates
        let mut yaw = offset.z.atan2(offset.x);
        let mut pitch = (offset.y / distance).acos();

        // Apply rotation
        yaw -= delta_x * sensitivity;
        pitch = (pitch - delta_y * sensitivity).clamp(0.1, std::f32::consts::PI - 0.1);

        // Convert back to cartesian
        let x = distance * pitch.sin() * yaw.cos();
        let y = distance * pitch.cos();
        let z = distance * pitch.sin() * yaw.sin();

        self.position = target + Vec3::new(x, y, z);

        // Look at target
        let forward = (target - self.position).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward).normalize();

        self.rotation = Quat::from_mat3(&glam::Mat3::from_cols(right, up, -forward));
    }

    /// Pan camera (move in screen space)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.01;
        let right = self.rotation * Vec3::X;
        let up = self.rotation * Vec3::Y;

        self.position += right * delta_x * sensitivity;
        self.position += up * delta_y * sensitivity;
    }

    /// Zoom camera (move forward/backward)
    pub fn zoom(&mut self, delta: f32) {
        let forward = self.rotation * Vec3::NEG_Z;
        let distance = (self.position - Vec3::ZERO).length();
        let zoom_speed = (distance * 0.1).max(0.1);

        self.position += forward * delta * zoom_speed;
    }
}

/// Vertex data for rendering (matches engine format)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2 + std::mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Uniform data for camera (matches engine format)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    view_projection: [[f32; 4]; 4],
}

/// Uniform data for models
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    model: [[f32; 4]; 4],
}

/// Simple viewport renderer
pub struct ViewportRenderer {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline,
    pub camera: ViewportCamera,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub grid_mesh: Option<GridMesh>,
    pub scene_objects: Vec<SceneObject>,
    // For canvas-based rendering
    pub offscreen_texture: Option<wgpu::Texture>,
    pub offscreen_view: Option<wgpu::TextureView>,
    pub readback_buffer: Option<wgpu::Buffer>,
    // Cached resources to avoid recreation
    pub grid_model_buffer: wgpu::Buffer,
    pub grid_model_bind_group: wgpu::BindGroup,
    pub model_bind_group_layout: wgpu::BindGroupLayout,
}

/// Grid mesh for viewport floor
pub struct GridMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

/// Scene object (mesh + transform)
pub struct SceneObject {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub transform: Mat4,
    pub model_buffer: wgpu::Buffer,
    pub model_bind_group: wgpu::BindGroup,
}

impl ViewportRenderer {
    /// Create viewport renderer from a winit window
    pub fn new_from_winit(
        window: Arc<winit::window::Window>,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        log::info!("Creating viewport renderer from winit window ({}x{})...", width, height);

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        log::info!("wgpu instance created");

        // Create surface from winit window
        log::info!("Creating surface from winit window...");
        let surface = instance.create_surface(window.clone())?;
        log::info!("Surface created successfully");

        Self::init_renderer(surface, instance, width, height)
    }

    /// Create viewport renderer from a Tauri window
    pub fn new(
        window: Arc<tauri::WebviewWindow>,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        log::info!("Creating viewport renderer from Tauri window ({}x{})...", width, height);

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        log::info!("wgpu instance created");

        // Create surface from Tauri window
        log::info!("Creating surface from Tauri window...");
        let surface = unsafe {
            let raw_handle = window.as_ref();
            instance.create_surface_unsafe(
                wgpu::SurfaceTargetUnsafe::from_window(raw_handle)?
            )?
        };
        log::info!("Surface created successfully");

        Self::init_renderer(surface, instance, width, height)
    }

    /// Initialize the renderer with a surface
    fn init_renderer(
        surface: wgpu::Surface<'static>,
        instance: wgpu::Instance,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {

        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))?;

        log::info!("Using adapter: {:?}", adapter.get_info());

        // Request device and queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Viewport Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: Default::default(),
            },
        ))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            // Use Mailbox for lower latency (allows tearing but better FPS)
            // If Mailbox not supported, falls back to Fifo
            present_mode: if surface_caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Fifo
            },
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create camera
        let aspect = width as f32 / height as f32;
        let camera = ViewportCamera::new(aspect);

        // Create camera buffer and bind group
        let camera_uniform = CameraUniform {
            view: camera.view_matrix().to_cols_array_2d(),
            projection: camera.projection_matrix().to_cols_array_2d(),
            view_projection: camera.view_projection_matrix().to_cols_array_2d(),
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Viewport Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/viewport.wgsl").into()),
        });

        // Create pipeline layout
        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Model Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Viewport Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &model_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Viewport Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create grid mesh
        log::info!("Creating grid mesh...");
        let grid_mesh = Some(Self::create_grid_mesh(&device));
        log::info!("Grid mesh created");

        // Create sample scene objects
        log::info!("Creating sample scene objects...");
        let scene_objects = Self::create_scene_objects(&device, &model_bind_group_layout);
        log::info!("Created {} scene objects", scene_objects.len());

        // Create cached grid model resources
        let grid_model_uniform = ModelUniform {
            model: Mat4::IDENTITY.to_cols_array_2d(),
        };
        let grid_model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Model Buffer (Cached)"),
            contents: bytemuck::cast_slice(&[grid_model_uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let grid_model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Model Bind Group (Cached)"),
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: grid_model_buffer.as_entire_binding(),
            }],
        });

        log::info!("Viewport renderer initialized successfully ({}x{})", width, height);

        Ok(Self {
            device,
            queue,
            surface,
            config,
            pipeline,
            camera,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            depth_view,
            grid_mesh,
            scene_objects,
            offscreen_texture: None,
            offscreen_view: None,
            readback_buffer: None,
            grid_model_buffer,
            grid_model_bind_group,
            model_bind_group_layout,
        })
    }

    /// Create a grid mesh for the floor
    fn create_grid_mesh(device: &wgpu::Device) -> GridMesh {
        let size = 20;
        let spacing = 1.0;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let grid_color = [0.3, 0.3, 0.3, 1.0];
        let axis_x_color = [1.0, 0.3, 0.3, 1.0];
        let axis_z_color = [0.3, 0.3, 1.0, 1.0];

        for i in 0..=size {
            let offset = (i as f32 - size as f32 / 2.0) * spacing;

            // Choose color based on axis
            let color = if i == size / 2 {
                axis_x_color // Z-axis
            } else {
                grid_color
            };

            // Lines parallel to X-axis
            let idx = vertices.len() as u32;
            vertices.push(Vertex {
                position: [-size as f32 / 2.0 * spacing, 0.0, offset],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [size as f32 / 2.0 * spacing, 0.0, offset],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color,
            });
            indices.extend_from_slice(&[idx, idx + 1]);

            // Lines parallel to Z-axis
            let color = if i == size / 2 {
                axis_z_color // X-axis
            } else {
                grid_color
            };

            let idx = vertices.len() as u32;
            vertices.push(Vertex {
                position: [offset, 0.0, -size as f32 / 2.0 * spacing],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [offset, 0.0, size as f32 / 2.0 * spacing],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color,
            });
            indices.extend_from_slice(&[idx, idx + 1]);
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        GridMesh {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }

    /// Create sample scene objects (cubes at different positions)
    fn create_scene_objects(
        device: &wgpu::Device,
        model_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Vec<SceneObject> {
        let mut objects = Vec::new();

        // Helper to create a cube mesh
        fn create_cube_mesh(color: [f32; 4]) -> (Vec<Vertex>, Vec<u32>) {
            let size = 0.5;
            let vertices = vec![
                // Front face (Z+)
                Vertex { position: [-size, -size, size], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0], color },
                Vertex { position: [size, -size, size], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0], color },
                Vertex { position: [size, size, size], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0], color },
                Vertex { position: [-size, size, size], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0], color },
                // Back face (Z-)
                Vertex { position: [size, -size, -size], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0], color },
                Vertex { position: [-size, -size, -size], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0], color },
                Vertex { position: [-size, size, -size], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0], color },
                Vertex { position: [size, size, -size], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0], color },
                // Top face (Y+)
                Vertex { position: [-size, size, size], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0], color },
                Vertex { position: [size, size, size], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0], color },
                Vertex { position: [size, size, -size], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0], color },
                Vertex { position: [-size, size, -size], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0], color },
                // Bottom face (Y-)
                Vertex { position: [-size, -size, -size], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], color },
                Vertex { position: [size, -size, -size], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0], color },
                Vertex { position: [size, -size, size], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0], color },
                Vertex { position: [-size, -size, size], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0], color },
                // Right face (X+)
                Vertex { position: [size, -size, size], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0], color },
                Vertex { position: [size, -size, -size], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0], color },
                Vertex { position: [size, size, -size], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0], color },
                Vertex { position: [size, size, size], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0], color },
                // Left face (X-)
                Vertex { position: [-size, -size, -size], normal: [-1.0, 0.0, 0.0], uv: [0.0, 0.0], color },
                Vertex { position: [-size, -size, size], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0], color },
                Vertex { position: [-size, size, size], normal: [-1.0, 0.0, 0.0], uv: [1.0, 1.0], color },
                Vertex { position: [-size, size, -size], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0], color },
            ];

            let indices = vec![
                0, 1, 2, 2, 3, 0,       // Front
                4, 5, 6, 6, 7, 4,       // Back
                8, 9, 10, 10, 11, 8,    // Top
                12, 13, 14, 14, 15, 12, // Bottom
                16, 17, 18, 18, 19, 16, // Right
                20, 21, 22, 22, 23, 20, // Left
            ];

            (vertices, indices)
        }

        // Create a few cubes at different positions with different colors
        let cube_positions = vec![
            (Vec3::new(0.0, 0.5, 0.0), [1.0, 0.3, 0.3, 1.0]), // Red cube at origin
            (Vec3::new(2.0, 0.5, 0.0), [0.3, 1.0, 0.3, 1.0]), // Green cube
            (Vec3::new(-2.0, 0.5, 0.0), [0.3, 0.3, 1.0, 1.0]), // Blue cube
            (Vec3::new(0.0, 0.5, 2.0), [1.0, 1.0, 0.3, 1.0]), // Yellow cube
            (Vec3::new(0.0, 0.5, -2.0), [1.0, 0.3, 1.0, 1.0]), // Magenta cube
        ];

        for (position, color) in cube_positions {
            let (vertices, indices) = create_cube_mesh(color);

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let transform = Mat4::from_translation(position);

            let model_uniform = ModelUniform {
                model: transform.to_cols_array_2d(),
            };

            let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Buffer"),
                contents: bytemuck::cast_slice(&[model_uniform]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Model Bind Group"),
                layout: model_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_buffer.as_entire_binding(),
                }],
            });

            objects.push(SceneObject {
                vertex_buffer,
                index_buffer,
                num_indices: indices.len() as u32,
                transform,
                model_buffer,
                model_bind_group,
            });
        }

        objects
    }

    /// Update camera buffer
    pub fn update_camera(&mut self) {
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        let camera_uniform = CameraUniform {
            view: self.camera.view_matrix().to_cols_array_2d(),
            projection: self.camera.projection_matrix().to_cols_array_2d(),
            view_projection: self.camera.view_projection_matrix().to_cols_array_2d(),
        };
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    /// Resize viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            // Recreate depth texture
            self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.update_camera();
            log::debug!("Viewport resized to {}x{}", width, height);
        }
    }

    /// Render the viewport
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Viewport Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Render grid with cached bind group (no buffer recreation!)
            if let Some(ref grid) = self.grid_mesh {
                render_pass.set_bind_group(1, &self.grid_model_bind_group, &[]);
                render_pass.set_vertex_buffer(0, grid.vertex_buffer.slice(..));
                render_pass.set_index_buffer(grid.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..grid.num_indices, 0, 0..1);
            }

            // Render scene objects
            for (i, object) in self.scene_objects.iter().enumerate() {
                render_pass.set_bind_group(1, &object.model_bind_group, &[]);
                render_pass.set_vertex_buffer(0, object.vertex_buffer.slice(..));
                render_pass.set_index_buffer(object.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..object.num_indices, 0, 0..1);
                if i == 0 {
                    log::trace!("Rendered scene object {} with {} indices", i, object.num_indices);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Initialize offscreen rendering resources (for canvas-based rendering)
    pub fn init_offscreen(&mut self) {
        let width = self.config.width;
        let height = self.config.height;

        // Create offscreen texture
        let offscreen_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let offscreen_view = offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create readback buffer with proper alignment
        // wgpu requires bytes_per_row to be aligned to COPY_BYTES_PER_ROW_ALIGNMENT (256)
        let unpadded_bytes_per_row = width * 4; // RGBA8
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;
        let buffer_size = (padded_bytes_per_row * height) as wgpu::BufferAddress;

        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.offscreen_texture = Some(offscreen_texture);
        self.offscreen_view = Some(offscreen_view);
        self.readback_buffer = Some(readback_buffer);

        log::info!("Offscreen rendering initialized for {}x{} (padded row: {})", width, height, padded_bytes_per_row);
    }

    /// Render to offscreen texture and return frame data as RGBA bytes
    ///
    /// ⚠️ PERFORMANCE WARNING: This method is SLOW!
    /// - Copies entire frame from GPU to CPU (millions of bytes)
    /// - Blocks waiting for GPU to finish (kills parallelism)
    /// - Transfers data through IPC to JavaScript
    /// This defeats GPU acceleration and limits FPS to ~6-10 FPS.
    ///
    /// For better performance, consider using direct surface rendering instead.
    pub fn get_frame_data(&mut self) -> Result<Vec<u8>, wgpu::SurfaceError> {
        // Initialize offscreen resources if not done
        if self.offscreen_texture.is_none() {
            self.init_offscreen();
        }

        let offscreen_view = self.offscreen_view.as_ref().unwrap();
        let offscreen_texture = self.offscreen_texture.as_ref().unwrap();
        let readback_buffer = self.readback_buffer.as_ref().unwrap();

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Offscreen Encoder"),
        });

        // Render to offscreen texture
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Offscreen Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: offscreen_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Render grid with cached bind group (no buffer recreation!)
            if let Some(ref grid) = self.grid_mesh {
                render_pass.set_bind_group(1, &self.grid_model_bind_group, &[]);
                render_pass.set_vertex_buffer(0, grid.vertex_buffer.slice(..));
                render_pass.set_index_buffer(grid.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..grid.num_indices, 0, 0..1);
            }

            // Render scene objects
            for object in &self.scene_objects {
                render_pass.set_bind_group(1, &object.model_bind_group, &[]);
                render_pass.set_vertex_buffer(0, object.vertex_buffer.slice(..));
                render_pass.set_index_buffer(object.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..object.num_indices, 0, 0..1);
            }
        }

        // Calculate aligned bytes_per_row
        let width = self.config.width;
        let height = self.config.height;
        let unpadded_bytes_per_row = width * 4; // RGBA8
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: offscreen_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data
        let buffer_slice = readback_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        // Poll device to trigger map_async callback
        // Use wgpu_types::PollType since it's not re-exported by wgpu
        self.device.poll(wgpu_types::PollType::wait_indefinitely()).unwrap();
        receiver.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        // Remove padding from each row if necessary
        let result = if padded_bytes_per_row == unpadded_bytes_per_row {
            // No padding, return as-is
            data.to_vec()
        } else {
            // Need to remove padding from each row
            let mut unpadded = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
            for row in 0..height {
                let row_start = (row * padded_bytes_per_row) as usize;
                let row_end = row_start + unpadded_bytes_per_row as usize;
                unpadded.extend_from_slice(&data[row_start..row_end]);
            }
            unpadded
        };

        drop(data);
        readback_buffer.unmap();

        Ok(result)
    }
}

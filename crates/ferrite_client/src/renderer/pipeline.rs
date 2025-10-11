//! Render pipeline and buffer management.

use super::mesh::Vertex;
use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;
use wgpu::{
    Buffer, Device, RenderPipeline, SurfaceConfiguration, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
};

/// GPU buffers for a mesh
#[derive(Component)]
pub struct MeshBuffers {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
}

impl MeshBuffers {
    /// Create GPU buffers from vertex and index data
    pub fn new(device: &Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

/// Create bind group layout for camera uniforms
pub fn create_camera_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
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
    })
}

/// Create bind group layout for model (transform) uniforms
pub fn create_model_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
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
    })
}

/// Create bind group layout for textures
pub fn create_texture_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}

/// Create the basic render pipeline for colored meshes
pub fn create_basic_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    model_bind_group_layout: &wgpu::BindGroupLayout,
) -> RenderPipeline {
    // Load shader
    let shader_source = include_str!("shaders/basic.wgsl");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Basic Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Define vertex buffer layout
    let vertex_buffer_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &[
            // Position
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            },
            // Normal
            VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: VertexFormat::Float32x3,
            },
            // UV
            VertexAttribute {
                offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                shader_location: 2,
                format: VertexFormat::Float32x2,
            },
            // Color
            VertexAttribute {
                offset: (std::mem::size_of::<[f32; 3]>() * 2 + std::mem::size_of::<[f32; 2]>())
                    as wgpu::BufferAddress,
                shader_location: 3,
                format: VertexFormat::Float32x4,
            },
        ],
    };

    // Create pipeline layout with camera and model bind groups
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Basic Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout, model_bind_group_layout],
        push_constant_ranges: &[],
    });

    // Create render pipeline
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Basic Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[vertex_buffer_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
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
    })
}

/// Create the textured render pipeline for textured meshes
pub fn create_textured_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    model_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> RenderPipeline {
    // Load shader
    let shader_source = include_str!("shaders/textured.wgsl");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Textured Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Define vertex buffer layout (same as basic pipeline)
    let vertex_buffer_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &[
            // Position
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            },
            // Normal
            VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: VertexFormat::Float32x3,
            },
            // UV
            VertexAttribute {
                offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                shader_location: 2,
                format: VertexFormat::Float32x2,
            },
            // Color
            VertexAttribute {
                offset: (std::mem::size_of::<[f32; 3]>() * 2 + std::mem::size_of::<[f32; 2]>())
                    as wgpu::BufferAddress,
                shader_location: 3,
                format: VertexFormat::Float32x4,
            },
        ],
    };

    // Create pipeline layout with camera, model, and texture bind groups
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Textured Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout, model_bind_group_layout, texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    // Create render pipeline
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Textured Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[vertex_buffer_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
    })
}

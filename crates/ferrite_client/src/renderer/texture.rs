//! Texture management and loading.

use bevy_ecs::prelude::*;
use wgpu::{Device, Queue, Texture, TextureView, Sampler, BindGroup, Extent3d};

/// Texture resource that can be attached to entities
#[derive(Component)]
pub struct TextureHandle {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub bind_group: BindGroup,
}

impl TextureHandle {
    /// Create a texture from RGBA8 data with custom filtering
    pub fn from_rgba8_with_filter(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        data: &[u8],
        label: Option<&str>,
        filter_mode: wgpu::FilterMode,
    ) -> Self {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    /// Create a texture from RGBA8 data (defaults to linear filtering)
    pub fn from_rgba8(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        data: &[u8],
        label: Option<&str>,
    ) -> Self {
        Self::from_rgba8_with_filter(
            device,
            queue,
            bind_group_layout,
            width,
            height,
            data,
            label,
            wgpu::FilterMode::Linear,
        )
    }

    /// Create a test checkerboard texture
    pub fn create_checkerboard(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        size: u32,
    ) -> Self {
        Self::create_checkerboard_with_filter(
            device,
            queue,
            bind_group_layout,
            size,
            wgpu::FilterMode::Nearest,
        )
    }

    /// Create a test checkerboard texture with custom filtering
    pub fn create_checkerboard_with_filter(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        size: u32,
        filter_mode: wgpu::FilterMode,
    ) -> Self {
        let mut data = Vec::with_capacity((size * size * 4) as usize);

        for y in 0..size {
            for x in 0..size {
                let checker = ((x / 32) + (y / 32)) % 2 == 0;
                let color = if checker { 255 } else { 64 };
                data.push(color);
                data.push(color);
                data.push(color);
                data.push(255);
            }
        }

        Self::from_rgba8_with_filter(
            device,
            queue,
            bind_group_layout,
            size,
            size,
            &data,
            Some("Checkerboard Texture"),
            filter_mode,
        )
    }

    /// Create a gradient texture for testing
    pub fn create_gradient(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Self {
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let r = (x as f32 / width as f32 * 255.0) as u8;
                let g = (y as f32 / height as f32 * 255.0) as u8;
                let b = 128;
                data.push(r);
                data.push(g);
                data.push(b);
                data.push(255);
            }
        }

        Self::from_rgba8(
            device,
            queue,
            bind_group_layout,
            width,
            height,
            &data,
            Some("Gradient Texture"),
        )
    }
}

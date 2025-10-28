use anyhow::Result;
use bevy_ecs::prelude::World;
use wgpu::{BindGroup, Buffer, CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub surface_config: &'a SurfaceConfiguration,
    pub surface_view: &'a TextureView,
    pub camera_buffer: &'a Buffer,
    pub camera_bind_group: Option<&'a BindGroup>,
    pub depth_view: &'a TextureView,
    pub msaa_color_view: Option<&'a TextureView>,
    pub msaa_depth_view: Option<&'a TextureView>,
    pub msaa_sample_count: u32,
}

pub trait RenderNode: Send + Sync {
    fn name(&self) -> &str;

    fn dependencies(&self) -> &[&str] {
        &[]
    }

    fn execute(
        &mut self,
        world: &mut World,
        context: &RenderContext,
        encoder: &mut CommandEncoder,
    ) -> Result<()>;
}

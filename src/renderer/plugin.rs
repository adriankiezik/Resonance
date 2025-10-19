use crate::app::{Engine, Plugin, Stage};
use crate::renderer::{GpuMeshCache, MeshPipeline, Renderer};
use crate::window::Window;
use std::any::TypeId;
use std::sync::Arc;

#[derive(Default)]
pub struct RenderPlugin;

impl RenderPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, engine: &mut Engine) {
        if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
            schedule.add_systems((
                initialize_renderer,
                recreate_camera_bind_group,
                crate::renderer::systems::upload_meshes,
            ));
        }

        if let Some(schedule) = engine.schedules.get_mut(Stage::PostUpdate) {
            schedule.add_systems((
                crate::renderer::systems::create_model_buffers,
                crate::renderer::systems::update_model_buffers,
                crate::renderer::systems::cleanup_mesh_components,
                crate::renderer::systems::cleanup_unused_meshes,
            ));
        }

        if let Some(schedule) = engine.schedules.get_mut(Stage::Render) {
            schedule.add_systems(render_system);
        }
    }

    fn dependencies(&self) -> Vec<(TypeId, &str)> {
        vec![(
            TypeId::of::<crate::window::WindowPlugin>(),
            "resonance::window::WindowPlugin",
        )]
    }

    fn is_client_plugin(&self) -> bool {
        true
    }

    fn is_server_plugin(&self) -> bool {
        false
    }
}

fn initialize_renderer(world: &mut bevy_ecs::prelude::World) {
    if world.contains_resource::<Renderer>() {
        return;
    }

    let Some(window) = world.get_resource::<Window>() else {
        return;
    };

    let window_arc = Arc::clone(&window.window);

    match crate::renderer::create_renderer_sync(window_arc) {
        Ok(mut renderer) => {
            let surface_format = renderer.config().format;
            let device = renderer.device();

            let mesh_pipeline = MeshPipeline::new(device, surface_format);
            let gpu_mesh_cache = GpuMeshCache::new();

            let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &mesh_pipeline.camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: renderer.camera_buffer().as_entire_binding(),
                }],
            });

            renderer.set_camera_bind_group(camera_bind_group);

            world.insert_resource(renderer);
            world.insert_resource(mesh_pipeline);
            world.insert_resource(gpu_mesh_cache);

            log::info!("Renderer initialized successfully");
        }
        Err(e) => {
            log::error!("Failed to initialize renderer: {}", e);
        }
    }
}

fn recreate_camera_bind_group(world: &mut bevy_ecs::prelude::World) {
    if world.get_resource::<Renderer>().is_none() || world.get_resource::<MeshPipeline>().is_none() {
        return;
    }

    world.resource_scope(|world, mut renderer: bevy_ecs::prelude::Mut<Renderer>| {
        if renderer.has_camera_bind_group() {
            return;
        }

        let pipeline = world.get_resource::<MeshPipeline>().unwrap();
        let device = renderer.device();

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &pipeline.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: renderer.camera_buffer().as_entire_binding(),
            }],
        });

        renderer.set_camera_bind_group(camera_bind_group);
    });
}

fn render_system(world: &mut bevy_ecs::prelude::World) {
    world.resource_scope(|world, mut renderer: bevy_ecs::prelude::Mut<Renderer>| {
        if let Err(e) = renderer.render(world) {
            log::error!("Failed to render frame: {}", e);
        }
    });
}

use crate::app::{Plugin, Resonance, Stage};
use crate::renderer::{
    GpuMeshCache, GraphicsSettings, MainPassNode, MeshPipeline, RenderGraph, Renderer,
    WireframePassNode, WireframePipeline,
};
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
    fn build(&self, engine: &mut Resonance) {
        if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
            schedule.add_systems((
                initialize_renderer,
                update_graphics_settings,
                recreate_camera_bind_group,
                crate::renderer::systems::initialize_lighting,
                crate::renderer::systems::update_camera_aspect_ratio,
                crate::renderer::systems::upload_meshes,
                crate::renderer::systems::compute_mesh_aabbs,
            ));
        }

        if let Some(schedule) = engine.schedules.get_mut(Stage::PostUpdate) {
            use bevy_ecs::schedule::IntoScheduleConfigs;

            // IMPORTANT: System ordering dependency for frustum culling.
            // prepare_indirect_draw_data must run AFTER propagate_transforms (from TransformPlugin).
            //
            // Why: The camera's Transform is updated by FlyCam in the Update stage. In PostUpdate,
            // we need to sync Transform → GlobalTransform. The camera's GlobalTransform must be
            // fully synchronized before prepare_indirect_draw_data reads it to compute the frustum.
            //
            // Without this ordering: Systems might execute in parallel, causing prepare_indirect
            // to read a stale GlobalTransform and compute frustum from previous frame's camera
            // position/rotation. This causes one-frame lag with flickering chunks at frustum edges.
            //
            // See: https://github.com/bevyengine/bevy/issues/XXXX (system scheduling stability)
            schedule.add_systems((
                crate::renderer::systems::cleanup_mesh_components,
                crate::renderer::systems::cleanup_unused_meshes,
                crate::renderer::systems::update_lighting,
                crate::renderer::systems::prepare_indirect_draw_data
                    .after(crate::transform::systems::propagate_transforms),
                crate::renderer::systems::update_gpu_memory_stats,
                submit_gpu_work,
            ));
        }

        if let Some(schedule) = engine.schedules.get_mut(Stage::Render) {
            schedule.add_systems(render_system);
        }
    }

    fn dependencies(&self) -> Vec<(TypeId, &str)> {
        vec![
            (
                TypeId::of::<crate::window::WindowPlugin>(),
                "resonance::window::WindowPlugin",
            ),
            (
                TypeId::of::<crate::transform::TransformPlugin>(),
                "resonance::transform::TransformPlugin",
            ),
        ]
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
            if !world.contains_resource::<GraphicsSettings>() {
                world.insert_resource(GraphicsSettings::default());
            }

            let graphics_settings = world.get_resource::<GraphicsSettings>().unwrap();
            let sample_count = graphics_settings.msaa_sample_count().as_u32();
            let vsync_enabled = graphics_settings.vsync_enabled();

            renderer.update_vsync(vsync_enabled);
            renderer.update_msaa_settings(sample_count);

            let surface_format = renderer.config().format;
            let device = renderer.device();

            let mesh_pipeline = MeshPipeline::new(device, surface_format, sample_count);
            // SSAO and depth prepass removed for simplicity
            // let depth_prepass_pipeline = DepthPrepassPipeline::new(device, sample_count);
            // let ssao_pipeline = SSAOPipeline::new(device, queue);
            // let ssao_blur_pipeline = SSAOBlurPipeline::new(device, width, height);
            // let ssao_debug_pipeline = SSAODebugPipeline::new(device, surface_format, sample_count);
            let wireframe_pipeline = WireframePipeline::new(device, surface_format, sample_count);
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

            let mut render_graph = RenderGraph::new();
            // Only add MainPassNode - depth prepass and SSAO are disabled/removed for simplicity
            render_graph.add_node(Box::new(MainPassNode::new()));
            render_graph.add_node(Box::new(WireframePassNode::new()));

            world.insert_resource(renderer);
            world.insert_resource(mesh_pipeline);
            // SSAO and depth prepass pipelines removed
            // world.insert_resource(depth_prepass_pipeline);
            // world.insert_resource(ssao_pipeline);
            // world.insert_resource(ssao_blur_pipeline);
            // world.insert_resource(ssao_debug_pipeline);
            world.insert_resource(wireframe_pipeline);
            world.insert_resource(gpu_mesh_cache);
            world.insert_resource(render_graph);


            log::info!("Renderer initialized successfully");
        }
        Err(e) => {
            log::error!("Failed to initialize renderer: {}", e);
        }
    }
}

fn recreate_camera_bind_group(world: &mut bevy_ecs::prelude::World) {
    if world.get_resource::<Renderer>().is_none() || world.get_resource::<MeshPipeline>().is_none()
    {
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

fn update_graphics_settings(world: &mut bevy_ecs::prelude::World) {
    if world.get_resource::<GraphicsSettings>().is_none()
        || world.get_resource::<Renderer>().is_none()
    {
        return;
    }

    let mut graphics_settings = world.get_resource_mut::<GraphicsSettings>().unwrap();
    if !graphics_settings.take_changed() {
        return;
    }

    let sample_count = graphics_settings.msaa_sample_count().as_u32();
    let vsync_enabled = graphics_settings.vsync_enabled();
    drop(graphics_settings);

    world.resource_scope(|world, mut renderer: bevy_ecs::prelude::Mut<Renderer>| {
        renderer.update_vsync(vsync_enabled);
        renderer.update_msaa_settings(sample_count);

        let device = renderer.device();
        let surface_format = renderer.config().format;

        let mesh_pipeline = MeshPipeline::new(device, surface_format, sample_count);
        // SSAO and depth prepass pipelines removed
        // let depth_prepass_pipeline = DepthPrepassPipeline::new(device, sample_count);
        // let ssao_debug_pipeline = SSAODebugPipeline::new(device, surface_format, sample_count);
        let wireframe_pipeline = WireframePipeline::new(device, surface_format, sample_count);

        world.insert_resource(mesh_pipeline);
        // world.insert_resource(depth_prepass_pipeline);
        // world.insert_resource(ssao_debug_pipeline);
        world.insert_resource(wireframe_pipeline);

        // Note: We don't invalidate camera_bind_group since the bind group layout
        // (camera uniform layout) doesn't change with graphics settings changes (MSAA/vsync)
    });
}

fn submit_gpu_work(world: &mut bevy_ecs::prelude::World) {
    if let Some(renderer) = world.get_resource::<Renderer>() {
        // Submit all queued GPU work before Render stage starts
        // This is critical for proper synchronization of buffer writes from prepare_indirect_draw_data
        // which uses queue.write_buffer() and must complete before render uses those buffers
        let device = renderer.device();
        let queue = renderer.queue();

        // Create an empty command encoder and submit it
        // This flushes all pending write_buffer operations to the GPU
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("GPU Work Flush"),
        });
        queue.submit(std::iter::once(encoder.finish()));

        // CRITICAL: In wgpu, queue.submit() queues work asynchronously
        // The empty encoder submit ensures pending write_buffer calls are flushed to GPU
    }
}

fn render_system(world: &mut bevy_ecs::prelude::World) {
    if world.get_resource::<RenderGraph>().is_none() || world.get_resource::<Renderer>().is_none() {
        return;
    }

    world.resource_scope(
        |world, mut render_graph: bevy_ecs::prelude::Mut<RenderGraph>| {
            world.resource_scope(|world, mut renderer: bevy_ecs::prelude::Mut<Renderer>| {
                if let Err(e) = render_graph.execute(world, &mut renderer) {
                    log::error!("Failed to render frame: {}", e);
                }
            });
        },
    );
}

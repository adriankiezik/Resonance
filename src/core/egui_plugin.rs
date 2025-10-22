use crate::app::{Plugin, Resonance, Stage};
use crate::renderer::{RenderContext, RenderGraph, RenderNode, Renderer};
use crate::window::Window;
use bevy_ecs::prelude::Resource;
use std::sync::{Arc, Mutex};

pub struct EguiContext {
    pub context: egui::Context,
    pub state: Mutex<egui_winit::State>,
    pub renderer: egui_wgpu::Renderer,
    pub output: Option<egui::FullOutput>,
}

unsafe impl Send for EguiContext {}
unsafe impl Sync for EguiContext {}

impl Resource for EguiContext {}

impl EguiContext {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        window: Arc<winit::window::Window>,
    ) -> Self {
        let context = egui::Context::default();

        let viewport_id = context.viewport_id();
        let state = egui_winit::State::new(context.clone(), viewport_id, &window, None, None, None);

        let renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );

        Self {
            context,
            state: Mutex::new(state),
            renderer,
            output: None,
        }
    }

    pub fn handle_event(
        &self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.lock().unwrap().on_window_event(window, event)
    }

    pub fn begin_frame(&self, window: &winit::window::Window) -> egui::Context {
        let raw_input = self.state.lock().unwrap().take_egui_input(window);
        self.context.begin_pass(raw_input);
        self.context.clone()
    }

    pub fn end_frame(&mut self) {
        self.output = Some(self.context.end_pass());
    }
}

#[derive(Default)]
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, engine: &mut Resonance) {
        if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
            schedule.add_systems(initialize_egui);
        }

        if let Some(schedule) = engine.schedules.get_mut(Stage::Render) {
            schedule.add_systems(prepare_egui_frame);
        }
    }

    fn dependencies(&self) -> Vec<(std::any::TypeId, &str)> {
        vec![
            (
                std::any::TypeId::of::<crate::window::WindowPlugin>(),
                "resonance::window::WindowPlugin",
            ),
            (
                std::any::TypeId::of::<crate::renderer::RenderPlugin>(),
                "resonance::renderer::RenderPlugin",
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

fn initialize_egui(world: &mut bevy_ecs::prelude::World) {
    if world.contains_resource::<EguiContext>() {
        return;
    }

    let Some(renderer) = world.get_resource::<Renderer>() else {
        log::warn!("egui initialization: waiting for Renderer");
        return;
    };

    let Some(window) = world.get_resource::<Window>() else {
        log::warn!("egui initialization: waiting for Window");
        return;
    };

    let device = renderer.device();
    let surface_format = renderer.config().format;
    let window_arc = Arc::clone(&window.window);

    log::info!("egui: Creating EguiContext...");
    let egui_context = EguiContext::new(device, surface_format, window_arc);

    world.insert_resource(egui_context);

    if let Some(mut render_graph) = world.get_resource_mut::<RenderGraph>() {
        render_graph.add_node(Box::new(EguiRenderNode));
        log::info!("egui: Added EguiRenderNode to render graph");
    } else {
        log::error!("egui: RenderGraph not found!");
    }

    log::info!("egui initialized successfully");
}

fn prepare_egui_frame(world: &mut bevy_ecs::prelude::World) {
    if !world.contains_resource::<EguiContext>() {
        return;
    }

    let window_arc = {
        let Some(window) = world.get_resource::<Window>() else {
            return;
        };
        Arc::clone(&window.window)
    };

    let ctx = {
        let egui_ctx = world.get_resource::<EguiContext>().unwrap();
        egui_ctx.begin_frame(&window_arc)
    };

    render_ui(world, &ctx);

    let mut egui_ctx = world.get_resource_mut::<EguiContext>().unwrap();
    egui_ctx.end_frame();

    log::trace!("egui: Frame prepared");
}

pub fn render_ui(world: &mut bevy_ecs::prelude::World, ctx: &egui::Context) {
    crate::core::performance_ui::render_performance_panel(world, ctx);
    crate::core::profiler_ui::render_profiler_panel(world, ctx);
    crate::core::general_info_ui::render_general_info_panel(world, ctx);

    render_game_ui(world, ctx);
}

fn render_game_ui(world: &mut bevy_ecs::prelude::World, ctx: &egui::Context) {
    if let Some(render_fn) = world.get_non_send_resource::<EditorUiRenderFn>() {
        (render_fn.0)(world, ctx);
    }
}

pub struct EditorUiRenderFn(pub fn(&mut bevy_ecs::prelude::World, &egui::Context));

pub struct EguiRenderNode;

impl RenderNode for EguiRenderNode {
    fn name(&self) -> &str {
        "egui"
    }

    fn dependencies(&self) -> &[&str] {
        &["main_pass"]
    }

    fn execute(
        &mut self,
        world: &mut bevy_ecs::prelude::World,
        context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
    ) -> anyhow::Result<()> {
        log::trace!("egui: Executing render node");

        let output = {
            let mut egui_ctx = world
                .get_resource_mut::<EguiContext>()
                .ok_or_else(|| anyhow::anyhow!("EguiContext not found"))?;
            egui_ctx
                .output
                .take()
                .ok_or_else(|| anyhow::anyhow!("No egui output"))?
        };

        let pixels_per_point = {
            let window = world
                .get_resource::<Window>()
                .ok_or_else(|| anyhow::anyhow!("Window not found"))?;
            window.window.scale_factor() as f32
        };

        {
            let mut egui_ctx = world
                .get_resource_mut::<EguiContext>()
                .ok_or_else(|| anyhow::anyhow!("EguiContext not found"))?;

            for (id, image_delta) in &output.textures_delta.set {
                egui_ctx
                    .renderer
                    .update_texture(context.device, context.queue, *id, image_delta);
            }

            for id in &output.textures_delta.free {
                egui_ctx.renderer.free_texture(id);
            }
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [context.surface_config.width, context.surface_config.height],
            pixels_per_point,
        };

        let clipped_primitives = {
            let egui_ctx = world
                .get_resource::<EguiContext>()
                .ok_or_else(|| anyhow::anyhow!("EguiContext not found"))?;
            egui_ctx.context.tessellate(output.shapes, pixels_per_point)
        };

        {
            let mut egui_ctx = world
                .get_resource_mut::<EguiContext>()
                .ok_or_else(|| anyhow::anyhow!("EguiContext not found"))?;
            egui_ctx.renderer.update_buffers(
                context.device,
                context.queue,
                encoder,
                &clipped_primitives,
                &screen_descriptor,
            );
        }

        {
            let mut egui_ctx = world
                .get_resource_mut::<EguiContext>()
                .ok_or_else(|| anyhow::anyhow!("EguiContext not found"))?;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: context.surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            unsafe {
                let render_pass_static: &mut wgpu::RenderPass<'static> =
                    std::mem::transmute(&mut render_pass);
                egui_ctx.renderer.render(
                    render_pass_static,
                    &clipped_primitives,
                    &screen_descriptor,
                );
            }
        }

        Ok(())
    }
}

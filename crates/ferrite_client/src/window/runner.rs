//! Window runner that integrates the engine with winit's event loop.

use super::{Window, WindowConfig, WindowEvent};
use crate::input::Input;
use ferrite_app::Engine;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, StartCause, WindowEvent as WinitWindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

/// Window application handler for winit
pub struct WindowApp {
    engine: Option<Engine>,
    window_config: WindowConfig,
    last_update: Option<Instant>,
    target_frame_time: Duration,
    should_update: bool,
}

impl WindowApp {
    pub fn new(engine: Engine, window_config: WindowConfig) -> Self {
        Self {
            engine: Some(engine),
            window_config,
            last_update: None,
            target_frame_time: Duration::from_micros(1_000_000 / 60), // 60 FPS
            should_update: false,
        }
    }

    fn update_engine(&mut self) {
        let now = Instant::now();

        // Check if enough time has passed for next frame
        if let Some(last) = self.last_update {
            let elapsed = now.duration_since(last);
            if elapsed < self.target_frame_time {
                return;
            }
        }

        // Update the engine
        if let Some(ref mut engine) = self.engine {
            if engine.is_running() {
                engine.update();

                // Update input state AFTER systems have read it (clear just_pressed/just_released/delta)
                if let Some(mut input) = engine.world.get_resource_mut::<Input>() {
                    input.update();
                }
            }
        }

        self.last_update = Some(now);
    }
}

impl ApplicationHandler for WindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window if we don't have one yet
        if let Some(ref mut engine) = self.engine {
            if !engine.world.contains_resource::<Window>() {
                // Create window
                let window = match Window::new(event_loop, &self.window_config) {
                    Ok(w) => w,
                    Err(e) => {
                        log::error!("Failed to create window: {}", e);
                        event_loop.exit();
                        return;
                    }
                };

                log::info!("Window created and added to engine");

                // Initialize renderer with wgpu
                let renderer = match crate::renderer::backend::create_renderer_sync(Arc::clone(&window.window)) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("Failed to create renderer: {}", e);
                        event_loop.exit();
                        return;
                    }
                };

                log::info!("Renderer created and added to engine");

                // Add resources to engine
                engine.world.insert_resource(window);
                engine.world.insert_resource(renderer);

                // Run startup after window and renderer are created
                engine.startup();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WinitWindowEvent,
    ) {
        match event {
            WinitWindowEvent::CloseRequested => {
                log::info!("Window close requested");
                if let Some(ref mut engine) = self.engine {
                    engine.stop();
                }
                event_loop.exit();
            }
            WinitWindowEvent::Resized(size) => {
                log::debug!("Window resized: {}x{}", size.width, size.height);
                // Resize the renderer surface
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut renderer) = engine.world.get_resource_mut::<crate::renderer::Renderer>() {
                        renderer.resize(size.width, size.height);
                    }
                    // Queue window resize event
                    engine.world.write_message(WindowEvent::Resized {
                        width: size.width,
                        height: size.height,
                    });
                }
            }
            WinitWindowEvent::Focused(focused) => {
                log::debug!("Window focus changed: {}", focused);
                if let Some(ref mut engine) = self.engine {
                    engine.world.write_message(WindowEvent::Focused(focused));
                }
            }
            WinitWindowEvent::KeyboardInput { event, .. } => {
                // Handle keyboard input
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut input) = engine.world.get_resource_mut::<Input>() {
                        // Extract KeyCode from PhysicalKey
                        if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                            match event.state {
                                ElementState::Pressed => {
                                    input.keyboard.press(key_code);
                                }
                                ElementState::Released => {
                                    input.keyboard.release(key_code);
                                }
                            }
                        }
                    }
                }
            }
            WinitWindowEvent::CursorMoved { position, .. } => {
                // Handle mouse movement
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut input) = engine.world.get_resource_mut::<Input>() {
                        input.mouse.update_position(position.x as f32, position.y as f32);
                    }
                }
            }
            WinitWindowEvent::MouseInput { state, button, .. } => {
                // Handle mouse button input
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut input) = engine.world.get_resource_mut::<Input>() {
                        match state {
                            ElementState::Pressed => {
                                input.mouse.press_button(button);
                            }
                            ElementState::Released => {
                                input.mouse.release_button(button);
                            }
                        }
                    }
                }
            }
            WinitWindowEvent::MouseWheel { delta, .. } => {
                // Handle mouse wheel
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut input) = engine.world.get_resource_mut::<Input>() {
                        match delta {
                            winit::event::MouseScrollDelta::LineDelta(_, y) => {
                                input.mouse.scroll(y * 10.0);
                            }
                            winit::event::MouseScrollDelta::PixelDelta(pos) => {
                                input.mouse.scroll(pos.y as f32);
                            }
                        }
                    }
                }
            }
            WinitWindowEvent::RedrawRequested => {
                // Request update on next redraw
                self.should_update = true;
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                // Handle raw mouse motion (used when cursor is locked)
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut input) = engine.world.get_resource_mut::<Input>() {
                        input.mouse.add_motion_delta(delta.0 as f32, delta.1 as f32);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Update the engine every frame
        self.update_engine();

        // Request redraw if we have a window
        if let Some(ref engine) = self.engine {
            if let Some(window) = engine.world.get_resource::<Window>() {
                window.window.request_redraw();
            }

            // Check if engine is still running
            if !engine.is_running() {
                event_loop.exit();
            }
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        // Set control flow to poll for continuous updates
        event_loop.set_control_flow(ControlFlow::Poll);
    }
}

/// Run the engine with a window using winit event loop
pub fn run_with_window(engine: Engine, config: WindowConfig) -> anyhow::Result<()> {
    // Create event loop
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    log::info!("Starting window event loop");

    // Create application handler
    let mut app = WindowApp::new(engine, config);

    // Run the event loop (this takes ownership and blocks)
    event_loop.run_app(&mut app)?;

    log::info!("Window event loop exited");
    Ok(())
}

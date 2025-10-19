use crate::app::Resonance;
use crate::input::Input;
use crate::window::{Window, WindowConfig, WindowEvent};

#[cfg(feature = "renderer")]
use crate::renderer::Renderer;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, StartCause, WindowEvent as WinitWindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

pub struct WindowApp {
    engine: Option<Resonance>,
    window_config: WindowConfig,
    last_update: Option<Instant>,
    target_frame_time: Duration,
    should_update: bool,
}

impl WindowApp {
    pub fn new(engine: Resonance, window_config: WindowConfig) -> Self {
        Self {
            engine: Some(engine),
            window_config,
            last_update: None,
            target_frame_time: Duration::from_micros(1_000_000 / 60),
            should_update: false,
        }
    }

    fn update_engine(&mut self) {
        let now = Instant::now();

        if let Some(last) = self.last_update {
            let elapsed = now.duration_since(last);
            if elapsed < self.target_frame_time {
                return;
            }
        }

        if let Some(ref mut engine) = self.engine {
            if engine.is_running() {
                engine.update();

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
        if let Some(ref mut engine) = self.engine {
            if !engine.world.contains_resource::<Window>() {
                let window = match Window::new(event_loop, &self.window_config) {
                    Ok(w) => w,
                    Err(e) => {
                        log::error!("Failed to create window: {}", e);
                        event_loop.exit();
                        return;
                    }
                };

                engine.world.insert_resource(window);

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
                if let Some(ref mut engine) = self.engine {
                    engine.stop();
                }
                event_loop.exit();
            }
            WinitWindowEvent::Resized(size) => {
                log::debug!("Window resized: {}x{}", size.width, size.height);
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut renderer) = engine.world.get_resource_mut::<Renderer>() {
                        renderer.resize(size.width, size.height);
                    }

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
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut input) = engine.world.get_resource_mut::<Input>() {
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
                if let Some(ref mut engine) = self.engine {
                    if let Some(mut input) = engine.world.get_resource_mut::<Input>() {
                        input
                            .mouse
                            .update_position(position.x as f32, position.y as f32);
                    }
                }
            }
            WinitWindowEvent::MouseInput { state, button, .. } => {
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
        self.update_engine();

        if let Some(ref engine) = self.engine {
            if let Some(window) = engine.world.get_resource::<Window>() {
                window.window.request_redraw();
            }

            if !engine.is_running() {
                event_loop.exit();
            }
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        event_loop.set_control_flow(ControlFlow::Poll);
    }
}

pub fn run(engine: Resonance) {
    let config = if let Some(config) = engine.world.get_resource::<WindowConfig>() {
        config.clone()
    } else {
        WindowConfig::default()
    };

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = WindowApp::new(engine, config);

    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
}

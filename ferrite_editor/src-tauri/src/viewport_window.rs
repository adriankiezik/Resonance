//! Separate native window for 3D viewport
//!
//! This module creates a native window using winit that runs the viewport renderer.
//! The window runs in its own thread with its own event loop.

use crate::viewport::ViewportRenderer;
use std::sync::Arc;
use std::thread;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes},
};

/// Spawn a new viewport window
pub fn spawn_viewport_window() -> anyhow::Result<()> {
    log::info!("Spawning viewport window");

    // Spawn in a new thread so it doesn't block
    thread::spawn(|| {
        if let Err(e) = run_viewport_window() {
            log::error!("Viewport window error: {}", e);
        }
    });

    Ok(())
}

/// Run the viewport window event loop
fn run_viewport_window() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ViewportApp {
        window: None,
        renderer: None,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}

/// Viewport window application
struct ViewportApp {
    window: Option<Arc<Window>>,
    renderer: Option<ViewportRenderer>,
}

impl ApplicationHandler for ViewportApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("Creating viewport window");

            // Create the window
            let window_attrs = WindowAttributes::default()
                .with_title("Ferrite 3D Viewport")
                .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));

            match event_loop.create_window(window_attrs) {
                Ok(window) => {
                    let window = Arc::new(window);
                    let size = window.inner_size();

                    log::info!("Window created, initializing renderer");

                    // Create renderer
                    match ViewportRenderer::new_from_winit(Arc::clone(&window), size.width, size.height) {
                        Ok(renderer) => {
                            log::info!("Renderer initialized");
                            self.renderer = Some(renderer);
                            self.window = Some(window);
                        }
                        Err(e) => {
                            log::error!("Failed to create renderer: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create window: {}", e);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Viewport window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    log::debug!("Viewport resized to {}x{}", new_size.width, new_size.height);
                    renderer.resize(new_size.width, new_size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.update_camera();
                    if let Err(e) = renderer.render() {
                        log::error!("Render error: {:?}", e);
                    }
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw on every event loop iteration
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

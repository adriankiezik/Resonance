//! Window management using winit.

pub mod runner;
pub mod systems;

pub use runner::run_with_window;

use bevy_ecs::prelude::*;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window as WinitWindow, WindowAttributes, Fullscreen, CursorGrabMode},
};

/// Window display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    /// Regular windowed mode with decorations
    Windowed,
    /// Exclusive fullscreen mode (changes display mode)
    Fullscreen,
    /// Borderless fullscreen window (no mode change)
    BorderlessFullscreen,
}

/// Window resource holding the actual winit window
#[derive(Resource, Clone)]
pub struct Window {
    pub window: Arc<WinitWindow>,
}

impl Window {
    /// Create a new window from an event loop and config
    pub fn new(event_loop: &ActiveEventLoop, config: &WindowConfig) -> anyhow::Result<Self> {
        let mut attributes = WindowAttributes::default()
            .with_title(config.title.clone())
            .with_inner_size(PhysicalSize::new(config.width, config.height))
            .with_resizable(config.resizable);

        // Apply window mode
        attributes = match config.mode {
            WindowMode::Windowed => {
                // Already configured as windowed
                attributes
            }
            WindowMode::Fullscreen => {
                // Get the primary monitor and set exclusive fullscreen with current video mode
                if let Some(monitor) = event_loop.primary_monitor() {
                    // Use the current video mode for exclusive fullscreen
                    if let Some(video_mode) = monitor.video_modes().next() {
                        log::info!(
                            "Setting exclusive fullscreen mode: {}x{} @ {}Hz",
                            video_mode.size().width,
                            video_mode.size().height,
                            video_mode.refresh_rate_millihertz() / 1000
                        );
                        attributes.with_fullscreen(Some(Fullscreen::Exclusive(video_mode)))
                    } else {
                        log::warn!("No video modes available, using borderless fullscreen");
                        attributes.with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))))
                    }
                } else {
                    log::warn!("No primary monitor found, falling back to windowed mode");
                    attributes
                }
            }
            WindowMode::BorderlessFullscreen => {
                // Set borderless fullscreen on primary monitor
                if let Some(monitor) = event_loop.primary_monitor() {
                    log::info!("Setting borderless fullscreen mode on primary monitor");
                    attributes.with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))))
                } else {
                    log::warn!("No primary monitor found, falling back to windowed mode");
                    attributes
                }
            }
        };

        let window = Arc::new(event_loop.create_window(attributes)?);
        log::info!(
            "Window created: {}x{} '{}' (mode: {:?})",
            config.width,
            config.height,
            config.title,
            config.mode
        );

        Ok(Self { window })
    }

    /// Get window size
    pub fn size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    /// Check if window should close
    pub fn should_close(&self) -> bool {
        false // Will be managed by event loop
    }

    /// Set the window mode at runtime
    pub fn set_mode(&self, mode: WindowMode) {
        match mode {
            WindowMode::Windowed => {
                log::info!("Switching to windowed mode");
                self.window.set_fullscreen(None);
            }
            WindowMode::Fullscreen => {
                // Get current monitor and set exclusive fullscreen
                if let Some(monitor) = self.window.current_monitor() {
                    if let Some(video_mode) = monitor.video_modes().next() {
                        log::info!(
                            "Switching to exclusive fullscreen: {}x{} @ {}Hz",
                            video_mode.size().width,
                            video_mode.size().height,
                            video_mode.refresh_rate_millihertz() / 1000
                        );
                        self.window.set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
                    } else {
                        log::warn!("No video modes available, using borderless fullscreen");
                        self.window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
                    }
                } else {
                    log::warn!("No monitor detected, cannot switch to fullscreen");
                }
            }
            WindowMode::BorderlessFullscreen => {
                // Get current monitor and set borderless fullscreen
                if let Some(monitor) = self.window.current_monitor() {
                    log::info!("Switching to borderless fullscreen mode");
                    self.window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
                } else {
                    log::warn!("No monitor detected, cannot switch to borderless fullscreen");
                }
            }
        }
    }

    /// Toggle between windowed and borderless fullscreen
    pub fn toggle_fullscreen(&self) {
        if self.window.fullscreen().is_some() {
            self.set_mode(WindowMode::Windowed);
        } else {
            self.set_mode(WindowMode::BorderlessFullscreen);
        }
    }

    /// Get current window mode
    pub fn current_mode(&self) -> WindowMode {
        match self.window.fullscreen() {
            None => WindowMode::Windowed,
            Some(Fullscreen::Exclusive(_)) => WindowMode::Fullscreen,
            Some(Fullscreen::Borderless(_)) => WindowMode::BorderlessFullscreen,
        }
    }

    /// Set cursor visibility
    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    /// Lock/confine cursor to window
    pub fn set_cursor_grab(&self, grab: bool) -> anyhow::Result<()> {
        let mode = if grab {
            // Use Locked mode for FPS-style camera control with infinite rotation
            // Falls back to Confined if Locked is not supported on the platform
            match self.window.set_cursor_grab(CursorGrabMode::Locked) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    log::warn!("CursorGrabMode::Locked not supported, falling back to Confined");
                    CursorGrabMode::Confined
                }
            }
        } else {
            CursorGrabMode::None
        };
        self.window.set_cursor_grab(mode)?;
        Ok(())
    }
}

/// Window configuration
#[derive(Resource, Clone)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub resizable: bool,
    pub vsync: bool,
    pub mode: WindowMode,
}

impl WindowConfig {
    pub fn new(width: u32, height: u32, title: impl Into<String>) -> Self {
        Self {
            width,
            height,
            title: title.into(),
            resizable: true,
            vsync: true,
            mode: WindowMode::Windowed,
        }
    }

    /// Set the window mode
    pub fn with_mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set to fullscreen mode
    pub fn fullscreen(mut self) -> Self {
        self.mode = WindowMode::Fullscreen;
        self
    }

    /// Set to borderless fullscreen mode
    pub fn borderless(mut self) -> Self {
        self.mode = WindowMode::BorderlessFullscreen;
        self
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            title: "Ferrite Engine".to_string(),
            resizable: true,
            vsync: true,
            mode: WindowMode::Windowed,
        }
    }
}

/// Events that can happen to the window
#[derive(Message, Debug)]
pub enum WindowEvent {
    Resized { width: u32, height: u32 },
    CloseRequested,
    Focused(bool),
    Moved { x: i32, y: i32 },
}

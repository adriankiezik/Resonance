
use bevy_ecs::prelude::*;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window as WinitWindow, WindowAttributes, Fullscreen, CursorGrabMode},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    Windowed,
    Fullscreen,
    BorderlessFullscreen,
}

#[derive(Resource, Clone)]
pub struct Window {
    pub window: Arc<WinitWindow>,
}

impl Window {
    pub fn new(event_loop: &ActiveEventLoop, config: &WindowConfig) -> anyhow::Result<Self> {
        let mut attributes = WindowAttributes::default()
            .with_title(config.title.clone())
            .with_inner_size(PhysicalSize::new(config.width, config.height))
            .with_resizable(config.resizable);

        attributes = match config.mode {
            WindowMode::Windowed => {

                attributes
            }
            WindowMode::Fullscreen => {

                if let Some(monitor) = event_loop.primary_monitor() {

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

    pub fn size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    pub fn should_close(&self) -> bool {
        false
    }

    pub fn set_mode(&self, mode: WindowMode) {
        match mode {
            WindowMode::Windowed => {
                log::info!("Switching to windowed mode");
                self.window.set_fullscreen(None);
            }
            WindowMode::Fullscreen => {

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

                if let Some(monitor) = self.window.current_monitor() {
                    log::info!("Switching to borderless fullscreen mode");
                    self.window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
                } else {
                    log::warn!("No monitor detected, cannot switch to borderless fullscreen");
                }
            }
        }
    }

    pub fn toggle_fullscreen(&self) {
        if self.window.fullscreen().is_some() {
            self.set_mode(WindowMode::Windowed);
        } else {
            self.set_mode(WindowMode::BorderlessFullscreen);
        }
    }

    pub fn current_mode(&self) -> WindowMode {
        match self.window.fullscreen() {
            None => WindowMode::Windowed,
            Some(Fullscreen::Exclusive(_)) => WindowMode::Fullscreen,
            Some(Fullscreen::Borderless(_)) => WindowMode::BorderlessFullscreen,
        }
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    pub fn set_cursor_grab(&self, grab: bool) -> anyhow::Result<()> {
        let mode = if grab {

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

    pub fn with_mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn fullscreen(mut self) -> Self {
        self.mode = WindowMode::Fullscreen;
        self
    }

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
            title: "Resonance Engine".to_string(),
            resizable: true,
            vsync: true,
            mode: WindowMode::Windowed,
        }
    }
}

#[derive(Message, Debug)]
pub enum WindowEvent {
    Resized { width: u32, height: u32 },
    CloseRequested,
    Focused(bool),
    Moved { x: i32, y: i32 },
}

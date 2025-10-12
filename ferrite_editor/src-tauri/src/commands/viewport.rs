use crate::viewport::ViewportRenderer;
use crate::viewport_window;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

/// Global viewport storage
pub struct ViewportState {
    pub renderers: RwLock<HashMap<String, Arc<RwLock<ViewportRenderer>>>>,
}

impl ViewportState {
    pub fn new() -> Self {
        Self {
            renderers: RwLock::new(HashMap::new()),
        }
    }
}

/// Initialize viewport for a window
///
/// Note: This must run on the main thread for macOS Metal compatibility
#[tauri::command]
pub fn init_viewport(
    app: AppHandle,
    window_label: String,
    width: u32,
    height: u32,
    viewport_state: State<'_, ViewportState>,
) -> Result<(), String> {
    log::info!("Initializing viewport for window: {}", window_label);

    let window = app
        .get_webview_window(&window_label)
        .ok_or_else(|| format!("Window '{}' not found", window_label))?;

    let window_arc = Arc::new(window);

    // PERFORMANCE: Render at 50% resolution to reduce GPU-CPU transfer overhead
    // This quadruples performance for canvas-based rendering
    let render_width = (width / 2).max(320);
    let render_height = (height / 2).max(240);

    log::info!("Creating viewport at {}x{} (display: {}x{})", render_width, render_height, width, height);

    // Create renderer (this will run on whatever thread calls this command)
    // Tauri ensures non-async commands run on the main thread
    let renderer = ViewportRenderer::new(window_arc, render_width, render_height)
        .map_err(|e| format!("Failed to create viewport renderer: {}", e))?;

    viewport_state
        .renderers
        .write()
        .insert(window_label.clone(), Arc::new(RwLock::new(renderer)));

    log::info!("Viewport initialized for window: {}", window_label);
    Ok(())
}

/// Resize viewport
#[tauri::command]
pub fn resize_viewport(
    window_label: String,
    width: u32,
    height: u32,
    viewport_state: State<ViewportState>,
) -> Result<(), String> {
    let renderers = viewport_state.renderers.read();
    let renderer = renderers
        .get(&window_label)
        .ok_or_else(|| format!("Viewport '{}' not found", window_label))?;

    // PERFORMANCE: Render at 50% resolution to reduce GPU-CPU transfer overhead
    let render_width = (width / 2).max(320);
    let render_height = (height / 2).max(240);

    renderer.write().resize(render_width, render_height);
    Ok(())
}

/// Render viewport frame
#[tauri::command]
pub fn render_viewport(
    window_label: String,
    viewport_state: State<ViewportState>,
) -> Result<(), String> {
    let renderers = viewport_state.renderers.read();
    let renderer = renderers
        .get(&window_label)
        .ok_or_else(|| format!("Viewport '{}' not found", window_label))?;

    let mut renderer = renderer.write();
    renderer.update_camera();
    renderer
        .render()
        .map_err(|e| format!("Render error: {:?}", e))?;

    Ok(())
}

/// Orbit camera
#[tauri::command]
pub fn orbit_camera(
    window_label: String,
    delta_x: f32,
    delta_y: f32,
    viewport_state: State<ViewportState>,
) -> Result<(), String> {
    let renderers = viewport_state.renderers.read();
    let renderer = renderers
        .get(&window_label)
        .ok_or_else(|| format!("Viewport '{}' not found", window_label))?;

    let mut renderer = renderer.write();
    renderer
        .camera
        .orbit(glam::Vec3::ZERO, delta_x, delta_y);
    Ok(())
}

/// Pan camera
#[tauri::command]
pub fn pan_camera(
    window_label: String,
    delta_x: f32,
    delta_y: f32,
    viewport_state: State<ViewportState>,
) -> Result<(), String> {
    let renderers = viewport_state.renderers.read();
    let renderer = renderers
        .get(&window_label)
        .ok_or_else(|| format!("Viewport '{}' not found", window_label))?;

    let mut renderer = renderer.write();
    renderer.camera.pan(delta_x, delta_y);
    Ok(())
}

/// Zoom camera
#[tauri::command]
pub fn zoom_camera(
    window_label: String,
    delta: f32,
    viewport_state: State<ViewportState>,
) -> Result<(), String> {
    let renderers = viewport_state.renderers.read();
    let renderer = renderers
        .get(&window_label)
        .ok_or_else(|| format!("Viewport '{}' not found", window_label))?;

    let mut renderer = renderer.write();
    renderer.camera.zoom(delta);
    Ok(())
}

/// Get camera info
#[tauri::command]
pub fn get_camera_info(
    window_label: String,
    viewport_state: State<ViewportState>,
) -> Result<CameraInfo, String> {
    let renderers = viewport_state.renderers.read();
    let renderer = renderers
        .get(&window_label)
        .ok_or_else(|| format!("Viewport '{}' not found", window_label))?;

    let renderer = renderer.read();
    let camera = &renderer.camera;

    Ok(CameraInfo {
        position: [camera.position.x, camera.position.y, camera.position.z],
        rotation: [
            camera.rotation.x,
            camera.rotation.y,
            camera.rotation.z,
            camera.rotation.w,
        ],
        fov: camera.fov.to_degrees(),
    })
}

#[derive(serde::Serialize)]
pub struct CameraInfo {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub fov: f32,
}

/// Open a separate viewport window
#[tauri::command]
pub fn open_viewport_window() -> Result<(), String> {
    log::info!("Opening separate viewport window");
    viewport_window::spawn_viewport_window()
        .map_err(|e| format!("Failed to open viewport window: {}", e))?;
    Ok(())
}

/// Get viewport frame data for canvas rendering
#[tauri::command]
pub fn get_viewport_frame(
    window_label: String,
    viewport_state: State<ViewportState>,
) -> Result<Vec<u8>, String> {
    let renderers = viewport_state.renderers.read();
    let renderer = renderers
        .get(&window_label)
        .ok_or_else(|| format!("Viewport '{}' not found", window_label))?;

    let mut renderer = renderer.write();
    renderer.update_camera();
    renderer
        .get_frame_data()
        .map_err(|e| format!("Failed to get frame data: {:?}", e))
}

# Viewport Main Thread Fix

If you still see the "get_metal_layer cannot be called in non-ui thread" error after making `init_viewport` synchronous, use this alternative approach:

## Alternative Fix: Explicit Main Thread Execution

### Option 1: Use Tauri's setup hook (Recommended)

Create the viewport renderer during Tauri's setup phase, which guarantees main thread execution:

```rust
// In main.rs
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Get the main window
            let window = app.get_webview_window("main").expect("Main window not found");

            // Initialize viewport on main thread during setup
            let viewport_state: tauri::State<ViewportState> = app.state();
            let window_arc = Arc::new(window);

            let renderer = ViewportRenderer::new(window_arc, 800, 600)
                .expect("Failed to create viewport renderer");

            viewport_state.renderers.write().insert(
                "main".to_string(),
                Arc::new(RwLock::new(renderer))
            );

            Ok(())
        })
        // ... rest of the builder
}
```

### Option 2: Use dispatch to main thread

Modify the command to explicitly dispatch to main thread:

```rust
#[tauri::command]
pub fn init_viewport(
    app: AppHandle,
    window_label: String,
    width: u32,
    height: u32,
    viewport_state: State<'_, ViewportState>,
) -> Result<(), String> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    // Dispatch to main thread
    let window_label_clone = window_label.clone();
    app.run_on_main_thread(move || {
        let result = (|| -> Result<(), String> {
            let window = app
                .get_webview_window(&window_label_clone)
                .ok_or_else(|| format!("Window '{}' not found", window_label_clone))?;

            let window_arc = Arc::new(window);
            let renderer = ViewportRenderer::new(window_arc, width, height)
                .map_err(|e| format!("Failed to create viewport renderer: {}", e))?;

            Ok(renderer)
        })();

        tx.send(result).unwrap();
    })?;

    let renderer = rx.recv().map_err(|e| format!("Channel error: {}", e))??;

    viewport_state
        .renderers
        .write()
        .insert(window_label, Arc::new(RwLock::new(renderer)));

    Ok(())
}
```

## Why This Happens

macOS Metal requires that:
1. `NSView` metal layer creation happens on the main thread
2. wgpu's `Surface::configure()` internally calls Metal APIs that need main thread
3. Tauri async commands run on tokio worker threads by default

## Current Fix

The current fix (making the command synchronous) should work because Tauri 2.x runs non-async commands on the main thread by default. If it doesn't, use one of the alternatives above.

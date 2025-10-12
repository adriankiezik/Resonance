# Multi-WebView Architecture for Ferrite Editor

## Overview
The editor uses Tauri v2's multi-webview feature to render wgpu 3D content with UI panels on top.

## Window Layout

```
┌─────────────────────────────────────────────────────────┐
│              Top Toolbar (WebView "toolbar")            │  40px height
├──────────────┬──────────────────┬──────────────────────┤
│              │                  │                      │
│   Left       │    3D Viewport   │      Right          │
│   Sidebar    │    (wgpu only)   │      Properties     │
│  (WebView    │   NO WEBVIEW     │     (WebView        │
│  "sidebar")  │                  │     "properties")   │
│   300px      │    TRANSPARENT   │      350px          │
│              │                  │                     │
└──────────────┴──────────────────┴─────────────────────┘
```

## WebView Components

### 1. Toolbar WebView ("toolbar")
- **Position**: (0, 0)
- **Size**: (window_width, 40)
- **Purpose**: Main menu, tool buttons, view controls
- **Route**: `/toolbar`
- **Transparency**: Yes
- **Background**: Semi-transparent dark

### 2. Sidebar WebView ("sidebar")
- **Position**: (0, 40)
- **Size**: (300, window_height - 40)
- **Purpose**: Scene hierarchy, entity list
- **Route**: `/sidebar`
- **Transparency**: Yes
- **Background**: Semi-transparent dark

### 3. Properties WebView ("properties")
- **Position**: (window_width - 350, 40)
- **Size**: (350, window_height - 40)
- **Purpose**: Entity properties, component inspector
- **Route**: `/properties`
- **Transparency**: Yes
- **Background**: Semi-transparent dark

### 4. Viewport Area
- **Position**: (300, 40)
- **Size**: (window_width - 650, window_height - 40)
- **Purpose**: 3D scene rendering
- **Implementation**: wgpu renders directly to window surface
- **NO WEBVIEW**: This area is left clear for GPU rendering

## Technical Implementation

### Window Creation
```rust
// Create transparent window
let window = WindowBuilder::new()
    .with_title("Ferrite Scene Editor")
    .with_inner_size(LogicalSize::new(1600, 900))
    .with_transparent(true)
    .build(&event_loop)?;
```

### wgpu Surface
```rust
// Create wgpu surface on the entire window
let surface = instance.create_surface(&window)?;
// wgpu will render to the entire window, but webviews will be on top
```

### Child WebViews
```rust
// Create toolbar webview
let toolbar = WebViewBuilder::new()
    .with_url("tauri://localhost/toolbar")
    .with_bounds(Rect {
        position: LogicalPosition::new(0, 0).into(),
        size: LogicalSize::new(window_width, 40).into(),
    })
    .with_transparent(true)
    .build_as_child(&window)?;

// Similar for sidebar and properties...
```

## React Application Structure

### New Structure
```
src/
├── main.tsx              # Entry point - detects route
├── routes/
│   ├── toolbar/
│   │   └── Toolbar.tsx   # Top toolbar component
│   ├── sidebar/
│   │   └── Sidebar.tsx   # Left sidebar component
│   └── properties/
│       └── Properties.tsx # Right properties panel
├── components/
│   ├── shared/           # Shared components
│   └── ...
└── lib/
    └── webview-bridge.ts # Inter-webview communication
```

### Router Logic
```typescript
// Detect which webview we're in
const route = window.location.pathname;

if (route === '/toolbar') {
  render(<Toolbar />);
} else if (route === '/sidebar') {
  render(<Sidebar />);
} else if (route === '/properties') {
  render(<Properties />);
}
```

## Communication

### Webview-to-Webview
```typescript
// Use Tauri events for inter-webview communication
await emit('entity-selected', { entityId: '123' });
await listen('entity-selected', (event) => { ... });
```

### Webview-to-Viewport (Rust)
```typescript
// Use Tauri commands for viewport control
await invoke('orbit_camera', { deltaX, deltaY });
await invoke('select_entity', { entityId });
```

## Benefits

1. **True GPU Rendering**: wgpu renders directly to the window, no compositing overhead
2. **Flexible UI**: Each panel is independent, can be shown/hidden, resized
3. **Performance**: GPU rendering is not blocked by web rendering
4. **Transparency**: UI panels can be semi-transparent over the 3D view
5. **Native Feel**: Proper integration of web UI and native graphics

## Implementation Phases

### Phase 1: Core Setup ✅
- [x] Enable unstable feature
- [ ] Create window with transparency
- [ ] Initialize wgpu rendering
- [ ] Verify wgpu renders to window

### Phase 2: WebView Integration
- [ ] Create child webviews with proper bounds
- [ ] Set up routing in React app
- [ ] Implement basic UI in each webview
- [ ] Test transparency and layering

### Phase 3: Communication
- [ ] Implement webview-to-webview events
- [ ] Update viewport commands for new architecture
- [ ] Handle window resize events
- [ ] Update webview bounds on resize

### Phase 4: UI Implementation
- [ ] Implement toolbar UI
- [ ] Implement sidebar (scene hierarchy)
- [ ] Implement properties panel
- [ ] Add viewport camera info overlay

### Phase 5: Polish
- [ ] Optimize rendering loop
- [ ] Add panel show/hide toggles
- [ ] Implement panel resizing
- [ ] Handle edge cases and errors

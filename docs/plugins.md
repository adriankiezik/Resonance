# Plugin Guide

Complete guide to all available plugins, their dependencies, and configuration options.

## Plugin Dependency Graph

```
CorePlugin (required by all, auto-added by DefaultPlugins)
  ├─→ TimePlugin (time tracking)
  ├─→ TransformPlugin (entity transforms and hierarchy)
  └─→ AssetsPlugin (asset loading system)
      └─→ WindowPlugin (window management)
          ├─→ RenderPlugin (graphics rendering)
          │   ├─ Requires: TransformPlugin (for entity positions)
          │   └─ Requires: WindowPlugin (for render surface)
          ├─→ InputPlugin (keyboard/mouse input)
          └─→ AudioPlugin (audio playback)
              └─→ PerformancePlugin (optional, performance tracking)
```

## Core Plugins

### CorePlugin

**Purpose**: Essential engine resources (time, game tick, memory tracking)

**Dependencies**: None

**Client/Server**: Both

**Configuration**: None

**Added by DefaultPlugins**: ✅ Yes

**Resources**:
- `Time` - Frame delta time and total elapsed time
- `FixedTime` - Fixed timestep accumulator for physics
- `GameTick` - Tick counter for networking/determinism
- `MemoryTracker` - Memory usage statistics

**Usage**: Automatically included with `DefaultPlugins`. Should never be manually added.

---

### TransformPlugin

**Purpose**: Entity positioning, rotation, and hierarchy system

**Dependencies**: None

**Client/Server**: Both

**Configuration**: None

**Added by DefaultPlugins**: ✅ Yes

**Components**:
- `Transform` - Local position, rotation, scale
- `GlobalTransform` - World-space transform (computed)
- `Parent` - Parent entity reference
- `Children` - Child entity list

**Systems**:
- `propagate_transforms` (PostUpdate) - Syncs Transform → GlobalTransform

**Usage**:
```rust
use resonance::prelude::*;

fn spawn_entity(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        // GlobalTransform is auto-added and computed
    ));
}
```

---

### AssetsPlugin

**Purpose**: Asynchronous asset loading with caching

**Dependencies**: None

**Client/Server**: Both

**Configuration**: Optional `AssetSourceConfig`

**Added by DefaultPlugins**: ✅ Yes

**Resources**:
- `Assets` - Main asset loading interface
- `AssetCache` - Shared asset cache

**Loaders**:
- `TextureLoader` - PNG/JPEG images
- `ObjLoader` / `GltfLoader` - 3D models
- `AudioLoader` - Audio files
- `TtfLoader` - Fonts
- `WgslLoader` - Shaders

**Usage**: See [Asset Loading Patterns](../src/assets/mod.rs) documentation

---

### WindowPlugin

**Purpose**: Window creation and event handling (winit)

**Dependencies**: None

**Client/Server**: Client only

**Configuration**: `WindowConfig` resource (optional)

**Added by DefaultPlugins**: ✅ Yes

**Resources**:
- `Window` - Window handle and state

**Configuration Example**:
```rust
use resonance::prelude::*;

Resonance::new()
    .with_resource(WindowConfig {
        width: 1920,
        height: 1080,
        title: "My Game".to_string(),
        mode: WindowMode::Windowed,
    })
    .add_plugin(DefaultPlugins)
    .run();
```

---

### RenderPlugin

**Purpose**: GPU rendering with wgpu

**Dependencies**:
- ⚠️ **Required**: `WindowPlugin` (for render surface)
- ⚠️ **Required**: `TransformPlugin` (for entity transforms)

**Client/Server**: Client only

**Configuration**: `GraphicsSettings` resource (optional)

**Added by DefaultPlugins**: ✅ Yes

**Resources**:
- `Renderer` - wgpu device/queue/surface
- `RenderGraph` - Render pass graph
- `GraphicsSettings` - MSAA, VSync settings
- `GpuMeshCache` - GPU mesh buffers

**Components**:
- `Camera` - Camera with projection matrix
- `Mesh` - 3D mesh reference
- `DirectionalLight` / `PointLight` / `AmbientLight`

**Configuration Example**:
```rust
use resonance::prelude::*;

Resonance::new()
    .with_graphics_settings(GraphicsSettings::new(
        MsaaSampleCount::X4,
        true, // VSync enabled
    ))
    .add_plugin(DefaultPlugins)
    .run();
```

---

### InputPlugin

**Purpose**: Keyboard and mouse input handling

**Dependencies**: None (works best with WindowPlugin)

**Client/Server**: Client only

**Configuration**: None

**Added by DefaultPlugins**: ✅ Yes

**Resources**:
- `Input<KeyCode>` - Keyboard state
- `Input<MouseButton>` - Mouse button state

**Usage**:
```rust
use resonance::prelude::*;

fn handle_input(input: Res<Input<KeyCode>>) {
    if input.pressed(KeyCode::Space) {
        println!("Space pressed!");
    }
}
```

---

### AudioPlugin

**Purpose**: Audio playback with spatial 3D audio

**Dependencies**: None

**Client/Server**: Client only

**Configuration**: None

**Added by DefaultPlugins**: ✅ Yes

**Components**:
- `AudioSource` - Audio emitter
- `AudioListener` - Audio receiver (camera)
- `Spatial3dAudio` - 3D audio settings

---

### PerformancePlugin

**Purpose**: Performance profiling and analytics

**Dependencies**: None

**Client/Server**: Both

**Configuration**: None

**Added by DefaultPlugins**: ✅ Yes (as of latest update)

**Resources**:
- `PerformanceAnalytics` - Frame time statistics
- `Profiler` - Per-system timing (optional)

---

## Addon Plugins

### WireframePlugin

**Purpose**: Wireframe rendering overlay

**Dependencies**: RenderPlugin

**Location**: `resonance::addons::WireframePlugin`

**Added by DefaultPlugins**: ❌ No

**Usage**:
```rust
use resonance::prelude::*;
use resonance::addons::WireframePlugin;

Resonance::new()
    .add_plugin(DefaultPlugins)
    .add_plugin(WireframePlugin)
    .run();
```

---

## Custom Plugin Creation

To create a custom plugin, implement the `Plugin` trait:

```rust
use resonance::prelude::*;

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, engine: &mut Resonance) {
        // Add resources
        engine.world.insert_resource(MyResource::default());

        // Add systems
        if let Some(schedule) = engine.schedules.get_mut(Stage::Update) {
            schedule.add_systems(my_system);
        }
    }

    fn dependencies(&self) -> Vec<(std::any::TypeId, &str)> {
        vec![
            (std::any::TypeId::of::<RenderPlugin>(), "resonance::renderer::RenderPlugin"),
        ]
    }

    fn is_client_plugin(&self) -> bool {
        true  // Only load in client mode
    }

    fn is_server_plugin(&self) -> bool {
        false  // Don't load in server mode
    }
}

fn my_system() {
    println!("My system runs every frame!");
}
```

## Plugin Best Practices

1. **Declare Dependencies**: Always specify plugin dependencies via `dependencies()`
2. **Client/Server Mode**: Set `is_client_plugin()` and `is_server_plugin()` correctly
3. **Use DefaultPlugins**: Start with `DefaultPlugins` for standard games
4. **Add Specialized Last**: Add game-specific plugins after engine plugins
5. **Order Matters**: Plugin order determines initialization order (dependencies first)

## Troubleshooting

### "Missing required dependency" Error

If you see:
```
Plugin 'MyPlugin' is missing required dependency 'RenderPlugin'
```

**Solution**: Add the dependency plugin before your plugin:

```rust
Resonance::new()
    .add_plugin(DefaultPlugins)  // Includes RenderPlugin
    .add_plugin(MyPlugin)         // Now works!
    .run();
```

### Plugin Not Loading in Server Mode

If your plugin doesn't load when running in server mode:

**Cause**: Plugin has `is_server_plugin() = false`

**Solution**: Enable server mode or create a server-compatible version:

```rust
impl Plugin for MyPlugin {
    fn is_server_plugin(&self) -> bool {
        true  // Now loads on server
    }
}
```

# Resonance Plugins

## DefaultPlugins
Bundles all commonly required plugins for indie game development. Use this for quick setup.

**Includes:**
- CorePlugin
- TransformPlugin
- AssetsPlugin
- WindowPlugin (client-only)
- RenderPlugin (client-only)
- AudioPlugin (client-only)

**Usage:**
```rust
Resonance::new().add_plugin(DefaultPlugins).run();
```

---

## CorePlugin
- **Dependencies:** None
- **In DefaultPlugins:** Yes
- Core engine functionality: time management, game ticks, logging

## TransformPlugin
- **Dependencies:** None
- **In DefaultPlugins:** Yes
- Entity transforms and hierarchical parent-child relationships

## WindowPlugin
- **Dependencies:** None
- **In DefaultPlugins:** Yes
- Window creation and management with event handling

## AssetsPlugin
- **Dependencies:** None
- **In DefaultPlugins:** Yes
- Asset loading and caching (textures, meshes, audio, fonts, shaders)

## RenderPlugin
- **Dependencies:** WindowPlugin
- **In DefaultPlugins:** Yes
- 3D mesh rendering with camera support using wgpu

## AudioPlugin
- **Dependencies:** None
- **In DefaultPlugins:** Yes
- Audio playback with spatial 3D audio and doppler effects

# Resonance

A modular game engine built on Bevy ECS.

⚠️ **Heavy work in progress** - Breaking changes expected. Usage not recommended.

## Features

- Entity Component System (ECS) using Bevy
- Transform hierarchy system
- Asset management (textures, meshes, audio, fonts)
- Input handling
- Audio system
- Graphics rendering with wgpu
- Window management

## Status

This project is in very early stages. Core systems are being implemented and the API is subject to change.

## Quick Start

```rust
use resonance::prelude::*;

fn main() {
    Engine::new()
        .add_plugin(CorePlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(WindowPlugin::default())
        .add_plugin(AssetsPlugin::default())
        .add_plugin(RenderPlugin::default())
        .run();
}
```

## Documentation

- [Plugin Guide](docs/plugins.md) - Complete guide to all available plugins, their dependencies, and configuration options

## License

MIT

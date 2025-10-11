# Ferrite Engine Examples

This directory contains example projects demonstrating various features of the Ferrite engine.

## Running Examples

```bash
# Run an example
cargo run --example hello_engine

# Run with logging
RUST_LOG=info cargo run --example ecs_demo

# Run server example
cargo run --example server_basic --features server
```

## Examples

### Basic Examples

- **hello_engine**: Minimal engine setup showing the basic game loop
- **ecs_demo**: Demonstrates ECS usage with custom components and systems
- **physics_sim**: Basic physics simulation with gravity and collisions

### Window & Rendering Examples

- **hello_window**: Create a window and render a clear color
- **window_modes**: Demonstrates fullscreen and borderless window modes

### Networking Examples

- **server_basic**: Dedicated server setup in headless mode

## TODO Examples (To be implemented)
- **hello_triangle**: Render a simple triangle
- **sprites**: 2D sprite rendering
- **camera_3d**: 3D camera controls
- **multiplayer_client**: Client connecting to a server
- **multiplayer_game**: Complete client-server game example
- **asset_loading**: Loading and using assets (textures, meshes, audio)
- **audio_playback**: Playing sound effects and music

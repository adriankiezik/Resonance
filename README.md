# Ferrite Game Engine

A multiplayer-ready game engine built in Rust with ECS architecture and first-class networking support.

## Features

- **ECS Architecture**: Built on `bevy_ecs` for high-performance entity-component-system design
- **Multiplayer Ready**: Client-server architecture with entity replication and state synchronization
- **Modular Design**: Plugin-based system for easy extensibility
- **Cross-Platform**: Runs on Windows, macOS, and Linux
- **Modern Graphics**: wgpu-based rendering for Vulkan/Metal/DX12 support
- **Physics**: Integrated physics simulation with collision detection
- **Asset Management**: Flexible asset loading with caching and hot-reloading support

## Project Structure

```
ferrite/
├── crates/
│   ├── ferrite_core/      # Core utilities (time, math, logging)
│   ├── ferrite_app/       # Application framework (engine, plugins)
│   ├── ferrite_transform/ # Transform system and hierarchy
│   ├── ferrite_physics/   # Physics simulation
│   ├── ferrite_network/   # Multiplayer networking
│   ├── ferrite_client/    # Client features (rendering, input, audio)
│   ├── ferrite_server/    # Server features (authority, validation)
│   ├── ferrite_assets/    # Asset loading and management
│   └── ferrite/           # Main crate (re-exports all)
├── examples/              # Example projects
└── assets/                # Example assets
```

## Quick Start

### Prerequisites

- Rust 1.70 or later
- A GPU with Vulkan, Metal, or DirectX 12 support

### Building

```bash
# Build the entire workspace
cargo build

# Build with optimizations
cargo build --release
```

### Running Examples

```bash
# Run the hello_engine example
cargo run --example hello_engine

# Run with logging
RUST_LOG=info cargo run --example ecs_demo

# Run the physics simulation
cargo run --example physics_sim

# Run a dedicated server
cargo run --example server_basic --features server
```

## Getting Started

```rust
use ferrite::prelude::*;

fn main() {
    // Initialize logging
    init_logger();

    // Create and configure the engine
    Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(TransformPlugin)
        .add_startup_system(setup)
        .add_system(Stage::Update, game_logic)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn entities during startup
    commands.spawn((
        Transform::from_position(Vec3::ZERO),
        // Your components here
    ));
}

fn game_logic(/* your system parameters */) {
    // Your game logic here
}
```

## Development Status

**Current Status**: Phase 2 - ECS & Application Framework Complete ✅ | **Dependencies: Latest Stable** ✅

Phase 2 implementation is complete! The engine now has a robust plugin system with dependency tracking, advanced system scheduling with run conditions and ordering, and comprehensive ECS capabilities. See [ROADMAP.md](ROADMAP.md) for the complete development plan.

### Dependency Versions (Updated 2025-10-11)
- **bevy_ecs 0.17** - Latest ECS with improved `IntoScheduleConfigs` API
- **wgpu 27.0** - Modern graphics backend (Vulkan/Metal/DX12)
- **glam 0.30** - High-performance SIMD math library
- **renet 1.2** - Reliable UDP networking for multiplayer
- **rodio 0.21** - Cross-platform audio playback
- **winit 0.30** - Window management
- **bincode 2.0** - Fast binary serialization
- **thiserror 2.0** - Error handling
- **tokio 1.47** - Async runtime

### What Works
- ✅ ECS architecture with bevy_ecs 0.17
- ✅ Engine initialization and game loop with FPS limiting
- ✅ Time management with pause/resume and time scale
- ✅ Fixed timestep (60Hz) for deterministic simulation
- ✅ Game tick counter for multiplayer
- ✅ Transform components with parent-child hierarchy
- ✅ Transform propagation system
- ✅ **Plugin system with dependency tracking and state management**
- ✅ **System ordering (before/after) and run conditions**
- ✅ **System chaining for sequential execution**
- ✅ **Plugin mode filtering (client/server)**
- ✅ Basic physics components (boilerplate)
- ✅ Networking protocol definitions (boilerplate)
- ✅ Asset management system (boilerplate)
- ✅ All dependencies updated to latest stable versions

### Next Up (Phase 3)
- 📋 Window management with winit
- 📋 Rendering pipeline with wgpu
- 📋 Camera system
- 📋 Mesh and material system

See [ROADMAP.md](ROADMAP.md) for detailed implementation plan and testing strategy.

## Features

- `client` (default): Client-only features (rendering, audio, input)
- `server`: Server-only features (authority, validation)
- `full`: Both client and server features

## Documentation

- [ROADMAP.md](ROADMAP.md) - Development plan and testing strategy
- [examples/](examples/) - Example projects and tutorials
- API Documentation: Run `cargo doc --open`

## Architecture

### ECS (Entity Component System)
Ferrite uses `bevy_ecs`, providing:
- High performance with parallel system execution
- Flexible component composition
- Built-in change detection for networking

### Multiplayer Architecture
- **Client-Server Model**: Authoritative server with client prediction
- **Entity Replication**: Automatic state synchronization
- **Deterministic Simulation**: Fixed timestep physics
- **Headless Server**: Can run without graphics for dedicated servers

### Plugin System
Features are organized into plugins that can be easily added or removed:
- `CorePlugin` - Time, logging, core utilities
- `TransformPlugin` - Spatial transforms and hierarchy
- `PhysicsPlugin` - Physics simulation
- `ClientPlugin` - Rendering, input, audio
- `ServerPlugin` - Server authority and validation
- `NetworkClientPlugin` / `NetworkServerPlugin` - Networking

## Contributing

This is currently a personal project. Contributions, suggestions, and feedback are welcome!

## License

MIT OR Apache-2.0

## Acknowledgments

- Built with [bevy_ecs](https://github.com/bevyengine/bevy) - High-performance ECS
- Graphics via [wgpu](https://wgpu.rs/) - Modern graphics API
- Windowing via [winit](https://github.com/rust-windowing/winit)
- Math via [glam](https://github.com/bitshifter/glam-rs)
- Networking via [renet](https://github.com/lucaspoffo/renet)

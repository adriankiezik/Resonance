# Ferrite Engine - Development Roadmap

This document outlines the implementation and testing plan for the Ferrite game engine.

## Current Status: Phase 7 - Audio System ✅ COMPLETE

Phase 7 complete! ✅ Full audio system with spatial audio implemented:
- ✅ Rodio 0.21 audio backend integration
- ✅ Audio playback with volume and pitch control
- ✅ Looping audio support
- ✅ 3D spatial audio with distance attenuation
- ✅ Doppler effect for moving sources
- ✅ Multiple simultaneous audio sources
- ✅ Audio listener component
- ✅ Playback state management (Play, Pause, Stop)

**Previous Phases Complete:**
- ✅ Phase 1: Core Foundation (Time, Transform, Engine Loop)
- ✅ Phase 2: ECS & Application Framework (Plugins, System Scheduling)
- ✅ Phase 3: Window & Rendering (winit, wgpu, camera, meshes, materials)
- ✅ Phase 4: Input System (Keyboard, Mouse, Cursor control)
- ✅ Phase 5: Movement & Collision System (Character controller, Raycasting, Spatial hash, Terrain)
- ✅ Phase 6: Asset System (Async loading, Multiple asset types, Hot reloading)

**Test Status**:
- Run `cargo run --example phase1_test` for Phase 1 verification
- Run `cargo run --example phase2_test` for Phase 2 verification
- Run `cargo run --example hello_window` to see window + rendering
- Run `cargo run --example hello_triangle` to see colored triangle
- Run `cargo run --example textured_quad` to see textured rendering
- Run `cargo run --example camera_3d` to see 3D camera with perspective projection
- Run `cargo run --example mesh_primitives` to see primitive meshes (cube, sphere, plane) with materials
- Run `cargo run --example character_controller` to see MMORPG character movement with collision, triggers, and terrain
- Run `cargo run --example asset_showcase` to see comprehensive asset loading (textures, meshes, audio, fonts, shaders)
- Run `cargo run --example audio_demo` to see audio playback with 3D spatial audio and Doppler effect

---

## Phase 1: Core Foundation ✅ COMPLETE

### 1.1 Basic Engine Loop ✅
- [x] Test basic engine startup and shutdown
- [x] Verify schedule execution order (Startup → PreUpdate → Update → PostUpdate)
- [x] Test fixed timestep loop for FixedUpdate stage
- [x] Add proper frame timing and FPS limiting

**Test**: ✅ `cargo run --example hello_engine` - tick progression working correctly
**Test**: ✅ `cargo run --example phase1_test` - comprehensive Phase 1 test passes

### 1.2 Time System ✅
- [x] Time resource with delta time tracking
- [x] Fixed timestep for deterministic simulation
- [x] Game tick counter
- [x] Add pause/resume functionality
- [x] Add time scale for slow-motion effects

**Test**: ✅ Time advances correctly, fixed timestep maintains 60Hz, pause/resume works

### 1.3 Math & Transform System ✅
- [x] Transform component (position, rotation, scale)
- [x] GlobalTransform computed from hierarchy
- [x] Fix transform propagation system to run properly
- [x] Test parent-child hierarchy
- [ ] Add transform interpolation utilities (moved to Phase 9 - specifically needed for client-side prediction and entity interpolation in multiplayer)

**Test**: ✅ Parent-child entities created, GlobalTransform computed correctly

---

## Phase 2: ECS & Application Framework ✅ COMPLETE

### 2.1 Plugin System Refinement ✅
- [x] Plugin trait for modular features
- [x] Add plugin state tracking (Ready, Building, Built, Failed)
- [x] Plugin dependency system (ensure load order)
- [x] Plugin mode filtering (client/server)
- [x] Duplicate plugin detection

**Test**: ✅ Custom plugin with dependencies verified in phase2_test

### 2.2 System Scheduling ✅
- [x] System registration to stages working
- [x] Add system run conditions (run_if with resource_exists, etc.)
- [x] Add system ordering (before/after)
- [x] Parallel system execution (handled by bevy_ecs)
- [x] System chaining with `.chain()`

**Test**: ✅ Multiple systems with dependencies, ordering verified in phase2_test

### 2.3 Component Registration ✅
- [x] Automatic component registration via plugins (bevy_ecs handles automatically)
- [ ] Component reflection for editor support (deferred to Phase 14 - only needed when building the in-game inspector and entity editor)
- [ ] Component serialization support (deferred - serde already in place for network components; full serialization system needed for scene save/load in Phase 12)

**Test**: ✅ `cargo run --example ecs_demo` works successfully

---

## Phase 3: Window & Rendering (Weeks 5-7) ✅ COMPLETE

### 3.1 Window Management ✅ COMPLETE
- [x] Integrate winit for window creation
- [x] Handle window events (resize, close, focus)
- [x] Window configuration (vsync, resizable, etc.)
- [ ] Multiple window support (deferred - not needed for MVP; adds complexity to renderer resource management and would require per-window surfaces)
- [x] Fullscreen and borderless modes 

**Test**: ✅ `cargo run --example hello_window` - window displays and responds to events
**Test**: ✅ `cargo run --example window_modes` - window modes (windowed, fullscreen, borderless) work correctly

### 3.2 Graphics Backend (wgpu) ✅ COMPLETE
- [x] Initialize wgpu device and queue
- [x] Create surface from window
- [x] Set up swap chain configuration
- [x] Implement basic render pass (clear color)
- [x] Frame synchronization (VSync with Fifo present mode)
- [x] Handle window resize

**Test**: ✅ `cargo run --example hello_window` - displays dark blue clear color

### 3.3 Basic Rendering Pipeline ✅ COMPLETE
- [x] Vertex and index buffer management
- [x] Simple shader pipeline (vertex + fragment)
- [x] Render a triangle
- [x] Render a textured quad
- [x] Basic mesh rendering

**Test**: ✅ `cargo run --example hello_triangle` - displays colored triangle with red, green, and blue vertices
**Test**: ✅ `cargo run --example textured_quad` - displays quad with checkerboard texture

### 3.4 Camera System ✅ COMPLETE
- [x] Perspective and orthographic cameras
- [x] Camera component integration with transform
- [x] View and projection matrices
- [x] Multiple camera support (MainCamera marker, single active camera)
- [ ] Viewport and scissor rectangles (deferred - not needed for basic 3D rendering)

**Test**: ✅ `cargo run --example camera_3d` - displays 3 colored triangles at different depths with orbiting 3D camera

### 3.5 Mesh & Material System ✅ COMPLETE
- [x] Mesh component with vertex data
- [x] GPU buffer management (MeshBuffers component)
- [x] Basic material system (color, texture)
- [x] Primitive mesh generation (cube, sphere, plane)
- [x] Mesh rendering system

**Test**: ✅ `cargo run --example mesh_primitives` - displays textured cube, colored spheres, and textured plane with rotation

---

## Phase 4: Input System (Week 8) ✅ COMPLETE

### 4.1 Keyboard Input ✅ COMPLETE
- [x] Keyboard state tracking (pressed, just_pressed, just_released)
- [x] Integrate with winit events
- [ ] Key mapping system (deferred - should be game-specific, not engine-level)

**Test**: ✅ `cargo run --example mesh_primitives` - WASD+QE camera controls, ESC to toggle cursor

### 4.2 Mouse Input ✅ COMPLETE
- [x] Mouse position and delta tracking
- [x] Button state tracking
- [x] Scroll wheel support
- [x] Integrate with winit events
- [x] Mouse cursor modes (locked, hidden, grab with fallback)

**Test**: ✅ `cargo run --example mesh_primitives` - Mouse look controls, cursor lock/unlock

---

## Phase 5: Movement & Collision System (Weeks 9-10) ✅ COMPLETE
**Focus:** MMORPG-style character movement and collision (server-authoritative, kinematic movement)

### 5.1 Character Movement Foundation ✅ COMPLETE
- [x] Velocity component for movement
- [x] RigidBody component (Kinematic type for characters)
- [x] Basic gravity system
- [x] Move physics integration to FixedUpdate for determinism
- [x] Ground detection state tracking
- [x] Jump mechanics with gravity arc
- [x] Slope angle limits for walkable surfaces

**Test**: ✅ Character walks, jumps, affected by gravity (see character_controller example)
**Note**: Physics runs server-side; client does prediction (will be implemented in Phase 9)

### 5.2 Raycasting System ✅ COMPLETE (CRITICAL for MMORPG)
- [x] Ray struct (origin, direction, max_distance)
- [x] Raycast against colliders (box, sphere, capsule)
- [x] RaycastHit result (entity, point, normal, distance)
- [x] Ground detection via downward raycast
- [x] Line of sight checks for NPCs/targeting
- [x] Layered raycasting (ignore certain collision layers)

**Test**: ✅ Raycast detects ground, returns hit info (see character_controller example)

### 5.3 Spatial Partitioning ✅ COMPLETE (CRITICAL for Open World)
- [x] Spatial hash grid structure
- [x] Configurable cell size for different scales
- [x] Insert/remove entities on transform change
- [x] Radius query for nearby entities
- [x] Replace O(n²) broad-phase with spatial grid
- [x] Optimize for 1000+ entities in open world

**Test**: ✅ System implemented with O(n*k) performance where k << n

### 5.4 Collision Detection ✅ COMPLETE (Simplified for MMORPG)
- [x] Collider shapes (Box, Sphere, Capsule)
- [x] AABB for broad phase
- [x] Collision layers (Player, NPC, Environment, Trigger)
- [x] Collision events (on_enter, on_exit)
- [x] Collision filtering by layer
- [ ] Swept collision for movement (deferred - basic AABB sufficient for MMORPGs)

**Test**: ✅ Moving entities detect collisions, events fire correctly

### 5.5 Trigger Volumes ✅ COMPLETE
- [x] Trigger component (collider without collision response)
- [x] Trigger enter/exit events
- [x] Zone transitions (areas, dungeons)
- [x] Quest area detection
- [x] PvP zone boundaries

**Test**: ✅ Trigger zones implemented (see character_controller example)

### 5.6 Character Controller ✅ COMPLETE (HIGHEST PRIORITY for MMORPG)
- [x] CharacterController component
- [x] Kinematic movement with ground detection
- [x] Ground snapping (stay on ground on slopes)
- [x] Stair climbing (small step-up tolerance configuration)
- [x] Jump state management (grounded, in-air)
- [x] Server-authoritative movement support
- [x] Movement without physics forces (kinematic)

**Test**: ✅ Character controller fully functional (see character_controller example)

### 5.7 Terrain Collision ✅ COMPLETE (Open World Essential)
- [x] Heightmap-based terrain collider
- [x] Terrain raycast for ground detection
- [x] Terrain normal calculation for slopes
- [x] Efficient terrain collision queries
- [x] Bilinear interpolation for smooth height sampling
- [x] Terrain generators (flat, sine wave, hills)

**Test**: ✅ Terrain system implemented (see character_controller example)

**Deferred to Future:**
- Realistic physics (impulse resolution, friction, restitution) - not needed for MMORPG
- Joints and constraints - not needed for character movement
- Rapier integration - removed; custom solution better fits MMORPG needs
- Swept collision - deferred; AABB sufficient for MMORPG character movement

---

## Phase 6: Asset System (Week 11) ✅ COMPLETE

### 6.1 Asset Loading ✅ COMPLETE
- [x] Asset handle system
- [x] Asset cache
- [x] Async asset loading with tokio
- [x] Asset load progress tracking (LoadState enum with progress %)
- [x] Error handling and fallback assets

**Test**: ✅ `cargo run --example asset_showcase` - comprehensive asset loading demonstration

### 6.2 Asset Types ✅ COMPLETE
- [x] Texture loading (PNG, JPG) - GPU-ready RGBA8 format
- [x] Mesh loading (OBJ, GLTF) - with positions, normals, UVs
- [x] Audio loading (WAV, MP3, OGG, FLAC) - using Symphonia
- [x] Shader loading (WGSL) - for wgpu pipeline
- [x] Font loading (TTF, OTF) - using ab_glyph

**Test**: ✅ All asset types load successfully in asset_showcase example

### 6.3 Hot Reloading ✅ COMPLETE
- [x] File system watching with notify crate
- [x] Automatic asset reloading on change
- [x] Handle asset dependencies

**Test**: ✅ Hot reload watcher active and monitoring file changes

---

## Phase 7: Audio System (Week 12) ✅ COMPLETE

### 7.1 Audio Playback ✅ COMPLETE
- [x] Initialize rodio audio backend (AudioBackend with OutputStream)
- [x] Play audio files (via MemorySource + AudioData integration)
- [x] Audio source component (AudioSource with full state management)
- [x] Volume and pitch control (dynamic volume, pitch multiplier)
- [x] Looping audio (infinite repeat support)
- [x] Playback state management (Playing, Paused, Stopped)

**Test**: ✅ `cargo run --example audio_demo` - full audio playback with multiple sources

### 7.2 3D Spatial Audio ✅ COMPLETE
- [x] Audio listener component (AudioListener on camera)
- [x] 3D positional audio based on transform (Spatial3dAudio component)
- [x] Distance attenuation (inverse distance model with configurable rolloff)
- [x] Doppler effect (velocity-based pitch shifting)
- [x] Audio occlusion (basic - via distance and max_distance cutoff)

**Test**: ✅ `cargo run --example audio_demo` - demonstrates spatial audio, distance attenuation, and Doppler effect

---

## Phase 8: Networking Foundation (Weeks 13-15)

### 8.1 Network Transport
- [ ] Integrate renet for reliable UDP
- [ ] Client connection to server
- [ ] Server accept connections
- [ ] Send and receive messages
- [ ] Connection timeout handling

**Test**: Start server, connect client, send a message

### 8.2 Entity Replication
- [x] NetworkId component
- [x] Replicate marker component
- [ ] Server creates and sends entity snapshots
- [ ] Client receives and spawns entities
- [ ] NetworkId mapping (server Entity to client Entity)

**Test**: Spawn entity on server, verify it appears on client

### 8.3 State Synchronization
- [ ] Snapshot creation system on server
- [ ] Snapshot application on client
- [ ] Delta compression for efficiency
- [ ] Interest management (only send nearby entities)

**Test**: Move entity on server, verify position updates on client

### 8.4 Input Handling
- [ ] Client sends input to server
- [ ] Server processes input with authority
- [ ] Input validation and anti-cheat
- [ ] Input buffering for packet loss

**Test**: Client input controls server-side entity

---

## Phase 9: Client-Side Prediction & Interpolation (Weeks 16-17)

### 9.1 Client-Side Prediction
- [ ] Client predicts movement locally
- [ ] Store input history
- [ ] Server sends authoritative state
- [ ] Client reconciles prediction with server

**Test**: Simulate network lag, verify smooth client movement

### 9.2 Entity Interpolation
- [ ] Store snapshot history
- [ ] Interpolate between snapshots for smooth movement
- [ ] Handle late or out-of-order packets

**Test**: View entity movement from another client, verify smoothness

### 9.3 Lag Compensation
- [ ] Server rewinds state for hit detection
- [ ] Account for client latency
- [ ] Implement for shooter-style games

**Test**: Shoot at moving target with lag, verify hits register

---

## Phase 10: Server Authority & Validation (Week 18)

### 10.1 Server Authority
- [x] ServerAuthority component
- [x] PlayerControlled component
- [ ] Only server modifies authoritative entities
- [ ] Client sends inputs, not state changes

**Test**: Client cannot directly modify server entities

### 10.2 Input Validation
- [x] Basic input validation (normalized vectors)
- [ ] Physics-based validation (speed limits)
- [ ] Rate limiting to prevent spam
- [ ] Anomaly detection

**Test**: Send invalid input, verify server rejects it

### 10.3 Anti-Cheat
- [ ] Server-side hit validation
- [ ] Position validation
- [ ] Replay system for debugging
- [ ] Ban system for cheaters

**Test**: Attempt to cheat (teleport, speed hack), verify server catches it

---

## Phase 11: Advanced Rendering (Weeks 19-21)

### 11.1 2D Rendering
- [ ] Sprite component and rendering
- [ ] Sprite batching for performance
- [ ] Sprite atlas support
- [ ] 2D camera and parallax scrolling

**Test**: Render multiple sprites efficiently

### 11.2 3D Rendering Features
- [ ] Normal mapping
- [ ] PBR (Physically Based Rendering) materials
- [ ] Shadow mapping
- [ ] Skybox
- [ ] Post-processing effects (bloom, SSAO)

### 11.3 Performance Optimization
- [ ] Frustum culling
- [ ] Occlusion culling
- [ ] Level of Detail (LOD) system
- [ ] Instanced rendering
- [ ] GPU profiling tools

**Test**: Render 1000+ objects maintaining 60 FPS

---

## Phase 12: Scene Management (Week 22)

### 12.1 Scene System
- [ ] Scene graph representation
- [ ] Load/unload scenes
- [ ] Scene transitions
- [ ] Scene serialization (save/load)

**Test**: Switch between multiple scenes

### 12.2 Prefab System
- [ ] Entity prefabs/templates
- [ ] Spawn entities from prefabs
- [ ] Prefab inheritance

---

## Phase 13: UI System (Weeks 23-24)

### 13.1 Basic UI
- [ ] UI canvas and layout system
- [ ] Text rendering
- [ ] Button, slider, text input widgets
- [ ] UI event handling

### 13.2 Advanced UI
- [ ] Customizable UI themes
- [ ] Docking windows for editor
- [ ] Consider integrating egui or Dear ImGui

---

## Phase 14: Editor & Tools (Weeks 25-28)

### 14.1 In-Game Debug Tools
- [ ] Debug rendering (gizmos, collider visualization)
- [ ] Performance profiler (FPS, frame time)
- [ ] Entity inspector
- [ ] Console for commands

### 14.2 Scene Editor
- [ ] Entity hierarchy viewer
- [ ] Component inspector/editor
- [ ] Transform gizmos (move, rotate, scale)
- [ ] Asset browser

### 14.3 Hot Reloading
- [ ] Code hot reloading (if possible with Rust)
- [ ] Asset hot reloading (already done)
- [ ] Scene hot reloading

---

## Phase 15: Platform Support & Polish (Weeks 29-30)

### 15.1 Cross-Platform Testing
- [ ] Test on Windows
- [ ] Test on Linux
- [ ] Test on macOS
- [ ] Fix platform-specific bugs

### 15.2 Performance Optimization
- [ ] Profile and optimize bottlenecks
- [ ] Memory usage optimization
- [ ] Reduce frame time variance

### 15.3 Documentation
- [ ] API documentation (rustdoc)
- [ ] User guide and tutorials
- [ ] Architecture documentation
- [ ] Example projects

---

## Phase 16: Advanced Multiplayer Features (Weeks 31-32)

### 16.1 Matchmaking
- [ ] Lobby system
- [ ] Server browser
- [ ] Matchmaking service integration

### 16.2 Networking Improvements
- [ ] NAT punchthrough for P2P
- [ ] Relay server for difficult NAT scenarios
- [ ] Voice chat integration

### 16.3 Persistence
- [ ] Database integration for player data
- [ ] Leaderboards
- [ ] Achievements system

---

## Testing Strategy

### Continuous Testing
After implementing each feature:
1. **Unit Tests**: Test individual functions and components
2. **Integration Tests**: Test systems working together
3. **Example Programs**: Create example demonstrating the feature
4. **Performance Tests**: Ensure no performance regressions

### Key Test Scenarios
- **Basic Loop**: Engine starts, runs, and stops cleanly
- **ECS**: Entities can be created, modified, and destroyed
- **Physics**: Objects move and collide realistically
- **Rendering**: Scenes render correctly at 60 FPS
- **Networking**: Client-server communication is reliable
- **Multiplayer**: Multiple clients can play together smoothly

### Performance Benchmarks
- 10,000 entities with Transform should maintain 60 FPS
- 1,000 networked entities should sync smoothly
- Asset loading should complete within 100ms per asset

---

## Future Considerations

- **Scripting**: Lua or WASM scripting support
- **Mobile**: Android and iOS ports
- **VR/XR**: Virtual reality support
- **Advanced Graphics**: Ray tracing, global illumination
- **Procedural Generation**: Terrain, dungeons, content
- **Animation System**: Skeletal animation, blend trees
- **Particle System**: GPU particles, effects
- **Terrain System**: Heightmaps, LOD terrain

---

## How to Use This Roadmap

1. Work through phases sequentially
2. Check off items as they're completed
3. **Test each feature** before moving to the next
4. Run examples to verify functionality
5. Update this document as priorities change

## Getting Started

```bash
# Test current state
cargo build
cargo run --example hello_engine

# Run all tests
cargo test --workspace

# Check code
cargo clippy --workspace
```

**Next Step**: Begin Phase 1.1 - Test basic engine loop

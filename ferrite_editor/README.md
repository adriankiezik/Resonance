# Ferrite Scene Editor

A professional, modern scene editor for the Ferrite game engine built with Tauri, React 19 (with React Compiler), and shadcn/ui.

## Features

- **Modern UI**: Built with shadcn/ui and Tailwind CSS for a professional, industry-standard interface
- **Real-time Editing**: Edit scenes in real-time with instant visual feedback
- **Entity Management**: Create, delete, and organize entities in a hierarchical structure
- **Component Inspector**: Edit entity components with intuitive UI controls
- **3D Viewport**: Integrated viewport for visualizing and manipulating scenes
- **Scene Management**: Create, load, and save scenes in RON format
- **Designer-Friendly**: No coding required - perfect for game designers and artists

## Tech Stack

- **Backend**: Rust + Tauri 2.x + Ferrite Engine
- **Frontend**: React 19 with React Compiler
- **UI Framework**: shadcn/ui (Radix UI primitives)
- **Styling**: Tailwind CSS
- **State Management**: Zustand
- **Build Tool**: Vite

## Prerequisites

- **Node.js**: v20 or later
- **npm**: v10 or later
- **Rust**: 1.70 or later (with `cargo`)
- **System dependencies**: See Tauri's [prerequisites guide](https://tauri.app/v2/guides/prerequisites/)

## Installation

### 1. Clone the repository

If you haven't already, clone the Ferrite repository:

```bash
git clone https://github.com/yourusername/ferrite.git
cd ferrite
```

### 2. Navigate to the editor directory

```bash
cd ferrite_editor
```

### 3. Install Node dependencies

```bash
npm install
```

### 4. Install Rust dependencies

The Rust dependencies will be installed automatically when you run the app, but you can manually trigger it:

```bash
cd src-tauri
cargo build
cd ..
```

## Development

### Run in development mode

```bash
npm run tauri:dev
```

This will:
1. Start the Vite dev server with hot-reload
2. Compile the Rust backend
3. Launch the Tauri application

### Build for production

```bash
npm run tauri:build
```

This creates optimized binaries in `src-tauri/target/release/`.

## Project Structure

```
ferrite_editor/
├── src/                      # React frontend
│   ├── components/
│   │   ├── ui/              # shadcn UI components
│   │   ├── layout/          # Editor layout (MenuBar, EditorLayout)
│   │   ├── hierarchy/       # Scene hierarchy panel
│   │   ├── inspector/       # Component inspector panel
│   │   └── viewport/        # 3D viewport
│   ├── lib/
│   │   ├── api.ts           # Tauri IPC API wrappers
│   │   ├── types.ts         # TypeScript type definitions
│   │   └── utils.ts         # Utility functions
│   ├── stores/              # Zustand state management
│   ├── App.tsx              # Main app component
│   ├── main.tsx             # React entry point
│   └── index.css            # Global styles + Tailwind
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── commands/        # Tauri IPC commands
│   │   │   ├── scene.rs     # Scene operations
│   │   │   ├── entity.rs    # Entity CRUD
│   │   │   └── component.rs # Component editing
│   │   ├── engine.rs        # Ferrite engine wrapper
│   │   ├── state.rs         # Editor state management
│   │   └── main.rs          # Tauri entry point
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── package.json
├── vite.config.ts           # Vite + React Compiler config
├── tailwind.config.js
└── tsconfig.json
```

## Usage

### Creating a New Scene

1. Click **File > New** or press the "New" button in the menu bar
2. Enter a scene name
3. Start adding entities!

### Managing Entities

- **Create Entity**: Click the `+` button in the Scene Hierarchy panel
- **Select Entity**: Click on an entity in the hierarchy
- **Delete Entity**: Select an entity and click the trash icon
- **Rename Entity**: Double-click an entity name (coming soon)

### Editing Components

1. Select an entity in the Scene Hierarchy
2. View and edit its components in the Inspector panel
3. For Transform component:
   - Edit Position (X, Y, Z)
   - Edit Rotation (Quaternion: X, Y, Z, W)
   - Edit Scale (X, Y, Z)
4. Changes are applied when you blur the input field

### Saving and Loading

- **Save Scene**: Click **File > Save** and choose a location
- **Load Scene**: Click **File > Open** and select a `.ron` file
- Scenes are saved in human-readable RON format for easy version control

## Keyboard Shortcuts (Planned)

- `Ctrl+N` - New Scene
- `Ctrl+O` - Open Scene
- `Ctrl+S` - Save Scene
- `Ctrl+D` - Duplicate Entity
- `Delete` - Delete Selected Entity
- `F` - Focus on Selected Entity (in viewport)
- `W/E/R` - Transform Gizmo Modes (Translate/Rotate/Scale)

## Roadmap

### Phase 1: Foundation ✅
- [x] Tauri + React 19 setup with React Compiler
- [x] Basic editor layout with resizable panels
- [x] Scene hierarchy panel
- [x] Inspector panel with Transform editor
- [x] Menu bar with basic controls
- [x] Zustand state management

### Phase 2: Core Features (In Progress)
- [ ] 3D viewport with wgpu integration
- [ ] Camera controls (WASD, mouse look)
- [ ] Entity selection in viewport
- [ ] Transform gizmos (translate, rotate, scale)
- [ ] File dialogs for save/load
- [ ] More component editors (Mesh, Material, Camera, etc.)

### Phase 3: Advanced Features
- [ ] Prefab system
- [ ] Asset browser
- [ ] Play mode
- [ ] Undo/redo system
- [ ] Drag-and-drop entity reparenting
- [ ] Grid snapping
- [ ] Multiple scene support

### Phase 4: Polish
- [ ] Keyboard shortcuts
- [ ] Context menus
- [ ] Customizable layout
- [ ] Dark/light theme toggle
- [ ] User preferences
- [ ] Documentation and tutorials

## Contributing

This is part of the Ferrite game engine project. Contributions are welcome!

## License

MIT OR Apache-2.0

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- UI components from [shadcn/ui](https://ui.shadcn.com/)
- Powered by [Ferrite Engine](../README.md)

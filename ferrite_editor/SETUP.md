# Ferrite Scene Editor - Setup Guide

## What's Been Created

A complete foundation for a professional scene editor with:

### Backend (Rust + Tauri)
- ✅ Ferrite engine integration
- ✅ Scene management commands (create, load, save)
- ✅ Entity CRUD operations
- ✅ Component management system
- ✅ ECS state management with bevy_ecs

### Frontend (React 19 + TypeScript)
- ✅ React 19 with React Compiler enabled
- ✅ Modern UI with shadcn/ui components
- ✅ Responsive layout with resizable panels
- ✅ Scene hierarchy panel
- ✅ Inspector panel with Transform editor
- ✅ 3D viewport (placeholder for wgpu integration)
- ✅ Zustand state management
- ✅ Tauri IPC API wrappers

## Quick Start

### 1. Install Dependencies

```bash
cd ferrite_editor
npm install
```

### 2. Run the Editor

```bash
npm run tauri:dev
```

This will:
- Install Rust dependencies (first run takes a few minutes)
- Build the Tauri backend
- Start Vite dev server
- Launch the editor window

### 3. Test the Editor

Once launched, try:
1. **Create a Scene**: Click "New" in the menu bar
2. **Add Entities**: Click the `+` button in Scene Hierarchy
3. **Edit Transform**: Select an entity and modify its Position/Rotation/Scale
4. **Save/Load**: Use File menu (requires implementing file dialogs)

## Architecture Overview

```
User Action (React UI)
    ↓
Zustand Store
    ↓
Tauri API Wrapper (lib/api.ts)
    ↓
Tauri IPC (invoke())
    ↓
Rust Command (commands/*.rs)
    ↓
EditorState (ECS World)
    ↓
Ferrite Engine Components
```

## Next Steps for Full Implementation

### Priority 1: File Dialogs
Add file dialog support for saving/loading scenes:

```bash
cargo add tauri-plugin-dialog --features="tauri/dialog-all"
```

Then update `MenuBar.tsx` to use the dialog API.

### Priority 2: 3D Viewport Integration
Integrate wgpu rendering into the viewport:
- Create a window surface in Tauri
- Render Ferrite scene into that surface
- Handle mouse/keyboard input for camera controls

### Priority 3: More Component Editors
Add editors for:
- Mesh (dropdown for primitives)
- Material (color picker, texture selector)
- Camera (projection type, FOV, near/far)
- RigidBody (type dropdown)
- Collider (shape selector, size inputs)
- Audio (file selector, volume slider)

### Priority 4: Additional Features
- Context menus (right-click on entities)
- Drag-and-drop entity reparenting
- Transform gizmos in viewport
- Undo/redo system
- Prefab system
- Asset browser

## File Structure Explanation

### Frontend Key Files

- **`src/App.tsx`**: Main app entry point
- **`src/components/layout/EditorLayout.tsx`**: Main layout with 3 panels
- **`src/components/hierarchy/SceneHierarchy.tsx`**: Left panel (entity tree)
- **`src/components/inspector/Inspector.tsx`**: Right panel (component editor)
- **`src/components/viewport/Viewport3D.tsx`**: Center panel (3D view)
- **`src/stores/editorStore.ts`**: Zustand state management
- **`src/lib/api.ts`**: Tauri command wrappers
- **`src/lib/types.ts`**: TypeScript type definitions

### Backend Key Files

- **`src-tauri/src/main.rs`**: Tauri entry point
- **`src-tauri/src/state.rs`**: Editor state (ECS World wrapper)
- **`src-tauri/src/engine.rs`**: Entity/Component serialization helpers
- **`src-tauri/src/commands/scene.rs`**: Scene operations
- **`src-tauri/src/commands/entity.rs`**: Entity CRUD
- **`src-tauri/src/commands/component.rs`**: Component management

## Troubleshooting

### "Failed to resolve module"
Make sure you ran `npm install` in the `ferrite_editor` directory.

### "Cannot find ferrite crates"
Make sure the `ferrite_editor` is inside the `ferrite` project directory so relative paths work.

### Tauri build errors
Ensure you have Tauri prerequisites installed:
- https://tauri.app/v2/guides/prerequisites/

### Hot reload not working
Restart the dev server: `Ctrl+C` then `npm run tauri:dev`

## Development Tips

### React Compiler
The React Compiler is enabled via Babel plugin. It automatically optimizes:
- Memoization (no need for `useMemo`/`useCallback`)
- Re-render prevention
- Component optimization

Just write normal React code!

### Tailwind + shadcn
All UI components use Tailwind utility classes. Colors are theme-aware:
- Use `bg-background`, `text-foreground` for theme colors
- Use `border-border` for borders
- Use `text-muted-foreground` for secondary text

### State Management
Use Zustand for global state:
```tsx
const { selectedEntityId, setSelectedEntityId } = useEditorStore();
```

Async actions are built into the store:
```tsx
await createEntity('MyEntity');
await refreshHierarchy();
```

## Performance Considerations

- React Compiler handles most optimizations automatically
- Use `ScrollArea` for long lists (hierarchy, components)
- Zustand updates are efficient (only re-renders subscribers)
- Tauri IPC is fast but async - always handle loading states

## Contributing

When adding features:
1. Add Rust commands in `src-tauri/src/commands/`
2. Register commands in `main.rs`
3. Add TypeScript wrappers in `lib/api.ts`
4. Add types in `lib/types.ts`
5. Update store actions in `stores/editorStore.ts`
6. Create UI components in `components/`

## Support

For issues with:
- **Tauri**: https://tauri.app/v2/guides/
- **React**: https://react.dev/
- **shadcn/ui**: https://ui.shadcn.com/
- **Ferrite Engine**: See main repository

Happy editing! 🎮✨

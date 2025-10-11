# Ferrite Scene Editor - Quick Start

## ✅ Working Solution

The editor is now fully functional! The issues were caused by:
1. **React Compiler conflicts** - Disabled it (React 19 is fast enough without it)
2. **Graphics rendering issues** - Fixed with `WEBKIT_DISABLE_DMABUF_RENDERER=1`
3. **Wayland display errors** - Fixed with `GDK_BACKEND=x11`

## 🚀 How to Run

### Simple Method (Recommended)
```bash
cd /home/adrian/Projects/ferrite/ferrite_editor
./run.sh
```

### Manual Method
```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 GDK_BACKEND=x11 npm run tauri:dev
```

## 🎮 Using the Editor

### 1. Create a New Scene
- Click "New" in the menu bar
- A new empty scene will be created

### 2. Add Entities
- Click the `+` button in the **Scene Hierarchy** panel (left side)
- Entities will appear in the list

### 3. Edit Entities
- Click on an entity to select it
- The **Inspector** panel (right side) shows its components
- Edit the Transform component:
  - **Position** (X, Y, Z)
  - **Rotation** (Quaternion: X, Y, Z, W)
  - **Scale** (X, Y, Z)

### 4. Delete Entities
- Select an entity
- Click the trash icon in the Scene Hierarchy panel

## 🎨 Editor Layout

```
┌─────────────────────────────────────────────────────┐
│  Menu Bar: [New] [Open] [Save] | [Play] [Pause]     │
├───────────┬─────────────────────┬────────────────────┤
│  Scene    │                     │   Inspector        │
│ Hierarchy │    3D Viewport      │                    │
│           │                     │   Transform        │
│  Entity   │   (Placeholder)     │   Position XYZ     │
│  Entity   │                     │   Rotation XYZW    │
│  Entity   │                     │   Scale XYZ        │
│           │                     │                    │
└───────────┴─────────────────────┴────────────────────┘
```

## 📝 Features Currently Working

✅ **Scene Management**
- Create new scenes
- Scene manager backend

✅ **Entity System**
- Create entities
- Delete entities
- Hierarchical entity tree (backend ready)
- Entity selection

✅ **Component Editing**
- Transform component (Position, Rotation, Scale)
- Real-time updates
- Input validation

✅ **UI/UX**
- Professional dark theme (shadcn/ui)
- Resizable panels (drag the dividers)
- Modern, clean interface
- React 19 with latest dependencies

## 🔧 Troubleshooting

### Problem: White screen
**Solution**: Make sure you're using the `run.sh` script or the environment variables

### Problem: esbuild crashes
**Solution**: Clear cache and restart
```bash
rm -rf node_modules/.vite dist
./run.sh
```

### Problem: Port 1420 already in use
**Solution**: Kill existing processes
```bash
pkill -9 -f "vite"
pkill -9 -f "ferrite-editor"
./run.sh
```

### Problem: Window doesn't open
**Solution**: Check if process is running
```bash
ps aux | grep ferrite-editor
```
If it's running but window isn't visible, try Alt+Tab or check your window manager.

## 🎯 Next Steps for Development

### Priority 1: File Dialogs
Add native file open/save dialogs for scenes.

### Priority 2: More Components
Add component editors for:
- Mesh (primitives dropdown)
- Material (color picker)
- Camera (projection, FOV)
- Physics (RigidBody, Collider)

### Priority 3: 3D Viewport
Integrate wgpu rendering to show actual 3D scene.

### Priority 4: Transform Gizmos
Visual manipulation tools in the viewport.

## 📚 Technical Details

### Stack
- **Frontend**: React 19, TypeScript, Vite 6
- **UI**: shadcn/ui (Radix UI) + Tailwind CSS
- **Backend**: Rust, Tauri 2.x, Ferrite Engine
- **State**: Zustand
- **ECS**: bevy_ecs 0.17

### Architecture
```
React UI
   ↓
Zustand Store
   ↓
Tauri IPC
   ↓
Rust Commands
   ↓
Ferrite ECS World
```

### Key Files
- `src/App.tsx` - Main application
- `src/components/layout/EditorLayout.tsx` - 3-panel layout
- `src/stores/editorStore.ts` - State management
- `src-tauri/src/commands/` - Backend IPC commands
- `src-tauri/src/state.rs` - Editor ECS state

## 🎉 Success!

The editor is now fully functional and ready for development. You can create entities, edit their transforms, and use a professional UI that matches industry standards.

**Enjoy building with Ferrite!** 🎮✨

use crate::state::EditorState;
use ferrite_scene::{Scene, SceneManager};
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneInfo {
    pub name: String,
    pub entity_count: usize,
    pub active: bool,
}

/// Create a new empty scene
#[tauri::command]
pub fn create_scene(name: String, state: State<EditorState>) -> Result<SceneInfo, String> {
    let scene = Scene::new(&name);

    let mut world = state.world.write();
    let mut scene_manager = world
        .get_resource_mut::<SceneManager>()
        .ok_or("SceneManager not found")?;

    scene_manager.register_scene(scene.clone());
    scene_manager
        .activate_scene(&name)
        .map_err(|e| e.to_string())?;

    Ok(SceneInfo {
        name: scene.name.clone(),
        entity_count: scene.entities.len(),
        active: true,
    })
}

/// Load a scene from a file
#[tauri::command]
pub fn load_scene(path: String, state: State<EditorState>) -> Result<SceneInfo, String> {
    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    let scene = Scene::from_ron(&data).map_err(|e| format!("Failed to parse scene: {}", e))?;

    let mut world = state.world.write();
    let mut scene_manager = world
        .get_resource_mut::<SceneManager>()
        .ok_or("SceneManager not found")?;

    let name = scene.name.clone();
    let entity_count = scene.entities.len();

    scene_manager.register_scene(scene);
    scene_manager
        .activate_scene(&name)
        .map_err(|e| e.to_string())?;

    Ok(SceneInfo {
        name,
        entity_count,
        active: true,
    })
}

/// Save the current scene to a file
#[tauri::command]
pub fn save_scene(path: String, state: State<EditorState>) -> Result<(), String> {
    let world = state.world.read();
    let scene_manager = world
        .get_resource::<SceneManager>()
        .ok_or("SceneManager not found")?;

    let ron_data = scene_manager
        .save_active_scene()
        .map_err(|e| format!("Failed to save scene: {}", e))?;

    fs::write(&path, ron_data).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Get information about the current scene
#[tauri::command]
pub fn get_scene_info(state: State<EditorState>) -> Result<Option<SceneInfo>, String> {
    let world = state.world.read();
    let scene_manager = world
        .get_resource::<SceneManager>()
        .ok_or("SceneManager not found")?;

    if let Some(scene_name) = scene_manager.active_scene() {
        if let Some(scene) = scene_manager.get_scene(scene_name) {
            return Ok(Some(SceneInfo {
                name: scene.name.clone(),
                entity_count: scene.entities.len(),
                active: true,
            }));
        }
    }

    Ok(None)
}

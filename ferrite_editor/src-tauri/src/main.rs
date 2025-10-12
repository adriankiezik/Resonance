// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod engine;
mod state;
mod viewport;
mod viewport_window;

use commands::viewport::ViewportState;
use state::EditorState;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(EditorState::new())
        .manage(ViewportState::new())
        .invoke_handler(tauri::generate_handler![
            commands::scene::create_scene,
            commands::scene::load_scene,
            commands::scene::save_scene,
            commands::scene::get_scene_info,
            commands::entity::create_entity,
            commands::entity::delete_entity,
            commands::entity::rename_entity,
            commands::entity::get_entity_hierarchy,
            commands::entity::set_entity_parent,
            commands::entity::get_entity,
            commands::component::add_component,
            commands::component::update_component,
            commands::component::remove_component,
            commands::component::get_entity_components,
            commands::viewport::init_viewport,
            commands::viewport::resize_viewport,
            commands::viewport::render_viewport,
            commands::viewport::orbit_camera,
            commands::viewport::pan_camera,
            commands::viewport::zoom_camera,
            commands::viewport::get_camera_info,
            commands::viewport::open_viewport_window,
            commands::viewport::get_viewport_frame,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

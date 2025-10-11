//! Scene management system for loading, unloading, and transitioning between scenes.

use crate::Scene;
use bevy_ecs::prelude::*;
use ferrite_core::FerriteError;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during scene operations
#[derive(Debug, Error)]
pub enum SceneError {
    #[error("Scene '{0}' not found")]
    SceneNotFound(String),
    #[error("Failed to load scene: {0}")]
    LoadError(String),
    #[error("Failed to save scene: {0}")]
    SaveError(String),
    #[error("Core error: {0}")]
    CoreError(#[from] FerriteError),
}

/// Scene transition state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneTransition {
    /// No transition in progress
    None,
    /// Loading a new scene
    Loading,
    /// Unloading current scene
    Unloading,
    /// Transitioning between scenes
    Transitioning,
}

/// Manages scene loading, unloading, and transitions
#[derive(Debug, Resource)]
pub struct SceneManager {
    /// Currently active scene
    active_scene: Option<String>,
    /// Loaded scenes (name -> scene data)
    loaded_scenes: HashMap<String, Scene>,
    /// Entities spawned from current scene
    scene_entities: Vec<Entity>,
    /// Current transition state
    transition_state: SceneTransition,
    /// Scene to transition to (if any)
    pending_scene: Option<String>,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneManager {
    /// Create a new scene manager
    pub fn new() -> Self {
        Self {
            active_scene: None,
            loaded_scenes: HashMap::new(),
            scene_entities: Vec::new(),
            transition_state: SceneTransition::None,
            pending_scene: None,
        }
    }

    /// Get the currently active scene name
    pub fn active_scene(&self) -> Option<&str> {
        self.active_scene.as_deref()
    }

    /// Get the current transition state
    pub fn transition_state(&self) -> SceneTransition {
        self.transition_state
    }

    /// Check if a scene is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded_scenes.contains_key(name)
    }

    /// Load a scene from RON data
    pub fn load_scene_from_ron(&mut self, name: impl Into<String>, ron_data: &str) -> Result<(), SceneError> {
        let scene = Scene::from_ron(ron_data)?;
        let name = name.into();
        self.loaded_scenes.insert(name, scene);
        Ok(())
    }

    /// Register a scene
    pub fn register_scene(&mut self, scene: Scene) {
        let name = scene.name.clone();
        self.loaded_scenes.insert(name, scene);
    }

    /// Get a loaded scene
    pub fn get_scene(&self, name: &str) -> Option<&Scene> {
        self.loaded_scenes.get(name)
    }

    /// Activate a loaded scene
    /// Note: This only marks the scene as active. Actual entity spawning
    /// should be handled by systems that process the scene data
    pub fn activate_scene(&mut self, name: impl Into<String>) -> Result<(), SceneError> {
        let name = name.into();

        if !self.loaded_scenes.contains_key(&name) {
            return Err(SceneError::SceneNotFound(name));
        }

        self.active_scene = Some(name);
        self.transition_state = SceneTransition::None;
        log::info!("Activated scene: {}", self.active_scene.as_ref().unwrap());
        Ok(())
    }

    /// Request a scene transition
    pub fn transition_to(&mut self, scene_name: impl Into<String>) -> Result<(), SceneError> {
        let scene_name = scene_name.into();

        if !self.loaded_scenes.contains_key(&scene_name) {
            return Err(SceneError::SceneNotFound(scene_name));
        }

        self.pending_scene = Some(scene_name.clone());
        self.transition_state = SceneTransition::Transitioning;
        log::info!("Transitioning to scene: {}", scene_name);
        Ok(())
    }

    /// Mark entities as spawned from the current scene
    pub fn register_scene_entities(&mut self, entities: Vec<Entity>) {
        self.scene_entities = entities;
    }

    /// Get entities spawned from current scene
    pub fn scene_entities(&self) -> &[Entity] {
        &self.scene_entities
    }

    /// Clear current scene entities
    pub fn clear_scene_entities(&mut self) {
        self.scene_entities.clear();
    }

    /// Complete a pending scene transition
    pub fn complete_transition(&mut self) -> Result<(), SceneError> {
        if let Some(scene_name) = self.pending_scene.take() {
            self.activate_scene(scene_name)?;
        }
        self.transition_state = SceneTransition::None;
        Ok(())
    }

    /// Unload current scene
    pub fn unload_current_scene(&mut self) {
        self.active_scene = None;
        self.scene_entities.clear();
        self.transition_state = SceneTransition::None;
        log::info!("Unloaded current scene");
    }

    /// Remove a scene from memory
    pub fn remove_scene(&mut self, name: &str) -> Option<Scene> {
        self.loaded_scenes.remove(name)
    }

    /// Get all loaded scene names
    pub fn loaded_scene_names(&self) -> Vec<&String> {
        self.loaded_scenes.keys().collect()
    }

    /// Save active scene to RON format
    pub fn save_active_scene(&self) -> Result<String, SceneError> {
        if let Some(scene_name) = &self.active_scene {
            if let Some(scene) = self.loaded_scenes.get(scene_name) {
                return Ok(scene.to_ron()?);
            }
        }
        Err(SceneError::SceneNotFound("No active scene".to_string()))
    }
}

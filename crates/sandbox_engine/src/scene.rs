// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scene management for saving and loading entity hierarchies.
//!
//! This module provides scene serialization using Bevy's `DynamicScene` system
//! with RON format output.

use bevy::prelude::*;
use bevy::scene::serde::SceneDeserializer;
use bevy::scene::DynamicSceneBuilder;
use serde::de::DeserializeSeed;
use std::path::PathBuf;

/// Plugin that sets up scene management.
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneManager>();
    }
}

/// Manages the current scene state and provides save/load operations.
#[derive(Resource, Default)]
pub struct SceneManager {
    /// Path to the currently loaded scene file, if any.
    pub current_scene_path: Option<PathBuf>,
    /// Whether the scene has unsaved changes.
    pub dirty: bool,
}

impl SceneManager {
    /// Marks the scene as having unsaved changes.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clears the dirty flag (called after save).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

/// Result type for scene operations.
pub type SceneResult<T> = Result<T, SceneError>;

/// Errors that can occur during scene operations.
#[derive(Debug)]
pub enum SceneError {
    /// Failed to serialize scene to RON.
    Serialization(String),
    /// Failed to write scene file.
    Io(std::io::Error),
    /// Failed to read scene file.
    FileRead(std::io::Error),
    /// Failed to deserialize scene from RON.
    Deserialization(String),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneError::Serialization(e) => write!(f, "Failed to serialize scene: {}", e),
            SceneError::Io(e) => write!(f, "Failed to write scene file: {}", e),
            SceneError::FileRead(e) => write!(f, "Failed to read scene file: {}", e),
            SceneError::Deserialization(e) => write!(f, "Failed to deserialize scene: {}", e),
        }
    }
}

impl std::error::Error for SceneError {}

/// Checks if an entity should be included in scene serialization.
///
/// Excludes cameras and other runtime-only entities.
pub fn should_serialize_entity(entity: &EntityRef) -> bool {
    // Exclude cameras - they're runtime only
    if entity.contains::<Camera2d>() {
        return false;
    }

    // Exclude entities without transforms (usually internal bevy entities)
    if !entity.contains::<Transform>() {
        return false;
    }

    true
}

/// Saves the current world state to a scene file.
///
/// # Arguments
/// * `world` - The world to save
/// * `path` - Path to save the scene file
///
/// # Returns
/// `Ok(())` on success, or a `SceneError` on failure.
pub fn save_scene(world: &mut World, path: &PathBuf) -> SceneResult<()> {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    // Collect entities to save
    let entities_to_save: Vec<Entity> = world
        .iter_entities()
        .filter(|e| should_serialize_entity(e))
        .map(|e| e.id())
        .collect();

    // Build the dynamic scene
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(entities_to_save.into_iter())
        .build();

    // Serialize to RON
    let serialized = scene
        .serialize(&type_registry)
        .map_err(|e| SceneError::Serialization(e.to_string()))?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(SceneError::Io)?;
    }

    // Write to file
    std::fs::write(path, serialized).map_err(SceneError::Io)?;

    // Update scene manager
    if let Some(mut manager) = world.get_resource_mut::<SceneManager>() {
        manager.current_scene_path = Some(path.clone());
        manager.mark_clean();
    }

    Ok(())
}

/// Clears all serializable entities from the world.
///
/// This removes entities that would be saved in a scene, leaving cameras
/// and other runtime entities intact.
pub fn clear_scene_entities(world: &mut World) {
    let entities_to_despawn: Vec<Entity> = world
        .iter_entities()
        .filter(|e| should_serialize_entity(e))
        .map(|e| e.id())
        .collect();

    for entity in entities_to_despawn {
        // Check if entity still exists (may have been despawned as a child of another entity)
        if world.get_entity(entity).is_ok() {
            world.despawn(entity);
        }
    }
}

/// Loads a scene from a file, replacing the current scene.
///
/// This clears existing scene entities before loading the new scene.
///
/// # Arguments
/// * `world` - The world to load into
/// * `path` - Path to the scene file
///
/// # Returns
/// `Ok(())` on success, or a `SceneError` on failure.
pub fn load_scene(world: &mut World, path: &PathBuf) -> SceneResult<()> {
    // Read the scene file
    let scene_data = std::fs::read_to_string(path).map_err(SceneError::FileRead)?;

    // Get type registry for deserialization
    let type_registry = world.resource::<AppTypeRegistry>().clone();

    // Deserialize the scene using RON
    let mut deserializer = bevy::scene::ron::de::Deserializer::from_str(&scene_data)
        .map_err(|e| SceneError::Deserialization(e.to_string()))?;

    let scene_deserializer = SceneDeserializer {
        type_registry: &type_registry.read(),
    };

    let scene: DynamicScene = scene_deserializer
        .deserialize(&mut deserializer)
        .map_err(|e| SceneError::Deserialization(e.to_string()))?;

    // Clear existing scene entities
    clear_scene_entities(world);

    // Spawn the scene entities
    scene
        .write_to_world(world, &mut bevy::ecs::entity::EntityHashMap::default())
        .map_err(|e| SceneError::Deserialization(format!("{:?}", e)))?;

    // Update scene manager
    if let Some(mut manager) = world.get_resource_mut::<SceneManager>() {
        manager.current_scene_path = Some(path.clone());
        manager.mark_clean();
    }

    Ok(())
}

/// Spawns a prefab into the current scene without clearing existing entities.
///
/// # Arguments
/// * `world` - The world to spawn into
/// * `path` - Path to the prefab file
///
/// # Returns
/// `Ok(())` on success, or a `SceneError` on failure.
pub fn spawn_prefab(world: &mut World, path: &PathBuf) -> SceneResult<()> {
    // Read the prefab file
    let prefab_data = std::fs::read_to_string(path).map_err(SceneError::FileRead)?;

    // Get type registry for deserialization
    let type_registry = world.resource::<AppTypeRegistry>().clone();

    // Deserialize the prefab using RON
    let mut deserializer = bevy::scene::ron::de::Deserializer::from_str(&prefab_data)
        .map_err(|e| SceneError::Deserialization(e.to_string()))?;

    let scene_deserializer = SceneDeserializer {
        type_registry: &type_registry.read(),
    };

    let scene: DynamicScene = scene_deserializer
        .deserialize(&mut deserializer)
        .map_err(|e| SceneError::Deserialization(e.to_string()))?;

    // Spawn the prefab entities (don't clear existing)
    scene
        .write_to_world(world, &mut bevy::ecs::entity::EntityHashMap::default())
        .map_err(|e| SceneError::Deserialization(format!("{:?}", e)))?;

    // Mark scene as dirty since we added entities
    if let Some(mut manager) = world.get_resource_mut::<SceneManager>() {
        manager.mark_dirty();
    }

    Ok(())
}

/// Creates a new empty scene, clearing all existing entities.
pub fn new_scene(world: &mut World) {
    clear_scene_entities(world);

    if let Some(mut manager) = world.get_resource_mut::<SceneManager>() {
        manager.current_scene_path = None;
        manager.mark_clean();
    }
}

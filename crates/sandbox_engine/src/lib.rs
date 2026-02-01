// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sandbox Engine - A Bevy-based 2D game engine
//!
//! This crate provides common 2D game setup and utilities built on top of Bevy.

pub use bevy;

pub mod editor_state;
pub mod scene;

pub mod prelude {
    pub use crate::editor_state::{
        EditorPlayState, EditorSnapshot, EditorStatePlugin, EntityState, GameplaySystemSet,
    };
    pub use crate::scene::{
        clear_scene_entities, load_scene, new_scene, save_scene, spawn_prefab, SceneError,
        SceneManager, ScenePlugin, SceneResult,
    };
    pub use crate::SandboxPlugin;
    pub use bevy::prelude::*;
}

use bevy::prelude::*;

/// Main plugin that bundles common 2D game setup.
///
/// This plugin adds:
/// - Default Bevy plugins
/// - A 2D camera
pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sandbox Engine".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

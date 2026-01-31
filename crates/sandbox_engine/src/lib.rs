//! Sandbox Engine - A Bevy-based 2D game engine
//!
//! This crate provides common 2D game setup and utilities built on top of Bevy.

pub use bevy;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use crate::SandboxPlugin;
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

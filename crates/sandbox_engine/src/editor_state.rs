// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Editor state machine for play/pause/stop functionality.
//!
//! This module provides the state machine that controls editor execution modes
//! and the system set for gating gameplay systems.

use bevy::prelude::*;

/// Editor execution state for play/pause/stop controls.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditorPlayState {
    /// Editor is stopped - entities can be selected and manipulated.
    #[default]
    Stopped,
    /// Game is running - gameplay systems are active.
    Playing,
    /// Game is paused - gameplay systems are suspended.
    Paused,
}

/// System set for gameplay systems that should only run when playing.
///
/// Configure your gameplay systems to run in this set:
/// ```ignore
/// app.add_systems(Update, my_system.in_set(GameplaySystemSet));
/// ```
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameplaySystemSet;

/// Snapshot of entity states for restoring after stopping playback.
#[derive(Resource, Default)]
pub struct EditorSnapshot {
    /// Stored entity transforms and velocities for restoration.
    pub entity_states: Vec<EntityState>,
}

/// State of a single entity for snapshot/restore.
#[derive(Clone)]
pub struct EntityState {
    /// The entity this state belongs to.
    pub entity: Entity,
    /// The entity's transform at snapshot time.
    pub transform: Transform,
    /// Optional velocity (as Vec2) if the entity had one.
    pub velocity: Option<Vec2>,
}

/// Plugin that sets up editor state management.
pub struct EditorStatePlugin;

impl Plugin for EditorStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EditorPlayState>()
            .init_resource::<EditorSnapshot>()
            .configure_sets(
                Update,
                GameplaySystemSet.run_if(in_state(EditorPlayState::Playing)),
            )
            .add_systems(OnEnter(EditorPlayState::Playing), capture_snapshot)
            .add_systems(OnEnter(EditorPlayState::Stopped), restore_snapshot);
    }
}

/// Captures entity states when entering play mode.
fn capture_snapshot(mut snapshot: ResMut<EditorSnapshot>, query: Query<(Entity, &Transform)>) {
    snapshot.entity_states.clear();
    for (entity, transform) in &query {
        snapshot.entity_states.push(EntityState {
            entity,
            transform: *transform,
            velocity: None, // Will be filled by game-specific systems if needed
        });
    }
}

/// Restores entity states when returning to stopped mode.
fn restore_snapshot(snapshot: Res<EditorSnapshot>, mut query: Query<&mut Transform>) {
    for state in &snapshot.entity_states {
        if let Ok(mut transform) = query.get_mut(state.entity) {
            *transform = state.transform;
        }
    }
}

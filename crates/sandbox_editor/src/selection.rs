// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Entity selection system for the editor.

use bevy::prelude::*;

/// Resource tracking the currently selected entity in the editor.
#[derive(Resource, Default)]
pub struct EditorSelection {
    /// The currently selected entity, if any.
    pub selected_entity: Option<Entity>,
}

/// Marker component added to selected entities.
#[derive(Component)]
pub struct EditorSelected;

/// Plugin that sets up the selection system.
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorSelection>()
            .add_systems(PostUpdate, sync_selection_marker);
    }
}

/// Syncs the EditorSelected marker component with the EditorSelection resource.
fn sync_selection_marker(
    selection: Res<EditorSelection>,
    mut commands: Commands,
    selected_query: Query<Entity, With<EditorSelected>>,
) {
    if !selection.is_changed() {
        return;
    }

    // Remove marker from previously selected entities
    for entity in &selected_query {
        if Some(entity) != selection.selected_entity {
            commands.entity(entity).remove::<EditorSelected>();
        }
    }

    // Add marker to newly selected entity
    if let Some(entity) = selection.selected_entity {
        if !selected_query.contains(entity) {
            commands.entity(entity).insert(EditorSelected);
        }
    }
}

// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scene hierarchy panel for the editor.

use bevy::prelude::*;
use bevy_egui::egui;
use sandbox_engine::scene::SceneManager;

use crate::selection::EditorSelection;

/// Counter for generating unique entity names.
#[derive(Resource, Default)]
pub struct EntityCounter(pub u32);

/// Displays the scene hierarchy panel.
pub fn hierarchy_panel(ui: &mut egui::Ui, world: &mut World) {
    ui.heading("Hierarchy");

    // Add Entity button
    ui.horizontal(|ui| {
        if ui.button("+ Add Entity").clicked() {
            add_new_entity(world);
        }
        if ui.button("Delete").clicked() {
            delete_selected_entity(world);
        }
    });

    ui.separator();

    // Get entities without a parent (root entities)
    let mut root_entities: Vec<Entity> = Vec::new();
    let mut entity_data: Vec<(Entity, Option<String>, bool)> = Vec::new();

    // First pass: collect entity data
    {
        let mut query = world.query::<(Entity, Option<&Name>, Option<&Children>)>();
        for (entity, name, children) in query.iter(world) {
            let name_str = name.map(|n| n.to_string());
            let has_children = children.is_some_and(|c| !c.is_empty());
            entity_data.push((entity, name_str, has_children));
        }

        // Filter to root entities (no Parent component)
        let mut parent_query = world.query::<&Parent>();
        for (entity, name_str, has_children) in &entity_data {
            if parent_query.get(world, *entity).is_err() {
                root_entities.push(*entity);
            }
            let _ = (name_str, has_children); // Suppress unused warning
        }
    }

    // Sort by entity index for consistent ordering
    root_entities.sort_by_key(|e| e.index());

    let selected_entity = world.resource::<EditorSelection>().selected_entity;

    // Display hierarchy
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for entity in root_entities {
                display_entity_tree(ui, world, entity, selected_entity, 0);
            }
        });
}

/// Recursively displays an entity and its children in the hierarchy.
fn display_entity_tree(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    selected_entity: Option<Entity>,
    depth: usize,
) {
    let indent = depth as f32 * 16.0;

    // Get entity info
    let (display_name, children) = {
        let name = world.get::<Name>(entity).map(|n| n.to_string());
        let children: Vec<Entity> = world
            .get::<Children>(entity)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();

        let display_name = name.unwrap_or_else(|| generate_entity_name(world, entity));
        (display_name, children)
    };

    let is_selected = selected_entity == Some(entity);
    let has_children = !children.is_empty();

    ui.horizontal(|ui| {
        ui.add_space(indent);

        // Expand/collapse indicator (placeholder - always expanded for now)
        if has_children {
            ui.label("▼");
        } else {
            ui.add_space(12.0);
        }

        // Entity button
        let response = ui.selectable_label(is_selected, &display_name);
        if response.clicked() {
            world.resource_mut::<EditorSelection>().selected_entity = Some(entity);
        }
    });

    // Display children
    for child in children {
        display_entity_tree(ui, world, child, selected_entity, depth + 1);
    }
}

/// Generates a display name for an entity based on its components.
fn generate_entity_name(world: &World, entity: Entity) -> String {
    // Check for common marker components to generate a meaningful name
    let components: Vec<_> = world.inspect_entity(entity).collect();

    // Look for component names that might be useful identifiers
    for component_info in &components {
        let name = component_info.name();
        // Use the first "interesting" component as the name
        // Skip common components that don't identify the entity well
        if !matches!(
            name,
            "bevy_transform::components::transform::Transform"
                | "bevy_transform::components::global_transform::GlobalTransform"
                | "bevy_sprite::sprite::Sprite"
                | "bevy_render::view::visibility::Visibility"
                | "bevy_render::view::visibility::InheritedVisibility"
                | "bevy_render::view::visibility::ViewVisibility"
        ) {
            // Extract just the type name from the full path
            if let Some(short_name) = name.rsplit("::").next() {
                return format!("{} ({:?})", short_name, entity);
            }
        }
    }

    format!("Entity ({:?})", entity)
}

/// Deletes the currently selected entity.
fn delete_selected_entity(world: &mut World) {
    let selected = world.resource::<EditorSelection>().selected_entity;

    if let Some(entity) = selected {
        // Check entity exists before despawning
        if world.get_entity(entity).is_ok() {
            world.despawn(entity);

            // Clear selection
            world.resource_mut::<EditorSelection>().selected_entity = None;

            // Mark scene as dirty
            if let Some(mut manager) = world.get_resource_mut::<SceneManager>() {
                manager.mark_dirty();
            }
        }
    }
}

/// Spawns a new entity with default components.
fn add_new_entity(world: &mut World) {
    // Initialize counter if not present
    if !world.contains_resource::<EntityCounter>() {
        world.init_resource::<EntityCounter>();
    }

    // Get next entity number
    let entity_num = {
        let mut counter = world.resource_mut::<EntityCounter>();
        counter.0 += 1;
        counter.0
    };

    // Spawn the entity
    let entity = world
        .spawn((
            Name::new(format!("Entity {}", entity_num)),
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::default(),
        ))
        .id();

    // Select the new entity
    world.resource_mut::<EditorSelection>().selected_entity = Some(entity);

    // Mark scene as dirty
    if let Some(mut manager) = world.get_resource_mut::<SceneManager>() {
        manager.mark_dirty();
    }
}

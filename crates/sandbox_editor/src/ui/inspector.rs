// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Entity inspector panel for the editor.

use bevy::prelude::*;
use bevy_egui::egui;
use sandbox_engine::assets::{AssetPath, SpriteAnimation};

use crate::assets::AssetBrowser;
use crate::selection::EditorSelection;

/// Displays the entity inspector panel.
pub fn inspector_panel(ui: &mut egui::Ui, world: &mut World) {
    ui.heading("Inspector");
    ui.separator();

    let selected_entity = world.resource::<EditorSelection>().selected_entity;

    let Some(entity) = selected_entity else {
        ui.label("Select an entity to inspect its components.");
        return;
    };

    // Check if entity still exists
    if world.get_entity(entity).is_err() {
        ui.label("Selected entity no longer exists.");
        world.resource_mut::<EditorSelection>().selected_entity = None;
        return;
    }

    // Display entity ID
    ui.label(format!("Entity: {:?}", entity));
    ui.separator();

    // Get component names and IDs for display
    let component_data: Vec<_> = world
        .inspect_entity(entity)
        .map(|info| (info.name().to_string(), info.id(), info.type_id()))
        .collect();

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (name, component_id, type_id) in &component_data {
                display_component(ui, world, entity, name, *component_id, *type_id);
            }
        });
}

/// Displays a single component in the inspector.
fn display_component(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    component_name: &str,
    component_id: bevy::ecs::component::ComponentId,
    type_id: Option<std::any::TypeId>,
) {
    // Extract short name from full path
    let short_name = component_name.rsplit("::").next().unwrap_or(component_name);

    egui::CollapsingHeader::new(short_name)
        .default_open(is_common_component(component_name))
        .show(ui, |ui| {
            // Try to display editable fields based on component type
            if component_name.contains("Transform") && !component_name.contains("GlobalTransform") {
                display_transform(ui, world, entity);
            } else if component_name.contains("SpriteAnimation") {
                display_sprite_animation(ui, world, entity);
            } else if component_name.contains("Sprite") {
                display_sprite(ui, world, entity);
            } else if component_name.contains("AssetPath") {
                display_asset_path(ui, world, entity);
            } else if component_name.contains("Velocity") {
                display_velocity(ui, world, entity, component_id, type_id);
            } else if component_name.contains("Camera2d") {
                ui.label("2D Camera");
            } else if component_name.contains("Name") && !component_name.contains("TypePath") {
                display_name_component(ui, world, entity);
            } else {
                // Read-only fallback
                ui.label(format!("(type: {})", short_name));
            }
        });
}

/// Checks if a component should be expanded by default.
fn is_common_component(name: &str) -> bool {
    (name.contains("Transform") && !name.contains("GlobalTransform"))
        || (name.contains("Sprite") && !name.contains("SpriteAnimation"))
        || name.contains("Velocity")
        || (name.contains("Name") && !name.contains("TypePath"))
        || name.contains("AssetPath")
        || name.contains("SpriteAnimation")
}

/// Displays and edits Transform component.
fn display_transform(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    let Some(mut transform) = world.get_mut::<Transform>(entity) else {
        ui.label("Transform not accessible");
        return;
    };

    ui.horizontal(|ui| {
        ui.label("Position:");
    });
    ui.horizontal(|ui| {
        ui.label("X:");
        let mut x = transform.translation.x;
        if ui.add(egui::DragValue::new(&mut x).speed(1.0)).changed() {
            transform.translation.x = x;
        }
        ui.label("Y:");
        let mut y = transform.translation.y;
        if ui.add(egui::DragValue::new(&mut y).speed(1.0)).changed() {
            transform.translation.y = y;
        }
        ui.label("Z:");
        let mut z = transform.translation.z;
        if ui.add(egui::DragValue::new(&mut z).speed(0.1)).changed() {
            transform.translation.z = z;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Rotation (deg):");
        // Convert quaternion to euler angle (Z rotation for 2D)
        let (_, _, z_rad) = transform.rotation.to_euler(EulerRot::XYZ);
        let mut z_deg = z_rad.to_degrees();
        if ui
            .add(egui::DragValue::new(&mut z_deg).speed(1.0))
            .changed()
        {
            transform.rotation = Quat::from_rotation_z(z_deg.to_radians());
        }
    });

    ui.horizontal(|ui| {
        ui.label("Scale:");
    });
    ui.horizontal(|ui| {
        ui.label("X:");
        let mut sx = transform.scale.x;
        if ui.add(egui::DragValue::new(&mut sx).speed(0.01)).changed() {
            transform.scale.x = sx;
        }
        ui.label("Y:");
        let mut sy = transform.scale.y;
        if ui.add(egui::DragValue::new(&mut sy).speed(0.01)).changed() {
            transform.scale.y = sy;
        }
    });
}

/// Displays and edits Sprite component.
fn display_sprite(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    let Some(mut sprite) = world.get_mut::<Sprite>(entity) else {
        ui.label("Sprite not accessible");
        return;
    };

    // Color editor
    ui.horizontal(|ui| {
        ui.label("Color:");
        let srgba = sprite.color.to_srgba();
        let mut color = [srgba.red, srgba.green, srgba.blue, srgba.alpha];
        if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
            sprite.color = Color::srgba(color[0], color[1], color[2], color[3]);
        }
    });

    // Custom size editor
    ui.horizontal(|ui| {
        ui.label("Size:");
        if let Some(size) = sprite.custom_size.as_mut() {
            ui.label("W:");
            ui.add(egui::DragValue::new(&mut size.x).speed(1.0));
            ui.label("H:");
            ui.add(egui::DragValue::new(&mut size.y).speed(1.0));
        } else {
            ui.label("(default)");
        }
    });
}

/// Displays velocity component (generic Vec2 wrapper).
fn display_velocity(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    component_id: bevy::ecs::component::ComponentId,
    type_id: Option<std::any::TypeId>,
) {
    // Try to get velocity via reflection
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    if let Some(type_id) = type_id {
        if let Some(registration) = type_registry.get(type_id) {
            if let Some(reflect_from_ptr) = registration.data::<bevy::reflect::ReflectFromPtr>() {
                // Get the component data pointer
                if let Some(component_ptr) = world.get_by_id(entity, component_id) {
                    // SAFETY: We're getting the reflect from a valid component pointer
                    let reflect = unsafe { reflect_from_ptr.as_reflect(component_ptr) };

                    // Try to display as tuple struct with Vec2
                    if let Ok(tuple_struct) = reflect.reflect_ref().as_tuple_struct() {
                        if let Some(field) = tuple_struct.field(0) {
                            if let Some(vec2) = field.try_downcast_ref::<Vec2>() {
                                ui.label(format!("Velocity: ({:.1}, {:.1})", vec2.x, vec2.y));
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    ui.label("Velocity: (read-only)");
}

/// Displays Name component.
fn display_name_component(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    let Some(name) = world.get::<Name>(entity) else {
        return;
    };

    ui.label(format!("Name: {}", name.as_str()));
}

/// Displays and edits AssetPath component.
fn display_asset_path(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // Get current path value
    let current_path = world
        .get::<AssetPath>(entity)
        .map(|ap| ap.path.clone())
        .unwrap_or_default();

    ui.horizontal(|ui| {
        ui.label("Path:");
    });

    // Text edit for path
    let mut path = current_path.clone();
    let response = ui.text_edit_singleline(&mut path);
    if response.changed() {
        if let Some(mut asset_path) = world.get_mut::<AssetPath>(entity) {
            asset_path.path = path.clone();
        }
    }

    // Preview of the texture
    if !current_path.is_empty() {
        ui.separator();
        ui.label("Preview:");

        // Load and display the texture preview
        let handle: Handle<Image> = world.resource::<AssetServer>().load(&current_path);

        // Store handle to keep it alive
        world
            .resource_mut::<AssetBrowser>()
            .preview_handles
            .insert(format!("inspector:{}", current_path), handle.clone());

        let images = world.resource::<Assets<Image>>();
        if let Some(image) = images.get(&handle) {
            let (width, height) = (image.width(), image.height());
            ui.label(format!("Size: {}x{}", width, height));

            // Calculate scaled size (max 128x128 for inspector preview)
            let max_size = 128.0;
            let scale = (max_size / width as f32)
                .min(max_size / height as f32)
                .min(1.0);
            let display_size = egui::vec2(width as f32 * scale, height as f32 * scale);

            // Register texture with egui and display
            let texture_id = {
                let mut egui_user_textures = world.resource_mut::<bevy_egui::EguiUserTextures>();
                egui_user_textures.add_image(handle.clone())
            };

            ui.image(egui::load::SizedTexture::new(texture_id, display_size));
        } else {
            ui.spinner();
            ui.label("Loading...");
        }
    }

    // "Browse..." button for file selection
    if ui.button("Browse...").clicked() {
        // Set the asset browser to show the current selection
        // (Full file dialog integration would require native dialog support)
        ui.label("Use Asset Browser panel to select files");
    }
}

/// Displays and edits SpriteAnimation component.
fn display_sprite_animation(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // Get animation data (clone to avoid borrow issues)
    let animation_data = world.get::<SpriteAnimation>(entity).map(|anim| {
        (
            anim.playing,
            anim.looping,
            anim.current_frame,
            anim.frames.len(),
            anim.timer,
        )
    });

    let Some((playing, looping, current_frame, frame_count, timer)) = animation_data else {
        ui.label("Animation not accessible");
        return;
    };

    // Playback controls
    ui.horizontal(|ui| {
        ui.label("Playback:");

        if playing {
            if ui.button("⏸ Pause").clicked() {
                if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                    anim.stop();
                }
            }
        } else if ui.button("▶ Play").clicked() {
            if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                anim.play();
            }
        }

        if ui.button("⏹ Reset").clicked() {
            if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                anim.reset();
            }
        }
    });

    // Loop toggle
    ui.horizontal(|ui| {
        ui.label("Loop:");
        let mut loop_val = looping;
        if ui.checkbox(&mut loop_val, "").changed() {
            if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                anim.looping = loop_val;
            }
        }
    });

    // Frame info
    ui.separator();
    ui.label(format!(
        "Frame: {} / {} (timer: {:.2}s)",
        current_frame + 1,
        frame_count,
        timer
    ));

    // Frame list
    if frame_count > 0 {
        ui.separator();
        ui.label("Frames:");

        // Clone frames to display
        let frames = world
            .get::<SpriteAnimation>(entity)
            .map(|anim| anim.frames.clone())
            .unwrap_or_default();

        for (i, frame) in frames.iter().enumerate() {
            let is_current = i == current_frame;
            let prefix = if is_current { "▶ " } else { "  " };

            ui.horizontal(|ui| {
                ui.label(format!(
                    "{}Frame {}: rect({:.0},{:.0},{:.0},{:.0}) {:.2}s",
                    prefix,
                    i + 1,
                    frame.rect.min.x,
                    frame.rect.min.y,
                    frame.rect.max.x,
                    frame.rect.max.y,
                    frame.duration
                ));
            });
        }
    } else {
        ui.label("No frames defined");
        ui.label("Use Animation Editor to add frames");
    }
}

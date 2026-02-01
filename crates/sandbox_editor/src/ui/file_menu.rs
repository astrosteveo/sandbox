// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! File menu for scene management operations.

use bevy::prelude::*;
use bevy_egui::egui;
use sandbox_engine::scene::{load_scene, new_scene, save_scene, spawn_prefab, SceneManager};

use super::AnimationEditorState;

/// State for tracking pending file operations.
#[derive(Resource, Default)]
pub struct FileMenuState {
    /// Error message to display, if any.
    pub error_message: Option<String>,
    /// Success message to display, if any.
    pub success_message: Option<String>,
}

/// Renders the menu bar with File menu.
pub fn menu_bar(ctx: &egui::Context, world: &mut World) {
    // Handle keyboard shortcuts
    handle_keyboard_shortcuts(ctx, world);

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            file_menu(ui, world);
            window_menu(ui, world);
        });
    });
}

/// Handles keyboard shortcuts for file operations.
fn handle_keyboard_shortcuts(ctx: &egui::Context, world: &mut World) {
    // Only process shortcuts if no text input is focused
    if ctx.wants_keyboard_input() {
        return;
    }

    ctx.input_mut(|input| {
        // Ctrl+S: Save Scene
        if input.consume_key(egui::Modifiers::CTRL, egui::Key::S) {
            handle_save_scene(world, false);
        }
        // Ctrl+Shift+S: Save Scene As
        else if input.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::S) {
            handle_save_scene(world, true);
        }
        // Ctrl+O: Load Scene
        else if input.consume_key(egui::Modifiers::CTRL, egui::Key::O) {
            handle_load_scene(world);
        }
        // Ctrl+N: New Scene
        else if input.consume_key(egui::Modifiers::CTRL, egui::Key::N) {
            new_scene(world);
            set_success_message(world, "Created new scene");
        }
    });
}

/// Renders the File menu.
fn file_menu(ui: &mut egui::Ui, world: &mut World) {
    ui.menu_button("File", |ui| {
        // New Scene
        if menu_item(ui, "New Scene", "Ctrl+N") {
            new_scene(world);
            set_success_message(world, "Created new scene");
            ui.close_menu();
        }

        ui.separator();

        // Save Scene
        if menu_item(ui, "Save Scene", "Ctrl+S") {
            handle_save_scene(world, false);
            ui.close_menu();
        }

        // Save Scene As
        if menu_item(ui, "Save Scene As...", "Ctrl+Shift+S") {
            handle_save_scene(world, true);
            ui.close_menu();
        }

        ui.separator();

        // Load Scene
        if menu_item(ui, "Load Scene...", "Ctrl+O") {
            handle_load_scene(world);
            ui.close_menu();
        }

        ui.separator();

        // Spawn Prefab
        if ui.button("Spawn Prefab...").clicked() {
            handle_spawn_prefab(world);
            ui.close_menu();
        }

        // Save as Prefab (save current selection or all as prefab)
        if ui.button("Save as Prefab...").clicked() {
            handle_save_prefab(world);
            ui.close_menu();
        }
    });
}

/// Renders the Window menu.
fn window_menu(ui: &mut egui::Ui, world: &mut World) {
    ui.menu_button("Window", |ui| {
        // Animation Editor
        if ui.button("Animation Editor").clicked() {
            if let Some(mut state) = world.get_resource_mut::<AnimationEditorState>() {
                state.open = true;
            }
            ui.close_menu();
        }
    });
}

/// Renders a menu item with a keyboard shortcut hint.
fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: &str) -> bool {
    ui.horizontal(|ui| {
        let response = ui.button(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.weak(shortcut);
        });
        response.clicked()
    })
    .inner
}

/// Handles saving a scene.
fn handle_save_scene(world: &mut World, force_dialog: bool) {
    // Check if we have an existing path and don't need to show dialog
    let existing_path = world
        .get_resource::<SceneManager>()
        .and_then(|m| m.current_scene_path.clone());

    let path = if force_dialog || existing_path.is_none() {
        // Show save dialog
        let dialog = rfd::FileDialog::new()
            .set_title("Save Scene")
            .add_filter("Scene files", &["scn.ron"])
            .set_directory("assets/scenes")
            .set_file_name("scene.scn.ron");

        dialog.save_file()
    } else {
        existing_path
    };

    if let Some(path) = path {
        match save_scene(world, &path) {
            Ok(()) => {
                set_success_message(world, &format!("Saved: {}", path.display()));
            }
            Err(e) => {
                set_error_message(world, &format!("Failed to save: {}", e));
            }
        }
    }
}

/// Handles loading a scene.
fn handle_load_scene(world: &mut World) {
    let dialog = rfd::FileDialog::new()
        .set_title("Load Scene")
        .add_filter("Scene files", &["scn.ron"])
        .set_directory("assets/scenes");

    if let Some(path) = dialog.pick_file() {
        match load_scene(world, &path) {
            Ok(()) => {
                set_success_message(world, &format!("Loaded: {}", path.display()));
            }
            Err(e) => {
                set_error_message(world, &format!("Failed to load: {}", e));
            }
        }
    }
}

/// Handles spawning a prefab.
fn handle_spawn_prefab(world: &mut World) {
    let dialog = rfd::FileDialog::new()
        .set_title("Spawn Prefab")
        .add_filter("Prefab files", &["scn.ron"])
        .set_directory("assets/prefabs");

    if let Some(path) = dialog.pick_file() {
        match spawn_prefab(world, &path) {
            Ok(()) => {
                set_success_message(world, &format!("Spawned prefab: {}", path.display()));
            }
            Err(e) => {
                set_error_message(world, &format!("Failed to spawn prefab: {}", e));
            }
        }
    }
}

/// Handles saving the current scene as a prefab.
fn handle_save_prefab(world: &mut World) {
    // Save the previous scene path before save_scene overwrites it
    let previous_path = world
        .get_resource::<SceneManager>()
        .and_then(|m| m.current_scene_path.clone());

    let dialog = rfd::FileDialog::new()
        .set_title("Save as Prefab")
        .add_filter("Prefab files", &["scn.ron"])
        .set_directory("assets/prefabs")
        .set_file_name("prefab.scn.ron");

    if let Some(path) = dialog.save_file() {
        // Prefabs use the same format as scenes
        match save_scene(world, &path) {
            Ok(()) => {
                set_success_message(world, &format!("Saved prefab: {}", path.display()));
                // Restore the previous scene path since prefabs shouldn't change it
                if let Some(mut manager) = world.get_resource_mut::<SceneManager>() {
                    manager.current_scene_path = previous_path;
                }
            }
            Err(e) => {
                set_error_message(world, &format!("Failed to save prefab: {}", e));
            }
        }
    }
}

/// Displays status messages (errors and success).
pub fn status_messages(ctx: &egui::Context, world: &mut World) {
    // Initialize FileMenuState if it doesn't exist
    if !world.contains_resource::<FileMenuState>() {
        world.init_resource::<FileMenuState>();
    }

    let mut clear_error = false;
    let mut clear_success = false;

    // Show error message if present
    if let Some(state) = world.get_resource::<FileMenuState>() {
        if let Some(ref error) = state.error_message {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(error);
                    if ui.button("OK").clicked() {
                        clear_error = true;
                    }
                });
        }

        if let Some(ref success) = state.success_message {
            // Show success message as a toast-like notification at the bottom
            egui::TopBottomPanel::bottom("status_toast")
                .frame(egui::Frame::none().fill(egui::Color32::from_rgb(40, 80, 40)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(success);
                        if ui.small_button("×").clicked() {
                            clear_success = true;
                        }
                    });
                });
        }
    }

    if clear_error {
        if let Some(mut state) = world.get_resource_mut::<FileMenuState>() {
            state.error_message = None;
        }
    }

    if clear_success {
        if let Some(mut state) = world.get_resource_mut::<FileMenuState>() {
            state.success_message = None;
        }
    }
}

fn set_error_message(world: &mut World, message: &str) {
    if !world.contains_resource::<FileMenuState>() {
        world.init_resource::<FileMenuState>();
    }
    if let Some(mut state) = world.get_resource_mut::<FileMenuState>() {
        state.error_message = Some(message.to_string());
        state.success_message = None;
    }
}

fn set_success_message(world: &mut World, message: &str) {
    if !world.contains_resource::<FileMenuState>() {
        world.init_resource::<FileMenuState>();
    }
    if let Some(mut state) = world.get_resource_mut::<FileMenuState>() {
        state.success_message = Some(message.to_string());
        state.error_message = None;
    }
}

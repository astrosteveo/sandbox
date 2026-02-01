// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Asset browser panel UI.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::assets::{AssetBrowser, AssetEntry, AssetType, AudioPreviewMarker};

/// Displays the asset browser panel.
pub fn asset_browser_panel(ui: &mut egui::Ui, world: &mut World) {
    ui.horizontal(|ui| {
        ui.heading("Assets");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("⟳ Refresh").clicked() {
                world.resource_mut::<AssetBrowser>().scan_assets_directory();
            }
        });
    });
    ui.separator();

    // Get asset browser state (clone to avoid borrow issues)
    let files = world.resource::<AssetBrowser>().files.clone();
    let selected_path = world.resource::<AssetBrowser>().selected_path.clone();

    // Layout: file tree on left, preview on right
    ui.columns(2, |columns| {
        // Left column: File tree
        egui::ScrollArea::vertical()
            .id_salt("asset_tree")
            .auto_shrink([false; 2])
            .show(&mut columns[0], |ui| {
                for entry in &files {
                    display_asset_entry(ui, world, entry, &selected_path);
                }

                if files.is_empty() {
                    ui.label("No assets found.");
                    ui.label("Add files to the 'assets' directory.");
                }
            });

        // Right column: Preview
        columns[1].vertical(|ui| {
            if let Some(selected) = &selected_path {
                display_asset_preview(ui, world, selected);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a file to preview");
                });
            }
        });
    });
}

/// Recursively displays an asset entry in the tree.
fn display_asset_entry(
    ui: &mut egui::Ui,
    world: &mut World,
    entry: &AssetEntry,
    selected_path: &Option<String>,
) {
    let is_selected = selected_path.as_ref() == Some(&entry.path);

    if entry.is_directory {
        // Directory entry with expandable header
        let expanded = entry.expanded;
        ui.horizontal(|ui| {
            let icon = if expanded { "▼" } else { "▶" };
            if ui.small_button(icon).clicked() {
                world
                    .resource_mut::<AssetBrowser>()
                    .toggle_directory(&entry.path);
            }
            ui.label(format!("📁 {}", entry.name));
        });

        // Show children if expanded
        if expanded {
            ui.indent(&entry.path, |ui| {
                for child in &entry.children {
                    display_asset_entry(ui, world, child, selected_path);
                }
            });
        }
    } else {
        // File entry
        let asset_type = AssetBrowser::get_asset_type(&entry.path);
        let icon = asset_type.icon();

        let response = ui.selectable_label(is_selected, format!("{} {}", icon, entry.name));

        if response.clicked() {
            world.resource_mut::<AssetBrowser>().selected_path = Some(entry.path.clone());
        }
    }
}

/// Displays a preview of the selected asset.
fn display_asset_preview(ui: &mut egui::Ui, world: &mut World, path: &str) {
    let asset_type = AssetBrowser::get_asset_type(path);

    ui.label(format!("Path: {}", path));
    ui.separator();

    match asset_type {
        AssetType::Image => {
            display_image_preview(ui, world, path);
        }
        AssetType::Audio => {
            display_audio_preview(ui, world, path);
        }
        AssetType::Scene => {
            ui.label("Scene file");
            ui.label("Use File > Load Scene to open");
        }
        AssetType::Unknown => {
            ui.label("Unknown file type");
        }
    }
}

/// Displays an image preview.
fn display_image_preview(ui: &mut egui::Ui, world: &mut World, path: &str) {
    // Load the image if not already loaded
    let asset_path = path.to_string();
    let handle: Handle<Image> = world.resource::<AssetServer>().load(&asset_path);

    // Store handle in browser to keep it alive
    world
        .resource_mut::<AssetBrowser>()
        .preview_handles
        .insert(path.to_string(), handle.clone());

    // Check if image is loaded
    let images = world.resource::<Assets<Image>>();
    if let Some(image) = images.get(&handle) {
        // Get image dimensions
        let (width, height) = (image.width(), image.height());
        ui.label(format!("Size: {}x{}", width, height));
        ui.separator();

        // Calculate scaled size to fit preview area
        let available_size = ui.available_size();
        let scale = (available_size.x / width as f32)
            .min(available_size.y / height as f32)
            .min(1.0); // Don't upscale

        let display_size = egui::vec2(width as f32 * scale, height as f32 * scale);

        // Register the texture with egui
        let texture_id = {
            let mut egui_user_textures = world.resource_mut::<bevy_egui::EguiUserTextures>();
            egui_user_textures.add_image(handle.clone())
        };

        // Display the image
        ui.image(egui::load::SizedTexture::new(texture_id, display_size));
    } else {
        ui.spinner();
        ui.label("Loading...");
    }
}

/// Audio action to perform after UI interaction.
enum AudioAction {
    None,
    Play(String),
    Stop,
}

/// Displays audio preview controls.
fn display_audio_preview(ui: &mut egui::Ui, world: &mut World, path: &str) {
    // Get current playback state
    let is_playing = world.resource::<AssetBrowser>().is_playing(path);
    let existing_entity = world.resource::<AssetBrowser>().audio_playback_entity;

    // Display audio file info
    let extension = path.rsplit('.').next().unwrap_or("unknown");
    ui.label(format!("Format: {}", extension.to_uppercase()));
    ui.separator();

    // Playback controls - collect action without borrowing world mutably
    let action = ui
        .horizontal(|ui| {
            if is_playing {
                if ui.button("⏹ Stop").clicked() {
                    return AudioAction::Stop;
                }
                ui.label("▶ Playing...");
            } else if ui.button("▶ Play").clicked() {
                return AudioAction::Play(path.to_string());
            }
            AudioAction::None
        })
        .inner;

    // Execute the action
    match action {
        AudioAction::Play(path_string) => {
            // Stop any existing playback first
            if let Some(entity) = existing_entity {
                world.despawn(entity);
            }

            // Load and play the audio
            let handle: Handle<AudioSource> = world.resource::<AssetServer>().load(&path_string);
            let entity = world
                .spawn((
                    AudioPlayer::<AudioSource>(handle),
                    PlaybackSettings::DESPAWN,
                    AudioPreviewMarker,
                ))
                .id();

            let mut browser = world.resource_mut::<AssetBrowser>();
            browser.audio_playback_entity = Some(entity);
            browser.playing_audio_path = Some(path_string);
        }
        AudioAction::Stop => {
            if let Some(entity) = existing_entity {
                world.despawn(entity);
            }
            let mut browser = world.resource_mut::<AssetBrowser>();
            browser.audio_playback_entity = None;
            browser.playing_audio_path = None;
        }
        AudioAction::None => {}
    }

    // Audio tips
    ui.separator();
    ui.label("Tip: Audio plays in the editor.");
    ui.label("Supported: .ogg, .wav, .mp3");
}

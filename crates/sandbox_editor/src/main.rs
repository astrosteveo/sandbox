// Copyright (C) 2025 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sandbox Editor - Integrated editor for the Sandbox Engine

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sandbox Editor".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, editor_ui)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn editor_ui(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    // Left panel - Viewport
    egui::SidePanel::left("viewport_panel")
        .resizable(true)
        .default_width(ctx.screen_rect().width() * 0.7)
        .min_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Viewport");
            ui.separator();

            // This is where the game viewport will render
            let available_size = ui.available_size();
            let (rect, _response) = ui.allocate_exact_size(
                available_size,
                egui::Sense::hover(),
            );

            // Draw a placeholder background for the viewport area
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(30, 30, 40),
            );

            // Center text in the viewport
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Game Viewport",
                egui::FontId::proportional(20.0),
                egui::Color32::from_rgb(100, 100, 120),
            );
        });

    // Right panel - Inspector
    egui::SidePanel::right("inspector_panel")
        .resizable(true)
        .default_width(300.0)
        .min_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();

            ui.label("Select an entity to inspect its components.");
            ui.add_space(20.0);

            // Placeholder for future inspector content
            ui.group(|ui| {
                ui.label("No entity selected");
            });
        });
}

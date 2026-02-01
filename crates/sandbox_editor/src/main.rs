// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sandbox Editor - Integrated editor for the Sandbox Engine

use bevy::prelude::*;
use bevy_egui::{egui, EguiPlugin};
use sandbox_engine::editor_state::{EditorPlayState, EditorStatePlugin};

mod gizmo;
mod selection;
mod ui;

use gizmo::{draw_translation_gizmo, GizmoPlugin};
use selection::SelectionPlugin;
use ui::{hierarchy_panel, inspector_panel};

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
        .add_plugins(EditorStatePlugin)
        .add_plugins(SelectionPlugin)
        .add_plugins(GizmoPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, editor_ui)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn some test entities for the editor
    commands.spawn((
        Name::new("Player Ship"),
        Sprite {
            color: Color::srgb(0.2, 0.6, 0.9),
            custom_size: Some(Vec2::new(40.0, 50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));

    commands.spawn((
        Name::new("Asteroid 1"),
        Sprite {
            color: Color::srgb(0.6, 0.5, 0.4),
            custom_size: Some(Vec2::new(30.0, 30.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(100.0, 50.0, 0.0)),
    ));

    commands.spawn((
        Name::new("Asteroid 2"),
        Sprite {
            color: Color::srgb(0.5, 0.4, 0.3),
            custom_size: Some(Vec2::new(45.0, 40.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-80.0, 120.0, 0.0)),
    ));
}

fn editor_ui(world: &mut World) {
    // Extract egui context
    let mut egui_ctx = world
        .query::<&mut bevy_egui::EguiContext>()
        .iter_mut(world)
        .next()
        .expect("EguiContext should exist")
        .clone();

    let ctx = egui_ctx.get_mut();

    // Top toolbar with play/pause/stop controls
    egui::TopBottomPanel::top("toolbar")
        .exact_height(36.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                toolbar_ui(ui, world);
            });
        });

    // Left panel - Hierarchy (narrow)
    egui::SidePanel::left("hierarchy_panel")
        .resizable(true)
        .default_width(200.0)
        .min_width(150.0)
        .max_width(400.0)
        .show(ctx, |ui| {
            hierarchy_panel(ui, world);
        });

    // Right panel - Inspector
    egui::SidePanel::right("inspector_panel")
        .resizable(true)
        .default_width(300.0)
        .min_width(200.0)
        .max_width(500.0)
        .show(ctx, |ui| {
            inspector_panel(ui, world);
        });

    // Central panel - Viewport
    egui::CentralPanel::default().show(ctx, |ui| {
        viewport_panel(ui, world);
    });
}

/// Renders the toolbar with play/pause/stop controls.
fn toolbar_ui(ui: &mut egui::Ui, world: &mut World) {
    let current_state = *world.resource::<State<EditorPlayState>>().get();

    ui.add_space(10.0);

    // Play button
    let play_enabled = current_state != EditorPlayState::Playing;
    if ui
        .add_enabled(play_enabled, egui::Button::new("▶ Play"))
        .clicked()
    {
        if let Some(mut next_state) = world.get_resource_mut::<NextState<EditorPlayState>>() {
            next_state.set(EditorPlayState::Playing);
        }
    }

    // Pause button
    let pause_enabled = current_state == EditorPlayState::Playing;
    if ui
        .add_enabled(pause_enabled, egui::Button::new("⏸ Pause"))
        .clicked()
    {
        if let Some(mut next_state) = world.get_resource_mut::<NextState<EditorPlayState>>() {
            next_state.set(EditorPlayState::Paused);
        }
    }

    // Resume button (only shown when paused)
    if current_state == EditorPlayState::Paused && ui.button("▶ Resume").clicked() {
        if let Some(mut next_state) = world.get_resource_mut::<NextState<EditorPlayState>>() {
            next_state.set(EditorPlayState::Playing);
        }
    }

    // Stop button
    let stop_enabled = current_state != EditorPlayState::Stopped;
    if ui
        .add_enabled(stop_enabled, egui::Button::new("⏹ Stop"))
        .clicked()
    {
        if let Some(mut next_state) = world.get_resource_mut::<NextState<EditorPlayState>>() {
            next_state.set(EditorPlayState::Stopped);
        }
    }

    ui.separator();

    // Display current state
    let state_text = match current_state {
        EditorPlayState::Stopped => "Stopped",
        EditorPlayState::Playing => "Playing",
        EditorPlayState::Paused => "Paused",
    };
    ui.label(format!("State: {}", state_text));
}

/// Renders the viewport panel with the game view and gizmos.
fn viewport_panel(ui: &mut egui::Ui, world: &mut World) {
    ui.heading("Viewport");
    ui.separator();

    // Allocate viewport area
    let available_size = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

    // Draw placeholder background
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 40));

    // Draw "Game Viewport" text
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Game Viewport",
        egui::FontId::proportional(20.0),
        egui::Color32::from_rgb(100, 100, 120),
    );

    // Draw grid (subtle reference lines)
    draw_viewport_grid(&painter, rect, world);

    // Draw gizmos for selected entity
    draw_translation_gizmo(&painter, rect, world, &response);

    // Handle viewport click for future entity picking
    if response.clicked() && world.resource::<gizmo::GizmoDragState>().dragging.is_none() {
        // Deselect when clicking empty space (basic behavior)
        // Full entity picking will be implemented later
    }
}

/// Draws a subtle grid in the viewport for spatial reference.
fn draw_viewport_grid(painter: &egui::Painter, viewport_rect: egui::Rect, world: &mut World) {
    let grid_spacing = 50.0;
    let grid_color = egui::Color32::from_rgba_unmultiplied(100, 100, 120, 30);

    // Get camera position
    let camera_offset = {
        let mut query = world.query_filtered::<&Transform, With<Camera2d>>();
        query
            .iter(world)
            .next()
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO)
    };

    let center = viewport_rect.center();

    // Calculate grid offset based on camera
    let offset_x = camera_offset.x % grid_spacing;
    let offset_y = camera_offset.y % grid_spacing;

    // Vertical lines
    let mut x = viewport_rect.left() - offset_x;
    while x < viewport_rect.right() {
        painter.line_segment(
            [
                egui::pos2(x, viewport_rect.top()),
                egui::pos2(x, viewport_rect.bottom()),
            ],
            egui::Stroke::new(1.0, grid_color),
        );
        x += grid_spacing;
    }

    // Horizontal lines
    let mut y = viewport_rect.top() + offset_y;
    while y < viewport_rect.bottom() {
        painter.line_segment(
            [
                egui::pos2(viewport_rect.left(), y),
                egui::pos2(viewport_rect.right(), y),
            ],
            egui::Stroke::new(1.0, grid_color),
        );
        y += grid_spacing;
    }

    // Draw origin axes (more visible)
    let origin_x = center.x - camera_offset.x;
    let origin_y = center.y + camera_offset.y;

    if viewport_rect.x_range().contains(origin_x) {
        painter.line_segment(
            [
                egui::pos2(origin_x, viewport_rect.top()),
                egui::pos2(origin_x, viewport_rect.bottom()),
            ],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(200, 100, 100, 60),
            ),
        );
    }

    if viewport_rect.y_range().contains(origin_y) {
        painter.line_segment(
            [
                egui::pos2(viewport_rect.left(), origin_y),
                egui::pos2(viewport_rect.right(), origin_y),
            ],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(100, 200, 100, 60),
            ),
        );
    }
}

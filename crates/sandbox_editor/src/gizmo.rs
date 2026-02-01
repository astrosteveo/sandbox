// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Transform gizmo rendering and interaction for the editor.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::selection::EditorSelection;
use sandbox_engine::editor_state::EditorPlayState;

/// Resource tracking gizmo drag state.
#[derive(Resource, Default)]
pub struct GizmoDragState {
    /// Which axis is being dragged, if any.
    pub dragging: Option<GizmoAxis>,
    /// Screen position where drag started.
    pub drag_start: Option<egui::Pos2>,
    /// World position of entity when drag started.
    pub entity_start_pos: Option<Vec3>,
}

/// Gizmo axis being manipulated.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GizmoAxis {
    X,
    Y,
    Center,
}

/// Gizmo visual constants.
const GIZMO_LENGTH: f32 = 80.0;
const GIZMO_THICKNESS: f32 = 3.0;
const GIZMO_HEAD_SIZE: f32 = 12.0;
const GIZMO_CENTER_SIZE: f32 = 16.0;
const GIZMO_HIT_RADIUS: f32 = 12.0;

const COLOR_X: egui::Color32 = egui::Color32::from_rgb(230, 80, 80);
const COLOR_Y: egui::Color32 = egui::Color32::from_rgb(80, 200, 80);
const COLOR_CENTER: egui::Color32 = egui::Color32::from_rgb(255, 255, 100);

const COLOR_X_HOVER: egui::Color32 = egui::Color32::from_rgb(255, 120, 120);
const COLOR_Y_HOVER: egui::Color32 = egui::Color32::from_rgb(120, 255, 120);
const COLOR_CENTER_HOVER: egui::Color32 = egui::Color32::from_rgb(255, 255, 180);

/// Plugin that sets up the gizmo system.
pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GizmoDragState>();
    }
}

/// Renders translation gizmos for the selected entity.
///
/// Call this from the viewport drawing code, passing the painter and viewport rect.
pub fn draw_translation_gizmo(
    painter: &egui::Painter,
    viewport_rect: egui::Rect,
    world: &mut World,
    response: &egui::Response,
) {
    // Only show gizmos when stopped
    let play_state = *world.resource::<State<EditorPlayState>>().get();
    if play_state != EditorPlayState::Stopped {
        return;
    }

    let selected_entity = world.resource::<EditorSelection>().selected_entity;
    let Some(entity) = selected_entity else {
        return;
    };

    // Get entity's transform
    let Some(transform) = world.get::<Transform>(entity) else {
        return;
    };
    let entity_world_pos = transform.translation;

    // Get camera for world-to-screen conversion
    let camera_transform = {
        let mut query = world.query_filtered::<&Transform, With<Camera2d>>();
        query.iter(world).next().copied()
    };

    let Some(camera_transform) = camera_transform else {
        return;
    };

    // Convert world position to screen position
    let screen_pos = world_to_screen(
        entity_world_pos.truncate(),
        camera_transform.translation.truncate(),
        viewport_rect,
    );

    // Check for hover/interaction
    let pointer_pos = response.hover_pos();
    let mut drag_state = world.resource_mut::<GizmoDragState>();

    let hovered_axis = if let Some(pos) = pointer_pos {
        hit_test_gizmo(pos, screen_pos)
    } else {
        None
    };

    // Handle drag interaction
    if response.drag_started() {
        if let Some(axis) = hovered_axis {
            drag_state.dragging = Some(axis);
            drag_state.drag_start = pointer_pos;
            drag_state.entity_start_pos = Some(entity_world_pos);
        }
    }

    if response.drag_stopped() {
        drag_state.dragging = None;
        drag_state.drag_start = None;
        drag_state.entity_start_pos = None;
    }

    // Calculate new position if dragging
    let new_position = {
        if let (Some(axis), Some(start_pos), Some(entity_start)) = (
            drag_state.dragging,
            drag_state.drag_start,
            drag_state.entity_start_pos,
        ) {
            if let Some(current_pos) = pointer_pos {
                let delta = current_pos - start_pos;

                // Convert screen delta to world delta (assuming 1:1 scale for now)
                let world_delta = Vec3::new(
                    match axis {
                        GizmoAxis::X | GizmoAxis::Center => delta.x,
                        GizmoAxis::Y => 0.0,
                    },
                    match axis {
                        GizmoAxis::Y | GizmoAxis::Center => -delta.y, // Screen Y is inverted
                        GizmoAxis::X => 0.0,
                    },
                    0.0,
                );

                Some(entity_start + world_delta)
            } else {
                None
            }
        } else {
            None
        }
    };

    // Apply new position if calculated
    if let Some(new_pos) = new_position {
        if let Some(mut transform) = world.get_mut::<Transform>(entity) {
            transform.translation = new_pos;
        }
    }

    // Draw gizmo
    let is_dragging = world.resource::<GizmoDragState>().dragging;

    // X axis (horizontal, red)
    let x_color = if is_dragging == Some(GizmoAxis::X) || hovered_axis == Some(GizmoAxis::X) {
        COLOR_X_HOVER
    } else {
        COLOR_X
    };
    draw_arrow(painter, screen_pos, egui::vec2(GIZMO_LENGTH, 0.0), x_color);

    // Y axis (vertical, green) - note: screen Y is down, so we draw up with negative Y
    let y_color = if is_dragging == Some(GizmoAxis::Y) || hovered_axis == Some(GizmoAxis::Y) {
        COLOR_Y_HOVER
    } else {
        COLOR_Y
    };
    draw_arrow(painter, screen_pos, egui::vec2(0.0, -GIZMO_LENGTH), y_color);

    // Center handle (yellow square)
    let center_color =
        if is_dragging == Some(GizmoAxis::Center) || hovered_axis == Some(GizmoAxis::Center) {
            COLOR_CENTER_HOVER
        } else {
            COLOR_CENTER
        };
    let center_rect =
        egui::Rect::from_center_size(screen_pos, egui::vec2(GIZMO_CENTER_SIZE, GIZMO_CENTER_SIZE));
    painter.rect_filled(center_rect, 2.0, center_color);
}

/// Draws an arrow from origin in the given direction.
fn draw_arrow(
    painter: &egui::Painter,
    origin: egui::Pos2,
    direction: egui::Vec2,
    color: egui::Color32,
) {
    let end = origin + direction;

    // Draw shaft
    painter.line_segment([origin, end], egui::Stroke::new(GIZMO_THICKNESS, color));

    // Draw arrowhead
    let dir_normalized = direction.normalized();
    let perpendicular = egui::vec2(-dir_normalized.y, dir_normalized.x);

    let head_base = end - dir_normalized * GIZMO_HEAD_SIZE;
    let head_left = head_base + perpendicular * (GIZMO_HEAD_SIZE * 0.5);
    let head_right = head_base - perpendicular * (GIZMO_HEAD_SIZE * 0.5);

    painter.add(egui::Shape::convex_polygon(
        vec![end, head_left, head_right],
        color,
        egui::Stroke::NONE,
    ));
}

/// Hit tests the gizmo to determine which axis (if any) is under the pointer.
fn hit_test_gizmo(pointer: egui::Pos2, gizmo_center: egui::Pos2) -> Option<GizmoAxis> {
    // Check center first (highest priority)
    let center_rect = egui::Rect::from_center_size(
        gizmo_center,
        egui::vec2(
            GIZMO_CENTER_SIZE + GIZMO_HIT_RADIUS,
            GIZMO_CENTER_SIZE + GIZMO_HIT_RADIUS,
        ),
    );
    if center_rect.contains(pointer) {
        return Some(GizmoAxis::Center);
    }

    // Check X axis
    let x_end = gizmo_center + egui::vec2(GIZMO_LENGTH, 0.0);
    if distance_to_line_segment(pointer, gizmo_center, x_end) < GIZMO_HIT_RADIUS {
        return Some(GizmoAxis::X);
    }

    // Check Y axis
    let y_end = gizmo_center + egui::vec2(0.0, -GIZMO_LENGTH);
    if distance_to_line_segment(pointer, gizmo_center, y_end) < GIZMO_HIT_RADIUS {
        return Some(GizmoAxis::Y);
    }

    None
}

/// Calculates the distance from a point to a line segment.
fn distance_to_line_segment(point: egui::Pos2, start: egui::Pos2, end: egui::Pos2) -> f32 {
    let line = end - start;
    let len_sq = line.length_sq();

    if len_sq < 0.001 {
        return (point - start).length();
    }

    let t = ((point - start).dot(line) / len_sq).clamp(0.0, 1.0);
    let projection = start + line * t;

    (point - projection).length()
}

/// Converts world coordinates to screen coordinates.
fn world_to_screen(world_pos: Vec2, camera_pos: Vec2, viewport_rect: egui::Rect) -> egui::Pos2 {
    let relative = world_pos - camera_pos;
    let viewport_center = viewport_rect.center();

    egui::pos2(
        viewport_center.x + relative.x,
        viewport_center.y - relative.y, // Screen Y is inverted
    )
}

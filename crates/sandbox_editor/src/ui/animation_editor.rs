// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Animation editor window for sprite animations.

use bevy::prelude::*;
use bevy_egui::egui;
use sandbox_engine::assets::{AnimationFrame, SpriteAnimation};

use crate::selection::EditorSelection;

/// Resource to track animation editor window state.
#[derive(Resource, Default)]
pub struct AnimationEditorState {
    /// Whether the animation editor window is open.
    pub open: bool,
    /// Index of the frame being edited (-1 for none).
    pub editing_frame_index: Option<usize>,
    /// Temporary values for frame editing.
    pub edit_rect_min_x: f32,
    pub edit_rect_min_y: f32,
    pub edit_rect_max_x: f32,
    pub edit_rect_max_y: f32,
    pub edit_duration: f32,
}

impl AnimationEditorState {
    /// Opens the editor for a specific frame.
    pub fn edit_frame(&mut self, index: usize, frame: &AnimationFrame) {
        self.editing_frame_index = Some(index);
        self.edit_rect_min_x = frame.rect.min.x;
        self.edit_rect_min_y = frame.rect.min.y;
        self.edit_rect_max_x = frame.rect.max.x;
        self.edit_rect_max_y = frame.rect.max.y;
        self.edit_duration = frame.duration;
    }

    /// Clears the frame editing state.
    pub fn clear_edit(&mut self) {
        self.editing_frame_index = None;
    }

    /// Creates an AnimationFrame from the current edit state.
    pub fn to_frame(&self) -> AnimationFrame {
        AnimationFrame {
            rect: Rect::new(
                self.edit_rect_min_x,
                self.edit_rect_min_y,
                self.edit_rect_max_x,
                self.edit_rect_max_y,
            ),
            duration: self.edit_duration,
        }
    }
}

/// Displays the animation editor window.
pub fn animation_editor_window(ctx: &egui::Context, world: &mut World) {
    // Check if window should be open
    let is_open = world
        .get_resource::<AnimationEditorState>()
        .map(|s| s.open)
        .unwrap_or(false);

    if !is_open {
        return;
    }

    // Get selected entity
    let selected_entity = world.resource::<EditorSelection>().selected_entity;

    let mut should_close = false;

    egui::Window::new("Animation Editor")
        .default_width(400.0)
        .default_height(500.0)
        .resizable(true)
        .show(ctx, |ui| {
            // Close button
            ui.horizontal(|ui| {
                ui.heading("Sprite Animation Editor");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("X Close").clicked() {
                        should_close = true;
                    }
                });
            });
            ui.separator();

            match selected_entity {
                None => {
                    ui.label("No entity selected.");
                    ui.label("Select an entity with a SpriteAnimation component.");
                }
                Some(entity) => {
                    if world.get::<SpriteAnimation>(entity).is_some() {
                        display_animation_editor(ui, world, entity);
                    } else {
                        ui.label("Selected entity has no SpriteAnimation component.");
                        ui.separator();

                        if ui.button("+ Add SpriteAnimation").clicked() {
                            world.entity_mut(entity).insert(SpriteAnimation::default());
                        }
                    }
                }
            }
        });

    if should_close {
        if let Some(mut state) = world.get_resource_mut::<AnimationEditorState>() {
            state.open = false;
        }
    }
}

/// Displays the animation editor UI for an entity with SpriteAnimation.
fn display_animation_editor(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // Get animation data (clone to avoid borrow issues)
    let animation = world.get::<SpriteAnimation>(entity).cloned();
    let Some(animation) = animation else {
        ui.label("Animation not accessible");
        return;
    };

    // Playback controls section
    ui.heading("Playback");
    ui.horizontal(|ui| {
        if animation.playing {
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

        ui.separator();

        let mut looping = animation.looping;
        if ui.checkbox(&mut looping, "Loop").changed() {
            if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                anim.looping = looping;
            }
        }
    });

    ui.label(format!(
        "Current: Frame {} / {} (timer: {:.2}s)",
        animation.current_frame + 1,
        animation.frames.len(),
        animation.timer
    ));

    ui.separator();

    // Frames section
    ui.horizontal(|ui| {
        ui.heading("Frames");
        if ui.button("+ Add Frame").clicked() {
            if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                anim.frames.push(AnimationFrame {
                    rect: Rect::new(0.0, 0.0, 32.0, 32.0),
                    duration: 0.1,
                });
            }
        }
    });

    // Frame list
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            let mut action = FrameAction::None;

            for (i, frame) in animation.frames.iter().enumerate() {
                let is_current = i == animation.current_frame;
                let prefix = if is_current { "▶ " } else { "   " };

                ui.horizontal(|ui| {
                    ui.label(format!("{}Frame {}:", prefix, i + 1));

                    // Show frame info
                    ui.label(format!(
                        "({:.0},{:.0})-({:.0},{:.0}) {:.2}s",
                        frame.rect.min.x,
                        frame.rect.min.y,
                        frame.rect.max.x,
                        frame.rect.max.y,
                        frame.duration
                    ));

                    // Edit button
                    if ui.small_button("✏ Edit").clicked() {
                        action = FrameAction::Edit(i);
                    }

                    // Delete button
                    if ui.small_button("🗑").clicked() {
                        action = FrameAction::Delete(i);
                    }

                    // Move up/down
                    if i > 0 && ui.small_button("↑").clicked() {
                        action = FrameAction::MoveUp(i);
                    }
                    if i < animation.frames.len() - 1 && ui.small_button("↓").clicked() {
                        action = FrameAction::MoveDown(i);
                    }
                });
            }

            // Handle frame actions
            match action {
                FrameAction::Edit(index) => {
                    if let Some(mut state) = world.get_resource_mut::<AnimationEditorState>() {
                        state.edit_frame(index, &animation.frames[index]);
                    }
                }
                FrameAction::Delete(index) => {
                    if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                        anim.frames.remove(index);
                        if anim.current_frame >= anim.frames.len() && !anim.frames.is_empty() {
                            anim.current_frame = anim.frames.len() - 1;
                        }
                    }
                }
                FrameAction::MoveUp(index) => {
                    if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                        anim.frames.swap(index, index - 1);
                    }
                }
                FrameAction::MoveDown(index) => {
                    if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                        anim.frames.swap(index, index + 1);
                    }
                }
                FrameAction::None => {}
            }
        });

    if animation.frames.is_empty() {
        ui.label("No frames. Click '+ Add Frame' to create one.");
    }

    ui.separator();

    // Frame editor section
    let editing_index = world
        .get_resource::<AnimationEditorState>()
        .and_then(|s| s.editing_frame_index);

    if let Some(index) = editing_index {
        ui.heading(format!("Edit Frame {}", index + 1));

        // Get current edit values
        let (rect_min_x, rect_min_y, rect_max_x, rect_max_y, duration) = {
            let state = world.resource::<AnimationEditorState>();
            (
                state.edit_rect_min_x,
                state.edit_rect_min_y,
                state.edit_rect_max_x,
                state.edit_rect_max_y,
                state.edit_duration,
            )
        };

        let mut new_rect_min_x = rect_min_x;
        let mut new_rect_min_y = rect_min_y;
        let mut new_rect_max_x = rect_max_x;
        let mut new_rect_max_y = rect_max_y;
        let mut new_duration = duration;

        ui.horizontal(|ui| {
            ui.label("Rect Min:");
            ui.label("X:");
            ui.add(egui::DragValue::new(&mut new_rect_min_x).speed(1.0));
            ui.label("Y:");
            ui.add(egui::DragValue::new(&mut new_rect_min_y).speed(1.0));
        });

        ui.horizontal(|ui| {
            ui.label("Rect Max:");
            ui.label("X:");
            ui.add(egui::DragValue::new(&mut new_rect_max_x).speed(1.0));
            ui.label("Y:");
            ui.add(egui::DragValue::new(&mut new_rect_max_y).speed(1.0));
        });

        ui.horizontal(|ui| {
            ui.label("Duration (s):");
            ui.add(
                egui::DragValue::new(&mut new_duration)
                    .speed(0.01)
                    .range(0.001..=10.0),
            );
        });

        // Update state with new values
        {
            let mut state = world.resource_mut::<AnimationEditorState>();
            state.edit_rect_min_x = new_rect_min_x;
            state.edit_rect_min_y = new_rect_min_y;
            state.edit_rect_max_x = new_rect_max_x;
            state.edit_rect_max_y = new_rect_max_y;
            state.edit_duration = new_duration;
        }

        ui.horizontal(|ui| {
            if ui.button("✓ Apply").clicked() {
                // Apply changes to the animation
                let new_frame = world.resource::<AnimationEditorState>().to_frame();
                if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
                    if index < anim.frames.len() {
                        anim.frames[index] = new_frame;
                    }
                }
                world.resource_mut::<AnimationEditorState>().clear_edit();
            }

            if ui.button("✗ Cancel").clicked() {
                world.resource_mut::<AnimationEditorState>().clear_edit();
            }
        });
    } else {
        ui.label("Select a frame to edit its properties.");
    }

    ui.separator();

    // Quick setup helpers
    ui.heading("Quick Setup");
    ui.label("Create frames from a sprite sheet grid:");

    let mut cols = 4;
    let mut rows = 4;
    let mut frame_width = 32.0;
    let mut frame_height = 32.0;
    let mut frame_duration = 0.1;

    ui.horizontal(|ui| {
        ui.label("Grid:");
        ui.label("Cols:");
        ui.add(egui::DragValue::new(&mut cols).range(1..=32));
        ui.label("Rows:");
        ui.add(egui::DragValue::new(&mut rows).range(1..=32));
    });

    ui.horizontal(|ui| {
        ui.label("Frame Size:");
        ui.label("W:");
        ui.add(egui::DragValue::new(&mut frame_width).speed(1.0));
        ui.label("H:");
        ui.add(egui::DragValue::new(&mut frame_height).speed(1.0));
    });

    ui.horizontal(|ui| {
        ui.label("Duration:");
        ui.add(
            egui::DragValue::new(&mut frame_duration)
                .speed(0.01)
                .range(0.001..=10.0),
        );
        ui.label("s per frame");
    });

    if ui.button("Generate Grid Frames").clicked() {
        if let Some(mut anim) = world.get_mut::<SpriteAnimation>(entity) {
            anim.frames.clear();
            for row in 0..rows {
                for col in 0..cols {
                    anim.frames.push(AnimationFrame {
                        rect: Rect::new(
                            col as f32 * frame_width,
                            row as f32 * frame_height,
                            (col + 1) as f32 * frame_width,
                            (row + 1) as f32 * frame_height,
                        ),
                        duration: frame_duration,
                    });
                }
            }
            anim.reset();
        }
    }
}

/// Actions that can be taken on a frame.
enum FrameAction {
    None,
    Edit(usize),
    Delete(usize),
    MoveUp(usize),
    MoveDown(usize),
}

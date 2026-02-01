// SPDX-FileCopyrightText: 2026 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Asset components for serializable asset references.
//!
//! This module provides `AssetPath`, a component that stores a path string
//! that gets automatically synced to a `Handle<Image>` at runtime. This solves
//! Bevy's handle serialization limitation by storing the path instead of the handle.

use bevy::prelude::*;

/// A serializable asset path component that syncs to `Handle<Image>`.
///
/// When this component is added to an entity with a `Sprite`, the asset system
/// will automatically load the image and update the sprite's texture.
///
/// # Example
/// ```ignore
/// commands.spawn((
///     Sprite::default(),
///     AssetPath { path: "textures/player.png".to_string() },
///     Transform::default(),
/// ));
/// ```
#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component)]
pub struct AssetPath {
    /// Path to the asset file, relative to the assets directory.
    pub path: String,
}

impl AssetPath {
    /// Creates a new AssetPath with the given path.
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

/// Animation frame data for sprite sheet animations.
#[derive(Clone, Debug, Reflect, Default)]
pub struct AnimationFrame {
    /// Rectangle in the sprite sheet for this frame.
    pub rect: Rect,
    /// Duration of this frame in seconds.
    pub duration: f32,
}

/// Sprite animation component for frame-based animations.
///
/// Works with sprite sheets to animate through multiple frames.
#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component)]
pub struct SpriteAnimation {
    /// List of animation frames.
    pub frames: Vec<AnimationFrame>,
    /// Current frame index.
    pub current_frame: usize,
    /// Time accumulator for frame timing.
    pub timer: f32,
    /// Whether the animation is currently playing.
    pub playing: bool,
    /// Whether the animation should loop.
    pub looping: bool,
}

impl SpriteAnimation {
    /// Creates a new animation with the given frames.
    pub fn new(frames: Vec<AnimationFrame>) -> Self {
        Self {
            frames,
            current_frame: 0,
            timer: 0.0,
            playing: false,
            looping: true,
        }
    }

    /// Starts playing the animation.
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Stops the animation.
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Resets the animation to the first frame.
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.timer = 0.0;
    }
}

/// Plugin that sets up asset path syncing.
pub struct AssetPathPlugin;

impl Plugin for AssetPathPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AssetPath>()
            .register_type::<SpriteAnimation>()
            .register_type::<AnimationFrame>()
            .add_systems(Update, (sync_asset_paths, animate_sprites));
    }
}

/// System that syncs `AssetPath` components to `Sprite` textures.
///
/// When an entity has both `AssetPath` and `Sprite` components, this system
/// loads the image from the path and updates the sprite's image handle.
fn sync_asset_paths(
    asset_server: Res<AssetServer>,
    mut query: Query<(&AssetPath, &mut Sprite), Changed<AssetPath>>,
) {
    for (asset_path, mut sprite) in &mut query {
        if asset_path.path.is_empty() {
            continue;
        }

        // Load the image from the asset path
        let handle: Handle<Image> = asset_server.load(&asset_path.path);

        // Update the sprite's image
        sprite.image = handle;

        // Clear custom_size if it was set (let the texture determine size)
        // This is optional - users can override this in the inspector
    }
}

/// System that advances sprite animations.
fn animate_sprites(time: Res<Time>, mut query: Query<(&mut SpriteAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
        if !animation.playing || animation.frames.is_empty() {
            continue;
        }

        animation.timer += time.delta_secs();

        // Get current frame duration
        let current_frame = &animation.frames[animation.current_frame];
        if animation.timer >= current_frame.duration {
            animation.timer -= current_frame.duration;

            // Advance to next frame
            animation.current_frame += 1;
            if animation.current_frame >= animation.frames.len() {
                if animation.looping {
                    animation.current_frame = 0;
                } else {
                    animation.current_frame = animation.frames.len() - 1;
                    animation.playing = false;
                }
            }

            // Update sprite rect if we have a valid frame
            if animation.current_frame < animation.frames.len() {
                let frame = &animation.frames[animation.current_frame];
                sprite.rect = Some(frame.rect);
            }
        }
    }
}

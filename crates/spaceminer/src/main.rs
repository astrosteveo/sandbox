// Copyright (C) 2025 the Sandbox contributors
// SPDX-License-Identifier: GPL-3.0-or-later

//! Spaceminer - A 2D space mining game

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spaceminer".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (ship_input, apply_drag, apply_velocity, camera_follow).chain())
        .run();
}

/// Marker component for the player's ship
#[derive(Component)]
struct Ship;

/// Velocity component for physics-based movement
#[derive(Component, Default)]
struct Velocity(Vec2);

/// Movement constants
const THRUST: f32 = 500.0;
const DRAG: f32 = 0.98;
const MAX_SPEED: f32 = 400.0;

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Spawn player ship as a colored rectangle
    commands.spawn((
        Ship,
        Velocity::default(),
        Sprite {
            color: Color::srgb(0.2, 0.6, 0.9),
            custom_size: Some(Vec2::new(40.0, 50.0)),
            ..default()
        },
        Transform::default(),
    ));

    // Spawn some background stars for visual reference
    for i in 0..50 {
        let x = ((i * 137) % 2000) as f32 - 1000.0;
        let y = ((i * 251) % 2000) as f32 - 1000.0;
        let size = ((i % 3) + 1) as f32 * 2.0;

        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, -1.0)),
        ));
    }
}

fn ship_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Velocity, With<Ship>>,
) {
    let Ok(mut velocity) = query.get_single_mut() else {
        return;
    };

    let mut thrust_dir = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        thrust_dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        thrust_dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        thrust_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        thrust_dir.x += 1.0;
    }

    if thrust_dir != Vec2::ZERO {
        thrust_dir = thrust_dir.normalize();
        velocity.0 += thrust_dir * THRUST * time.delta_secs();
    }

    // Clamp to max speed
    if velocity.0.length() > MAX_SPEED {
        velocity.0 = velocity.0.normalize() * MAX_SPEED;
    }
}

fn apply_drag(mut query: Query<&mut Velocity>) {
    for mut velocity in &mut query {
        velocity.0 *= DRAG;

        // Stop completely if velocity is very small
        if velocity.0.length() < 0.1 {
            velocity.0 = Vec2::ZERO;
        }
    }
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

fn camera_follow(
    ship_query: Query<&Transform, With<Ship>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Ship>)>,
) {
    let Ok(ship_transform) = ship_query.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    // Smooth camera follow
    let target = ship_transform.translation;
    camera_transform.translation = camera_transform.translation.lerp(target, 0.1);
}

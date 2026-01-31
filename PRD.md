<!--
SPDX-FileCopyrightText: 2026 the Sandbox contributors
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Sandbox Engine - Product Requirements Document

## Vision
A Bevy-based 2D game engine with an integrated editor, designed for rapid prototyping and small-scale game development.

## Core Principles
- **Pragmatic**: Build what's needed, when it's needed
- **Minimal**: Keep the codebase lean and understandable
- **Build-as-needed**: Features emerge from actual game requirements (spaceminer drives development)

## Target Users
- Solo developers
- Small indie teams (2-5 people)
- Game jam participants
- Developers learning Bevy who want editor tooling

## High-Level Feature Roadmap

### Phase 1: Foundation (Complete)
- [x] Cargo workspace structure
- [x] Basic engine plugin with 2D defaults
- [x] Editor with egui split viewport (placeholder viewport)
- [x] Demo game (spaceminer) for dogfooding

### Phase 2: Editor Essentials
- [ ] Entity inspector panel
- [ ] Scene hierarchy view
- [ ] Play/pause/stop controls
- [ ] Basic transform gizmos

### Phase 3: Scene System
- [ ] Scene serialization (RON format)
- [ ] Scene loading/saving
- [ ] Prefab support
- [ ] Hot reloading

### Phase 4: Asset Pipeline
- [ ] Asset browser panel
- [ ] Sprite/texture preview
- [ ] Basic animation editor
- [ ] Audio preview

### Future Considerations
- Tilemap editor integration
- Particle system editor
- Physics debug visualization
- Plugin system for game-specific tools

## Non-Goals
- **3D support**: This is a 2D-focused engine
- **Visual scripting**: Code-first approach
- **Cross-platform editor**: Desktop only (Linux/Windows/macOS)
- **Asset store**: No marketplace infrastructure
- **Multiplayer networking**: Out of scope for initial versions
- **Mobile deployment**: Desktop games only initially
- **Competing with Unity/Godot**: This is a learning/hobby tool, not a commercial engine

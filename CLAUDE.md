<!--
SPDX-FileCopyrightText: 2026 the Sandbox contributors
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Sandbox Engine - Project Context

## Project Structure
```
sandbox/
├── Cargo.toml                    # Workspace manifest
├── PRD.md                        # Engine product requirements
├── GDD.md                        # Spaceminer game design
├── CLAUDE.md                     # This file
├── crates/
│   ├── sandbox_engine/           # Core engine library
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── sandbox_editor/           # Editor application
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── spaceminer/               # Demo game
│       ├── Cargo.toml
│       └── src/main.rs
```

## Build Commands
```bash
# Build everything
cargo build --workspace

# Run editor
cargo run -p sandbox_editor

# Run spaceminer game
cargo run -p spaceminer

# Check all crates
cargo check --workspace

# Run tests
cargo test --workspace
# Note: No tests exist yet - test infrastructure is a future addition

# Lint
cargo clippy --workspace

# Format
cargo fmt --all

# Check REUSE/license compliance
reuse lint
```

## Dependencies
- `bevy = "0.15"` - Game engine foundation
- `bevy_egui = "0.31"` - Editor UI integration

## Coding Conventions

### Rust Idioms
- Use `clippy` defaults
- Prefer `impl Into<T>` for flexible APIs
- Use `thiserror` for error types when needed
- Document public APIs with `///` comments

### Bevy Patterns
- One plugin per major feature
- Systems grouped by schedule (Update, FixedUpdate, etc.)
- Components are small, focused data
- Use events for cross-system communication
- Resources for global state
- Queries should be as specific as possible

### Editor-Specific Patterns
- Editor UI uses exclusive world access: `fn editor_ui(world: &mut World)`
- "Collect then iterate" pattern for complex queries (avoids borrow conflicts)
- Selection uses resource + marker component sync (`EditorSelection` + `EditorSelected`)
- Gizmos only visible when `EditorPlayState::Stopped`

### Project Conventions
- Engine re-exports bevy via `sandbox_engine::prelude`
- Games depend on `sandbox_engine`, not bevy directly
- Editor-only code stays in `sandbox_editor`
- Game-specific code stays in game crate
- New `.rs` files need SPDX header:
  ```rust
  // SPDX-FileCopyrightText: 2026 the Sandbox contributors
  // SPDX-License-Identifier: GPL-3.0-or-later
  ```
- Project is REUSE compliant - all source files must have SPDX headers

## Git Workflow

- Create feature branches off `main` for all changes
- Use `gh pr create` for pull requests
- Squash merge PRs, delete branch after merge

## Key Files

- `crates/sandbox_engine/src/lib.rs` - SandboxPlugin definition
- `crates/sandbox_engine/src/editor_state.rs` - Play/pause/stop state machine, snapshot/restore
- `crates/sandbox_editor/src/main.rs` - Editor UI layout
- `crates/sandbox_editor/src/ui/hierarchy.rs` - Scene hierarchy panel
- `crates/sandbox_editor/src/ui/inspector.rs` - Entity inspector panel
- `crates/sandbox_editor/src/gizmo.rs` - Transform gizmo interaction
- `crates/sandbox_editor/src/selection.rs` - Entity selection system
- `crates/spaceminer/src/main.rs` - Game loop and movement systems
- `PRD.md` - Roadmap and status tracking
- `GDD.md` - Spaceminer game design

## Architecture Notes

### SandboxPlugin
Bundles common 2D game setup:
- Default plugins (windowing, rendering, etc.)
- 2D camera
- Common game systems

### Editor Layout
- Top: Toolbar with play/pause/stop controls
- Left panel: Scene hierarchy (entity tree with selection)
- Center: Viewport with grid and transform gizmos
- Right panel: Inspector (component editing for selected entity)
- Future: Menu bar, console/logs

### Spaceminer Movement
- `Velocity` component stores current velocity
- `Ship` marker component identifies the player
- Input system reads WASD, applies thrust to velocity
- Movement system applies velocity to transform
- Light drag applied each frame to prevent infinite drift

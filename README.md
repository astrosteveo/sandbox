# Sandbox Engine

[![REUSE compliant](https://reuse.software/badge/reuse-compliant.svg)](https://reuse.software/)

A Bevy-based 2D game engine with an integrated editor.

## Project Structure

- **sandbox_engine** - Core engine library with common 2D setup
- **sandbox_editor** - Editor application with egui-based UI
- **spaceminer** - Demo game for dogfooding the engine

## Building

```bash
cargo build --workspace
```

## Running

```bash
# Run the editor
cargo run -p sandbox_editor

# Run the spaceminer demo
cargo run -p spaceminer
```

## Spaceminer Controls

- `W` - Thrust up
- `S` - Thrust down
- `A` - Thrust left
- `D` - Thrust right

## License

This project is licensed under the GPL-3.0-or-later. See [LICENSE](LICENSE) for details.

# OPGAME - GTA SA-like Game in Rust

A bare-minimum game framework inspired by Grand Theft Auto: San Andreas, built from scratch in Rust.

## Features (Minimal Base)
- **Windowed application** with resizable window
- **Basic camera system** with third-person view following the player
- **Player entity** with health, position, and rotation
- **Input handling** (WASD for movement, Space for jump, ESC to exit)
- **Renderer** using WGPU for cross-platform graphics

## Building

```bash
cargo build --release
cargo run --release
```

## Controls
- **W/A/S/D** - Move forward/left/backward/right
- **Space** - Jump (placeholder)
- **ESC** - Exit game

## Architecture

The project is organized as follows:

- `main.rs` - Entry point and event loop
- `game.rs` - Main game logic and state management
- `player.rs` - Player entity and stats
- `camera.rs` - Camera system with view/projection matrices
- `entity.rs` - Base entity trait for game objects
- `renderer.rs` - WGPU rendering backend

## Next Steps

You can extend this with:
- **Map/World** - Load and render 3D maps
- **NPCs** - AI characters and interactions
- **Vehicles** - Car physics and driving
- **Combat** - Weapons, damage, and fighting
- **Missions** - Quest system
- **UI** - HUD, menus, dialog
- **Audio** - Sound effects and music

## Dependencies

- **winit** - Window management and events
- **wgpu** - Graphics API abstraction
- **glam** - Math library (vectors, matrices)
- **serde** - Serialization (for saving/loading)

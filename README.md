# OPGAME - GTA SA-like Game in Rust

A bare-minimum game framework inspired by Grand Theft Auto: San Andreas, built from scratch in Rust.

## Current Features
- **Resizable game window**
- **Third-person orbit camera** with mouse look and scroll-wheel zoom
- **Player movement** relative to camera heading
- **Procedural starter map layout** with roads and city blocks
- **Player entity** with health, weapons, and movement physics
- **NPC, vehicle, combat, mission, UI, and audio systems** ready to be expanded
- **OpenGL renderer foundation**

## Building

```bash
cargo build --release
cargo run --release
```

## Controls
- **W/A/S/D** - Move relative to camera
- **Mouse** - Orbit camera
- **Mouse wheel** - Zoom in/out
- **Space** - Jump placeholder
- **Shift** - Sprint
- **Left mouse / F** - Fire weapon
- **R** - Reload
- **1/2/3** - Select weapon
- **P** - Pause
- **ESC** - Exit game

## Architecture

The project is organized as follows:

- `main.rs` - Entry point and event loop
- `game.rs` - Main game logic and state management
- `player.rs` - Player entity and stats
- `camera.rs` - Third-person orbit camera with view/projection matrices
- `world.rs` - Procedural map, roads, buildings, time, and weather
- `entity.rs` - Base entity trait and world objects
- `renderer.rs` - OpenGL rendering backend
- `npc.rs` - NPC and AI state management
- `vehicle.rs` - Vehicle simulation
- `combat.rs` - Combat system
- `physics.rs` - Physics helpers
- `mission.rs` - Mission system
- `ui.rs` - HUD/UI state
- `sound.rs` - Audio system foundation

## Next Steps

The next major rendering milestone is to add a real OpenGL context, shaders, vertex buffers, and draw the procedural roads/buildings as 3D geometry. After that, the starter map can grow into a streamed city map with models, textures, collision, NPCs, and vehicles.

## Dependencies

- **winit** - Window management and events
- **glutin/glutin-winit** - OpenGL context support
- **gl** - OpenGL bindings
- **glam** - Math library (vectors, matrices)
- **serde** - Serialization (for saving/loading)

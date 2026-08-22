# OPGAME

A from-scratch third-person open-world game framework written in Rust.

## Direction

The goal is to build a game that lets players create their own games without needing to write code—unless it’s absolutely necessary.

I’m planning to develop most of the core game first, then add an editor mode where players can edit the map, create their own missions, and customize other parts of the game.

Feel free to contribute in any way you can. Using tools like ChatGPT or Copilot is completely fine—I use AI myself to help develop the project.

## Asset Policy

Only assets that we created ourselves or have explicit permission/licenses to redistribute should be committed to this repository.

Do not add:
- ripped or extracted game assets
- proprietary characters or character models
- proprietary vehicles or vehicle models
- copyrighted game textures, sounds, animations, maps, or fonts without redistribution rights
- logos, trademarks, or branding copied from other games or companies

References to other games may be useful during private design discussion, but the repository should describe the project's own mechanics and assets without presenting another game's identity as part of OPGAME.

Before adding a third-party asset, record its source and license/permission in the repository documentation and verify that redistribution is permitted.

## Building

```bash
python tools/build.py
```

## Dependencies

- **winit** - Window management and events
- **glutin/glutin-winit** - Cross-platform OpenGL context, surface, and proc address loading
- **gl_generator** - Build-time OpenGL compatibility bindings
- **glam** - Math library (vectors, matrices)
- **serde** - Serialization (for saving/loading)

A TrueType font is needed for on-screen text. The packaged build looks in `assets/fonts/` first and then falls back to platform system fonts. Any font committed to the repository must have redistribution rights compatible with the project.

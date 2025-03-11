# WLRS - Dynamic Wallpaper for Wayland

A Rust-based dynamic wallpaper engine for Wayland compositors, inspired by Wallpaper Engine.

## Overview

WLRS is a dynamic wallpaper system that brings animated backgrounds to Wayland desktops. It uses a client-daemon architecture with Bevy's entity-component system (ECS) for rendering and animation.

## Architecture

- **Frontend** - CLI tool for user interaction and control
- **Daemon** - Background service that renders wallpapers using Bevy
- **Common** - Shared code for IPC communication between frontend and daemon

## Features (MVP)

- [x] Daemon process with configurable framerate
- [x] IPC communication between CLI and daemon
- [ ] Simple wallpaper loading from directory
- [ ] Basic manifest file (TOML) support
- [ ] Static image wallpaper support
- [ ] Simple sprite animation

## Wallpaper Structure

Each wallpaper will have a minimalist directory structure:

```
my-wallpaper/
├── manifest.toml      # Basic wallpaper configuration
└── assets/            # Images and other media
    └── background.png
```

## Manifest Format

A minimal `manifest.toml` for the MVP:

```toml
[wallpaper]
name = "My Wallpaper"
author = "Your Name"
version = "1.0.0"

[settings]
fps = 30
scale_mode = "fill"  # Options: fill, fit, stretch, center, tile

[background]
image = "assets/background.png"
color = "#000000"  # Optional background color
```

For more dynamic wallpapers, you can add effects:

```toml
[[effects]]
name = "rain"
effect_type = "weather"  # Options: particles, weather, overlay, custom
script = "scripts/rain.lua"

[[effects]]
name = "floating_leaves"
effect_type = "particles"
image = "assets/leaf.png"
params = { count = 100, speed = 1.5 }
```

## Extension Points

Though the MVP is minimal, the architecture is designed for easy extension:

1. **Component System** - Bevy ECS allows adding new components without changing core code
2. **Message-based Architecture** - New commands can be added to the IPC system
3. **Manifest Format** - Can be extended with new sections and properties
4. **Asset Loading** - The loader can be enhanced to support more file types

## Todo List (MVP)

- [ ] Create basic wallpaper directory loader
- [ ] Implement simple TOML manifest parser
- [ ] Add sprite/image rendering
- [ ] Create wallpaper switching command
- [ ] Add configuration persistence

## Future Extensions

1. Scripting support (Lua/WASM)
2. Advanced animation systems
3. Interactive elements
4. Audio-reactive components
5. Multiple monitor support
6. Community repository for sharing

## Usage (MVP)

```bash
# Load a wallpaper
wlrs load /path/to/wallpaper

# Set framerate
wlrs set-framerate --fps 60

# Start/stop the daemon
wlrs start
wlrs stop
```

## Contributing

Contributions are welcome! The focus is on building a solid, extensible foundation first.

## License

[MIT License](LICENSE)
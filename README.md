# WLRS - Dynamic Wallpaper for Wayland

A Rust-based dynamic wallpaper engine for Wayland compositors that brings animated and interactive backgrounds to your desktop.

## Overview

WLRS (Wayland Live Rendering System) is a dynamic wallpaper system for Wayland desktops. It uses a client-daemon architecture with WebGPU (wgpu) for high-performance rendering and Lua for scripting support.

## Features

- ✅ Seamless integration with Wayland compositors
- ✅ Multi-monitor support with per-monitor wallpaper configuration
- ✅ Static image wallpapers
- ✅ Solid color backgrounds
- ✅ Combined image + color backgrounds
- ✅ Shader-based visual effects
  - ✅ Wave distortion effect
  - ✅ Glitch effect
  - ✅ Gaussian blur effect
- ✅ Multiple effects can be layered and combined
- ✅ Configurable framerate for animations
- ✅ Simple and intuitive CLI interface
- 🚧 Lua scripting support for custom animations
- 🚧 Particle system effects

## Architecture

- **Frontend** - CLI tool for user interaction and control
- **Daemon** - Background service that renders wallpapers using wgpu
- **Common** - Shared code for IPC communication and wallpaper definitions

## Installation

### Dependencies

- Rust 1.70+
- Wayland libraries (`libwayland-dev`)
- OpenGL or Vulkan libraries (for GPU rendering)

### Building from source

```bash
git clone https://github.com/yourusername/wlrs.git
cd wlrs
cargo build --release
```

### Installing the binary

```bash
cargo install --path .
```

## Usage

### Starting the daemon

```bash
# Start the daemon service
wlrs-daemon
```

### Managing wallpapers

```bash
# List available wallpapers
wlrs list-wallpapers

# Set a wallpaper for all monitors
wlrs set-wallpaper "Wallpaper Name"

# Set wallpaper for a specific monitor
wlrs set-wallpaper "Wallpaper Name" --monitor "Monitor Name"

# Query active wallpapers
wlrs query
```

## Wallpaper Structure

Each wallpaper has a simple directory structure:

```
my-wallpaper/
├── manifest.toml      # Wallpaper configuration
├── assets/            # Images and other media files
│   ├── background.png
│   └── overlay.png
└── scripts/           # Optional Lua scripts
    └── animation.lua
```

## Manifest Format

### Basic wallpaper with image background

```toml
name = "Simple Wallpaper"
author = "Your Name"
version = "1.0.0"
description = "A simple static wallpaper"
fps = 0  # 0 for static wallpapers
scale_mode = "fill"  # Options: fill, fit, stretch, center, tile

# Image background
background = "assets/background.png"
```

### Wallpaper with color background

```toml
name = "Solid Color"
author = "Your Name"
version = "1.0.0"
description = "A solid color wallpaper"
fps = 0
scale_mode = "fill"

# Color background
background = "#0066CC"  # CSS-style hex colors
```

### Wallpaper with combined image and color

```toml
name = "Combined Background"
author = "Your Name"
version = "1.0.0"
description = "Image with background color"
fps = 0
scale_mode = "fill"

[background]
image = "assets/background.png"
color = "#000033"  # Dark blue background color
```

### Wallpaper with effects

```toml
name = "Effect Demo"
author = "Your Name"
version = "1.0.0"
description = "Wallpaper with visual effects"
fps = 60  # Set to desired framerate for animated effects
scale_mode = "fill"

# Background configuration
[background]
image = "assets/background.png"
color = "#000033"

# Wave effect
[[effects]]
name = "wave-effect"
effect_type = { shader = "wave" }
image = "assets/background.png"
z_index = 10
opacity = 0.7

# Glitch effect
[[effects]]
name = "glitch-effect"
effect_type = { shader = "glitch" }
image = "assets/overlay.png"
z_index = 20
opacity = 0.9
```

## Supported Effect Types

- Shader effects:
  - `wave`: Creates a wavy distortion effect
  - `glitch`: Creates a digital glitch/RGB split effect
  - `gaussian`: Applies a Gaussian blur (coming soon)
  - `custom`: Custom WGSL shader support (coming soon)

- Other effects:
  - `particles`: Particle system effects (coming soon)
  - `image`: Static image overlay

## Creating Custom Wallpapers

1. Create a directory for your wallpaper with the structure shown above
2. Create a `manifest.toml` file with your wallpaper configuration
3. Add your images to the `assets/` directory
4. Add effects as needed in the manifest
5. Place the wallpaper in `~/.local/share/wlrs/wallpapers/` or use the `--path` option to use wallpapers from custom locations

## Troubleshooting

### Common Issues

- **Blank screen**: Ensure your compositor supports Wayland layer shell protocol
- **High CPU usage**: Consider lowering the FPS in the manifest or using static wallpapers
- **Artifacts/glitches**: Check GPU driver compatibility or try simpler effects

### Logs

The daemon logs can help diagnose issues:

```bash
# Run with verbose logging
RUST_LOG=debug wlrs-daemon
```

## Contributing

Contributions are welcome! Feel free to:

- Report bugs and request features using Issues
- Submit Pull Requests for bug fixes and features
- Improve documentation and examples
- Share your custom wallpapers and effects

Please follow the Rust code style and include tests for new features.

## License

[MIT License](LICENSE)
> [!WARNING]
> The `daemon` is in the process of a major rewrite, for details, see #2

# WLRS - Dynamic Wallpaper for Wayland

A Rust-based dynamic wallpaper engine for Wayland compositors that brings animated and interactive backgrounds to your desktop. WLRS supports shader-based visual effects, animated wallpapers, and a flexible layer system.

## Overview

WLRS (`wallpape-rs`) is a dynamic wallpaper system for Wayland desktops. It uses a client-daemon architecture with WebGPU (wgpu) for high-performance rendering and Lua for scripting support.

## Features

- ✅ Seamless integration with Wayland compositors
- ✅ Multi-monitor support with per-monitor wallpaper configuration
- ✅ Static image wallpapers
- ✅ Solid color backgrounds
- ✅ Combined image + color backgrounds
- ✅ Shader-based visual effects
  - ✅ Wave distortion effect with dynamic animation
  - ✅ Glitch effect with customizable intensity
  - ✅ Gaussian blur effect with configurable radius
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
git clone https://github.com/werdxz/wlrs.git
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
framerate = 30        # Visual refresh rate (FPS)
tickrate = "compositor"  # Animation update rate - sync with compositor
scale_mode = "fill"

# Background color layer
[[layers]]
name = "background-color"
content = "#000033"  # Dark blue
z_index = -1000

# Background image layer (without effects)
[[layers]]
name = "background-image"
content = "assets/background.png"
z_index = -500

# Wave effect as a separate layer
[[layers]]
name = "wave-effect"
content = "assets/background.png"  # This image is used as the texture for the shader
effect_type = { shader = "wave" }
z_index = 500  # Higher z-index to render on top
opacity = 1.0  # Full opacity for maximum effect
params = { amplitude = 0.9, frequency = 0.4, speed = 2.0 }

# Glitch effect on top of everything
[[layers]]
name = "glitch-effect"
content = "assets/overlay.png"  
effect_type = { shader = "glitch" }
z_index = 600  # Even higher z-index to render on top of wave
opacity = 0.8
params = { intensity = 0.8, frequency = 0.5 }
```

## Supported Effect Types

- Shader effects:
  - `wave`: Creates a wavy distortion effect with customizable amplitude and frequency
    - Parameters: `amplitude` (0.0-1.0), `frequency` (0.0-1.0), `speed` (multiplier)
  - `glitch`: Creates a digital glitch/RGB split effect
    - Parameters: `intensity` (0.0-1.0), `frequency` (0.0-1.0)
  - `gaussian`: Applies a Gaussian blur
    - Parameters: `radius` (pixel radius of blur)
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

## Credits

- Wave, glitch, and gaussian effects enhancements by [werdxz](https://github.com/werdxz)
- WGPU rendering framework using the wgpu-rs crate
- Wayland integration using smithay-client-toolkit

## License

[MIT License](LICENSE)

# Creating Custom Wallpapers for WLRS

This guide explains how to create custom wallpapers for the WLRS Wayland wallpaper engine.

## Basic Wallpaper Structure

Each wallpaper is a directory with the following structure:

```
my-wallpaper/
├── manifest.toml      # Required configuration file
├── assets/            # Directory for images and other media
│   ├── background.png # Background image
│   └── overlay.png    # Optional overlay image
└── scripts/           # Optional directory for Lua scripts
    └── animation.lua  # Animation script
```

## Creating a Basic Wallpaper

### Step 1: Create the directory structure

```bash
mkdir -p my-wallpaper/assets
```

### Step 2: Add your background image

Place your background image in the `assets` directory:

```bash
cp path/to/your/image.png my-wallpaper/assets/background.png
```

### Step 3: Create the manifest file

Create a file called `manifest.toml` in the wallpaper directory:

```toml
name = "My Custom Wallpaper"
author = "Your Name"
version = "1.0.0"
description = "My first custom wallpaper"
fps = 0  # 0 for static wallpapers
scale_mode = "fill"  # Options: fill, fit, stretch, center, tile

# Simple image background
background = "assets/background.png"
```

### Step 4: Install the wallpaper

Copy your wallpaper directory to the WLRS wallpapers directory:

```bash
cp -r my-wallpaper ~/.local/share/wlrs/wallpapers/
```

### Step 5: Set as wallpaper

```bash
wlrs set-wallpaper "My Custom Wallpaper"
```

## Wallpaper with Solid Color Background

For a simple solid color background:

```toml
name = "Solid Blue"
author = "Your Name"
version = "1.0.0"
description = "A solid blue wallpaper"
fps = 0
scale_mode = "fill"

# Color background (CSS-style hex color)
background = "#0066CC"
```

## Wallpaper with Combined Background

For an image with a background color:

```toml
name = "Combined Background"
author = "Your Name"
version = "1.0.0"
description = "Image with colored background"
fps = 0
scale_mode = "fill"

[background]
image = "assets/background.png"
color = "#000033"  # Dark blue background color
```

## Adding Visual Effects

WLRS supports various visual effects that can be applied to your wallpaper.

### Wave Effect

The wave effect creates a fluid, wavy distortion of your image:

```toml
name = "Wave Effect Wallpaper"
author = "Your Name"
version = "1.0.0"
description = "A wallpaper with wave distortion effect"
fps = 60  # Required for animated effects
scale_mode = "fill"

# Background image
background = "assets/background.png"

# Wave effect
[[effects]]
name = "wave-effect"
effect_type = { shader = "wave" }
image = "assets/background.png"
z_index = 10  # Higher values render on top
opacity = 0.7  # 0.0 to 1.0
```

### Glitch Effect

The glitch effect creates a digital distortion/RGB split effect:

```toml
name = "Glitch Effect Wallpaper"
author = "Your Name"
version = "1.0.0"
description = "A wallpaper with digital glitch effect"
fps = 60
scale_mode = "fill"

# Background image
background = "assets/background.png"

# Glitch effect
[[effects]]
name = "glitch-effect"
effect_type = { shader = "glitch" }
image = "assets/overlay.png"  # Can be a different image
z_index = 20
opacity = 0.9
```

### Multiple Effects

You can combine multiple effects in one wallpaper:

```toml
name = "Multi-Effect Wallpaper"
author = "Your Name"
version = "1.0.0"
description = "A wallpaper with multiple effects"
fps = 60
scale_mode = "fill"

# Use a solid color background
[background]
color = "#000022"

# First effect layer
[[effects]]
name = "wave-effect"
effect_type = { shader = "wave" }
image = "assets/background.png"
z_index = 10
opacity = 0.8

# Second effect layer
[[effects]]
name = "glitch-effect"
effect_type = { shader = "glitch" }
image = "assets/overlay.png" 
z_index = 20
opacity = 0.6
```

## Tips for Creating Effective Wallpapers

### Image Selection

- Choose high-resolution images (at least as high as your screen resolution)
- Use images with good contrast that will work well with desktop icons
- Consider the overall color scheme of your desktop environment

### Performance Considerations

- Animated wallpapers (fps > 0) will use more resources
- Complex effects stacked together can impact performance
- Test your wallpaper on your hardware

### Opacity and Z-Index

- Use opacity to blend effects with the background
- Z-index determines rendering order (higher values on top)
- Experiment with different values to find the best visual result

## Scale Modes

WLRS supports different ways to scale your background image:

- `fill`: Scales the image to fill the screen, may crop edges
- `fit`: Scales the image to fit within the screen, may show borders
- `stretch`: Stretches the image to fill the screen (may distort)
- `center`: Places the image in the center without scaling
- `tile`: Repeats the image to fill the screen

## Troubleshooting

### Wallpaper Not Found

Make sure your wallpaper is in one of these locations:
- `~/.local/share/wlrs/wallpapers/`
- The current directory when running the command

### Image Not Displaying

Check that the paths in your manifest file match the actual locations of your image files relative to the wallpaper directory.

### Effects Not Working

For animated effects, ensure:
- FPS is set to a value greater than 0
- The image exists and is in the correct format

## Advanced Topics

### Custom Animation (Coming Soon)

WLRS will support Lua scripting for custom animations. A basic script will look like this:

```lua
-- animation.lua
function update(t, dt)
    -- t: current time in seconds
    -- dt: time since last frame in seconds
    
    -- Return parameters to control your effect
    return {
        amplitude = 0.1 + math.sin(t) * 0.05,
        frequency = 10 + math.cos(t * 0.5) * 5,
    }
end
```
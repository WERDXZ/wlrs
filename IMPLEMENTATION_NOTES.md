WebP Support Implementation for WLRS Wallpaper Engine

## Overview

We've added WebP support to the WLRS wallpaper engine. This includes support for both static and animated WebP images.

## Components Added

1. Added Bevy with WebP support to the daemon package
2. Created a BackgroundPlugin with WebP animation handling
3. Implemented ScaleMode processing logic for different display modes
4. Connected the wallpaper manager to the background rendering system
5. Added color parsing for background colors

## Key Files

- daemon/src/renderer/systems/background.rs - Background rendering with WebP support
- daemon/src/main.rs - Updated to use the background config
- daemon/src/renderer/mod.rs - Plugin initialization

## Future Improvements

1. Implement proper WebP animation frame extraction
2. Add proper buffer management for Wayland surfaces
3. Add transition effects between wallpapers
4. Support for other animated formats like GIF, MP4, etc.


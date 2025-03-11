#import "@preview/touying:0.5.3": *
#import themes.simple: *

#show: simple-theme.with(aspect-ratio: "16-9")

= WLRS - Dynamic Wallpaper for Wayland

== What I'm Building
- A Rust-based dynamic wallpaper system for Wayland compositors
- Inspired by Wallpaper Engine (popular on Windows)
- Fills a gap in the Linux ecosystem for animated backgrounds
- Client-daemon architecture with Bevy's entity-component system

== Why It Matters
- Wayland lacks dynamic wallpaper solutions that exist for other platforms
- Provides an extensible foundation for creative desktop environments
- Opportunity to build a robust Rust application with modern architecture
- Strong focus on performance and resource efficiency

== Current Implementation
- Client-daemon architecture with IPC communication
- WebP image support (both static and animated)
- Wallpaper manifest system with TOML configuration
- Different scaling modes (fill, fit, stretch, center, tile)

== Technical Architecture
- Frontend: CLI tool for user interaction
- Daemon: Background service with Bevy renderer
- Common: Shared code for IPC communication
- Manifest: TOML-based wallpaper configuration

```rust
// Background rendering system
fn update_webp_animations(
    time: Res<Time>,
    mut query: Query<(&mut WebpAnimation, &mut Sprite),
                With<WallpaperBackground>>,
) {
    for (mut animation, mut sprite) in query.iter_mut() {
        // Update timer with elapsed time
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() && animation.frames.len() > 1 {
            // Advance to the next frame with wrap-around
            animation.current_frame =
                (animation.current_frame + 1) % animation.frames.len();

            // Update sprite's texture
            sprite.image = animation.frames[animation.current_frame].clone();
        }
    }
}
```

== Sample Manifest
```yaml
[wallpaper]
name = "WebP Test"
author = "WLRS"
version = "1.0.0"
description = "A test wallpaper with WebP support"

[settings]
fps = 1
scale_mode = "fill"

[background]
image = "assets/background.webp"
color = "#000000"
)
```

== Guaranteed Features
- Complete wallpaper directory loading system
- Simple particle system for rain/snow effects
- User configuration and preferences system
- Simple scripting support

== Stretch Goals
- Switch to wgpu directly for custom shader support
- Audio reactivity for wallpapers
- Multiple monitor support with independent configurations
- Interactive elements (mouse interactions)
- Laptop power-state adaptation

== CLI Interface
```bash
# Load a wallpaper
wlrs load-wallpaper /path/to/wallpaper

# Set framerate
wlrs set-framerate --fps 60

# Start/stop the daemon
wlrs start
wlrs stop

# Set wallpaper by name
wlrs set-wallpaper "My Wallpaper"
```

== Scripting Engine
- Lua integration for dynamic effects:
```lua
-- Simple particle system in Lua
function update(dt)
  emit_particles({
    rate = 50,
    velocity = {0, -100, 0},
    color = {0.7, 0.7, 1.0, 0.8},
    lifetime = 5.0,
    size = {1.0, 3.0}
  })
end
```

== Thank You!
- GitHub: https://github.com/WERDXZ/wlrs
- Questions?

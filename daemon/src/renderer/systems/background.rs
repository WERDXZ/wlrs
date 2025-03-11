use bevy::prelude::*;
use common::manifest::ScaleMode;
use std::time::Duration;

/// Component that tracks animated WebP state
#[derive(Component)]
pub struct WebpAnimation {
    pub timer: Timer,
    pub frames: Vec<Handle<Image>>,
    pub current_frame: usize,
}

/// Component that marks the background image entity
#[derive(Component)]
pub struct WallpaperBackground;

/// Resource that contains configuration for the background
#[derive(Resource, Debug, Clone)]
pub struct BackgroundConfig {
    pub image_path: Option<String>,
    pub color: Option<Color>,
    pub scale_mode: ScaleMode,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            image_path: None,
            color: Some(Color::BLACK),
            scale_mode: ScaleMode::Fill,
        }
    }
}

pub struct BackgroundPlugin {
    /// Initial configuration for the background (optional)
    pub initial_config: Option<BackgroundConfig>,
}

impl Default for BackgroundPlugin {
    fn default() -> Self {
        Self {
            initial_config: None,
        }
    }
}

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the background config
        if let Some(config) = &self.initial_config {
            app.insert_resource(config.clone());
        } else {
            app.init_resource::<BackgroundConfig>();
        }

        // Add systems
        app.add_systems(Startup, setup_background)
            .add_systems(Update, (update_webp_animations, update_background));
    }
}

/// Helper function to create a background entity from a path
fn create_background_entity(commands: &mut Commands, asset_server: &AssetServer, path: &str) {
    info!("Creating background entity from path: {}", path);

    if path.ends_with(".webp") {
        // For WebP, set up animation support
        let texture_handle = asset_server.load(path);
        info!("Loaded WebP texture: {:?}", texture_handle);
        let mut s = Sprite::from_image(texture_handle.clone());
        s.custom_size = Some(Vec2::new(1920., 1080.));

        commands.spawn((
            s,
            WallpaperBackground,
            WebpAnimation {
                // For now, we'll use a dummy timer and just one frame
                timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
                frames: vec![texture_handle],
                current_frame: 0,
            },
        ));
    } else {
        // For other image formats
        info!("Loading regular image: {}", path);
        commands.spawn((
            Sprite::from_image(asset_server.load(path)),
            WallpaperBackground,
        ));
    }
}

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    background_config: Res<BackgroundConfig>,
) {
    // Create background entity
    commands.insert_resource(ClearColor(background_config.color.unwrap_or(Color::BLACK)));

    // If we have a background image, load it now
    if let Some(path) = &background_config.image_path {
        create_background_entity(&mut commands, &asset_server, path);
    }

    // Add camera for 2D rendering
    commands.spawn(Camera2d);
}

/// System to update WebP animations by cycling through frames
fn update_webp_animations(
    time: Res<Time>,
    mut query: Query<(&mut WebpAnimation, &mut Sprite), With<WallpaperBackground>>,
) {
    for (mut animation, mut sprite) in query.iter_mut() {
        // Update timer with elapsed time
        animation.timer.tick(time.delta());

        // Only advance to the next frame if:
        // 1. The timer has finished a cycle (based on FPS)
        // 2. We have more than one frame (actual animation)
        if animation.timer.just_finished() && animation.frames.len() > 1 {
            // Advance to the next frame with wrap-around
            animation.current_frame = (animation.current_frame + 1) % animation.frames.len();

            // Update the sprite's texture to the new frame
            sprite.image = animation.frames[animation.current_frame].clone();
        }
    }
}

fn update_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    background_config: Res<BackgroundConfig>,
    windows: Query<&Window>,
    mut query: Query<(Entity, &mut Sprite, &mut Transform), With<WallpaperBackground>>,
) {
    // Check if background_config has changed and we need to create a new background
    if background_config.is_changed() {
        info!("Background config changed: {:?}", *background_config);

        // Remove old background entities
        for (entity, _, _) in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Create a new background if path is provided
        if let Some(path) = &background_config.image_path {
            info!("Loading new background from: {}", path);
            create_background_entity(&mut commands, &asset_server, path);
        }

        // Update the clear color to match the background color
        commands.insert_resource(ClearColor(background_config.color.unwrap_or(Color::BLACK)));

        // Return early as we'll update the new entity on the next frame
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let window_size = Vec2::new(window.width(), window.height());

    for (_, mut sprite, mut transform) in query.iter_mut() {
        // Update sprite size based on scale mode
        match background_config.scale_mode {
            ScaleMode::Fill => {
                sprite.custom_size = Some(window_size);
            }
            ScaleMode::Fit => {
                // For a color background, fit is the same as fill
                sprite.custom_size = Some(window_size);
            }
            ScaleMode::Stretch => {
                sprite.custom_size = Some(window_size);
            }
            ScaleMode::Center => {
                // For center, we'll keep the original size
                sprite.custom_size = None;
            }
            ScaleMode::Tile => {
                // For tiling we'd need a different approach
                sprite.custom_size = Some(window_size);
            }
        }

        // Center the background
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

// Helper to calculate scaling for images based on scale mode
pub fn calculate_image_scale(image_size: Vec2, window_size: Vec2, scale_mode: &ScaleMode) -> Vec2 {
    match scale_mode {
        ScaleMode::Fill => {
            // Scale so the smaller dimension fits the window, may crop
            let scale_x = window_size.x / image_size.x;
            let scale_y = window_size.y / image_size.y;
            let scale = scale_x.max(scale_y);
            Vec2::new(scale, scale)
        }
        ScaleMode::Fit => {
            // Scale so the larger dimension fits the window, may have borders
            let scale_x = window_size.x / image_size.x;
            let scale_y = window_size.y / image_size.y;
            let scale = scale_x.min(scale_y);
            Vec2::new(scale, scale)
        }
        ScaleMode::Stretch => {
            // Stretch to fill the window, may distort
            Vec2::new(window_size.x / image_size.x, window_size.y / image_size.y)
        }
        ScaleMode::Center => {
            // Don't scale, just center
            Vec2::ONE
        }
        ScaleMode::Tile => {
            // For tiling, we'll handle this separately
            Vec2::ONE
        }
    }
}

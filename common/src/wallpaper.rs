use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::manifest::{Background, Effect, ManifestError, WallpaperManifest, ScaleMode, ShaderType, EffectType};

/// Errors that can occur when working with wallpapers
#[derive(Error, Debug)]
pub enum WallpaperError {
    #[error("Failed to read wallpaper directory: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] ManifestError),

    #[error("Invalid wallpaper: {0}")]
    ValidationError(String),

    #[error("Missing asset: {0}")]
    MissingAsset(String),
}

/// Represents a loaded wallpaper
#[derive(Debug, Clone)]
pub struct Wallpaper {
    /// The manifest data
    pub manifest: WallpaperManifest,

    /// The absolute path to the wallpaper directory
    pub path: PathBuf,
}

impl Wallpaper {
    pub fn test() -> Self {
        let path = PathBuf::from("/");
        // Construct an absolute path for the test image
        // This is needed because the renderer tries to load the image directly
        let test_image = path.join("test.png").to_string_lossy().to_string();

        Wallpaper {
            path,
            manifest: WallpaperManifest {
                name: "test".to_string(),
                author: "test".to_string(),
                version: "0.0.0".to_string(),
                description: "test".to_string(),
                fps: 30,
                scale_mode: ScaleMode::Fill,
                background: Background::Image(test_image.clone()),
                effects: vec![Effect {
                    name: "test-effect".to_string(),
                    effect_type: EffectType::Shader(ShaderType::Wave),
                    image: Some(test_image),
                    script: None,
                    z_index: 0,
                    opacity: 1.0,
                    params: HashMap::new(),
                }],
            },
        }
    }
    /// Load a wallpaper from a directory
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, WallpaperError> {
        let path = path.as_ref().to_path_buf();

        // Validate that it's a directory
        if !path.is_dir() {
            return Err(WallpaperError::ValidationError(format!(
                "Path is not a directory: {}",
                path.display()
            )));
        }

        // Look for manifest.toml
        let manifest_path = path.join("manifest.toml");
        if !manifest_path.exists() {
            return Err(WallpaperError::ValidationError(format!(
                "Manifest file not found at: {}",
                manifest_path.display()
            )));
        }

        // Parse the manifest
        let manifest = WallpaperManifest::from_file(&manifest_path)?;

        // Validate that the assets exist
        Self::validate_assets(&path, &manifest)?;

        Ok(Self { manifest, path })
    }

    /// Get the absolute path to an asset
    pub fn asset_path(&self, relative_path: &str) -> PathBuf {
        self.path.join(relative_path)
    }

    /// Validate that all assets referenced in the manifest exist
    fn validate_assets(
        wallpaper_path: &Path,
        manifest: &WallpaperManifest,
    ) -> Result<(), WallpaperError> {
        // Check background image
        match &manifest.background {
            Background::Image(image_path) => {
                let full_path = wallpaper_path.join(image_path);
                if !full_path.exists() {
                    return Err(WallpaperError::MissingAsset(format!(
                        "Background image not found: {image_path}"
                    )));
                }
            },
            Background::Combined { image, .. } => {
                let full_path = wallpaper_path.join(image);
                if !full_path.exists() {
                    return Err(WallpaperError::MissingAsset(format!(
                        "Background image not found: {image}"
                    )));
                }
            },
            _ => {}
        }

        // Check effects
        for effect in &manifest.effects {
            // Check image if specified
            if let Some(image_path) = &effect.image {
                let full_path = wallpaper_path.join(image_path);
                if !full_path.exists() {
                    return Err(WallpaperError::MissingAsset(format!(
                        "Effect image not found: {} for effect {}",
                        image_path, effect.name
                    )));
                }
            }

            // Check script if specified
            if let Some(script_path) = &effect.script {
                let full_path = wallpaper_path.join(script_path);
                if !full_path.exists() {
                    return Err(WallpaperError::MissingAsset(format!(
                        "Script not found: {} for effect {}",
                        script_path, effect.name
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get the name of the wallpaper
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// Get the author of the wallpaper
    pub fn author(&self) -> &str {
        &self.manifest.author
    }

    /// Get the version of the wallpaper
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// Get the description of the wallpaper
    pub fn description(&self) -> &str {
        &self.manifest.description
    }

    /// Get the FPS
    pub fn fps(&self) -> u32 {
        self.manifest.fps
    }

    /// Get the background image path if any
    pub fn background_image(&self) -> Option<PathBuf> {
        match &self.manifest.background {
            Background::Image(path) => Some(self.path.join(path)),
            Background::Combined { image, .. } => Some(self.path.join(image)),
            _ => None,
        }
    }

    /// Get the background color if any
    pub fn background_color(&self) -> Option<&str> {
        match &self.manifest.background {
            Background::Color(color) => Some(color),
            Background::Combined { color, .. } => Some(color),
            _ => None,
        }
    }

    /// Return true if the background has an image
    pub fn has_background_image(&self) -> bool {
        matches!(
            &self.manifest.background,
            Background::Image(_) | Background::Combined { .. }
        )
    }

    /// Return true if the background has a color
    pub fn has_background_color(&self) -> bool {
        matches!(
            &self.manifest.background,
            Background::Color(_) | Background::Combined { .. }
        )
    }

    /// Get the full background configuration
    pub fn background(&self) -> &Background {
        &self.manifest.background
    }

    /// Get the scale mode
    pub fn scale_mode(&self) -> &ScaleMode {
        &self.manifest.scale_mode
    }

    /// Get all effects
    pub fn effects(&self) -> &[Effect] {
        &self.manifest.effects
    }

    /// Get an effect by index
    pub fn effect(&self, index: usize) -> Option<&Effect> {
        self.manifest.effects.get(index)
    }

    /// Get an effect by name
    pub fn effect_by_name(&self, name: &str) -> Option<&Effect> {
        self.manifest.effects.iter().find(|e| e.name == name)
    }

    /// Get all layers in this wallpaper in rendering order
    pub fn get_layers(&self) -> Vec<Layer> {
        let mut layers = Vec::new();

        // Start with background
        match &self.manifest.background {
            Background::Image(image) => {
                layers.push(Layer::Background {
                    image_path: self.path.join(image),
                    color: None,
                });
            },
            Background::Color(color) => {
                layers.push(Layer::Color {
                    color: color.clone(),
                });
            },
            Background::Combined { image, color } => {
                layers.push(Layer::Background {
                    image_path: self.path.join(image),
                    color: Some(color.clone()),
                });
            },
            Background::None => {
                // No background, nothing to add
            }
        }

        // Add effect layers
        for effect in &self.manifest.effects {
            layers.push(Layer::from_effect(effect, &self.path));
        }

        // Sort by z-index
        layers.sort_by_key(|layer| layer.z_index());

        layers
    }
}

/// A visual layer in a wallpaper
#[derive(Debug, Clone)]
pub enum Layer {
    /// Background layer with image
    Background {
        /// Path to the background image
        image_path: PathBuf,
        /// Optional background color
        color: Option<String>,
    },
    /// Background with just a color
    Color {
        /// Background color (CSS-style hex or rgb/rgba)
        color: String,
    },
    /// Image layer
    Image {
        /// Name of the layer
        name: String,
        /// Path to the image
        image_path: PathBuf,
        /// Transparency (0.0 to 1.0)
        opacity: f32,
        /// Layer position (higher = on top)
        z_index: i32,
    },
    /// Particle effect layer
    Particle {
        /// Name of the effect
        name: String,
        /// Path to the particle image
        image_path: PathBuf,
        /// Path to the script if any
        script_path: Option<PathBuf>,
        /// Parameters for the effect
        params: HashMap<String, toml::Value>,
        /// Transparency (0.0 to 1.0)
        opacity: f32,
        /// Layer position (higher = on top)
        z_index: i32,
    },
    /// Shader effect layer
    Shader {
        /// Name of the effect
        name: String,
        /// Type of shader to use
        shader_type: ShaderType,
        /// Optional image for the shader
        image_path: Option<PathBuf>,
        /// Uniforms for the shader
        uniforms: HashMap<String, toml::Value>,
        /// Transparency (0.0 to 1.0)
        opacity: f32,
        /// Layer position (higher = on top)
        z_index: i32,
    },
}

impl Layer {
    /// Get the z-index for sorting
    fn z_index(&self) -> i32 {
        match self {
            Layer::Background { .. } => -1000,  // Always at the bottom
            Layer::Color { .. } => -999,        // Just above the background
            Layer::Image { z_index, .. } => *z_index,
            Layer::Particle { z_index, .. } => *z_index,
            Layer::Shader { z_index, .. } => *z_index,
        }
    }

    /// Create a layer from an effect
    fn from_effect(effect: &Effect, base_path: &Path) -> Self {
        match effect.effect_type {
            EffectType::Image => Layer::Image {
                name: effect.name.clone(),
                image_path: base_path.join(effect.image.as_ref().unwrap_or(&String::new())),
                opacity: effect.opacity,
                z_index: effect.z_index,
            },
            EffectType::Particles => Layer::Particle {
                name: effect.name.clone(),
                image_path: base_path.join(effect.image.as_ref().unwrap_or(&String::new())),
                script_path: effect.script.as_ref().map(|s| base_path.join(s)),
                params: effect.params.clone(),
                opacity: effect.opacity,
                z_index: effect.z_index,
            },
            EffectType::Shader(ref shader_type) => Layer::Shader {
                name: effect.name.clone(),
                shader_type: shader_type.clone(),
                image_path: effect.image.as_ref().map(|i| base_path.join(i)),
                uniforms: HashMap::new(), // In the future, extract shader-specific params
                opacity: effect.opacity,
                z_index: effect.z_index,
            },
        }
    }
}

/// A directory for storing and finding wallpapers
#[derive(Debug)]
pub struct WallpaperDirectory {
    /// Base path for wallpapers
    pub path: PathBuf,
}

impl WallpaperDirectory {
    /// Create a new wallpaper directory
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Ensure the wallpaper directory exists
    pub fn ensure_exists(&self) -> Result<(), std::io::Error> {
        if !self.path.exists() {
            fs::create_dir_all(&self.path)?;
        }
        Ok(())
    }

    /// List all available wallpapers
    pub fn list_wallpapers(&self) -> Result<Vec<String>, WallpaperError> {
        self.ensure_exists()?;

        let mut wallpapers = Vec::new();

        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let manifest_path = path.join("manifest.toml");
                if manifest_path.exists() {
                    // Get the directory name as the wallpaper name
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            wallpapers.push(name_str.to_string());
                        }
                    }
                }
            }
        }

        Ok(wallpapers)
    }
    /// Load a specific wallpaper by name
    pub fn load_wallpaper(&self, name: &str) -> Result<Wallpaper, WallpaperError> {
        let wallpaper_path = self.path.join(name);
        Wallpaper::load(wallpaper_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_wallpaper() {
        let dir = tempdir().unwrap();

        // Create a wallpaper directory
        let wallpaper_dir = dir.path().join("test-wallpaper");
        fs::create_dir_all(&wallpaper_dir).unwrap();

        // Create assets directory
        let assets_dir = wallpaper_dir.join("assets");
        fs::create_dir_all(&assets_dir).unwrap();

        // Create a scripts directory
        let scripts_dir = wallpaper_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).unwrap();

        // Create a dummy background image file
        let background_path = assets_dir.join("background.png");
        File::create(&background_path).unwrap();

        // Create a dummy particle image
        let particle_path = assets_dir.join("particle.png");
        File::create(&particle_path).unwrap();

        // Create a dummy script file
        let script_path = scripts_dir.join("particles.lua");
        File::create(&script_path).unwrap();

        // Create a manifest
        let manifest_path = wallpaper_dir.join("manifest.toml");
        let manifest_content = r#"
            name = "Test Wallpaper"
            author = "Test Author"
            version = "1.0.0"
            fps = 30
            scale_mode = "fill"
            background = "assets/background.png"

            [[effects]]
            name = "particles"
            effect_type = "particles"
            image = "assets/particle.png"
            script = "scripts/particles.lua"
        "#;

        let mut file = File::create(manifest_path).unwrap();
        file.write_all(manifest_content.as_bytes()).unwrap();

        // Load the wallpaper
        let wallpaper = Wallpaper::load(&wallpaper_dir).unwrap();

        // Check the wallpaper
        assert_eq!(wallpaper.name(), "Test Wallpaper");
        assert_eq!(wallpaper.fps(), 30);

        // Check background is an Image type
        match &wallpaper.manifest.background {
            Background::Image(path) => {
                assert_eq!(path, "assets/background.png");
            },
            _ => panic!("Expected Background::Image"),
        }

        assert_eq!(wallpaper.manifest.effects.len(), 1);
        assert_eq!(wallpaper.manifest.effects[0].name, "particles");
        assert_eq!(
            wallpaper.manifest.effects[0].effect_type,
            EffectType::Particles
        );
    }

    #[test]
    fn test_wallpaper_directory() {
        let dir = tempdir().unwrap();

        // Create wallpaper directory
        let wallpaper_dir = WallpaperDirectory::new(dir.path());
        wallpaper_dir.ensure_exists().unwrap();

        // Create two wallpaper subdirectories
        let wallpaper1_dir = dir.path().join("wallpaper1");
        fs::create_dir_all(&wallpaper1_dir).unwrap();

        let wallpaper2_dir = dir.path().join("wallpaper2");
        fs::create_dir_all(&wallpaper2_dir).unwrap();

        // Create manifest files
        let manifest1 = r#"
            name = "Wallpaper 1"
        "#;

        let manifest2 = r#"
            name = "Wallpaper 2"
        "#;

        fs::write(wallpaper1_dir.join("manifest.toml"), manifest1).unwrap();
        fs::write(wallpaper2_dir.join("manifest.toml"), manifest2).unwrap();

        // List wallpapers
        let wallpapers = wallpaper_dir.list_wallpapers().unwrap();

        // Check that both wallpapers are found
        assert_eq!(wallpapers.len(), 2);
        assert!(wallpapers.contains(&"wallpaper1".to_string()));
        assert!(wallpapers.contains(&"wallpaper2".to_string()));
    }

    #[test]
    fn test_wallpaper_layers() {
        // Create temp directory
        let dir = tempdir().unwrap();
        let wallpaper_dir = dir.path().join("layered-wallpaper");
        fs::create_dir_all(&wallpaper_dir).unwrap();

        // Create manifest file for a wallpaper with multiple layers
        let manifest = r###"
name = "Layered Wallpaper"
author = "Layer Test"
fps = 60
scale_mode = "fill"

[background]
image = "bg.png"
color = "#000000"

[[effects]]
name = "particles"
effect_type = "particles"
image = "particle.png"
script = "particle.lua"
z_index = 10
opacity = 0.8

[[effects]]
name = "shader-effect"
effect_type = { shader = "wave" }
z_index = 5
opacity = 0.5

[[effects]]
name = "overlay"
effect_type = "image"
image = "overlay.png"
z_index = 20
opacity = 1.0
"###;

        // Create dummy files
        fs::write(wallpaper_dir.join("bg.png"), b"dummy").unwrap();
        fs::write(wallpaper_dir.join("particle.png"), b"dummy").unwrap();
        fs::write(wallpaper_dir.join("particle.lua"), b"dummy").unwrap();
        fs::write(wallpaper_dir.join("overlay.png"), b"dummy").unwrap();
        fs::write(wallpaper_dir.join("manifest.toml"), manifest).unwrap();

        // Load the wallpaper
        let wallpaper = Wallpaper::load(&wallpaper_dir).unwrap();

        // Check the background type
        match &wallpaper.manifest.background {
            Background::Combined { image, color } => {
                assert_eq!(image, "bg.png");
                assert_eq!(color, "#000000");
            },
            _ => panic!("Expected Background::Combined"),
        }

        // Test layer extraction
        let layers = wallpaper.get_layers();

        // Should have 4 layers (background + 3 effects)
        assert_eq!(layers.len(), 4);

        // First layer should be background
        match &layers[0] {
            Layer::Background { image_path, color } => {
                assert!(image_path.ends_with("bg.png"));
                assert_eq!(color, &Some("#000000".to_string()));
            }
            _ => panic!("First layer should be background"),
        }

        // Check that layers are sorted by z-index
        // Last layer should be overlay (z-index 20)
        match &layers[3] {
            Layer::Image { name, .. } => {
                assert_eq!(name, "overlay");
            }
            _ => panic!("Last layer should be overlay image"),
        }

        // Second to last should be particles (z-index 10)
        match &layers[2] {
            Layer::Particle { name, .. } => {
                assert_eq!(name, "particles");
            }
            _ => panic!("Third layer should be particles"),
        }

        // Third to last should be shader (z-index 5)
        match &layers[1] {
            Layer::Shader { name, .. } => {
                assert_eq!(name, "shader-effect");
            }
            _ => panic!("Second layer should be shader"),
        }
    }
}

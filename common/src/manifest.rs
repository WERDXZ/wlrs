use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use thiserror::Error;

/// Errors that can occur when working with wallpaper manifests
#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Failed to read manifest file: {0}")]
    IoError(#[from] io::Error),

    #[error("Failed to parse manifest file: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid manifest: {0}")]
    ValidationError(String),
}

/// The root structure for a wallpaper manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperManifest {
    /// Metadata about the wallpaper
    pub wallpaper: WallpaperInfo,

    /// Performance and display settings
    #[serde(default)]
    pub settings: WallpaperSettings,

    /// Main background image configuration
    #[serde(default)]
    pub background: Background,

    /// Effects to apply to the wallpaper
    #[serde(default)]
    pub effects: Vec<Effect>,
}

/// Information about the wallpaper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperInfo {
    /// The name of the wallpaper
    pub name: String,

    /// The author of the wallpaper
    #[serde(default)]
    pub author: String,

    /// The version of the wallpaper
    #[serde(default = "default_version")]
    pub version: String,

    /// A description of the wallpaper
    #[serde(default)]
    pub description: String,
}

/// Settings for the wallpaper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperSettings {
    /// The default frames per second for the wallpaper
    /// Static wallpapers can use 0 or 1 for minimal resource usage
    #[serde(default = "default_fps")]
    pub fps: u32,

    /// Scale mode for the background image
    #[serde(default)]
    pub scale_mode: ScaleMode,
}

impl Default for WallpaperSettings {
    fn default() -> Self {
        Self {
            fps: default_fps(),
            scale_mode: ScaleMode::Fill,
        }
    }
}

/// Background image configuration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Background {
    /// Path to the background image, relative to the wallpaper directory
    pub image: Option<String>,

    /// Color to use when no image is provided or as a base color
    #[serde(default)]
    pub color: Option<String>,
}

/// Effect applied to the wallpaper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Effect {
    /// Name of the effect
    pub name: String,

    /// Type of effect
    pub effect_type: EffectType,

    /// Path to an image for the effect, if applicable
    #[serde(default)]
    pub image: Option<String>,

    /// Script to drive the effect, if applicable
    #[serde(default)]
    pub script: Option<String>,

    /// Additional parameters for the effect
    #[serde(default)]
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

/// Type of effect
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    /// Simple particles
    Particles,
    /// Weather effects
    Weather,
    /// Floating overlay images
    Overlay,
    /// Custom effect defined by a script
    Custom,
}

/// Scale mode for background images
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScaleMode {
    /// Scale to fill the screen, may crop
    #[default]
    Fill,
    /// Scale to fit the screen, may show borders
    Fit,
    /// Stretch to fill the screen, may distort
    Stretch,
    /// Center the image without scaling
    Center,
    /// Tile the image
    Tile,
}

/// Default functions for serde defaults
fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_fps() -> u32 {
    // Default to 30 FPS, but static wallpapers can use 0 or 1
    30
}

impl WallpaperManifest {
    /// Load a manifest from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ManifestError> {
        let content = fs::read_to_string(path)?;
        let manifest: WallpaperManifest = toml::from_str(&content)?;

        // Basic validation
        if manifest.wallpaper.name.is_empty() {
            return Err(ManifestError::ValidationError(
                "Wallpaper name cannot be empty".to_string(),
            ));
        }

        Ok(manifest)
    }

    /// Save the manifest to a TOML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ManifestError> {
        let content =
            toml::to_string(self).map_err(|e| ManifestError::ValidationError(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_minimal_manifest() {
        let toml_str = r#"
            [wallpaper]
            name = "My Wallpaper"
        "#;

        let manifest: WallpaperManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.wallpaper.name, "My Wallpaper");
        assert_eq!(manifest.wallpaper.version, "1.0.0"); // Default
        assert_eq!(manifest.settings.fps, 30); // Default
        assert_eq!(manifest.settings.scale_mode, ScaleMode::Fill); // Default
    }

    #[test]
    fn test_deserialize_full_manifest() {
        let toml_str = r##"
            [wallpaper]
            name = "My Awesome Wallpaper"
            author = "John Doe"
            version = "2.1.0"
            description = "A beautiful dynamic wallpaper"
            
            [settings]
            fps = 60
            scale_mode = "fit"
            
            [background]
            image = "assets/background.png"
            color = "#000000"
            
            [[effects]]
            name = "rain"
            effect_type = "weather"
            script = "scripts/rain.lua"
            
            [[effects]]
            name = "floating_leaves"
            effect_type = "particles"
            image = "assets/leaf.png"
            params = { count = 100, speed = 1.5 }
        "##;

        let manifest: WallpaperManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.wallpaper.name, "My Awesome Wallpaper");
        assert_eq!(manifest.wallpaper.author, "John Doe");
        assert_eq!(manifest.wallpaper.version, "2.1.0");
        assert_eq!(manifest.settings.fps, 60);
        assert_eq!(manifest.settings.scale_mode, ScaleMode::Fit);
        assert_eq!(
            manifest.background.image,
            Some("assets/background.png".to_string())
        );
        assert_eq!(manifest.background.color, Some("#000000".to_string()));
        assert_eq!(manifest.effects.len(), 2);
        assert_eq!(manifest.effects[0].name, "rain");
        assert_eq!(manifest.effects[0].effect_type, EffectType::Weather);
        assert_eq!(
            manifest.effects[0].script,
            Some("scripts/rain.lua".to_string())
        );
        assert_eq!(manifest.effects[1].name, "floating_leaves");
        assert_eq!(manifest.effects[1].effect_type, EffectType::Particles);
        assert_eq!(
            manifest.effects[1].image,
            Some("assets/leaf.png".to_string())
        );
    }

    #[test]
    fn test_serialize_manifest() {
        let manifest = WallpaperManifest {
            wallpaper: WallpaperInfo {
                name: "Test Wallpaper".to_string(),
                author: "Test Author".to_string(),
                version: "1.0.0".to_string(),
                description: "Test Description".to_string(),
            },
            settings: WallpaperSettings {
                fps: 30,
                scale_mode: ScaleMode::Fill,
            },
            background: Background {
                image: Some("assets/bg.png".to_string()),
                color: None,
            },
            effects: vec![Effect {
                name: "particles".to_string(),
                effect_type: EffectType::Particles,
                image: Some("assets/particle.png".to_string()),
                script: None,
                params: std::collections::HashMap::new(),
            }],
        };

        let toml_str = toml::to_string(&manifest).unwrap();
        assert!(toml_str.contains("name = \"Test Wallpaper\""));
        assert!(toml_str.contains("image = \"assets/bg.png\""));
        assert!(toml_str.contains("effect_type = \"particles\""));
    }
}

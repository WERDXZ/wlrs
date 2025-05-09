use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::Path};
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

/// Background configuration for a wallpaper
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Background {
    /// A solid color background (CSS-style color string)
    Color(String),

    /// An image file background (path relative to wallpaper directory)
    Image(String),

    /// A combination of image with a base color
    Combined {
        /// Image path
        image: String,
        /// Base color shown behind the image or for transparent areas
        color: String,
    },

    /// No background specified (defaults to black)
    #[default]
    None,
}

/// Custom serialization for the Background enum
impl Serialize for Background {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Background::Color(color) => serializer.serialize_str(color),
            Background::Image(path) => serializer.serialize_str(path),
            Background::Combined { image, color } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("image", image)?;
                map.serialize_entry("color", color)?;
                map.end()
            }
            Background::None => serializer.serialize_none(),
        }
    }
}

/// Custom deserialization for the Background enum
impl<'de> Deserialize<'de> for Background {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // This visitor handles the various ways to specify a background
        struct BackgroundVisitor;

        impl<'de> serde::de::Visitor<'de> for BackgroundVisitor {
            type Value = Background;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string, map, or null value")
            }

            // Handle string values - could be either a color or an image path
            fn visit_str<E>(self, value: &str) -> Result<Background, E>
            where
                E: serde::de::Error,
            {
                // If it starts with #, it's a color
                if value.starts_with('#') || value.starts_with("rgb") || value.starts_with("hsl") {
                    Ok(Background::Color(value.to_string()))
                } else {
                    // Otherwise assume it's an image path
                    Ok(Background::Image(value.to_string()))
                }
            }

            // Handle null/missing value
            fn visit_none<E>(self) -> Result<Background, E>
            where
                E: serde::de::Error,
            {
                Ok(Background::None)
            }

            // Handle combined image and color in a map format
            fn visit_map<A>(self, mut map: A) -> Result<Background, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut image = None;
                let mut color = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "image" => {
                            if image.is_some() {
                                return Err(serde::de::Error::duplicate_field("image"));
                            }
                            image = Some(map.next_value::<String>()?);
                        }
                        "color" => {
                            if color.is_some() {
                                return Err(serde::de::Error::duplicate_field("color"));
                            }
                            color = Some(map.next_value::<String>()?);
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                match (image, color) {
                    (Some(image), Some(color)) => Ok(Background::Combined { image, color }),
                    (Some(image), None) => Ok(Background::Image(image)),
                    (None, Some(color)) => Ok(Background::Color(color)),
                    (None, None) => Ok(Background::None),
                }
            }
        }

        // Use our custom visitor to deserialize the value
        deserializer.deserialize_any(BackgroundVisitor)
    }
}

/// The root structure for a wallpaper manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperManifest {
    // Core metadata - directly at the root level
    /// Name of the wallpaper
    pub name: String,

    /// The author of the wallpaper (optional)
    #[serde(default)]
    pub author: String,

    /// The version of the wallpaper (defaults to 1.0.0)
    #[serde(default = "default_version")]
    pub version: String,

    /// A description of the wallpaper (optional)
    #[serde(default)]
    pub description: String,

    // Performance and display settings
    /// The frames per second for the wallpaper
    #[serde(default = "default_fps")]
    pub fps: u32,

    /// Scale mode for the background image
    #[serde(default)]
    pub scale_mode: ScaleMode,

    // Background configuration - single field that handles different formats
    /// Background configuration (color, image, or both)
    #[serde(default)]
    pub background: Background,

    // Visual effects
    /// Effects to apply to the wallpaper
    #[serde(default)]
    pub effects: Vec<Effect>,
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

    /// Z-index for layer ordering (higher values are rendered on top)
    #[serde(default)]
    pub z_index: i32,

    /// Opacity of the effect (0.0 to 1.0)
    #[serde(default = "default_opacity")]
    pub opacity: f32,

    /// Additional parameters for the effect
    #[serde(default)]
    pub params: HashMap<String, toml::Value>,
}

/// Type of effect
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    /// Simple particles
    Particles,
    /// Shader effects
    Shader(ShaderType),
    /// Static image
    Image,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ShaderType {
    Wave,
    Glitch,
    Gaussian,
    Custom(String),
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

fn default_opacity() -> f32 {
    1.0
}

impl WallpaperManifest {
    /// Load a manifest from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ManifestError> {
        let content = fs::read_to_string(path)?;
        let manifest: WallpaperManifest = toml::from_str(&content)?;

        // Basic validation
        if manifest.name.is_empty() {
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
            name = "My Wallpaper"
        "#;

        let manifest: WallpaperManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.name, "My Wallpaper");
        assert_eq!(manifest.version, "1.0.0"); // Default
        assert_eq!(manifest.fps, 30); // Default
        assert_eq!(manifest.scale_mode, ScaleMode::Fill); // Default
    }

    #[test]
    fn test_deserialize_full_manifest() {
        // Test new flat format with Background enum
        let toml_str = r###"
                            name = "My Awesome Wallpaper"
                            author = "John Doe"
                            version = "2.1.0"
                            description = "A beautiful dynamic wallpaper"
                            fps = 60
                            scale_mode = "fit"

                            [background]
                            image = "assets/background.png"
                            color = "#000000"

                            [[effects]]
                            name = "rain"
                            effect_type = "particles"
                            script = "scripts/rain.lua"
                            opacity = 0.8
                            z_index = 10

                            [[effects]]
                            name = "floating_leaves"
                            effect_type = "particles"
                            image = "assets/leaf.png"
                            opacity = 1.0
                            z_index = 5
                            params = { count = 100, speed = 1.5 }
                            "###;

        let manifest: WallpaperManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.name, "My Awesome Wallpaper");
        assert_eq!(manifest.author, "John Doe");
        assert_eq!(manifest.version, "2.1.0");
        assert_eq!(manifest.fps, 60);
        assert_eq!(manifest.scale_mode, ScaleMode::Fit);

        // Check the background is Combined type
        match &manifest.background {
            Background::Combined { image, color } => {
                assert_eq!(image, "assets/background.png");
                assert_eq!(color, "#000000");
            }
            _ => panic!("Expected Background::Combined"),
        }

        assert_eq!(manifest.effects.len(), 2);
        assert_eq!(manifest.effects[0].name, "rain");
        assert_eq!(manifest.effects[0].effect_type, EffectType::Particles);
        assert_eq!(
            manifest.effects[0].script,
            Some("scripts/rain.lua".to_string())
        );
        assert_eq!(manifest.effects[0].opacity, 0.8);
        assert_eq!(manifest.effects[0].z_index, 10);

        assert_eq!(manifest.effects[1].name, "floating_leaves");
        assert_eq!(manifest.effects[1].effect_type, EffectType::Particles);
        assert_eq!(
            manifest.effects[1].image,
            Some("assets/leaf.png".to_string())
        );
        assert_eq!(manifest.effects[1].opacity, 1.0);
        assert_eq!(manifest.effects[1].z_index, 5);
    }

    // Legacy format deserializer has been removed, so we'll test a non-legacy format
    #[test]
    fn test_deserialize_legacy_full_manifest() {
        // Using flat format since legacy is no longer supported
        let toml_str = r###"
                            name = "My Awesome Wallpaper"
                            author = "John Doe"
                            version = "2.1.0"
                            description = "A beautiful dynamic wallpaper"
                            fps = 60
                            scale_mode = "fit"

                            [background]
                            image = "assets/background.png"
                            color = "#000000"

                            [[effects]]
                            name = "rain"
                            effect_type = "particles"
                            script = "scripts/rain.lua"

                            [[effects]]
                            name = "floating_leaves"
                            effect_type = "particles"
                            image = "assets/leaf.png"
                            params = { count = 100, speed = 1.5 }
                            "###;

        let manifest: WallpaperManifest = toml::from_str(toml_str).unwrap();

        // Verify the parsed manifest
        assert_eq!(manifest.name, "My Awesome Wallpaper");
        assert_eq!(manifest.author, "John Doe");
        assert_eq!(manifest.version, "2.1.0");
        assert_eq!(manifest.fps, 60);
        assert_eq!(manifest.scale_mode, ScaleMode::Fit);

        // Check that the background is a Combined type with both image and color
        match &manifest.background {
            Background::Combined { image, color } => {
                assert_eq!(image, "assets/background.png");
                assert_eq!(color, "#000000");
            }
            _ => panic!("Expected Background::Combined"),
        }
    }

    #[test]
    fn test_serialize_manifest() {
        let manifest = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            fps: 30,
            scale_mode: ScaleMode::Fill,
            background: Background::Image("assets/bg.png".to_string()),
            effects: vec![Effect {
                name: "particles".to_string(),
                effect_type: EffectType::Particles,
                image: Some("assets/particle.png".to_string()),
                script: None,
                z_index: 0,
                opacity: 1.0,
                params: HashMap::new(),
            }],
        };

        let toml_str = toml::to_string(&manifest).unwrap();

        // Check that the serialized output uses the new flattened format
        assert!(toml_str.contains("name = \"Test Wallpaper\""));
        assert!(toml_str.contains("author = \"Test Author\""));

        // Background should be serialized as a string for the Image variant
        assert!(toml_str.contains("background = \"assets/bg.png\""));

        // Legacy fields should not be serialized
        assert!(!toml_str.contains("[wallpaper]"));
        assert!(!toml_str.contains("[settings]"));
        assert!(!toml_str.contains("[background]"));

        // Effects array should be properly serialized
        assert!(toml_str.contains("[[effects]]"));
        assert!(toml_str.contains("name = \"particles\""));
        assert!(toml_str.contains("effect_type = \"particles\""));

        // Now test a Combined background
        let manifest2 = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            fps: 30,
            scale_mode: ScaleMode::Fill,
            background: Background::Combined {
                image: "assets/bg.png".to_string(),
                color: "#000000".to_string(),
            },
            effects: vec![],
        };

        let toml_str2 = toml::to_string(&manifest2).unwrap();

        // Check that the Combined variant is serialized as a table
        assert!(toml_str2.contains("name = \"Test Wallpaper\""));
        assert!(toml_str2.contains("[background]"));
        assert!(toml_str2.contains("image = \"assets/bg.png\""));
        assert!(toml_str2.contains("color = \"#000000\""));
    }
}

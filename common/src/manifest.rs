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

/// Content configuration for a layer
#[derive(Debug, Serialize, Clone, PartialEq, Default)]
#[serde(untagged)]
pub enum LayerContent {
    /// A solid color (CSS-style color string)
    Color(String),

    /// An image file (path relative to wallpaper directory)
    Image(String),

    /// No content specified (defaults to transparent)
    #[default]
    None,
}

impl<'de> Deserialize<'de> for LayerContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.starts_with('#') || value.contains("rgba") {
            Ok(LayerContent::Color(value))
        } else {
            Ok(LayerContent::Image(value))
        }
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
    /// The frames per second for visual updates
    /// Special values:
    ///   "compositor": Use compositor-driven refresh rate (-1)
    ///   "static": No automatic updates (0)
    ///   "default": Default framerate (30 FPS)
    ///   Any number > 0: Specific framerate
    #[serde(default = "default_fps", deserialize_with = "deserialize_framerate")]
    pub framerate: i32,

    /// The ticks per second for animation logic
    /// Special values:
    ///   "compositor": Use compositor-driven update rate (-1)
    ///   "static": No animation updates (0)
    ///   "default": Same as "compositor" (-1)
    ///   Any number > 0: Specific tickrate
    #[serde(default = "default_tps", deserialize_with = "deserialize_tickrate")]
    pub tickrate: i32,

    /// Scale mode for images
    #[serde(default)]
    pub scale_mode: ScaleMode,

    // All visual layers including background and effects
    #[serde(default)]
    pub layers: Vec<Layer>,
}

/// A layer within a wallpaper (background or effect)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Layer {
    /// Name of the layer
    pub name: String,

    /// Content of the layer (color, image, or none)
    #[serde(default)]
    pub content: LayerContent,

    /// Type of layer effect to apply (none for basic layers)
    #[serde(default)]
    pub effect_type: Option<EffectType>,

    /// Z-index for layer ordering (higher values are rendered on top)
    #[serde(default = "default_z_index")]
    pub z_index: i32,

    /// Opacity of the layer (0.0 to 1.0)
    #[serde(default = "default_opacity")]
    pub opacity: f32,

    /// Additional parameters for the layer effect
    #[serde(default)]
    pub params: HashMap<String, toml::Value>,
}

impl Layer {
    /// Create a new background color layer
    pub fn new_background_color(color: &str) -> Self {
        Self {
            name: "background".to_string(),
            content: LayerContent::Color(color.to_string()),
            effect_type: None,
            z_index: -1000, // Very bottom layer
            opacity: 1.0,
            params: HashMap::new(),
        }
    }

    /// Create a new background image layer
    pub fn new_background_image(image_path: &str) -> Self {
        Self {
            name: "background".to_string(),
            content: LayerContent::Image(image_path.to_string()),
            effect_type: None,
            z_index: -999, // Just above background color
            opacity: 1.0,
            params: HashMap::new(),
        }
    }

    /// Create a new effect layer
    pub fn new_effect(
        name: &str,
        effect_type: EffectType,
        content: LayerContent,
        z_index: i32,
    ) -> Self {
        Self {
            name: name.to_string(),
            content,
            effect_type: Some(effect_type),
            z_index,
            opacity: 1.0,
            params: HashMap::new(),
        }
    }

    /// Check if this is a background layer
    pub fn is_background(&self) -> bool {
        self.z_index < 0 || self.name.contains("background")
    }
}

/// Type of effect
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    /// Simple particles effect
    Particles,

    /// Shader effect
    Shader(ShaderType),

    /// No effect (plain image or color)
    #[default]
    None,
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

/// Function to deserialize framerate from either a number or a string
fn deserialize_framerate<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, Unexpected, Visitor};
    use std::fmt;

    struct RateVisitor;

    impl<'de> Visitor<'de> for RateVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number or string representing a framerate")
        }

        fn visit_i8<E>(self, value: i8) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_i16<E>(self, value: i16) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_i32<E>(self, value: i32) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_u8<E>(self, value: u8) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_u16<E>(self, value: u16) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_u32<E>(self, value: u32) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_u64<E>(self, value: u64) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_str<E>(self, value: &str) -> Result<i32, E>
        where
            E: Error,
        {
            match value.to_lowercase().as_str() {
                "compositor" => Ok(-1), // Use compositor-driven timing
                "static" => Ok(0),      // No updates
                "default" => Ok(30),    // Default value for framerate is 30 Hz
                _ => {
                    // Try to parse as number
                    value.parse::<i32>().map_err(|_| {
                        Error::invalid_value(
                            Unexpected::Str(value),
                            &"a valid framerate value ('compositor', 'static', 'default', or a number)",
                        )
                    })
                }
            }
        }
    }

    deserializer.deserialize_any(RateVisitor)
}

/// Function to deserialize tickrate from either a number or a string
fn deserialize_tickrate<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, Unexpected, Visitor};
    use std::fmt;

    struct RateVisitor;

    impl<'de> Visitor<'de> for RateVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number or string representing a tickrate")
        }

        fn visit_i8<E>(self, value: i8) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_i16<E>(self, value: i16) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_i32<E>(self, value: i32) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_u8<E>(self, value: u8) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_u16<E>(self, value: u16) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_u32<E>(self, value: u32) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_u64<E>(self, value: u64) -> Result<i32, E>
        where
            E: Error,
        {
            Ok(value as i32)
        }

        fn visit_str<E>(self, value: &str) -> Result<i32, E>
        where
            E: Error,
        {
            match value.to_lowercase().as_str() {
                "compositor" => Ok(-1), // Use compositor-driven timing
                "static" => Ok(0),      // No updates
                "default" => Ok(-1),    // Default value for tickrate is compositor-driven
                _ => {
                    // Try to parse as number
                    value.parse::<i32>().map_err(|_| {
                        Error::invalid_value(
                            Unexpected::Str(value),
                            &"a valid tickrate value ('compositor', 'static', 'default', or a number)",
                        )
                    })
                }
            }
        }
    }

    deserializer.deserialize_any(RateVisitor)
}

/// Default functions for serde defaults
fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_fps() -> i32 {
    // Default to 30 FPS
    // Special values:
    //  Negative: Use compositor-driven refresh rate
    //   0: Static wallpaper (no automatic updates)
    30
}

fn default_tps() -> i32 {
    // By default, use compositor-driven update rate
    -1 // Use compositor-driven update rate by default
}

fn default_opacity() -> f32 {
    1.0
}

fn default_z_index() -> i32 {
    0 // Default z-index, backgrounds should use negative values
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

    /// Get all layers sorted by z-index (bottom to top)
    pub fn get_sorted_layers(&self) -> Vec<&Layer> {
        let mut layers: Vec<&Layer> = self.layers.iter().collect();
        layers.sort_by_key(|layer| layer.z_index);
        layers
    }

    /// Get a layer by name
    pub fn get_layer_by_name(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.name == name)
    }

    /// Get background layers (any with negative z-index or named "background")
    pub fn get_background_layers(&self) -> Vec<&Layer> {
        let mut backgrounds: Vec<&Layer> = self
            .layers
            .iter()
            .filter(|layer| layer.z_index < 0 || layer.name.contains("background"))
            .collect();
        backgrounds.sort_by_key(|layer| layer.z_index);
        backgrounds
    }

    /// Get the effective tickrate
    /// Returns a positive value for actual tickrate, 0 for static content,
    /// or negative value for compositor-driven timing
    pub fn get_tickrate(&self) -> i32 {
        self.tickrate
    }

    /// Check if this wallpaper is animated
    pub fn is_animated(&self) -> bool {
        // Consider animated if either framerate or tickrate is non-zero (i.e., not static)
        (self.framerate != 0 || self.tickrate != 0)
            && self.layers.iter().any(|layer| {
                matches!(
                    layer.effect_type,
                    Some(EffectType::Particles) | Some(EffectType::Shader(_))
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tickrate_defaults() {
        // Test default framerate and tickrate
        let manifest = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: 30,
            tickrate: -1,
            scale_mode: ScaleMode::Fill,
            layers: vec![],
        };

        // Framerate is 30, tickrate is compositor-driven (-1)
        assert_eq!(manifest.framerate, 30);
        assert_eq!(manifest.get_tickrate(), -1);

        // Test with explicit positive tickrate
        let manifest_with_tickrate = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: 30,
            tickrate: 60,
            scale_mode: ScaleMode::Fill,
            layers: vec![],
        };

        assert_eq!(manifest_with_tickrate.get_tickrate(), 60);

        // Test compositor-driven framerate with static animations
        let compositor_static = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: -1,
            tickrate: 0,
            scale_mode: ScaleMode::Fill,
            layers: vec![],
        };

        assert_eq!(compositor_static.framerate, -1);
        assert_eq!(compositor_static.get_tickrate(), 0);

        // Test compositor-driven for both
        let compositor_both = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: -1,
            tickrate: -1,
            scale_mode: ScaleMode::Fill,
            layers: vec![],
        };

        assert_eq!(compositor_both.framerate, -1);
        assert_eq!(compositor_both.get_tickrate(), -1);
    }

    #[test]
    fn test_is_animated() {
        // Create a layer with effect for testing
        let effect_layer = Layer {
            name: "test_effect".to_string(),
            content: LayerContent::None,
            effect_type: Some(EffectType::Shader(ShaderType::Wave)),
            z_index: 0,
            opacity: 1.0,
            params: HashMap::new(),
        };

        // Non-animated wallpaper (framerate=0, tickrate=None, has effect)
        let non_animated = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: 0,
            tickrate: 0,
            scale_mode: ScaleMode::Fill,
            layers: vec![effect_layer.clone()],
        };

        // Should not be animated because framerate=0 and tickrate=None (defaults to 0)
        assert!(!non_animated.is_animated());

        // Animated by framerate (framerate>0, tickrate=None, has effect)
        let animated_by_framerate = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: 30,
            tickrate: 0,
            scale_mode: ScaleMode::Fill,
            layers: vec![effect_layer.clone()],
        };

        // Should be animated because framerate>0 and has effect
        assert!(animated_by_framerate.is_animated());

        // Animated by tickrate (framerate=0, tickrate>0, has effect)
        let animated_by_tickrate = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: 0,
            tickrate: 60,
            scale_mode: ScaleMode::Fill,
            layers: vec![effect_layer.clone()],
        };

        // Should be animated because tickrate>0 and has effect
        assert!(animated_by_tickrate.is_animated());

        // Animated by compositor-driven framerate (framerate=-1, has effect)
        let animated_by_compositor = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: -1,
            tickrate: -1,
            scale_mode: ScaleMode::Fill,
            layers: vec![effect_layer.clone()],
        };

        // Should be animated because framerate=-1 (compositor-driven) and has effect
        assert!(animated_by_compositor.is_animated());

        // Non-animated because no effects (framerate>0, tickrate>0, no effects)
        let no_effects = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: 30,
            tickrate: 60,
            scale_mode: ScaleMode::Fill,
            layers: vec![Layer {
                name: "no_effect".to_string(),
                content: LayerContent::Color("#000000".to_string()),
                effect_type: None,
                z_index: 0,
                opacity: 1.0,
                params: HashMap::new(),
            }],
        };

        // Should not be animated despite framerate/tickrate because no layer has effects
        assert!(!no_effects.is_animated());

        // Non-animated despite compositor-driven framerate (-1) because no effects
        let compositor_no_effects = WallpaperManifest {
            name: "Test Wallpaper".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Test Description".to_string(),
            framerate: -1,
            tickrate: -1,
            scale_mode: ScaleMode::Fill,
            layers: vec![Layer {
                name: "no_effect".to_string(),
                content: LayerContent::Color("#000000".to_string()),
                effect_type: None,
                z_index: 0,
                opacity: 1.0,
                params: HashMap::new(),
            }],
        };

        // Should not be animated despite framerate=-1 because no layer has effects
        assert!(!compositor_no_effects.is_animated());
    }

    #[test]
    fn test_string_rate_deserialization() {
        // Test deserialization of string values for framerate and tickrate
        let toml_str = r#"
            name = "String Values Test"
            author = "Test Author"
            version = "1.0.0"
            framerate = "compositor"
            tickrate = "static"
        "#;

        let manifest: WallpaperManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.framerate, -1); // "compositor" -> -1
        assert_eq!(manifest.tickrate, 0); // "static" -> 0

        let toml_str = r#"
            name = "Default Values Test"
            author = "Test Author"
            version = "1.0.0"
            framerate = "default"
            tickrate = "default"
        "#;

        let manifest: WallpaperManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.framerate, 30); // "default" -> 30 for framerate
        assert_eq!(manifest.tickrate, -1); // "default" -> -1 for tickrate

        let toml_str = r#"
            name = "Mixed Values Test"
            author = "Test Author"
            version = "1.0.0"
            framerate = 60
            tickrate = "compositor"
        "#;

        let manifest: WallpaperManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.framerate, 60); // 60 -> 60
        assert_eq!(manifest.tickrate, -1); // "compositor" -> -1
    }
}

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::manifest::{
    EffectType, Layer, LayerContent, ManifestError, ScaleMode, ShaderType, WallpaperManifest,
};

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
        // Check assets for all layers
        for layer in &manifest.layers {
            // Check content images
            if let LayerContent::Image(image_path) = &layer.content {
                let full_path = wallpaper_path.join(image_path);
                if !full_path.exists() {
                    return Err(WallpaperError::MissingAsset(format!(
                        "Image not found: {image_path} for layer {}",
                        layer.name
                    )));
                }
            }

            // Check if layer has script parameters
            if let Some(script_path) = layer.params.get("script").and_then(|v| v.as_str()) {
                let full_path = wallpaper_path.join(script_path);
                if !full_path.exists() {
                    return Err(WallpaperError::MissingAsset(format!(
                        "Script not found: {script_path} for layer {}",
                        layer.name
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

    /// Get the framerate (FPS)
    /// Special values:
    ///   -1: Use compositor-driven refresh rate
    ///    0: Static wallpaper (no automatic updates)
    ///  >0: Specific framerate
    pub fn framerate(&self) -> i32 {
        self.manifest.framerate
    }

    /// Get the tickrate (defaults to framerate if not specified)
    /// Returns a positive value for actual tickrate, 0 for static content,
    /// or -1 for compositor-driven timing
    pub fn tickrate(&self) -> i32 {
        self.manifest.get_tickrate()
    }

    /// Get the scale mode
    pub fn scale_mode(&self) -> &ScaleMode {
        &self.manifest.scale_mode
    }

    /// Get all layers
    pub fn layers(&self) -> &[Layer] {
        &self.manifest.layers
    }

    /// Get a layer by index
    pub fn layer(&self, index: usize) -> Option<&Layer> {
        self.manifest.layers.get(index)
    }

    /// Get a layer by name
    pub fn layer_by_name(&self, name: &str) -> Option<&Layer> {
        self.manifest.layers.iter().find(|l| l.name == name)
    }

    /// Get layers with effects
    pub fn effect_layers(&self) -> Vec<&Layer> {
        self.manifest
            .layers
            .iter()
            .filter(|l| l.effect_type.is_some())
            .collect()
    }

    /// Get all layers in this wallpaper in rendering order
    pub fn get_layers(&self) -> Vec<RenderLayer> {
        let mut render_layers = Vec::new();

        // Convert manifest layers to render layers
        for layer in &self.manifest.layers {
            render_layers.push(RenderLayer::from_manifest_layer(layer, &self.path));
        }

        // Sort by z-index
        render_layers.sort_by_key(|layer| layer.z_index);

        dbg!(render_layers)
    }
}

/// A visual layer in a wallpaper for rendering
#[derive(Debug, Clone)]
pub struct RenderLayer {
    /// Name of the layer (may be empty for background layers)
    pub name: String,
    /// Layer position (higher = on top)
    pub z_index: i32,
    /// Transparency (0.0 to 1.0)
    pub opacity: f32,
    /// Layer type
    pub layer_type: LayerType,
}

/// Types of layers in a wallpaper
#[derive(Debug, Clone)]
pub enum LayerType {
    /// Solid color layer
    Color {
        /// Color value (CSS-style hex or rgb/rgba)
        color: String,
    },
    /// Static image layer
    Image {
        /// Path to the image
        image_path: PathBuf,
    },
    /// Particle effect layer
    Particle {
        /// Path to the particle image
        image_path: PathBuf,
        /// Path to the script if any
        script_path: Option<PathBuf>,
        /// Parameters for the effect
        params: HashMap<String, toml::Value>,
    },
    /// Shader effect layer
    Shader {
        /// Type of shader to use
        shader_type: ShaderType,
        /// Optional image for the shader
        image_path: Option<PathBuf>,
        /// Uniforms for the shader
        uniforms: HashMap<String, toml::Value>,
    },
}

impl RenderLayer {
    /// Create a render layer from a manifest layer
    pub fn from_manifest_layer(layer: &Layer, base_path: &Path) -> Self {
        let layer_type = match &layer.content {
            LayerContent::Color(color) => LayerType::Color {
                color: color.clone(),
            },
            LayerContent::Image(image) => LayerType::Image {
                image_path: base_path.join(image),
            },
            LayerContent::None => {
                // Empty layer, fallback to a transparent layer
                LayerType::Color {
                    color: "transparent".to_string(),
                }
            }
        };

        // Apply effect if present
        let layer_type = if let Some(effect_type) = &layer.effect_type {
            match effect_type {
                EffectType::Particles => {
                    // Get script path from params if available
                    let script_path = layer
                        .params
                        .get("script")
                        .and_then(|v| v.as_str())
                        .map(|s| base_path.join(s));

                    LayerType::Particle {
                        image_path: if let LayerContent::Image(img) = &layer.content {
                            base_path.join(img)
                        } else {
                            // Default to an empty image if not specified
                            PathBuf::new()
                        },
                        script_path,
                        params: layer.params.clone(),
                    }
                }
                EffectType::Shader(shader_type) => LayerType::Shader {
                    shader_type: shader_type.clone(),
                    image_path: if let LayerContent::Image(img) = &layer.content {
                        Some(base_path.join(img))
                    } else {
                        None
                    },
                    uniforms: layer.params.clone(),
                },
                EffectType::None => layer_type, // No effect, use original layer type
            }
        } else {
            layer_type // No effect, use original layer type
        };

        Self {
            name: layer.name.clone(),
            z_index: layer.z_index,
            opacity: layer.opacity,
            layer_type,
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
    use tempfile::tempdir;

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
}

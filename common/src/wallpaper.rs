use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;

use crate::manifest::{ManifestError, WallpaperManifest};

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
            return Err(WallpaperError::ValidationError(
                format!("Path is not a directory: {}", path.display())
            ));
        }
        
        // Look for manifest.toml
        let manifest_path = path.join("manifest.toml");
        if !manifest_path.exists() {
            return Err(WallpaperError::ValidationError(
                format!("Manifest file not found at: {}", manifest_path.display())
            ));
        }
        
        // Parse the manifest
        let manifest = WallpaperManifest::from_file(&manifest_path)?;
        
        // Validate that the assets exist
        Self::validate_assets(&path, &manifest)?;
        
        Ok(Self {
            manifest,
            path,
        })
    }
    
    /// Get the absolute path to an asset
    pub fn asset_path(&self, relative_path: &str) -> PathBuf {
        self.path.join(relative_path)
    }
    
    /// Validate that all assets referenced in the manifest exist
    fn validate_assets(wallpaper_path: &Path, manifest: &WallpaperManifest) -> Result<(), WallpaperError> {
        // Check background image
        if let Some(image_path) = &manifest.background.image {
            let full_path = wallpaper_path.join(image_path);
            if !full_path.exists() {
                return Err(WallpaperError::MissingAsset(
                    format!("Background image not found: {}", image_path)
                ));
            }
        }
        
        // Check effects
        for effect in &manifest.effects {
            // Check image if specified
            if let Some(image_path) = &effect.image {
                let full_path = wallpaper_path.join(image_path);
                if !full_path.exists() {
                    return Err(WallpaperError::MissingAsset(
                        format!("Effect image not found: {} for effect {}", image_path, effect.name)
                    ));
                }
            }
            
            // Check script if specified
            if let Some(script_path) = &effect.script {
                let full_path = wallpaper_path.join(script_path);
                if !full_path.exists() {
                    return Err(WallpaperError::MissingAsset(
                        format!("Script not found: {} for effect {}", script_path, effect.name)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the name of the wallpaper
    pub fn name(&self) -> &str {
        &self.manifest.wallpaper.name
    }
    
    /// Get the FPS
    pub fn fps(&self) -> u32 {
        self.manifest.settings.fps
    }
    
    /// Get the background image path if any
    pub fn background_image(&self) -> Option<PathBuf> {
        self.manifest.background.image.as_ref().map(|path| self.path.join(path))
    }
    
    /// Get the background color if any
    pub fn background_color(&self) -> Option<&str> {
        self.manifest.background.color.as_deref()
    }
    
    /// Get the scale mode
    pub fn scale_mode(&self) -> &crate::manifest::ScaleMode {
        &self.manifest.settings.scale_mode
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
            [wallpaper]
            name = "Test Wallpaper"
            author = "Test Author"
            version = "1.0.0"
            
            [settings]
            fps = 30
            scale_mode = "fill"
            
            [background]
            image = "assets/background.png"
            
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
        assert_eq!(wallpaper.manifest.background.image, Some("assets/background.png".to_string()));
        assert_eq!(wallpaper.manifest.effects.len(), 1);
        assert_eq!(wallpaper.manifest.effects[0].name, "particles");
        assert_eq!(wallpaper.manifest.effects[0].effect_type, crate::manifest::EffectType::Particles);
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
            [wallpaper]
            name = "Wallpaper 1"
        "#;
        
        let manifest2 = r#"
            [wallpaper]
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
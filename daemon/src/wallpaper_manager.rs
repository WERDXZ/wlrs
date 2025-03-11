use bevy::prelude::*;
use common::wallpaper::{Wallpaper, WallpaperDirectory, WallpaperError};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;

/// Manages wallpaper storage and loading in the daemon
#[derive(Resource)]
pub struct WallpaperManager {
    /// Directory where wallpapers are stored
    pub wallpaper_dir: WallpaperDirectory,
    
    /// Currently active wallpaper
    pub current_wallpaper: Option<Wallpaper>,
    
    /// Name of the currently active wallpaper
    pub current_name: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum WallpaperManagerError {
    #[error("Wallpaper error: {0}")]
    WallpaperError(#[from] WallpaperError),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Wallpaper not found: {0}")]
    NotFound(String),
    
    #[error("Failed to install wallpaper: {0}")]
    InstallError(String),
}

impl Default for WallpaperManager {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let wallpaper_dir = WallpaperDirectory::new(home_dir.join(".wlrs/wallpapers"));
        
        // Ensure the directory exists
        let _ = wallpaper_dir.ensure_exists();
        
        Self {
            wallpaper_dir,
            current_wallpaper: None,
            current_name: None,
        }
    }
}

impl WallpaperManager {
    /// Create a new wallpaper manager with a custom directory
    pub fn new<P: AsRef<Path>>(wallpaper_dir: P) -> Self {
        let wallpaper_dir = WallpaperDirectory::new(wallpaper_dir);
        
        // Ensure the directory exists
        let _ = wallpaper_dir.ensure_exists();
        
        Self {
            wallpaper_dir,
            current_wallpaper: None,
            current_name: None,
        }
    }
    
    /// List all available wallpapers
    pub fn list_wallpapers(&self) -> Result<Vec<common::types::WallpaperInfo>, WallpaperManagerError> {
        let wallpaper_names = self.wallpaper_dir.list_wallpapers()?;
        
        let mut result = Vec::new();
        for name in wallpaper_names {
            let path = self.wallpaper_dir.path.join(&name);
            result.push(common::types::WallpaperInfo {
                name,
                path: path.to_string_lossy().to_string(),
            });
        }
        
        Ok(result)
    }
    
    /// Load a wallpaper by name
    pub fn load_wallpaper(&mut self, name: &str) -> Result<&Wallpaper, WallpaperManagerError> {
        let wallpaper = self.wallpaper_dir.load_wallpaper(name)?;
        
        self.current_wallpaper = Some(wallpaper);
        self.current_name = Some(name.to_string());
        
        Ok(self.current_wallpaper.as_ref().unwrap())
    }
    
    /// Load a wallpaper from a path
    pub fn load_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<&Wallpaper, WallpaperManagerError> {
        let wallpaper = Wallpaper::load(path.as_ref())?;
        
        self.current_wallpaper = Some(wallpaper);
        self.current_name = Some(path.as_ref().to_string_lossy().to_string());
        
        Ok(self.current_wallpaper.as_ref().unwrap())
    }
    
    /// Install a wallpaper from a path
    pub fn install_wallpaper<P: AsRef<Path>>(&self, source_path: P, custom_name: Option<String>) -> Result<String, WallpaperManagerError> {
        // Validate the source path is a valid wallpaper
        let wallpaper = Wallpaper::load(&source_path)?;
        
        // Determine target name
        let name = if let Some(custom_name) = custom_name {
            custom_name
        } else {
            // Use directory name as default
            source_path.as_ref()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unnamed".to_string())
        };
        
        // Create target directory
        let target_dir = self.wallpaper_dir.path.join(&name);
        if target_dir.exists() {
            return Err(WallpaperManagerError::InstallError(
                format!("Wallpaper with name '{}' already exists", name)
            ));
        }
        
        // Copy files
        copy_dir_all(source_path, &target_dir)?;
        
        Ok(name)
    }
    
    /// Get the current wallpaper info
    pub fn get_current_wallpaper(&self) -> Option<common::types::CurrentWallpaper> {
        if let Some(name) = &self.current_name {
            Some(common::types::CurrentWallpaper {
                name: Some(name.clone()),
                path: self.current_wallpaper.as_ref().map(|w| 
                    w.path.to_string_lossy().to_string()
                ),
            })
        } else {
            None
        }
    }
}

// Helper function to recursively copy directories
fn copy_dir_all<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(src_path, dst_path)?;
        } else {
            fs::copy(src_path, dst_path)?;
        }
    }
    
    Ok(())
}
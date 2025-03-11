use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check if the daemon is running
    Ping(PingArgs),
    /// Start the daemon process
    Start(StartArgs),
    /// Enable or disable the daemon's active state
    SetState(SetStateArgs),
    /// Set the daemon's framerate
    SetFramerate(SetFramerateArgs),
    /// Load a wallpaper from a directory
    LoadWallpaper(LoadWallpaperArgs),
    /// Get information about the current wallpaper
    CurrentWallpaper(CurrentWallpaperArgs),
    /// List all available wallpapers
    ListWallpapers(ListWallpapersArgs),
    /// Install a wallpaper from a directory
    InstallWallpaper(InstallWallpaperArgs),
    /// Set the current wallpaper by name
    SetWallpaper(SetWallpaperArgs),
}

#[derive(Args, Debug)]
pub struct PingArgs {}

#[derive(Args, Debug)]
pub struct StartArgs {
    #[arg(short, long)]
    pub detach: bool,
}

#[derive(Args, Debug)]
pub struct SetStateArgs {
    #[arg(short, long)]
    pub enabled: bool,
}

#[derive(Args, Debug)]
pub struct SetFramerateArgs {
    /// Frames per second (FPS)
    #[arg(short, long)]
    pub fps: u32,
}

#[derive(Args, Debug)]
pub struct LoadWallpaperArgs {
    /// Path to the wallpaper directory
    #[arg(required = true)]
    pub path: String,
}

#[derive(Args, Debug)]
pub struct CurrentWallpaperArgs {}

#[derive(Args, Debug)]
pub struct ListWallpapersArgs {}

#[derive(Args, Debug)]
pub struct InstallWallpaperArgs {
    /// Path to the wallpaper directory
    #[arg(required = true)]
    pub path: String,
    
    /// Custom name for the wallpaper (defaults to directory name)
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Args, Debug)]
pub struct SetWallpaperArgs {
    /// Name of the wallpaper
    #[arg(required = true)]
    pub name: String,
}
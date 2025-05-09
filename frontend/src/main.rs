mod cli;

use clap::Parser;
use std::{fs, path::Path};

use common::{
    ipc::{IpcError, IpcSocket, Stream},
    types::{
        Checkhealth, GetCurrentWallpaper, GetInstallDirectory, ListWallpapers, LoadWallpaper,
        QueryActiveWallpapers, SetCurrentWallpaper, StopServer,
    },
};
use fs_extra::dir::{copy, CopyOptions};

fn main() -> Result<(), IpcError> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Ping(_) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send ping request
                    match client.request(Checkhealth) {
                        Ok(pong) => {
                            println!("Daemon is running: {pong:?}");
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to get response from daemon: {e:?}");
                            Err(e)
                        }
                    }
                }
                Err(_) => {
                    println!("Daemon is not running");
                    Ok(())
                }
            }
        }
        cli::Commands::LoadWallpaper(args) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Check if this is a wallpaper name (without path separators) or a path
                    if !args.path.contains('/') && !args.path.contains('\\') {
                        // This looks like just a name, use SetCurrentWallpaper
                        println!("Loading wallpaper by name: {}", args.path);
                        let request = SetCurrentWallpaper {
                            name: args.path,
                            monitor: None,
                        };

                        match client.request(request) {
                            Ok(response) => {
                                if response.success {
                                    println!("Wallpaper '{}' loaded successfully", response.name);
                                } else {
                                    eprintln!(
                                        "Failed to load wallpaper: {}",
                                        response
                                            .error
                                            .unwrap_or_else(|| "Unknown error".to_string())
                                    );
                                }
                                Ok(())
                            }
                            Err(e) => {
                                eprintln!("Failed to load wallpaper: {e:?}");
                                Err(e)
                            }
                        }
                    } else {
                        // This is a path, use LoadWallpaper
                        println!("Loading wallpaper from path: {}", args.path);
                        let request = LoadWallpaper { path: args.path };

                        match client.request(request) {
                            Ok(response) => {
                                if response.success {
                                    println!("Wallpaper '{}' loaded successfully", response.name);
                                } else {
                                    eprintln!(
                                        "Failed to load wallpaper: {}",
                                        response
                                            .error
                                            .unwrap_or_else(|| "Unknown error".to_string())
                                    );
                                }
                                Ok(())
                            }
                            Err(e) => {
                                eprintln!("Failed to load wallpaper: {e:?}");
                                Err(e)
                            }
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Daemon is not running. Start it first with 'wlrs start'");
                    Err(IpcError::ConnectionClosed)
                }
            }
        }
        cli::Commands::CurrentWallpaper(_) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send get current wallpaper request
                    let request = GetCurrentWallpaper;
                    match client.request(request) {
                        Ok(status) => {
                            if let Some(name) = status.name {
                                println!("Current wallpaper: {name}");
                                if let Some(path) = status.path {
                                    println!("Path: {path}");
                                }
                            } else {
                                println!("No wallpaper currently loaded");
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to get current wallpaper: {e:?}");
                            Err(e)
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Daemon is not running. Start it first with 'wlrs start'");
                    Err(IpcError::ConnectionClosed)
                }
            }
        }
        cli::Commands::ListWallpapers(_) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send list wallpapers request
                    let request = ListWallpapers;
                    match client.request(request) {
                        Ok(list) => {
                            if list.wallpapers.is_empty() {
                                println!("No wallpapers installed");
                            } else {
                                println!("Available wallpapers:");
                                for wallpaper in list.wallpapers {
                                    println!("  {} - {}", wallpaper.name, wallpaper.path);
                                }
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to list wallpapers: {e:?}");
                            Err(e)
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Daemon is not running. Start it first with 'wlrs start'");
                    Err(IpcError::ConnectionClosed)
                }
            }
        }
        cli::Commands::InstallWallpaper(args) => {
            // Try to connect to the daemon to get the installation directory
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // First, check if the source directory exists and contains a manifest
                    let source_path = Path::new(&args.path);
                    if !source_path.exists() || !source_path.is_dir() {
                        eprintln!(
                            "The source path '{}' does not exist or is not a directory",
                            args.path
                        );
                        return Ok(());
                    }

                    let manifest_path = source_path.join("manifest.toml");
                    if !manifest_path.exists() {
                        eprintln!("The source directory does not contain a manifest.toml file");
                        return Ok(());
                    }

                    // Get the installation directory from the server
                    let request = GetInstallDirectory;
                    match client.request(request) {
                        Ok(install_dir_info) => {
                            if !install_dir_info.success {
                                eprintln!(
                                    "Failed to get install directory: {}",
                                    install_dir_info
                                        .error
                                        .unwrap_or_else(|| "Unknown error".to_string())
                                );
                                return Ok(());
                            }

                            // Create the installation directory if it doesn't exist
                            let install_dir = Path::new(&install_dir_info.path);
                            fs::create_dir_all(install_dir).unwrap_or_else(|e| {
                                eprintln!("Failed to create installation directory: {e}");
                                std::process::exit(1);
                            });

                            // Determine the target directory name
                            let wallpaper_name = match args.name {
                                Some(ref name) => name.clone(),
                                None => source_path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "unknown_wallpaper".to_string()),
                            };

                            let target_dir = install_dir.join(&wallpaper_name);

                            // If target directory already exists, remove it
                            if target_dir.exists() {
                                fs::remove_dir_all(&target_dir).unwrap_or_else(|e| {
                                    eprintln!("Failed to remove existing wallpaper directory: {e}");
                                    std::process::exit(1);
                                });
                            }

                            // Copy the wallpaper directory to the installation location
                            let mut options = CopyOptions::new();
                            options.overwrite = true;
                            options.copy_inside = true;

                            match copy(source_path, install_dir, &options) {
                                Ok(_) => {
                                    println!(
                                        "Wallpaper '{}' installed successfully to '{}'",
                                        wallpaper_name,
                                        target_dir.display()
                                    );

                                    // Rename the directory to the specified name if different
                                    let copied_dir = install_dir.join(
                                        source_path
                                            .file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "unknown_wallpaper".to_string()),
                                    );

                                    if copied_dir != target_dir && args.name.is_some() {
                                        fs::rename(copied_dir, target_dir).unwrap_or_else(|e| {
                                            eprintln!("Failed to rename wallpaper directory: {e}");
                                            std::process::exit(1);
                                        });
                                    }

                                    Ok(())
                                }
                                Err(e) => {
                                    eprintln!("Failed to copy wallpaper directory: {e}");
                                    Ok(())
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get installation directory: {e:?}");
                            Err(e)
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Daemon is not running. Start it first with 'wlrs start'");
                    Err(IpcError::ConnectionClosed)
                }
            }
        }
        cli::Commands::SetWallpaper(args) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send set current wallpaper request
                    let request = SetCurrentWallpaper {
                        name: args.name,
                        monitor: args.monitor,
                    };
                    match client.request(request) {
                        Ok(status) => {
                            if status.success {
                                println!("Current wallpaper set to '{}'", status.name);
                            } else {
                                eprintln!(
                                    "Failed to set wallpaper: {}",
                                    status.error.unwrap_or_else(|| "Unknown error".to_string())
                                );
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to set wallpaper: {e:?}");
                            Err(e)
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Daemon is not running. Start it first with 'wlrs start'");
                    Err(IpcError::ConnectionClosed)
                }
            }
        }
        cli::Commands::Query(_) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send query active wallpapers request
                    let request = QueryActiveWallpapers;
                    match client.request(request) {
                        Ok(result) => {
                            if result.success {
                                if result.wallpapers.is_empty() {
                                    println!("No active wallpapers found");
                                } else {
                                    println!("Active wallpapers:");
                                    for wallpaper in result.wallpapers {
                                        println!("  Monitor: {}", wallpaper.output_name);
                                        println!("    Name: {}", wallpaper.name);
                                        println!(
                                            "    Size: {}x{}",
                                            wallpaper.width, wallpaper.height
                                        );
                                        println!();
                                    }
                                }
                            } else {
                                eprintln!(
                                    "Failed to query active wallpapers: {}",
                                    result.error.unwrap_or_else(|| "Unknown error".to_string())
                                );
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to query active wallpapers: {e:?}");
                            Err(e)
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Daemon is not running. Start it first with 'wlrs start'");
                    Err(IpcError::ConnectionClosed)
                }
            }
        }
        cli::Commands::Stop(_) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send stop server request
                    let request = StopServer;
                    match client.request(request) {
                        Ok(status) => {
                            if status.success {
                                println!("Daemon is shutting down gracefully");
                            } else {
                                eprintln!("Failed to stop daemon");
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to stop daemon: {e:?}");
                            Err(e)
                        }
                    }
                }
                Err(_) => {
                    println!("Daemon is not running");
                    Ok(())
                }
            }
        }
    }
}

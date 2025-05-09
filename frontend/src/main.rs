mod cli;

use clap::Parser;
use common::{
    ipc::{IpcError, IpcSocket, Stream},
    types::{
        Checkhealth, GetCurrentWallpaper, InstallWallpaper, ListWallpapers, LoadWallpaper,
        QueryActiveWallpapers, SetCurrentWallpaper, StopServer,
    },
};

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
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send install wallpaper request
                    let request = InstallWallpaper {
                        path: args.path,
                        name: args.name,
                    };
                    match client.request(request) {
                        Ok(status) => {
                            if status.success {
                                println!("Wallpaper '{}' installed successfully", status.name);
                            } else {
                                eprintln!(
                                    "Failed to install wallpaper: {}",
                                    status.error.unwrap_or_else(|| "Unknown error".to_string())
                                );
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to install wallpaper: {e:?}");
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
                                        println!("    Size: {}x{}", wallpaper.width, wallpaper.height);
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

mod cli;

use clap::Parser;
use common::{
    ipc::{IpcError, IpcSocket, Stream},
    types::{
        GetCurrentWallpaper, InstallWallpaper, ListWallpapers, LoadWallpaper, Ping, Response,
        SetCurrentWallpaper, SetDaemonState, SetFramerate,
    },
};
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

fn main() -> Result<(), IpcError> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Ping(_) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send ping request
                    match client.request(Ping) {
                        Ok(pong) => {
                            println!("Daemon is running: {:?}", pong);
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to get response from daemon: {:?}", e);
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
        cli::Commands::Start(args) => {
            // Check if daemon is already running
            if IpcSocket::<Stream>::is_daemon_running() {
                println!("Daemon is already running");
                return Ok(());
            }

            // Start the daemon
            if args.detach {
                // Start detached
                let daemon = Command::new("wlrs-daemon")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();

                match daemon {
                    Ok(_) => {
                        println!("Daemon started in detached mode");

                        // Wait for the daemon to start
                        for _ in 0..10 {
                            thread::sleep(Duration::from_millis(100));
                            if IpcSocket::<Stream>::is_daemon_running() {
                                println!("Daemon is now running");
                                return Ok(());
                            }
                        }

                        println!("Warning: Daemon might not have started properly");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Failed to start daemon: {}", e);
                        Err(IpcError::Io(e))
                    }
                }
            } else {
                // Start and keep process in foreground
                println!("Starting daemon...");
                let status = Command::new("wlrs-daemon").status();

                match status {
                    Ok(exit_status) => {
                        if exit_status.success() {
                            println!("Daemon exited successfully");
                            Ok(())
                        } else {
                            eprintln!("Daemon exited with error: {:?}", exit_status);
                            Err(IpcError::Io(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Daemon exited with non-zero status",
                            )))
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to start daemon: {}", e);
                        Err(IpcError::Io(e))
                    }
                }
            }
        }
        cli::Commands::SetState(args) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send set state request
                    let request = SetDaemonState {
                        enabled: args.enabled,
                    };
                    match client.request(request) {
                        Ok(status) => {
                            println!("Daemon state updated: {:?}", status);
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to set daemon state: {:?}", e);
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
        cli::Commands::SetFramerate(args) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Send set framerate request
                    let request = SetFramerate { fps: args.fps };
                    match client.request(request) {
                        Ok(status) => {
                            println!("Daemon framerate updated: {:?}", status);
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to set daemon framerate: {:?}", e);
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
        cli::Commands::LoadWallpaper(args) => {
            // Try to connect to the daemon
            match IpcSocket::<Stream>::connect() {
                Ok(mut client) => {
                    // Check if this is a wallpaper name (without path separators) or a path
                    if !args.path.contains('/') && !args.path.contains('\\') {
                        // This looks like just a name, use SetCurrentWallpaper
                        println!("Loading wallpaper by name: {}", args.path);
                        let request = SetCurrentWallpaper { name: args.path };

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
                                eprintln!("Failed to load wallpaper: {:?}", e);
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
                                eprintln!("Failed to load wallpaper: {:?}", e);
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
                                println!("Current wallpaper: {}", name);
                                if let Some(path) = status.path {
                                    println!("Path: {}", path);
                                }
                            } else {
                                println!("No wallpaper currently loaded");
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Failed to get current wallpaper: {:?}", e);
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
                            eprintln!("Failed to list wallpapers: {:?}", e);
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
                            eprintln!("Failed to install wallpaper: {:?}", e);
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
                    let request = SetCurrentWallpaper { name: args.name };
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
                            eprintln!("Failed to set wallpaper: {:?}", e);
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
    }
}

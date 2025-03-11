use bevy::prelude::*;
use common::{
    ipc::{IpcSocket, Listener},
    types::{DaemonStatus, FramerateStatus, Pong, Request, Response},
};
use std::{process::exit, sync::mpsc, thread};

use common::wallpaper::Wallpaper;
use daemon::renderer::systems::background::BackgroundConfig;
use daemon::wallpaper_manager::WallpaperManager;

// Shared state for the daemon
#[derive(Debug, Clone)]
enum DaemonMessage {
    SetFramerate(u32),
    SetRunning(bool),
    LoadWallpaperFromPath(String),
    InstallWallpaper { path: String, name: Option<String> },
    SetCurrentWallpaper(String),
    // Add more message types here in the future
}

#[derive(Resource, Default)]
struct DaemonRunning(bool);

#[derive(Resource, Default)]
struct WallpaperState {
    manager: WallpaperManager,
}

#[derive(Default)]
struct MessageReceiver {
    receiver: Option<mpsc::Receiver<DaemonMessage>>,
}

/// Helper function to create a background configuration from a wallpaper
fn create_background_config(wallpaper: &Wallpaper) -> BackgroundConfig {
    let mut config = BackgroundConfig::default();
    
    // Set scale mode
    config.scale_mode = wallpaper.scale_mode().clone();
    
    // Set background image path
    config.image_path = wallpaper
        .background_image()
        .map(|p| p.to_string_lossy().to_string());
    
    // Set background color if specified
    if let Some(color_str) = wallpaper.background_color() {
        if color_str.starts_with('#') && (color_str.len() == 7 || color_str.len() == 9) {
            // Parse hex color (#RRGGBB or #RRGGBBAA)
            let r = u8::from_str_radix(&color_str[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&color_str[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&color_str[5..7], 16).unwrap_or(0);
            let a = if color_str.len() == 9 {
                u8::from_str_radix(&color_str[7..9], 16).unwrap_or(255)
            } else {
                255
            };

            config.color = Some(Color::srgba(r.into(), g.into(), b.into(), a.into()));
        } else {
            // Default to black if invalid color
            config.color = Some(Color::BLACK);
        }
    } else {
        // Use black as default if no color specified
        config.color = Some(Color::BLACK);
    }
    
    println!("Created background config: {:?}", config);
    config
}

// System to process messages from the channel
fn process_daemon_messages(
    message_receiver: NonSendMut<MessageReceiver>,
    mut daemon_running: ResMut<DaemonRunning>,
    mut sctk_config: ResMut<daemon::renderer::config::SctkLayerWindowConfig>,
    mut wallpaper_state: ResMut<WallpaperState>,
    mut background_config: ResMut<BackgroundConfig>,
) {
    if let Some(rx) = &message_receiver.receiver {
        while let Ok(message) = rx.try_recv() {
            match message {
                DaemonMessage::SetFramerate(fps) => {
                    println!("Updating framerate to {} FPS", fps);
                    // Calculate the new pace based on framerate
                    let new_pace = std::time::Duration::from_secs_f64(1.0 / fps as f64);
                    sctk_config.pace = new_pace;
                }
                DaemonMessage::SetRunning(running) => {
                    println!("Setting daemon running state to: {}", running);
                    daemon_running.0 = running;
                }
                // LoadWallpaper message has been replaced by SetCurrentWallpaper
                DaemonMessage::LoadWallpaperFromPath(path) => {
                    println!("Loading wallpaper from path: {}", path);
                    match wallpaper_state.manager.load_from_path(&path) {
                        Ok(wallpaper) => {
                            println!("Loaded wallpaper: {}", wallpaper.name());

                            // Set framerate from wallpaper if specified
                            let fps = wallpaper.fps();
                            if fps > 0 {
                                println!("Setting wallpaper framerate to {} FPS", fps);
                                let new_pace = std::time::Duration::from_secs_f64(1.0 / fps as f64);
                                sctk_config.pace = new_pace;
                            }

                            // Create new background config and update resource
                            *background_config = create_background_config(wallpaper);
                        }
                        Err(e) => {
                            eprintln!("Failed to load wallpaper: {:?}", e);
                        }
                    }
                }
                DaemonMessage::InstallWallpaper { path, name } => {
                    println!(
                        "Installing wallpaper from path: {} with name: {:?}",
                        path, name
                    );
                    match wallpaper_state.manager.install_wallpaper(&path, name) {
                        Ok(installed_name) => {
                            println!("Installed wallpaper: {}", installed_name);
                        }
                        Err(e) => {
                            eprintln!("Failed to install wallpaper: {:?}", e);
                        }
                    }
                }
                DaemonMessage::SetCurrentWallpaper(name) => {
                    println!("Setting current wallpaper to: {}", name);
                    match wallpaper_state.manager.load_wallpaper(&name) {
                        Ok(wallpaper) => {
                            println!("Set current wallpaper to: {}", wallpaper.name());

                            // Set framerate from wallpaper
                            let fps = wallpaper.fps();
                            if fps > 0 {
                                println!("Setting wallpaper framerate to {} FPS", fps);
                                let new_pace = std::time::Duration::from_secs_f64(1.0 / fps as f64);
                                sctk_config.pace = new_pace;
                            }

                            // Create new background config and update resource
                            *background_config = create_background_config(wallpaper);
                        }
                        Err(e) => {
                            eprintln!("Failed to set current wallpaper: {:?}", e);
                        }
                    }
                }
            }
        }
    }
}

// Handle IPC requests
fn handle_ipc_request(
    request: Request,
    msg_sender: mpsc::Sender<DaemonMessage>,
    wallpaper_state: &WallpaperState,
) -> Response {
    match request {
        Request::Ping(_) => {
            println!("Received ping request");
            Response::Pong(Pong(true))
        }
        Request::SetDaemonState(set_state) => {
            println!("Received set daemon state request: {:?}", set_state);

            // Send a message to update the running state
            let enabled = set_state.enabled;
            if let Err(e) = msg_sender.send(DaemonMessage::SetRunning(enabled)) {
                eprintln!("Failed to send running state message: {:?}", e);
            }

            // Return the current status
            Response::DaemonStatus(DaemonStatus { running: enabled })
        }
        Request::SetFramerate(set_framerate) => {
            println!(
                "Received set framerate request: {:?} fps",
                set_framerate.fps
            );

            // Send a message to update the framerate
            let fps = set_framerate.fps;
            if let Err(e) = msg_sender.send(DaemonMessage::SetFramerate(fps)) {
                eprintln!("Failed to send framerate message: {:?}", e);
            }

            // Return the current framerate
            Response::FramerateStatus(FramerateStatus { fps })
        }
        Request::LoadWallpaper(load_wallpaper) => {
            println!(
                "Received load wallpaper from path request: {:?}",
                load_wallpaper.path
            );

            // Send a message to load the wallpaper from path
            let path = load_wallpaper.path.clone();

            if let Err(e) = msg_sender.send(DaemonMessage::LoadWallpaperFromPath(path.clone())) {
                eprintln!("Failed to send load wallpaper message: {:?}", e);
                Response::WallpaperLoaded(common::types::WallpaperLoaded {
                    name: path,
                    success: false,
                    error: Some(format!("Failed to send message to daemon: {}", e)),
                })
            } else {
                Response::WallpaperLoaded(common::types::WallpaperLoaded {
                    name: path,
                    success: true,
                    error: None,
                })
            }
        }
        Request::GetCurrentWallpaper(_) => {
            println!("Received get current wallpaper request");

            // Return the current wallpaper info from manager
            match wallpaper_state.manager.get_current_wallpaper() {
                Some(current) => Response::CurrentWallpaper(current),
                None => Response::CurrentWallpaper(common::types::CurrentWallpaper {
                    name: None,
                    path: None,
                }),
            }
        }
        Request::ListWallpapers(_) => {
            println!("Received list wallpapers request");

            // Return list of wallpapers from manager
            match wallpaper_state.manager.list_wallpapers() {
                Ok(wallpapers) => {
                    Response::WallpaperList(common::types::WallpaperList { wallpapers })
                }
                Err(e) => {
                    eprintln!("Failed to list wallpapers: {:?}", e);
                    Response::WallpaperList(common::types::WallpaperList {
                        wallpapers: Vec::new(),
                    })
                }
            }
        }
        Request::InstallWallpaper(install) => {
            println!("Received install wallpaper request: {:?}", install);

            // Send a message to install the wallpaper
            let path = install.path.clone();
            let name = install.name.clone();

            if let Err(e) = msg_sender.send(DaemonMessage::InstallWallpaper {
                path: path.clone(),
                name: name.clone(),
            }) {
                eprintln!("Failed to send install wallpaper message: {:?}", e);
                Response::WallpaperInstalled(common::types::WallpaperInstalled {
                    name: name.unwrap_or(path),
                    success: false,
                    error: Some(format!("Failed to send message to daemon: {}", e)),
                })
            } else {
                // Note: this immediately returns success, the actual installation
                // happens asynchronously
                Response::WallpaperInstalled(common::types::WallpaperInstalled {
                    name: name.unwrap_or(path),
                    success: true,
                    error: None,
                })
            }
        }
        Request::SetCurrentWallpaper(set) => {
            println!("Received set current wallpaper request: {:?}", set);

            // Send a message to set the current wallpaper
            let name = set.name.clone();

            if let Err(e) = msg_sender.send(DaemonMessage::SetCurrentWallpaper(name.clone())) {
                eprintln!("Failed to send set current wallpaper message: {:?}", e);
                Response::WallpaperSet(common::types::WallpaperSet {
                    name,
                    success: false,
                    error: Some(format!("Failed to send message to daemon: {}", e)),
                })
            } else {
                Response::WallpaperSet(common::types::WallpaperSet {
                    name,
                    success: true,
                    error: None,
                })
            }
        }
    }
}

// Start the IPC server
fn start_ipc_server(msg_sender: mpsc::Sender<DaemonMessage>, wallpaper_state: WallpaperState) {
    thread::spawn(move || {
        // Create IPC socket
        let listener = match IpcSocket::<Listener>::listen() {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed to create IPC socket: {:?}", e);
                exit(1);
            }
        };

        println!("IPC server started");

        // Set daemon as running through message channel
        if let Err(e) = msg_sender.send(DaemonMessage::SetRunning(true)) {
            eprintln!("Failed to set initial running state: {:?}", e);
        }

        // Create shared wallpaper state
        let wallpaper_state = std::sync::Arc::new(std::sync::Mutex::new(wallpaper_state));

        // Handle requests in a loop
        loop {
            match listener.accept() {
                Ok(mut client) => {
                    let msg_sender_clone = msg_sender.clone();
                    let wallpaper_state = wallpaper_state.clone();

                    // Spawn a thread to handle this client
                    thread::spawn(move || match client.receive::<Request>() {
                        Ok(request) => {
                            let state = wallpaper_state.lock().unwrap();
                            let response = handle_ipc_request(request, msg_sender_clone, &state);
                            if let Err(e) = client.send(&response) {
                                eprintln!("Failed to send response: {:?}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to receive request: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept client: {:?}", e);
                }
            }
        }
    });
}

fn main() {
    // Create the message channel
    let (msg_sender, msg_receiver) = mpsc::channel::<DaemonMessage>();

    // Create initial wallpaper state with manager
    let wallpaper_state = WallpaperState::default();

    // Start IPC server
    start_ipc_server(msg_sender.clone(), wallpaper_state);

    // Set initial framerate (30 FPS)
    if let Err(e) = msg_sender.send(DaemonMessage::SetFramerate(30)) {
        eprintln!("Failed to set initial framerate: {:?}", e);
    }

    // For testing: Try to load a sample wallpaper with a WebP image
    let example_path =
        std::path::Path::new("/home/werdxz/Projects/rust/wlrs/examples/wallpapers/webp-test");
    if example_path.exists() {
        println!("Loading example WebP wallpaper for testing");
        if let Err(e) = msg_sender.send(DaemonMessage::LoadWallpaperFromPath(
            example_path.to_string_lossy().to_string(),
        )) {
            eprintln!("Failed to send test wallpaper message: {:?}", e);
        }
    }

    // Configure and run the app
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<bevy::winit::WinitPlugin>())
        .add_plugins(daemon::renderer::SctkPlugin)
        .init_resource::<DaemonRunning>()
        .init_resource::<WallpaperState>()
        .init_resource::<BackgroundConfig>()
        .insert_non_send_resource(MessageReceiver {
            receiver: Some(msg_receiver),
        })
        .add_systems(Update, process_daemon_messages)
        .run();
}

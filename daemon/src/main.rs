use epoll::Events;
use std::os::fd::{AsFd, AsRawFd};

use common::{
    ipc::{IpcSocket, Listener},
    types::{
        ActiveWallpaperInfo, ActiveWallpaperList, CurrentWallpaper, Health, InstallDirectory,
        Request, Response, ServerStopping, WallpaperList, WallpaperLoaded, WallpaperSet,
    },
};
use daemon::renderer::client::Client;

fn main() {
    env_logger::init();

    // Ensure wallpaper directory exists
    ensure_wallpaper_directory();

    // Create initial wallpaper state with manager
    let (mut client, mut event_queue) = Client::new(Some("wlrs"));
    let stream = IpcSocket::<Listener>::listen()
        .expect("A ipc socket need to be created for client-server functionality");

    let wayland_event_fd = event_queue.as_fd().as_raw_fd();
    let client_event_fd = stream.as_fd().as_raw_fd();

    let ep = epoll::create(false).expect("Epoll create failed");
    let wayland_event = epoll::Event::new(Events::EPOLLIN, wayland_event_fd as u64);
    epoll::ctl(
        ep,
        epoll::ControlOptions::EPOLL_CTL_ADD,
        wayland_event_fd,
        wayland_event,
    )
    .expect("Epoll ctl failed");
    let client_event = epoll::Event::new(Events::EPOLLIN, client_event_fd as u64);
    epoll::ctl(
        ep,
        epoll::ControlOptions::EPOLL_CTL_ADD,
        client_event_fd,
        client_event,
    )
    .expect("Epoll ctl failed");

    // Pre-allocate events array for epoll
    let mut events = [epoll::Event::new(Events::empty(), 0); 2];
    let mut wayland_event_ready = false;
    let mut client_event_ready = false;
    loop {
        event_queue.flush().unwrap();
        let wayland_event_read_guard = event_queue.prepare_read();
        if wayland_event_read_guard.is_none() {
            event_queue
                .dispatch_pending(&mut client)
                .expect("Failed to dispatch wayland events");
        }

        // Wait for events with epoll
        let num_events = epoll::wait(ep, -1, &mut events).unwrap();

        // Only process the number of events that were returned
        (0..num_events).for_each(|i| {
            let event = &events[i];
            if event.data == wayland_event_fd as u64 {
                log::debug!("Wayland event ready");
                wayland_event_ready = true;
            } else if event.data == client_event_fd as u64 {
                log::debug!("Client event ready");
                client_event_ready = true;
            }
        });

        if let Some(wayland_event_read_guard) = wayland_event_read_guard {
            log::debug!("Wayland event read guard");
            wayland_event_read_guard.read().unwrap();
            if wayland_event_ready {
                event_queue
                    .dispatch_pending(&mut client)
                    .expect("Failed to dispatch wayland events");
            }
        }

        if client_event_ready {
            // stream.handle_request(handler).unwrap();
            let mut client_socket = stream.accept().unwrap();
            let request: Request = client_socket.receive().unwrap();
            let response = match request {
                Request::Checkhealth(req) => Response::Health(Health(true)),
                Request::LoadWallpaper(req) => Response::WallpaperLoaded(WallpaperLoaded {
                    name: "".to_string(),
                    success: false,
                    error: Some("Not Supported".to_string()),
                }),
                Request::StopServer(_) => {
                    *daemon::EXIT.lock().unwrap() = true;
                    Response::ServerStopping(ServerStopping {
                        success: *daemon::EXIT.lock().unwrap(),
                    })
                }
                Request::ListWallpapers(_) => {
                    // Scan for available wallpapers in the standard directories
                    let wallpapers = find_available_wallpapers();

                    Response::WallpaperList(WallpaperList { wallpapers })
                }
                Request::GetCurrentWallpaper(req) => Response::CurrentWallpaper(CurrentWallpaper {
                    name: None,
                    path: None,
                }),
                Request::SetCurrentWallpaper(req) => Response::WallpaperSet(WallpaperSet {
                    name: "".to_string(),
                    success: false,
                    error: Some("Not supported".to_string()),
                }),
                Request::QueryActiveWallpapers(_) => {
                    // Get information about active wallpapers from client.wallpapers
                    let mut active_wallpapers = Vec::new();

                    // Iterate through wallpapers in client
                    for layer in client.wallpapers.iter() {
                        active_wallpapers.push(ActiveWallpaperInfo {
                            name: layer.name.clone(),
                            output_name: layer.name.clone(), // Using the same name since it's derived from output name
                            width: layer.width,
                            height: layer.height,
                        });
                    }

                    Response::ActiveWallpaperList(ActiveWallpaperList {
                        wallpapers: active_wallpapers,
                        success: true,
                        error: None,
                    })
                }
                Request::GetInstallDirectory(_) => {
                    // Return the standardized XDG data directory for wallpaper installations
                    let install_dir = directories::BaseDirs::new()
                        .map(|dirs| {
                            dirs.data_dir()
                                .join("wlrs")
                                .join("wallpapers")
                                .to_string_lossy()
                                .to_string()
                        })
                        .unwrap_or_else(|| String::from("/tmp/wlrs/wallpapers"));

                    Response::InstallDirectory(InstallDirectory {
                        path: install_dir,
                        success: true,
                        error: None,
                    })
                }
            };
            client_socket.send(&response).unwrap();
        }

        wayland_event_ready = false;
        client_event_ready = false;
        if *daemon::EXIT.lock().unwrap() {
            break;
        }
    }
}

/// Find all available wallpapers in standard directories
fn find_available_wallpapers() -> Vec<common::types::WallpaperInfo> {
    use common::types::WallpaperInfo;
    use common::wallpaper::WallpaperDirectory;
    use std::path::PathBuf;

    let mut all_wallpapers = Vec::new();

    // Define the standard directories where wallpapers can be located
    let possible_paths = vec![
        // User-specific wallpapers in data directory
        directories::BaseDirs::new()
            .map(|dirs| dirs.data_dir().join("wlrs").join("wallpapers"))
            .unwrap_or_else(|| PathBuf::from("/tmp/wlrs/wallpapers")),
        // Example wallpapers in the project directory (for development)
        PathBuf::from("examples/wallpapers"),
    ];

    // Check each directory for wallpapers
    for path in possible_paths {
        if !path.exists() || !path.is_dir() {
            continue;
        }

        // Create a wallpaper directory handler
        let wallpaper_dir = WallpaperDirectory::new(&path);

        // List all wallpapers in the directory
        match wallpaper_dir.list_wallpapers() {
            Ok(names) => {
                for name in names {
                    // Attempt to load each wallpaper to get its details
                    if let Ok(wallpaper) = wallpaper_dir.load_wallpaper(&name) {
                        all_wallpapers.push(WallpaperInfo {
                            name: wallpaper.manifest.name.clone(),
                            path: wallpaper.path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
            Err(_) => continue, // Skip directories that cannot be read
        }
    }

    all_wallpapers
}

/// Ensure that the wallpaper directory exists
fn ensure_wallpaper_directory() {
    use std::fs;
    use std::path::PathBuf;

    // Get the user-specific wallpapers directory
    let user_wallpaper_dir = directories::BaseDirs::new()
        .map(|dirs| dirs.data_dir().join("wlrs").join("wallpapers"))
        .unwrap_or_else(|| PathBuf::from("/tmp/wlrs/wallpapers"));

    // Create the directory if it doesn't exist
    if !user_wallpaper_dir.exists() {
        println!(
            "Creating user wallpaper directory: {}",
            user_wallpaper_dir.display()
        );
        if let Err(e) = fs::create_dir_all(&user_wallpaper_dir) {
            eprintln!("Failed to create user wallpaper directory: {e}");
        }
    }
}

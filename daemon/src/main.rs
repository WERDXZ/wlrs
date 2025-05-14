use epoll::Events;
use std::os::fd::{AsFd, AsRawFd};
use std::path::Path;

use common::{
    ipc::{IpcSocket, Listener},
    types::{
        ActiveWallpaperInfo, ActiveWallpaperList, Health, InstallDirectory, Request, Response,
        ServerStopping, WallpaperList, WallpaperLoaded,
    },
    wallpaper::Wallpaper,
};
use daemon::renderer::client::Client;
use daemon::utils::*;

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

    // Frame counter for animation timing (roughly ~60 frames per second)
    let qh = event_queue.handle();
    let mut last_render_time = std::time::Instant::now();
    let target_frame_time = std::time::Duration::from_millis(32); // ~60 FPS

    loop {
        // Handle rendering frames
        let current_time = std::time::Instant::now();
        if current_time.duration_since(last_render_time) >= target_frame_time {
            // Render a new frame
            client.request_update(&qh);
            last_render_time = current_time;
        }

        event_queue.flush().unwrap();
        let wayland_event_read_guard = event_queue.prepare_read();
        if wayland_event_read_guard.is_none() {
            event_queue
                .dispatch_pending(&mut client)
                .expect("Failed to dispatch wayland events");
        }

        // Wait for events with epoll with a timeout to ensure animations continue
        let tickrate = 5; // Short timeout to ensure animations remain smooth
        let num_events = epoll::wait(ep, tickrate, &mut events).unwrap();

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
                Request::Checkhealth(_) => Response::Health(Health(true)),
                Request::LoadWallpaper(req) => {
                    // Try to load the wallpaper from the specified path
                    match Wallpaper::load(&req.path) {
                        Ok(wallpaper) => Response::WallpaperLoaded(WallpaperLoaded {
                            name: wallpaper.name().to_string(),
                            success: true,
                            error: None,
                        }),
                        Err(e) => Response::WallpaperLoaded(WallpaperLoaded {
                            name: Path::new(&req.path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            success: false,
                            error: Some(format!("Failed to load wallpaper: {e}")),
                        }),
                    }
                }
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
                Request::SetCurrentWallpaper(req) => handle_set_wallpaper(&req, &mut client),
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

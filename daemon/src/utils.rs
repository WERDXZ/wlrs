use common::{
    types::{Response, SetCurrentWallpaper, WallpaperInfo, WallpaperSet},
    wallpaper::Wallpaper,
};

use crate::renderer::client::Client;

/// Handle a request to set the current wallpaper
pub fn handle_set_wallpaper(req: &SetCurrentWallpaper, client: &mut Client) -> Response {
    // Try to find the requested wallpaper
    let wallpaper_info = find_wallpaper_by_name(&req.name);

    // If wallpaper not found, return error
    if wallpaper_info.is_none() {
        return Response::WallpaperSet(WallpaperSet {
            name: req.name.clone(),
            success: false,
            error: Some("Wallpaper not found".to_string()),
        });
    }

    // Try to load the wallpaper
    let wallpaper_info = wallpaper_info.unwrap();
    let wallpaper_result = Wallpaper::load(&wallpaper_info.path);

    if let Err(e) = wallpaper_result {
        return Response::WallpaperSet(WallpaperSet {
            name: req.name.clone(),
            success: false,
            error: Some(format!("Failed to load wallpaper: {e}")),
        });
    }

    let wallpaper = wallpaper_result.unwrap();

    // If a specific monitor is requested, set only that monitor
    if let Some(ref monitor_name) = req.monitor {
        // Check if the monitor exists
        let found = client
            .wallpapers
            .iter()
            .any(|layer| layer.name == *monitor_name);
        if !found {
            return Response::WallpaperSet(WallpaperSet {
                name: req.name.clone(),
                success: false,
                error: Some(format!("Monitor '{monitor_name}' not found")),
            });
        }

        // Set the wallpaper for the specified monitor
        for layer in client.wallpapers.iter_mut() {
            if layer.name == *monitor_name {
                layer.wallpaper = crate::renderer::pipeline::Pipelines::from(
                    wallpaper.clone(),
                    &client.device,
                    &client.queue,
                    client.bindgroup_layout_manager.clone(),
                    client.pipeline_manager.clone(),
                );
                // Set the framerate and tickrate based on the wallpaper's manifest
                layer.set_framerate(wallpaper.framerate());
                layer.set_tickrate(wallpaper.tickrate());
                layer.damaged = true;
                break;
            }
        }
    } else {
        // Set the wallpaper for all monitors
        for layer in client.wallpapers.iter_mut() {
            layer.wallpaper = crate::renderer::pipeline::Pipelines::from(
                wallpaper.clone(),
                &client.device,
                &client.queue,
                client.bindgroup_layout_manager.clone(),
                client.pipeline_manager.clone(),
            );
            // Set the framerate and tickrate based on the wallpaper's manifest
            layer.set_framerate(wallpaper.framerate());
            layer.set_tickrate(wallpaper.tickrate());
            layer.damaged = true;
            println!("Setting wallpaper for monitor: {}", layer.name);

            println!("tickrate: {}", wallpaper.tickrate());
        }
    }

    Response::WallpaperSet(WallpaperSet {
        name: req.name.clone(),
        success: true,
        error: None,
    })
}

/// Find all available wallpapers in standard directories
pub fn find_available_wallpapers() -> Vec<WallpaperInfo> {
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

/// Find a wallpaper by name
pub fn find_wallpaper_by_name(name: &str) -> Option<WallpaperInfo> {
    // Get all available wallpapers
    let wallpapers = find_available_wallpapers();

    // Find the wallpaper with the matching name
    wallpapers.into_iter().find(|wp| wp.name == name)
}

/// Ensure that the wallpaper directory exists
pub fn ensure_wallpaper_directory() {
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

use std::{env::current_dir, path::Path};

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(
        asset_server.load(format!("{}/{}", current_dir().unwrap().display(), "examples/wallpapers/webp-test/assets/background.webp"))
    ));
}


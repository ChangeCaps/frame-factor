use crate::networking::*;
use bevy::prelude::*;

/// Runs the client.
pub fn run(ip: String) {
    App::build()
        // plugins
        .add_plugin(NetworkPlugin::client(ip))
        .add_plugins(DefaultPlugins)
        // systems
        // startup_systems
        .add_startup_system(setup_system.system())
        .run();
}

/// Setup system for the client.
fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sprites: Res<Assets<Texture>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let handles = asset_server.load_folder(".").unwrap();

    commands.spawn(OrthographicCameraBundle::new_2d());
    commands.spawn(SpriteBundle {
        material: color_materials.add(sprites.get_handle("arrow.png").into()),
        ..Default::default()
    });

    commands.insert_resource(handles);
}

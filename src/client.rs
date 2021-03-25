use crate::networking::*;
use crate::player::*;
use crate::world_transform::*;
use bevy::prelude::*;

/// Runs the client.
pub fn run(ip: String) {
    App::build()
        // plugins
        .add_plugin(NetworkPlugin::client(ip))
        .add_plugins(DefaultPlugins)
        // network events
        .register_network_event::<WorldTransformEvent>()
        // network spawnables
        .register_network_spawnable::<PlayerSpawner>()
        // systems
        .add_system(world_transform_system.system())
        .add_system(world_transform_network_system.system())
        // startup systems
        .add_startup_system(setup_system.system())
        .run();
}

/// Setup system for the client.
fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = asset_server.load_folder(".").unwrap();

    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());

    commands.insert_resource(handles);
}

use crate::game_state::*;
use crate::networking::*;
use crate::player::*;
use crate::world_transform::*;
use bevy::prelude::*;

pub fn run(ip: String) {
    App::build()
        // resources
        .insert_resource(bevy::app::ScheduleRunnerSettings::run_loop(
            std::time::Duration::from_secs_f32(1.0 / 40.0),
        ))
        // plugins
        .add_plugin(NetworkPlugin::server(ip))
        .add_plugins(MinimalPlugins)
        // network events
        .register_network_event::<WorldTransformEvent>()
        // network spawnables
        .register_network_spawnable::<PlayerSpawner>()
        // state
        .add_state(GameState::Connection)
        // systems
        .add_system(world_transform_system.system())
        .add_system(connection_system.system())
        // startup systems
        .run();
}

fn connection_system(
    mut event_reader: EventReader<ConnectionEvent>,
    network_spawner: Res<NetworkSpawner>,
) {
    for event in event_reader.iter() {
        match event {
            ConnectionEvent::Connected { id } => {
                network_spawner.spawn(PlayerSpawner { player_id: *id });
            }
            _ => {}
        }
    }
}
